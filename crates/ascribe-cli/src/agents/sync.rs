//! `ascribe agents sync`: which files to write for a project, and what each
//! should hold. [`plan`] works it all out and writes nothing, so a damaged
//! file stops the command before any file changes; [`Plan::write`] writes
//! what changed.
//!
//! The files, by target:
//!
//! - `agents-md`: `AGENTS.md` beside `ascribe.toml`, with the project's
//!   rules; when that isn't the repository's root, a short block of its own
//!   in the root's `AGENTS.md` too, since Codex reads no file below where it
//!   starts.
//! - `skills`: the skill, in `.agents/skills/ascribe/` at the root.
//! - `claude`: an import of `AGENTS.md` in each `CLAUDE.md` beside one
//!   (a `CLAUDE.md` stops Claude Code reading `AGENTS.md` by itself), and
//!   the skill in `.claude/skills/ascribe/`, where Claude Code looks.
//! - `claude-rules`: the rules in `.claude/rules/`, loaded only for pages.
//! - `copilot`: the rules in `.github/instructions/`, loaded only for pages.
//! - `codex`: nothing of its own, since Codex reads `AGENTS.md` and the
//!   skill; it's there for its hooks.
//!
//! With hooks (`--with-hook`, or once a file holds them), the hook entries
//! and the MCP server too (`super::settings`): `.claude/settings.json` and
//! `.mcp.json` for `claude`, `.codex/hooks.json` for `codex`, and
//! `.github/hooks/ascribe.json` for `copilot`, all at the root.
//!
//! For Copilot's cloud agent (`--cloud`, or once the files hold it,
//! `super::copilot`): a block of setup steps in
//! `.github/workflows/copilot-setup-steps.yml`, and with `--agent` the
//! custom agent `.github/agents/ascribe-docs.md` that carries the MCP
//! server; without it, the server's JSON for the repository's settings,
//! which no file can change.

use std::io;
use std::path::{Path, PathBuf};

use ascribe_check::Project;
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, RelPath};
use ascribe_diff::{DiffError, Repository};
use ascribe_query::rules::{content_glob, pointer, rules};
use ascribe_resolve::is_source_path;

use super::markers::{Comment, Damage, Markers};
use super::{command, copilot, settings, skill};

/// What `sync` can write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Target {
    /// `AGENTS.md`.
    AgentsMd,
    /// The skill, in `.agents/skills/`.
    Skills,
    /// `CLAUDE.md`'s import, and the skill in `.claude/skills/`.
    Claude,
    /// `.claude/rules/`.
    ClaudeRules,
    /// `.github/instructions/`.
    Copilot,
    /// Codex's hooks.
    Codex,
}

/// Why `sync` can't write a project's files.
#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    /// A file's markers are damaged.
    #[error("can't update {path}: {damage}; fix its `{name}` markers by hand")]
    Damaged {
        /// The file.
        path: String,
        /// The block's name.
        name: String,
        /// What's wrong.
        damage: Damage,
    },
    /// A file `sync` would write is one of the project's pages.
    #[error(
        "can't write {path}: it's under the content root, so it would be one of the \
         project's pages"
    )]
    InContent {
        /// The file.
        path: String,
    },
    /// Copilot's files are asked for, outside a git repository.
    #[error(
        "--target copilot and --cloud need a git repository: Copilot reads its files \
         from the repository's .github/"
    )]
    NoRepository,
    /// `git` couldn't say where the repository is.
    #[error(transparent)]
    Git(DiffError),
    /// A file couldn't be read.
    #[error("can't read {path}: {source}")]
    Read {
        /// The file.
        path: String,
        /// Why.
        source: io::Error,
    },
    /// A settings file `sync` merges its entries into can't be read as one.
    #[error("can't update {path}: {why}; fix it by hand")]
    Settings {
        /// The file.
        path: String,
        /// What's wrong.
        why: String,
    },
    /// A file couldn't be written.
    #[error("can't write {path}: {source}")]
    Write {
        /// The file.
        path: String,
        /// Why.
        source: io::Error,
    },
}

