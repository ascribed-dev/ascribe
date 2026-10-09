//! `ascribe agents`: the files agents read on their own. `sync` writes the
//! project's rules into the instruction files and the skill into the skills
//! folder; `skill` prints the skill.

use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use clap::{Args as ClapArgs, Subcommand, ValueEnum};

use crate::agents::skill;
use crate::agents::sync::{self, Plan};
use crate::answer;
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
    /// Print the Ascribe skill's SKILL.md.
    #[command(after_help = docs_page!("reference/cli/#ascribe-agents-skill"))]
    Skill,
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
        Command::Skill => {
            let result = out.write_all(skill::skill_md(None).as_bytes());
            answer::written(result, &mut err).unwrap_or(exit::OK)
        }
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
