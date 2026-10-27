//! `ascribe agents`: the files agents read on their own, and the prompts
//! for them. `sync` writes the project's rules into the instruction files
//! and the skill into the skills folder; `rules` prints the rules, `skill`
//! the skill, and `prompt` a named prompt. `index` is for the agents of the
//! product the docs are about: a compact index of the docs, for its
//! `AGENTS.md`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ascribe_check::{MODEL_FILE, Project};
use ascribe_workspace::LocateError;
use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use serde::Serialize;

use crate::agents::sync::{self, Agent, Plan};
use crate::agents::{checker, command, copilot, hook, index as index_file, prompts, skill};
use crate::answer::{self, Format, FromDisk};
use crate::guard;
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// Arguments of `ascribe agents`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

/// The subcommands of `ascribe agents`.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write the project's rules into the files agents read on their own,
    /// and the Ascribe skill, from the content model.
    #[command(after_help = docs_link!("cli-agents-sync"))]
    Sync(SyncArgs),
    /// Print the project's rules: the block `sync` writes into the
    /// AGENTS.md beside ascribe.toml.
    #[command(after_help = docs_link!("cli-agents-rules"))]
    Rules(RulesArgs),
    /// Write a compact index of the docs, one line a page, for the
    /// AGENTS.md of the product they're about.
    #[command(after_help = docs_link!("cli-agents-index"))]
    Index(IndexArgs),
    /// Print a file of the Ascribe skill: its SKILL.md, or its directive
    /// reference.
    #[command(after_help = docs_link!("cli-agents-skill"))]
    Skill(SkillArgs),
    /// Print a named prompt for an agent: `new-page`, `fix`, or `review`.
    #[command(after_help = docs_link!("cli-agents-prompt"))]
    Prompt(PromptArgs),
    /// Check an agent's edits, as its hook: after it writes a file, or
    /// before it finishes. Reads the hook's input on standard input.
    #[command(after_help = docs_link!("cli-agents-hook"))]
    Hook(HookArgs),
    /// Keep a project loaded for its hooks: run by the first hook in a
    /// project, in the background.
    #[command(hide = true)]
    HookServer {
        /// The program the hook ran, which the server stops when it
        /// changes.
        #[arg(long, value_name = "PATH")]
        binary: Option<std::path::PathBuf>,
    },
}

/// Arguments of `ascribe agents hook`.
#[derive(Debug, ClapArgs)]
pub struct HookArgs {
    /// The agent whose hook runs it: how its input is read and its answer
    /// written.
    #[arg(value_enum, value_name = "HARNESS")]
    pub harness: hook::Harness,

    /// When the hook runs: after the agent writes a file (`edit`), or when
    /// it's about to finish (`stop`).
    #[arg(long, value_enum, default_value_t = hook::Event::Edit, value_name = "EVENT")]
    pub event: hook::Event,

    /// The command the project runs Ascribe with, for the commands the
    /// hook's messages name.
    ///
    /// By default, each project's, as `ascribe agents sync` works it out.
    #[arg(long, value_name = "COMMAND", value_parser = project_command)]
    pub command: Option<String>,
}

/// A `--command`: one a hook's command line can carry in double quotes.
fn project_command(given: &str) -> Result<String, String> {
    if command::is_safe(given) {
        Ok(given.to_owned())
    } else {
        Err("a command has only letters, digits, spaces, and `._/@:+=-`".to_owned())
    }
}

/// Arguments of `ascribe agents rules`.
#[derive(Debug, ClapArgs)]
pub struct RulesArgs {
    /// A file or folder in the project, to find its `ascribe.toml` from.
    ///
    /// By default, the current directory.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

/// Arguments of `ascribe agents index`.
#[derive(Debug, ClapArgs)]
pub struct IndexArgs {
    /// The build whose pages to index.
    ///
    /// Needed when `ascribe.toml` has more than one build.
    #[arg(long, value_name = "NAME")]
    pub build: Option<String>,