impl Coded for SyncError {
    fn code(&self) -> &'static str {
        match self {
            SyncError::Damaged { .. } => "markers_damaged",
            SyncError::InContent { .. } => "instructions_in_content",
            SyncError::NoRepository => "copilot_needs_repository",
            SyncError::Git(e) => e.code(),
            SyncError::Read { .. } => "instructions_unreadable",
            SyncError::Settings { .. } => "settings_unreadable",
            SyncError::Write { .. } => "instructions_unwritable",
        }
    }
}

/// An agent that reads a file `sync` writes, as `sync` names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Agent {
    /// Claude Code.
    ClaudeCode,
    /// Codex.
    Codex,
    /// GitHub Copilot, in an editor or on GitHub.
    Copilot,
    /// Copilot's cloud agent only.
    CopilotCloud,
    /// Cursor.
    Cursor,
    /// Gemini CLI.
    GeminiCli,
}

impl Agent {
    /// The agent's name.
    pub fn name(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "Claude Code",
            Agent::Codex => "Codex",
            Agent::Copilot => "Copilot",
            Agent::CopilotCloud => "Copilot's cloud agent",
            Agent::Cursor => "Cursor",
            Agent::GeminiCli => "Gemini CLI",
        }
    }
}

/// Who reads `AGENTS.md`. Claude Code does by itself while there's no
/// `CLAUDE.md`, and through the import `sync` adds to one.
const READ_AGENTS_MD: &[Agent] = &[
    Agent::ClaudeCode,
    Agent::Codex,
    Agent::Copilot,
    Agent::Cursor,
];
/// Who reads the skill in `.agents/skills/`.
const READ_SKILLS: &[Agent] = &[
    Agent::Codex,
    Agent::Copilot,
    Agent::Cursor,
    Agent::GeminiCli,
];
const READ_CLAUDE: &[Agent] = &[Agent::ClaudeCode];
const READ_CODEX: &[Agent] = &[Agent::Codex];
const READ_COPILOT: &[Agent] = &[Agent::Copilot];
const READ_COPILOT_CLOUD: &[Agent] = &[Agent::CopilotCloud];

/// What `sync` says when Claude Code reads none of the skill it writes.
const CLAUDE_SKILL_UNREAD: &str = "Claude Code doesn't read the skill in .agents/skills/; \
    `--target claude` writes it to .claude/skills/ too, where it does";

/// One file `sync` writes.
#[derive(Debug)]
pub struct Planned {
    /// Where.
    pub path: PathBuf,
    /// Its text now; `None` when there's no file.
    pub old: Option<String>,
    /// Its text after `sync`.
    pub new: String,
    /// The agents that read it.
    pub readers: &'static [Agent],
}

impl Planned {
    /// Whether `sync` changes it.
    pub fn changes(&self) -> bool {
        self.old.as_deref() != Some(self.new.as_str())
    }
}

/// What `sync` would write, and what it has to say.
#[derive(Debug)]
pub struct Plan {
    /// Each file, in the order they're reported.
    pub files: Vec<Planned>,
    /// Notes for the person running it.
    pub notes: Vec<String>,
    /// What an agent doesn't read of what's written, and the target that
    /// adds it.
    pub unread: Vec<&'static str>,
    /// The MCP servers' JSON for the repository's settings on GitHub, one
    /// for each block of the setup steps, for the person to paste there,
    /// when the cloud agent's setup is asked for without a custom agent.
    pub mcp_settings: Option<String>,
    /// The repository's root, or the project's folder when there's no
    /// repository: what's written is under it, and no link on the way
    /// from it is written through.
    pub root: PathBuf,
}

/// What's asked for beyond the targets' files.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The hooks (`--with-hook`).
    pub hooks: bool,
    /// The setup of Copilot's cloud agent (`--cloud`).
    pub cloud: bool,
    /// The custom agent that carries the MCP server (`--agent`).
    pub agent: bool,
    /// The command the project runs Ascribe with, when it's given
    /// (`--command`) rather than worked out.
    pub command: Option<String>,
}

