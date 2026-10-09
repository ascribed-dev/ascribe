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

use std::io;
use std::path::{Path, PathBuf};

use ascribe_check::Project;
use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Coded, RelPath};
use ascribe_diff::{DiffError, Repository};
use ascribe_query::rules::{content_glob, pointer, rules};
use ascribe_resolve::is_source_path;

use super::markers::{Damage, Markers};
use super::skill;

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
        "--target copilot needs a git repository: Copilot reads its instructions from \
         the repository's .github/"
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
            SyncError::Write { .. } => "instructions_unwritable",
        }
    }
}

/// One file `sync` writes.
#[derive(Debug)]
pub struct Planned {
    /// Where.
    pub path: PathBuf,
    /// Its text now; `None` when there's no file.
    pub old: Option<String>,
    /// Its text after `sync`.
    pub new: String,
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
}

impl Plan {
    /// Writes each file that changes.
    pub fn write(&self) -> Result<(), SyncError> {
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

/// Works out the files for `targets` (with `agents-md` and `skills` always,
/// and each other target whose files exist already), without writing.
pub fn plan(project: &Project, asked: &[Target]) -> Result<Plan, SyncError> {
    let mut notes = Vec::new();
    let here = absolute(project.root());
    let prefix = match Repository::discover(&here) {
        Ok(repository) => repository.project_dir(),
        Err(DiffError::NotARepository { .. }) => {
            if asked.contains(&Target::Copilot) {
                return Err(SyncError::NoRepository);
            }
            notes.push(format!(
                "{} isn't in a git repository, so its folder is treated as the repository's root",
                here.display()
            ));
            RelPath::root()
        }
        Err(e) => return Err(SyncError::Git(e)),
    };
    // The root above the project's folder as written, not as `git` prints
    // it, so a symbolic link on the way doesn't change the paths shown.
    let mut root = here.clone();
    for _ in prefix.segments() {
        root.pop();
    }
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
    let with_copilot = wants(Target::Copilot, std::slice::from_ref(&copilot))?;

    let content = absolute(&project.root().join(project.content_root().as_str()));
    let mut files = Vec::new();
    let mut add =
        |path: PathBuf, new: &dyn Fn(Option<&str>) -> Result<String, Damage>, name: &str| {
            if let Some(rel) = relative_path(&content, &normalize(&path))
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
            files.push(Planned { path, old, new });
            Ok(())
        };

    // AGENTS.md: the rules beside ascribe.toml, and a pointer at the root.
    let note = "generated by `ascribe agents sync`; edit ascribe.toml, not this block";
    let body = rules(&index, &RelPath::root());
    let markers = Markers { name: BLOCK, note };
    add(
        here.join("AGENTS.md"),
        &|old| markers.place(old, &body),
        BLOCK,
    )?;
    let mut agents_dirs = vec![here.clone()];
    if !prefix.is_root() {
        let name = format!("{BLOCK}:{}", prefix.as_str());
        let note = format!("generated by `ascribe agents sync`; edit {model_file}, not this block");
        let body = pointer(&index, &prefix);
        let markers = Markers {
            name: &name,
            note: &note,
        };
        add(
            root.join("AGENTS.md"),
            &|old| markers.place(old, &body),
            &name,
        )?;
        agents_dirs.push(root.clone());
    }

    // CLAUDE.md: an import of AGENTS.md, in each one there is.
    for dir in &agents_dirs {
        let note = "generated by `ascribe agents sync`; it loads AGENTS.md";
        let markers = Markers { name: BLOCK, note };
        let create = asked.contains(&Target::Claude) && dir == &here;
        let (file, import) = match claude_md(dir, create)? {
            Some((file, import)) => (Some(file), import),
            None => (None, ""),
        };
        match file {
            Some(file) => add(file, &|old| markers.place(old, import), BLOCK)?,
            None => {
                if let Some(local) = read(&dir.join("CLAUDE.local.md"))?
                    && !local.contains("@AGENTS.md")
                {
                    notes.push(format!(
                        "{} exists, so Claude Code won't load AGENTS.md there by itself; add \
                         `@AGENTS.md` to it, or run with `--target claude`",
                        dir.join("CLAUDE.local.md").display()
                    ));
                }
            }
        }
    }

    // The skill.
    let glob = content_glob(&index, &prefix);
    let mut skill_dirs: Vec<(PathBuf, Option<Vec<String>>)> =
        vec![(under(&root, ".agents/skills").join(skill::NAME), None)];
    if claude {
        let mut paths = read(&claude_skill.join("SKILL.md"))?
            .map(|text| skill_paths(&text))
            .unwrap_or_default();
        paths.push(glob.clone());
        paths.sort();
        paths.dedup();
        skill_dirs.push((claude_skill.clone(), Some(paths)));
    }
    for (dir, paths) in &skill_dirs {
        for (file, text) in skill::files(paths.as_deref()) {
            add(under(dir, file), &|_| Ok(text.clone()), BLOCK)?;
        }
    }

    // Rules loaded only for the project's pages.
    let generated = format!(
        "<!-- Generated by `ascribe agents sync` from {model_file}. Edit that, not this file. -->"
    );
    let scoped = rules(&index, &prefix);
    if with_rules {
        let text = format!("---\npaths:\n  - \"{glob}\"\n---\n\n{generated}\n\n{scoped}");
        add(claude_rules, &|_| Ok(text.clone()), BLOCK)?;
    }
    if with_copilot {
        let text = format!("---\napplyTo: \"{glob}\"\n---\n\n{generated}\n\n{scoped}");
        add(copilot, &|_| Ok(text.clone()), BLOCK)?;
    }

    Ok(Plan { files, notes })
}

/// The `CLAUDE.md` in `dir` to import `AGENTS.md` into, and the import:
/// `CLAUDE.md`, or `.claude/CLAUDE.md`, or with `create` a new `CLAUDE.md`.
fn claude_md(dir: &Path, create: bool) -> Result<Option<(PathBuf, &'static str)>, SyncError> {
    let beside = dir.join("CLAUDE.md");
    let inside = under(dir, ".claude/CLAUDE.md");
    Ok(if read(&beside)?.is_some() {
        Some((beside, "@AGENTS.md\n"))
    } else if read(&inside)?.is_some() {
        Some((inside, "@../AGENTS.md\n"))
    } else if create {
        Some((beside, "@AGENTS.md\n"))
    } else {
        None
    })
}

/// A path in the project's folder, as written from the repository's root.
fn in_dir(prefix: &RelPath, name: &str) -> String {
    prefix
        .join(name)
        .map(|p| p.as_str().to_owned())
        .unwrap_or_else(|_| name.to_owned())
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
fn read(path: &Path) -> Result<Option<String>, SyncError> {
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
        let text = skill::skill_md(Some(&["a/**/*.md".to_owned(), "b/**/*.md".to_owned()]));
        assert_eq!(skill_paths(&text), ["a/**/*.md", "b/**/*.md"]);
        assert!(skill_paths(&skill::skill_md(None)).is_empty());
    }
}