    /// Write the index into this file, between its markers, instead of
    /// printing it.
    ///
    /// The rest of the file is left as it is, and a file without the
    /// markers gets the block at its end.
    #[arg(long, value_name = "FILE")]
    pub into: Option<PathBuf>,

    /// Also copy each page's Markdown version into this folder, and point
    /// the index's addresses into it: for a package that ships its docs.
    ///
    /// The addresses are from the folder of the file the index is written
    /// into, or from the current directory when it's printed.
    #[arg(long, value_name = "DIR")]
    pub pages: Option<PathBuf>,

    /// `--dry-run` and `--force`, for what `--into` and `--pages` write.
    #[command(flatten)]
    pub guard: guard::Options,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Arguments of `ascribe agents skill`.
#[derive(Debug, ClapArgs)]
pub struct SkillArgs {
    /// The file to print, relative to the skill's folder.
    #[arg(value_name = "FILE", default_value = "SKILL.md", value_parser = ["SKILL.md", skill::DIRECTIVES])]
    pub file: String,
}

/// Arguments of `ascribe agents prompt`.
#[derive(Debug, ClapArgs)]
pub struct PromptArgs {
    /// The prompt's name.
    #[arg(value_name = "NAME", required_unless_present = "list")]
    pub name: Option<String>,

    /// An argument of the prompt, as `key=value`.
    ///
    /// Repeat it for several. `--list` shows each prompt's arguments.
    #[arg(long = "arg", value_name = "KEY=VALUE", value_parser = key_value)]
    pub arguments: Vec<(String, String)>,

    /// List the prompts, with their arguments, instead.
    #[arg(long, conflicts_with = "name")]
    pub list: bool,

    /// How to show the prompt, or the list.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// `key=value`, split at the first `=`.
fn key_value(given: &str) -> Result<(String, String), String> {
    given
        .split_once('=')
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .ok_or_else(|| format!("`{given}` isn't `key=value`"))
}

/// Arguments of `ascribe agents sync`.
#[derive(Debug, ClapArgs)]
pub struct SyncArgs {
    /// Write nothing, print how each file out of date would change as a
    /// unified diff, and exit with 1 when one is: for CI.
    #[arg(long)]
    pub check: bool,

    /// `--dry-run` and `--force`.
    #[command(flatten)]
    pub guard: guard::Options,

    /// Also write this target's files.
    ///
    /// Repeat it for several. `agents-md` and `skills` are always written,
    /// and each other target whose files already exist is kept up to date.
    #[arg(long, value_enum, value_name = "TARGET")]
    pub target: Vec<Target>,

    /// Also write the hooks that check each edit and the whole project
    /// before an agent finishes, for `claude`, `codex`, and `copilot`; and
    /// for `claude` the MCP server. Hooks written before are kept up to date
    /// without it.
    #[arg(long)]
    pub with_hook: bool,

    /// Also write what Copilot's cloud agent needs on GitHub: steps in
    /// .github/workflows/copilot-setup-steps.yml that install Ascribe, and
    /// print the MCP server's JSON for the repository's settings. Implies
    /// `--target copilot`. Steps written before are kept up to date without
    /// it.
    #[arg(long)]
    pub cloud: bool,

    /// With --cloud, write the MCP server into a custom agent,
    /// .github/agents/ascribe-docs.md, instead of printing its JSON: for a
    /// repository whose settings you can't change.
    #[arg(long, requires = "cloud")]
    pub agent: bool,

    /// The command the project runs Ascribe with, which the files name in
    /// the commands they give: `npx ascribe`, for example.
    ///
    /// By default, `npx ascribe`, `pnpm exec ascribe`, or `yarn ascribe`
    /// when a package.json depends on @ascribed/cli or @ascribed/astro,
    /// by its lockfile; else `ascribe`. Give it again each time: with
    /// --check, a file written with another command is out of date.
    #[arg(long, value_name = "COMMAND", value_parser = project_command)]
    pub command: Option<String>,