impl Plan {
    /// Writes each file that changes. Nothing is written when a link is on
    /// the way to one of them from the root, or is where one goes: a
    /// committed link could lead anywhere.
    pub fn write(&self) -> Result<(), SyncError> {
        let mut links = ascribe_resolve::LinkCheck::new(&self.root);
        for file in self.files.iter().filter(|f| f.changes()) {
            let link = match file.path.strip_prefix(&self.root) {
                Ok(rel) => links.link(rel),
                Err(_) => ascribe_resolve::is_link(&file.path).then(|| file.path.clone()),
            };
            if let Some(link) = link {
                return Err(SyncError::Write {
                    path: file.path.display().to_string(),
                    source: io::Error::other(format!(
                        "{} is a link; replace it with a folder or a file, or remove it",
                        link.display()
                    )),
                });
            }
        }
        for file in self.files.iter().filter(|f| f.changes()) {
            let failed = |source| SyncError::Write {
                path: file.path.display().to_string(),
                source,
            };
            if let Some(parent) = file.path.parent() {
                std::fs::create_dir_all(parent).map_err(failed)?;
            }
            std::fs::write(&file.path, &file.new).map_err(failed)?;
        }
        Ok(())
    }
}

/// The marker name of the block in a project's own files.
const BLOCK: &str = "ascribe:agents";

/// The project's folder in its repository, and a note when there's no
/// repository and the folder is taken as its root.
fn prefix_of(here: &Path) -> Result<(RelPath, Option<String>), SyncError> {
    match Repository::discover(here) {
        Ok(repository) => Ok((repository.project_dir(), None)),
        Err(DiffError::NotARepository { .. }) => Ok((
            RelPath::root(),
            Some(format!(
                "{} isn't in a git repository, so its folder is treated as the repository's root",
                here.display()
            )),
        )),
        Err(e) => Err(SyncError::Git(e)),
    }
}

/// The project's rules as `sync` writes them into the `AGENTS.md` beside
/// `ascribe.toml`: what `ascribe agents rules` prints. `index` is the
/// project's source index.
pub fn rules_beside(
    project: &Project,
    index: &ascribe_resolve::Project,
) -> Result<String, SyncError> {
    let (here, prefix, root, _) = locate(project, false)?;
    let text = rules(index, project.file_system(), &RelPath::root(), &prefix);
    Ok(command::with_command(&text, &command::detect(&here, &root)))
}

/// Where `project` is in its repository: its folder, absolute; its folder
/// in the repository; the repository's root, absolute; and a note when
/// there's no repository, and the folder is taken as its root. With
/// `needs_repository`, an error instead of that note.
fn locate(
    project: &Project,
    needs_repository: bool,
) -> Result<(PathBuf, RelPath, PathBuf, Vec<String>), SyncError> {
    let here = absolute(project.root());
    let (prefix, note) = prefix_of(&here)?;
    if note.is_some() && needs_repository {
        return Err(SyncError::NoRepository);
    }
    // The root above the project's folder as written, not as `git` prints
    // it, so a symbolic link on the way doesn't change the paths shown.
    let mut root = here.clone();
    for _ in prefix.segments() {
        root.pop();
    }
    Ok((here, prefix, root, note.into_iter().collect()))
}

