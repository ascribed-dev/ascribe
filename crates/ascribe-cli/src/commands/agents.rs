//! `ascribe agents`: the files agents read on their own, and the prompts
//! for them. `sync` writes the project's rules into the instruction files
//! and the skill into the skills folder; `rules` prints the rules, `skill`
//! the skill, and `prompt` a named prompt.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args as ClapArgs, Subcommand, ValueEnum};

use crate::agents::sync::{self, Plan};
use crate::agents::{prompts, skill};
use crate::answer::{self, FromDisk};
use crate::cli::Global;
use crate::context::load_project;
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
    #[command(after_help = docs_page!("reference/cli/#ascribe-agents-sync"))]
    Sync(SyncArgs),
    /// Print the project's rules: the block `sync` writes into the
    /// AGENTS.md beside ascribe.toml.
    #[command(after_help = docs_page!("reference/cli/#ascribe-agents-rules"))]
    Rules(RulesArgs),
    /// Print a file of the Ascribe skill: its SKILL.md, or its directive
    /// reference.
    #[command(after_help = docs_page!("reference/cli/#ascribe-agents-skill"))]
    Skill(SkillArgs),
    /// Print a named prompt for an agent: `new-page`, `fix`, or `review`.
    #[command(after_help = docs_page!("reference/cli/#ascribe-agents-prompt"))]
    Prompt(PromptArgs),
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
    /// Write nothing, and exit with 1 when a file is out of date: for CI.
    #[arg(long)]
    pub check: bool,

    /// Also write this target's files.
    ///
    /// Repeat it for several. `agents-md` and `skills` are always written,
    /// and each other target whose files already exist is kept up to date.
    #[arg(long, value_enum, value_name = "TARGET")]
    pub target: Vec<Target>,
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
    /// ascribe.toml, and the skill in .claude/skills/ascribe/.
    Claude,
    /// The rules in .claude/rules/, loaded only for the project's pages.
    ClaudeRules,
    /// The rules in .github/instructions/, loaded only for the project's
    /// pages.
    Copilot,
}

impl Target {
    fn sync(self) -> sync::Target {
        match self {
            Target::AgentsMd => sync::Target::AgentsMd,
            Target::Skills => sync::Target::Skills,
            Target::Claude => sync::Target::Claude,
            Target::ClaudeRules => sync::Target::ClaudeRules,
            Target::Copilot => sync::Target::Copilot,
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
        Command::Skill(args) => {
            let text = if args.file == skill::DIRECTIVES {
                skill::directives_md()
            } else {
                skill::skill_md(None)
            };
            let result = out.write_all(text.as_bytes());
            answer::written(result, &mut err).unwrap_or(exit::OK)
        }
        Command::Prompt(args) => prompt(&args, &mut out, &mut err),
    };
    let _ = out.flush();
    exit::code(code)
}

fn sync(global: &Global, args: &SyncArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let project = match load_project(global) {
        Ok(project) => project,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let targets: Vec<sync::Target> = args.target.iter().map(|t| t.sync()).collect();
    let plan = match sync::plan(&project, &targets) {
        Ok(plan) => plan,
        Err(e) => return exit::fail(err, &e),
    };
    for note in &plan.notes {
        let _ = writeln!(err, "note: {note}");
    }
    if args.check {
        return check(&plan, out, err);
    }
    if let Err(e) = plan.write() {
        return exit::fail(err, &e);
    }
    let result = plan.files.iter().try_for_each(|file| {
        let done = if file.changes() { "wrote" } else { "unchanged" };
        writeln!(out, "{done:<9} {}", shown(&file.path))
    });
    answer::written(result, err).unwrap_or(exit::OK)
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

fn prompt(args: &PromptArgs, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
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
        Ok(text) => answer::written(out.write_all(text.as_bytes()), err).unwrap_or(exit::OK),
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            exit::FAILURE
        }
    }
}

fn check(plan: &Plan, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let result = plan.files.iter().try_for_each(|file| {
        let state = if file.changes() {
            "stale"
        } else {
            "up to date"
        };
        writeln!(out, "{state:<10} {}", shown(&file.path))
    });
    if let Some(code) = answer::written(result, err) {
        return code;
    }
    let stale = plan.files.iter().filter(|f| f.changes()).count();
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

/// A path as the person typed it: from the current directory, with `/`.
fn shown(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| {
            ascribe_core::path::relative_path(&ascribe_core::path::normalize(&cwd), path)
        })
        .map(|rel| rel.as_str().to_owned())
        .unwrap_or_else(|| path.display().to_string())
}