    /// How to show what was written, or with --check what's out of date.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// The version of `sync --format json`'s schema.
const SYNC_SCHEMA_VERSION: u32 = 1;

/// The version of `prompt --format json`'s schema, and of `prompt --list
/// --format json`'s.
const PROMPT_SCHEMA_VERSION: u32 = 1;

/// What `ascribe agents sync --format json` writes: each file it writes,
/// and what it has to say.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct SyncJson<'a> {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Whether it ran with `--check`, which writes nothing.
    check: bool,
    /// Whether it wrote the files: false with `--check` or `--dry-run`.
    written: bool,
    /// Each file, in the order it was first planned.
    files: Vec<SyncFile<'a>>,
    /// The notes for the person running it, which it also writes to
    /// standard error.
    notes: Vec<&'a str>,
    /// What an agent doesn't read of what's written, and the target that
    /// adds it, a sentence each.
    unread: Vec<&'a str>,
    /// With `--cloud` and no `--agent`, the MCP server's JSON to paste into
    /// the repository's settings for Copilot's cloud agent; else `null`.
    mcp_settings: Option<serde_json::Value>,
}

/// A file in `ascribe agents sync --format json`.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct SyncFile<'a> {
    /// Its path, from the current directory, `/`-separated.
    path: String,
    /// Whether it changed: with `check`, whether it's out of date.
    changed: bool,
    /// The agents that read it.
    read_by: &'a [Agent],
}

/// What `ascribe agents prompt <NAME> --format json` writes: the prompt.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct PromptJson<'a> {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The prompt's name.
    name: &'a str,
    /// The prompt, as `--format text` prints it.
    text: &'a str,
}

/// What `ascribe agents prompt --list --format json` writes: every prompt,
/// with its arguments.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct PromptListJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// The prompts, in order.
    prompts: Vec<PromptListed>,
}

/// A prompt in `ascribe agents prompt --list --format json`.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct PromptListed {
    /// Its name, which `ascribe agents prompt` takes.
    name: &'static str,
    /// A short name for people.
    title: &'static str,
    /// What it asks an agent to do.
    description: &'static str,
    /// Its arguments, which `--arg key=value` gives.
    arguments: Vec<PromptArgument>,
}

/// An argument of a prompt.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct PromptArgument {
    /// Its name, the `key` of `--arg key=value`.
    name: &'static str,
    /// What it is.
    description: &'static str,
    /// Whether the prompt needs it.
    required: bool,
}

/// What `sync` can write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Target {
    /// AGENTS.md beside ascribe.toml, and a short block in the repository
    /// root's.
    AgentsMd,
    /// The skill, in the repository root's .agents/skills/ascribe/.
    Skills,
    /// An import of AGENTS.md in CLAUDE.md, creating one beside
    /// ascribe.toml, and the skill in .claude/skills/ascribe/; with
    /// --with-hook, the hooks in .claude/settings.json and the MCP server in
    /// .mcp.json.
    Claude,
    /// The rules in .claude/rules/, loaded only for the project's pages.
    ClaudeRules,
    /// The rules in .github/instructions/, loaded only for the project's
    /// pages; with --with-hook, the hooks in .github/hooks/ascribe.json;
    /// with --cloud, the cloud agent's setup.
    Copilot,
    /// Nothing of its own, since Codex reads AGENTS.md and the skill; with
    /// --with-hook, the hooks in .codex/hooks.json.
    Codex,
}

impl Target {
    fn sync(self) -> sync::Target {
        match self {
            Target::AgentsMd => sync::Target::AgentsMd,
            Target::Skills => sync::Target::Skills,
            Target::Claude => sync::Target::Claude,
            Target::ClaudeRules => sync::Target::ClaudeRules,
            Target::Copilot => sync::Target::Copilot,
            Target::Codex => sync::Target::Codex,
        }
    }
}