/// Works out the files for `targets` (with `agents-md` and `skills` always,
/// and each other target whose files exist already), without writing.
/// With `options.hooks`, the hook entries of each of them too, and with
/// `options.cloud` the cloud agent's setup, which implies `copilot`;
/// without, those already written are kept up to date.
pub fn plan(project: &Project, asked: &[Target], options: &Options) -> Result<Plan, SyncError> {
    let needs_repository = asked.contains(&Target::Copilot) || options.cloud;
    let (here, prefix, root, mut notes) = locate(project, needs_repository)?;
    let (runs, skill_runs) = commands_of(options.command.as_deref(), &here, &root);
    let skill_config = config_of(&prefix, &here, &root);
    let commands = |text: &str| command::with_command(text, &runs);
    let index = project.index();
    let name = if prefix.is_root() {
        "ascribe".to_owned()
    } else {
        format!("ascribe-{}", prefix.as_str().replace('/', "-"))
    };
    let model_file = in_dir(&prefix, "ascribe.toml");
    let rules_file = |dir: &str, file: &str| under(&root, dir).join(file);
    let claude_rules = rules_file(".claude/rules", &format!("{name}.md"));
    let copilot = rules_file(".github/instructions", &format!("{name}.instructions.md"));
    let claude_skill = under(&root, ".claude/skills").join(skill::NAME);

    let claude_files = [
        here.join("CLAUDE.md"),
        under(&here, ".claude/CLAUDE.md"),
        root.join("CLAUDE.md"),
        under(&root, ".claude/CLAUDE.md"),
        claude_skill.join("SKILL.md"),
    ];
    let wants = |target: Target, files: &[PathBuf]| -> Result<bool, SyncError> {
        if asked.contains(&target) {
            return Ok(true);
        }
        for file in files {
            if read(file)?.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    };
    let claude = wants(Target::Claude, &claude_files)?;
    let with_rules = wants(Target::ClaudeRules, std::slice::from_ref(&claude_rules))?;
    let with_copilot = options.cloud || wants(Target::Copilot, std::slice::from_ref(&copilot))?;

    let content = absolute(&project.root().join(project.content_root().as_str()));
    let mut files = Vec::new();
    let mut add = |path: PathBuf, new: &New<'_>, name: &str, readers: &'static [Agent]| {
        files.push(planned(&content, path, new, name, readers)?);
        Ok::<(), SyncError>(())
    };

    // AGENTS.md: the rules beside ascribe.toml, and a pointer at the root.
    let note = commands("generated by `ascribe agents sync`; edit ascribe.toml, not this block");
    let files_of = project.file_system();
    let body = commands(&rules(index, files_of, &RelPath::root(), &prefix));
    let markers = Markers {
        name: BLOCK,
        note: &note,
        comment: Comment::Html,
    };
    add(
        here.join("AGENTS.md"),
        &|old| markers.place(old, &body),
        BLOCK,
        READ_AGENTS_MD,
    )?;
    let mut agents_dirs = vec![here.clone()];
    if !prefix.is_root() {
        let name = format!("{BLOCK}:{}", prefix.as_str());
        let note = commands(&format!(
            "generated by `ascribe agents sync`; edit {model_file}, not this block"
        ));
        let body = commands(&pointer(index, &prefix));
        let markers = Markers {
            name: &name,
            note: &note,
            comment: Comment::Html,
        };
        add(
            root.join("AGENTS.md"),
            &|old| markers.place(old, &body),
            &name,
            READ_AGENTS_MD,
        )?;
        agents_dirs.push(root.clone());
    }

    // CLAUDE.md: an import of AGENTS.md, in each one there is.
    let note = commands("generated by `ascribe agents sync`; it loads AGENTS.md");
    let markers = Markers {
        name: BLOCK,
        note: &note,
        comment: Comment::Html,
    };
    for dir in &agents_dirs {
        let create = asked.contains(&Target::Claude) && dir == &here;
        let (file, note) = claude_md(dir, create, &markers)?;
        notes.extend(note);
        if let Some((file, import)) = file {
            add(
                file,
                &|old| import_once(&markers, old, import),
                BLOCK,
                READ_CLAUDE,
            )?;
        }
    }

    // The skill, for every agent, and for Claude Code where it looks.
    let glob = content_glob(index, &prefix);
    let agents_skill = under(&root, ".agents/skills").join(skill::NAME);
    for (file, text) in skill::files(None, &skill_runs, skill_config.as_deref()) {
        let path = under(&agents_skill, file);
        add(path, &|_| Ok(text.clone()), BLOCK, READ_SKILLS)?;
    }
    if claude {
        let paths = claude_skill_paths(&claude_skill, &glob)?;
        for (file, text) in skill::files(Some(&paths), &skill_runs, skill_config.as_deref()) {
            let path = under(&claude_skill, file);
            add(path, &|_| Ok(text.clone()), BLOCK, READ_CLAUDE)?;
        }
    }

    // Rules loaded only for the project's pages.
    let generated = commands(&format!(
        "<!-- Generated by `ascribe agents sync` from {model_file}. Edit that, not this file. -->"
    ));
    let scoped = commands(&rules(index, files_of, &prefix, &prefix));
    if with_rules {
        let text = format!("---\npaths:\n  - \"{glob}\"\n---\n\n{generated}\n\n{scoped}");
        add(claude_rules, &|_| Ok(text.clone()), BLOCK, READ_CLAUDE)?;
    }
    if with_copilot {
        let text = format!("---\napplyTo: \"{glob}\"\n---\n\n{generated}\n\n{scoped}");
        add(copilot, &|_| Ok(text.clone()), BLOCK, READ_COPILOT)?;
    }

    // Outside FileSystem: whether the repository has a `.codex` folder,
    // which isn't the project's.
    let wants_hooks = Hooks {
        asked: options.hooks,
        claude,
        codex: asked.contains(&Target::Codex) || under(&root, ".codex").is_dir(),
        copilot: with_copilot,
    };
    let given = options.command.as_deref();
    plan_hooks(&wants_hooks, &here, &root, given, &mut files, &mut notes)?;

    // Copilot's cloud agent.
    let mcp_settings = plan_cloud(options, &here, &root, &mut files)?;

    Ok(Plan {
        files,
        notes,
        unread: if claude {
            Vec::new()
        } else {
            vec![CLAUDE_SKILL_UNREAD]
        },
        mcp_settings,
        root,
    })
}

/// A file's text after `sync`, from its text now; an error when its
/// markers are damaged.
type New<'a> = dyn Fn(Option<&str>) -> Result<String, Damage> + 'a;

/// The file at `path`, with its text now and after `sync`: an error when
/// it would be one of the pages under `content`, can't be read, or has
/// damaged `name` markers.
fn planned(
    content: &Path,
    path: PathBuf,
    new: &New<'_>,
    name: &str,
    readers: &'static [Agent],
) -> Result<Planned, SyncError> {
    if let Some(rel) = relative_path(content, &normalize(&path))
        && is_source_path(&rel)
    {
        return Err(SyncError::InContent {
            path: path.display().to_string(),
        });
    }
    let old = read(&path)?;
    let new = new(old.as_deref()).map_err(|damage| SyncError::Damaged {
        path: path.display().to_string(),
        name: name.to_owned(),
        damage,
    })?;
    Ok(Planned {
        path,
        old,
        new,
        readers,
    })
}

/// The cloud agent's setup steps and custom agent, when they're asked for
/// or written already; and the MCP servers' JSON for the repository's
/// settings, when the setup is asked for without the custom agent. The
/// settings and the custom agent carry a server for each block of the
/// steps, this project's and every other's.
fn plan_cloud(
    options: &Options,
    here: &Path,
    root: &Path,
    files: &mut Vec<Planned>,
) -> Result<Option<String>, SyncError> {
    // Projects that install the same way share a block: a global install
    // is the same for every project, and a pinned one for each
    // `package.json`.
    let install = install_of(here, root)?;
    let name = match &install {
        copilot::Install::Global { .. } => copilot::BLOCK.to_owned(),
        copilot::Install::Pinned { package, .. } if package.is_empty() => {
            format!("{}:pinned", copilot::BLOCK)
        }
        copilot::Install::Pinned { package, .. } => format!("{}:{package}", copilot::BLOCK),
    };
    let markers = Markers {
        name: &name,
        note: "generated by `ascribe agents sync --cloud`; run it again rather than editing this block",
        comment: Comment::Hash,
    };
    let setup = under(root, copilot::SETUP_STEPS);
    let old = read(&setup)?;
    let has_block = old
        .as_deref()
        .is_some_and(|text| text.lines().any(|line| markers.is_start(line.trim())));
    let steps = if options.cloud || has_block {
        let new = copilot::setup_steps(old.as_deref(), &markers, &install).map_err(|why| {
            SyncError::Settings {
                path: setup.display().to_string(),
                why,
            }
        })?;
        files.push(Planned {
            path: setup,
            old,
            new: new.clone(),
            readers: READ_COPILOT_CLOUD,
        });
        Some(new)
    } else {
        old
    };
    // A server for each block of the steps, other projects' too, so the
    // settings and the custom agent carry every one.
    let mut servers = steps.as_deref().map(copilot::servers).unwrap_or_default();
    if servers.is_empty() {
        servers.push(copilot::server_of(&install));
    }
    let agent = under(root, copilot::AGENT);
    let old = read(&agent)?;
    if (options.cloud && options.agent) || old.is_some() {
        files.push(Planned {
            path: agent,
            old,
            new: copilot::agent_md(&servers),
            readers: READ_COPILOT_CLOUD,
        });
        return Ok(None);
    }
    Ok(options.cloud.then(|| copilot::mcp_settings(&servers)))
}

/// How the cloud agent's setup installs `ascribe`: the dependencies of the
/// nearest `package.json` from the project's folder up to the repository's
/// root that pins `@ascribed/cli`, with the lockfile from there up; else
/// this version, globally.
fn install_of(here: &Path, root: &Path) -> Result<copilot::Install, SyncError> {
    let from_root = |dir: &Path| {
        relative_path(root, dir)
            .filter(|rel| !rel.is_root())
            .map(|rel| rel.as_str().to_owned())
            .unwrap_or_default()
    };
    for dir in here.ancestors() {
        if let Some(text) = read(&dir.join("package.json"))?
            && let Some(spec) = copilot::pinned_spec(&text)
        {
            let package = from_root(dir);
            let bin = if package.is_empty() {
                "node_modules/.bin".to_owned()
            } else {
                format!("{package}/node_modules/.bin")
            };
            let pinned = |install: &Path, command| copilot::Install::Pinned {
                package: package.clone(),
                install: from_root(install),
                command,
                bin: bin.clone(),
            };
            for lock_dir in dir.ancestors() {
                for (file, command) in copilot::LOCKFILES {
                    // Outside FileSystem: the repository's lockfiles, which
                    // aren't the project's.
                    if lock_dir.join(file).is_file() {
                        return Ok(pinned(lock_dir, command));
                    }
                }
                if let Some(lock) = read(&lock_dir.join(copilot::YARN_LOCK))? {
                    let yarnrc = read(&lock_dir.join(copilot::YARNRC))?;
                    return match copilot::yarn_command(&lock, yarnrc.as_deref()) {
                        Some(command) => Ok(pinned(lock_dir, command)),
                        None => plug_n_play(&spec, root),
                    };
                }
                if lock_dir == root {
                    break;
                }
            }
            return Ok(pinned(dir, copilot::NO_LOCKFILE));
        }
        if dir == root {
            break;
        }
    }
    Ok(copilot::Install::Global {
        version: env!("CARGO_PKG_VERSION").to_owned(),
    })
}

/// How a project installs the `@ascribed/cli` it pins at `spec` when Yarn
/// uses Plug'n'Play, which writes no `node_modules/.bin`: that version,
/// globally, or an error when npm can't install it.
fn plug_n_play(spec: &str, root: &Path) -> Result<copilot::Install, SyncError> {
    if copilot::installs_globally(spec) {
        return Ok(copilot::Install::Global {
            version: spec.to_owned(),
        });
    }
    Err(SyncError::Settings {
        path: under(root, copilot::SETUP_STEPS).display().to_string(),
        why: format!(
            "Yarn's Plug'n'Play leaves no node_modules/.bin, so the steps would install \
             @ascribed/cli globally, and npm can't install `{spec}`; pin a version in \
             package.json, or set `nodeLinker: node-modules` in {}",
            copilot::YARNRC
        ),
    })
}

/// Which agents' hooks `sync` may write, and whether they were asked for.
struct Hooks {
    asked: bool,
    claude: bool,
    codex: bool,
    copilot: bool,
}

/// The hook entries and the MCP server, for each agent of `hooks` that has
/// them already, or all of them when they're asked for.
/// A settings file merged from its text now: `None` when it's left out.
type Merge<'a> = dyn Fn(Option<&str>) -> Result<Option<String>, String> + 'a;

fn plan_hooks(
    hooks: &Hooks,
    here: &Path,
    root: &Path,
    command: Option<&str>,
    files: &mut Vec<Planned>,
    notes: &mut Vec<String>,
) -> Result<(), SyncError> {
    let claude_settings = under(root, ".claude/settings.json");
    let mcp = root.join(".mcp.json");
    let codex_hooks = under(root, ".codex/hooks.json");
    let copilot_hooks = under(root, ".github/hooks/ascribe.json");
    let ascribe = ascribe_command(here, root).with_command(command);
    let has_hook = |path: &Path| -> Result<bool, SyncError> {
        Ok(read(path)?.is_some_and(|text| text.contains("agents hook")))
    };
    // A file whose meaning wouldn't change is left as it is, however it's
    // laid out.
    let mut merge = |path: PathBuf, merged: &Merge<'_>, readers: &'static [Agent]| {
        let old = read(&path)?;
        let new = merged(old.as_deref()).map_err(|why| SyncError::Settings {
            path: path.display().to_string(),
            why,
        })?;
        if let Some(mut new) = new {
            if let Some(old) = &old
                && settings::same_meaning(old, &new)
            {
                new.clone_from(old);
            }
            files.push(Planned {
                path,
                old,
                new,
                readers,
            });
        }
        Ok::<(), SyncError>(())
    };
    if hooks.claude && (hooks.asked || has_hook(&claude_settings)?) {
        merge(
            claude_settings,
            &|old| settings::claude_settings(old, &ascribe).map(Some),
            READ_CLAUDE,
        )?;
        merge(
            mcp,
            &|old| settings::mcp_json(old, &ascribe, hooks.asked),
            READ_CLAUDE,
        )?;
    }
    if hooks.codex && (hooks.asked || has_hook(&codex_hooks)?) {
        merge(
            codex_hooks,
            &|old| settings::codex_hooks(old, &ascribe).map(Some),
            READ_CODEX,
        )?;
    }
    if (hooks.copilot && hooks.asked) || read(&copilot_hooks)?.is_some() {
        merge(
            copilot_hooks,
            &|_| Ok(Some(settings::copilot_hooks(&ascribe))),
            READ_COPILOT,
        )?;
    }
    if hooks.asked && !hooks.claude && !hooks.codex && !hooks.copilot {
        notes.push(
            "--with-hook writes hooks for `claude`, `codex`, and `copilot`; name one with --target"
                .to_owned(),
        );
    }
    Ok(())
}

/// How the hooks run `ascribe`: the binary of `@ascribed/cli` when the
/// project pins it, in a `node_modules` from the project's folder up to the
/// repository's root, else `ascribe` on the path.
fn ascribe_command(here: &Path, root: &Path) -> settings::Ascribe {
    // Outside FileSystem: the folders around the project, for an installed
    // `@ascribed/cli`.
    for dir in here.ancestors() {
        if dir
            .join("node_modules")
            .join("@ascribed")
            .join("cli")
            .is_dir()
            && let Some(rel) = relative_path(root, dir)
        {
            let bin = rel
                .join("node_modules/.bin/ascribe")
                .map(|p| p.as_str().to_owned())
                .unwrap_or_else(|_| "node_modules/.bin/ascribe".to_owned());
            return settings::Ascribe::pinned(&bin);
        }
        if dir == root {
            break;
        }
    }
    settings::Ascribe::on_path()
}

/// The command the project in `here` runs Ascribe with, `given` or worked
/// out, and the one the skill names. The skill at `root` is every
/// project's in the repository, so it names this project's command when
/// each of them runs Ascribe with it, else `ascribe`, and says how a
/// project that pins it runs it.
fn commands_of(given: Option<&str>, here: &Path, root: &Path) -> (String, String) {
    if let Some(given) = given {
        return (given.to_owned(), given.to_owned());
    }
    let runs = command::detect(here, root);
    if runs == command::PLAIN {
        return (runs.clone(), runs);
    }
    // Outside FileSystem: the repository's other projects.
    let mut projects = ascribe_workspace::find_below(root, usize::MAX);
    if root.join(ascribe_check::MODEL_FILE).is_file() {
        projects.push(root.to_owned());
    }
    let shared = projects
        .iter()
        .all(|dir| command::detect(dir, root) == runs);
    let skill = if shared {
        runs.clone()
    } else {
        command::PLAIN.to_owned()
    };
    (runs, skill)
}

/// The folder the skill's commands that work on the whole project are
/// given with `--config`, so that they work from the repository's root:
/// the project's, `prefix`, when it isn't at the root. The skill at `root`
/// is every project's in the repository, so only when the project in
/// `here` is the only one.
fn config_of(prefix: &RelPath, here: &Path, root: &Path) -> Option<String> {
    if prefix.is_root() {
        return None;
    }
    // Outside FileSystem: the repository's other projects.
    let projects = ascribe_workspace::find_below(root, 2);
    let only = projects == [here] && !root.join(ascribe_check::MODEL_FILE).is_file();
    only.then(|| prefix.as_str().to_owned())
}

/// A `CLAUDE.md`, and the line that imports `AGENTS.md` from it.
type Import = (PathBuf, &'static str);

/// The `CLAUDE.md` in `dir` to import `AGENTS.md` into, and the import:
/// `CLAUDE.md`, or `.claude/CLAUDE.md`, or with `create` a new `CLAUDE.md`;
/// and what `sync` says of it, or of a `CLAUDE.local.md` when there's none.
fn claude_md(
    dir: &Path,
    create: bool,
    markers: &Markers<'_>,
) -> Result<(Option<Import>, Option<String>), SyncError> {
    let beside = dir.join("CLAUDE.md");
    let inside = under(dir, ".claude/CLAUDE.md");
    let (path, text, import) = if let Some(text) = read(&beside)? {
        (beside, text, "@AGENTS.md\n")
    } else if let Some(text) = read(&inside)? {
        (inside, text, "@../AGENTS.md\n")
    } else if create {
        return Ok((Some((beside, "@AGENTS.md\n")), None));
    } else {
        let local = dir.join("CLAUDE.local.md");
        let note = read(&local)?
            .filter(|text| !text.contains("@AGENTS.md"))
            .map(|_| {
                format!(
                    "{} exists, so Claude Code won't load AGENTS.md there by itself; add \
                     `@AGENTS.md` to it, or run with `--target claude`",
                    local.display()
                )
            });
        return Ok((None, note));
    };
    // A file with damaged markers is said nothing of: placing the block
    // reports them.
    let note = matches!(own_import(markers, &text, import), Ok(Some(_))).then(|| {
        format!(
            "{} already loads AGENTS.md, so it gets no import",
            path.display()
        )
    });
    Ok((Some((path, import)), note))
}

/// A `CLAUDE.md`'s text with `import` once: in the block, or only outside
/// it when it's there already, so the block is left out.
fn import_once(markers: &Markers<'_>, old: Option<&str>, import: &str) -> Result<String, Damage> {
    if let Some(text) = old
        && let Some(outside) = own_import(markers, text, import)?
    {
        return Ok(outside);
    }
    markers.place(old, import)
}

/// `text` without the block when a line outside it, trimmed, is `import`:
/// one in a code span or in other words isn't an import.
fn own_import(markers: &Markers<'_>, text: &str, import: &str) -> Result<Option<String>, Damage> {
    let outside = markers.remove(text)?.unwrap_or_else(|| text.to_owned());
    let imports = outside.lines().any(|line| line.trim() == import.trim_end());
    Ok(imports.then_some(outside))
}

/// A path in the project's folder, as written from the repository's root.
fn in_dir(prefix: &RelPath, name: &str) -> String {
    prefix
        .join(name)
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|_| name.to_owned())
}

/// The `paths` the skill in `dir`, for Claude Code, lists: those it lists
/// now, and `glob`.
fn claude_skill_paths(dir: &Path, glob: &str) -> Result<Vec<String>, SyncError> {
    let mut paths = read(&dir.join("SKILL.md"))?
        .map(|text| skill_paths(&text))
        .unwrap_or_default();
    paths.push(glob.to_owned());
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// The `paths` a skill written for Claude Code lists: the content globs of
/// every project that wrote it.
fn skill_paths(text: &str) -> Vec<String> {
    text.lines()
        .skip_while(|line| *line != "paths:")
        .skip(1)
        .map_while(|line| line.strip_prefix("  - \""))
        .filter_map(|rest| rest.strip_suffix('"'))
        .map(str::to_owned)
        .collect()
}

/// `dir` joined with a `/`-separated relative path.
fn under(dir: &Path, rel: &str) -> PathBuf {
    rel.split('/')
        .fold(dir.to_path_buf(), |path, segment| path.join(segment))
}

/// `path` made absolute from the current directory, and normalized.
fn absolute(path: &Path) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    normalize(&cwd.join(path))
}

/// A file's text, or `None` when there's no file.
pub(crate) fn read(path: &Path) -> Result<Option<String>, SyncError> {
    // Outside FileSystem: the instruction files agents read, which aren't
    // the project's pages.
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(SyncError::Read {
            path: path.display().to_string(),
            source,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_skill_paths_are_read_back() {
        let text = skill::skill_md(
            Some(&["a/**/*.md".to_owned(), "b/**/*.md".to_owned()]),
            command::PLAIN,
            None,
        );
        assert_eq!(skill_paths(&text), ["a/**/*.md", "b/**/*.md"]);
        assert!(skill_paths(&skill::skill_md(None, command::PLAIN, None)).is_empty());
    }
}