/// Runs the command. Exit codes: 0 when the files are written, or with
/// `--check` up to date; 1 with `--check` when one is out of date; 2 when
/// they can't be written.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = match args.command {
        Command::Sync(args) => sync(global, &args, &mut out, &mut err),
        Command::Rules(args) => rules(global, &args, &mut out, &mut err),
        Command::Index(args) => index(global, &args, &mut out, &mut err),
        Command::Skill(args) => {
            let text = if args.file == skill::DIRECTIVES {
                skill::directives_md()
            } else {
                skill::skill_md(None, command::PLAIN, None)
            };
            let result = out.write_all(text.as_bytes());
            answer::written(result, &mut err).unwrap_or(exit::OK)
        }
        Command::Prompt(args) => prompt(&args, &mut out, &mut err),
        Command::Hook(args) => {
            let mut input = String::new();
            let read = io::Read::read_to_string(&mut io::stdin(), &mut input).map(|_| input);
            let answer = hook::run(
                args.harness,
                args.event,
                read,
                args.command.as_deref(),
                hook::Limits::default(),
            );
            let _ = out.write_all(answer.stdout.as_bytes());
            let _ = err.write_all(answer.stderr.as_bytes());
            answer.code
        }
        Command::HookServer { binary } => match global.config.as_deref() {
            Some(config) => match checker::serve(config, checker::IDLE, binary.as_deref()) {
                Ok(()) => exit::OK,
                Err(e) => {
                    let _ = writeln!(err, "error: the check server stopped: {e}");
                    exit::FAILURE
                }
            },
            None => {
                let _ = writeln!(err, "error: the check server needs --config");
                exit::FAILURE
            }
        },
    };
    let _ = out.flush();
    exit::code(code)
}

fn sync(global: &Global, args: &SyncArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let configs = match crate::context::locate(global) {
        Ok(config) => vec![config],
        // Above several projects and in none: each project's files.
        Err(LocateError::NotFound { dir, below }) if !below.is_empty() => {
            let below = ascribe_workspace::find_below(&dir, usize::MAX);
            let names: Vec<String> = below.iter().map(|d| shown(d)).collect();
            let _ = writeln!(
                err,
                "note: there's no {MODEL_FILE} here, so this syncs each project below: {}",
                names.join(", ")
            );
            below.iter().map(|d| d.join(MODEL_FILE)).collect()
        }
        Err(e) => return answer::report_failure(err, &Failure::Config(e)),
    };
    let targets: Vec<sync::Target> = args.target.iter().map(|t| t.sync()).collect();
    let options = sync::Options {
        hooks: args.with_hook,
        cloud: args.cloud,
        agent: args.agent,
        command: args.command.clone(),
    };
    let mut projects = Vec::new();
    for config in &configs {
        match Project::load(config) {
            Ok(project) => projects.push(project.with_today(crate::clock::today())),
            Err(e) => return answer::report_failure(err, &Failure::Load(e)),
        }
    }
    // Every project's plan first, so a damaged file stops the command
    // before any file changes.
    let mut plans = Vec::new();
    for project in &projects {
        match sync::plan(project, &targets, &options) {
            Ok(plan) => plans.push(plan),
            Err(e) => return exit::fail(err, &e),
        }
    }
    let mut notes: Vec<String> = Vec::new();
    for note in plans.iter().flat_map(|plan| &plan.notes) {
        if !notes.contains(note) {
            let _ = writeln!(err, "note: {note}");
            notes.push(note.clone());
        }
    }
    if args.check {
        return check(&plans, &notes, args.format, out, err);
    }
    let changed: Vec<&Path> = plans
        .iter()
        .flat_map(|plan| &plan.files)
        .filter(|file| file.changes())
        .map(|file| file.path.as_path())
        .collect();
    if let Err(e) = args.guard.allow(&changed) {
        return exit::fail(err, &e);
    }
    let dry_run = args.guard.dry_run;
    // Several projects share files at the root, so each is planned again
    // once the one before it is written.
    let mut written: Vec<Recorded> = Vec::new();
    let mut unread: Vec<&str> = Vec::new();
    // The last project's settings: it's planned once every block of the
    // setup steps is written, so its settings carry every server.
    let mut mcp_settings: Option<String> = None;
    for (i, project) in projects.iter().enumerate() {
        // A dry run writes nothing, so the plans made first are the ones.
        let plan = if dry_run {
            plans.remove(0)
        } else if i == 0 {
            plans.swap_remove(0)
        } else {
            match sync::plan(project, &targets, &options) {
                Ok(plan) => plan,
                Err(e) => return exit::fail(err, &e),
            }
        };
        if !dry_run && let Err(e) = plan.write() {
            return exit::fail(err, &e);
        }
        record(&mut written, &plan);
        for line in &plan.unread {
            if !unread.contains(line) {
                unread.push(line);
            }
        }
        if plan.mcp_settings.is_some() {
            mcp_settings = plan.mcp_settings;
        }
    }
    if args.format == Format::Json {
        let json = sync_json(
            (false, !dry_run),
            &written,
            &notes,
            &unread,
            mcp_settings.as_deref(),
        );
        let result = answer::write_json(out, &json);
        return answer::written(result, err).unwrap_or(exit::OK);
    }
    let paths: Vec<String> = written.iter().map(|(path, ..)| shown(path)).collect();
    let width = paths.iter().map(|p| p.chars().count()).max().unwrap_or(0);
    let result = written
        .iter()
        .zip(&paths)
        .try_for_each(|((_, changed, readers), path)| {
            let done = match (*changed, dry_run) {
                (false, _) => "unchanged",
                (true, false) => "wrote",
                (true, true) => "would write",
            };
            let pad = if dry_run { 11 } else { 9 };
            writeln!(out, "{done:<pad$} {path:<width$}  read by {}", list(readers))
        });
    let result = result.and_then(|()| {
        unread
            .iter()
            .try_for_each(|line| write!(out, "\n{line}.\n"))
    });
    let result = result.and_then(|()| {
        let Some(json) = &mcp_settings else {
            return Ok(());
        };
        write!(
            out,
            "\nCopilot's cloud agent reads its MCP servers from the repository's settings, \
             not from a file. Paste this into {}, beside any servers already there, or run \
             with --agent to write a custom agent that carries it:\n\n{}",
            copilot::MCP_SETTINGS_PLACE,
            json
        )
    });
    answer::written(result, err).unwrap_or(exit::OK)
}

/// What `sync --format json` writes about `files`: with `--check` or not,
/// and written or not.
fn sync_json<'a>(
    (check, written): (bool, bool),
    files: &'a [Recorded],
    notes: &'a [String],
    unread: &[&'a str],
    mcp_settings: Option<&str>,
) -> SyncJson<'a> {
    SyncJson {
        schema_version: SYNC_SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        check,
        written,
        files: files
            .iter()
            .map(|(path, changed, readers)| SyncFile {
                path: shown(path),
                changed: *changed,
                read_by: readers,
            })
            .collect(),
        notes: notes.iter().map(String::as_str).collect(),
        unread: unread.to_vec(),
        mcp_settings: mcp_settings.and_then(|json| serde_json::from_str(json).ok()),
    }
}

/// A file `sync` writes: where, whether it changes, and who reads it.
type Recorded = (PathBuf, bool, &'static [Agent]);

/// Adds `plan`'s files to `files`, each once, in the order they're first
/// planned: changed when any plan changes it.
fn record(files: &mut Vec<Recorded>, plan: &Plan) {
    for file in &plan.files {
        match files.iter_mut().find(|(path, ..)| *path == file.path) {
            Some((_, changed, _)) => *changed |= file.changes(),
            None => files.push((file.path.clone(), file.changes(), file.readers)),
        }
    }
}

/// Agents' names as a list in a sentence: `Codex, Copilot, and Cursor`.
fn list(agents: &[Agent]) -> String {
    let names: Vec<&str> = agents.iter().map(|a| a.name()).collect();
    match names.as_slice() {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

fn rules(global: &Global, args: &RulesArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let loaded = match answer::load(&FromDisk, global, args.path.as_deref()) {
        Ok(loaded) => loaded,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    match sync::rules_beside(&loaded.project, loaded.index()) {
        Ok(text) => answer::written(out.write_all(text.as_bytes()), err).unwrap_or(exit::OK),
        Err(e) => exit::fail(err, &e),
    }
}

fn index(global: &Global, args: &IndexArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let loaded = match answer::load(&FromDisk, global, None) {
        Ok(loaded) => loaded,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let project = &loaded.project;
    let build = match answer::one_build(project.model(), args.build.as_deref()) {
        Ok(build) => build,
        Err(e) => return exit::fail(err, &e),
    };
    let cwd = std::env::current_dir().unwrap_or_default();
    let absolute = |path: &Path| ascribe_core::path::normalize(&cwd.join(path));
    let file = args.into.as_deref().map(absolute);
    let folder = args.pages.as_deref().map(absolute);
    // The addresses are read from the file's folder, or from here.
    let reader = file
        .as_deref()
        .and_then(Path::parent)
        .map_or_else(|| absolute(Path::new(".")), Path::to_owned);
    let address = folder
        .as_deref()
        .map(|folder| index_file::folder_address(&reader, folder));
    let mut answer = match ascribe_query::agents_index(
        loaded.index(),
        project.root(),
        build,
        address.as_deref(),
        ascribe_resolve::llms::COMPACT_INDEX_LIMIT,
    ) {
        Ok(answer) => answer,
        Err(e) => return exit::fail(err, &e),
    };
    let mut written = index_file::Written::default();
    if file.is_some() || folder.is_some() {
        let content = absolute(&project.root().join(project.content_root().as_str()));
        written = match index_file::write(
            &answer,
            file.as_deref(),
            folder.as_deref(),
            &content,
            args.guard,
        ) {
            Ok(written) => written,
            Err(e) => return exit::fail(err, &e),
        };
        answer.written = !args.guard.dry_run;
    }
    answer.file = file.as_deref().map(shown);
    answer.changed = written.changed;
    answer.folder = folder.as_deref().map(shown);
    if args.format == Format::Json {
        let result = answer::write_json(out, &answer);
        return answer::written(result, err).unwrap_or(exit::OK);
    }
    // The index goes to standard output unless it's written into a file;
    // then what was written does.
    let wrote = if args.guard.dry_run {
        "would write"
    } else {
        "wrote"
    };
    let pad = wrote.len().max(9);
    let pages = answer.folder.as_deref().map(|folder| {
        let done = if written.pages_changed > 0 {
            wrote
        } else {
            "unchanged"
        };
        let count = answer.pages.len();
        let noun = if count == 1 { "page" } else { "pages" };
        format!(
            "{done:<pad$} {folder}  {count} {noun}, {} changed",
            written.pages_changed
        )
    });
    let result = match &answer.file {
        None => out.write_all(answer.text.as_bytes()).map(|()| {
            if let Some(pages) = &pages {
                let _ = writeln!(err, "{pages}");
            }
        }),
        Some(file) => {
            let done = if written.changed { wrote } else { "unchanged" };
            writeln!(out, "{done:<pad$} {file}")
                .and_then(|()| pages.map_or(Ok(()), |pages| writeln!(out, "{pages}")))
        }
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    if !answer.left_out.is_empty() || !answer.without_description.is_empty() {
        let _ = writeln!(
            err,
            "note: to stay under {} bytes, the index leaves out {} of {} pages, and the descriptions of {} more",
            answer.limit,
            answer.left_out.len(),
            answer.offered,
            answer.without_description.len()
        );
    }
    if !written.stray.is_empty() {
        let _ = writeln!(
            err,
            "note: {} has Markdown files that aren't pages of this index, perhaps from an earlier run: {}",
            answer.folder.as_deref().unwrap_or_default(),
            written.stray.join(", ")
        );
    }
    exit::OK
}

fn prompt(args: &PromptArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    if args.list && args.format == Format::Json {
        let list = PromptListJson {
            schema_version: PROMPT_SCHEMA_VERSION,
            ascribe_version: env!("CARGO_PKG_VERSION"),
            prompts: prompts::PROMPTS
                .iter()
                .map(|p| PromptListed {
                    name: p.name,
                    title: p.title,
                    description: p.description,
                    arguments: p
                        .arguments
                        .iter()
                        .map(|(name, description, required)| PromptArgument {
                            name,
                            description,
                            required: *required,
                        })
                        .collect(),
                })
                .collect(),
        };
        return answer::written(answer::write_json(out, &list), err).unwrap_or(exit::OK);
    }
    if args.list {
        let result = prompts::PROMPTS.iter().try_for_each(|p| {
            let arguments: Vec<String> = p
                .arguments
                .iter()
                .map(|(name, _, required)| {
                    if *required {
                        format!("{name}=…")
                    } else {
                        format!("[{name}=…]")
                    }
                })
                .collect();
            writeln!(out, "{} {}", p.name, arguments.join(" "))?;
            writeln!(out, "  {}", p.description)
        });
        return answer::written(result, err).unwrap_or(exit::OK);
    }
    let name = args.name.as_deref().unwrap_or_default();
    let Some(named) = prompts::find(name) else {
        let names: Vec<&str> = prompts::PROMPTS.iter().map(|p| p.name).collect();
        let _ = writeln!(
            err,
            "error: there's no prompt `{name}`; the prompts are {}",
            names.join(", ")
        );
        return exit::FAILURE;
    };
    let arguments: prompts::Arguments = args.arguments.iter().cloned().collect();
    match named.text(&FromDisk, &arguments) {
        Ok(text) => {
            let result = match args.format {
                Format::Text => out.write_all(text.as_bytes()),
                Format::Json => answer::write_json(
                    out,
                    &PromptJson {
                        schema_version: PROMPT_SCHEMA_VERSION,
                        ascribe_version: env!("CARGO_PKG_VERSION"),
                        name: named.name,
                        text: &text,
                    },
                ),
            };
            answer::written(result, err).unwrap_or(exit::OK)
        }
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            exit::FAILURE
        }
    }
}

fn check(
    plans: &[Plan],
    notes: &[String],
    format: Format,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let mut files = Vec::new();
    for plan in plans {
        record(&mut files, plan);
    }
    let result = match format {
        Format::Text => {
            let mut shown_once: Vec<&Path> = Vec::new();
            plans
                .iter()
                .flat_map(|plan| &plan.files)
                .filter(|file| file.changes())
                .try_for_each(|file| {
                    if shown_once.contains(&file.path.as_path()) {
                        return Ok(());
                    }
                    shown_once.push(&file.path);
                    let old = file.old.as_deref().unwrap_or_default();
                    answer::write_unified_diff(out, &shown(&file.path), old, &file.new)
                })
        }
        Format::Json => {
            let mut unread: Vec<&str> = Vec::new();
            for line in plans.iter().flat_map(|plan| &plan.unread) {
                if !unread.contains(line) {
                    unread.push(line);
                }
            }
            let mcp_settings = plans.iter().rev().find_map(|p| p.mcp_settings.as_deref());
            answer::write_json(
                out,
                &sync_json((true, false), &files, notes, &unread, mcp_settings),
            )
        }
    };
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    let stale = files.iter().filter(|(_, changed, _)| *changed).count();
    if stale == 0 {
        return exit::OK;
    }
    let files = if stale == 1 { "file is" } else { "files are" };
    let _ = writeln!(
        err,
        "error: {stale} {files} out of date; run `ascribe agents sync` to update them"
    );
    exit::PROBLEMS
}

/// A path as the person typed it: from the current directory, with `/`,
/// and `.` for the current directory itself.
fn shown(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            ascribe_core::path::relative_path(&ascribe_core::path::normalize(&cwd), path)
        })
        .map(|rel| match rel.as_str() {
            "" => ".".to_owned(),
            rel => rel.to_owned(),
        })
        .unwrap_or_else(|| path.display().to_string())
}
