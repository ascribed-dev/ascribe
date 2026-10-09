//! `ascribe model`: what the content model allows, resolved. The text is a
//! short Markdown summary an agent can read directly; JSON has every entry.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use ascribe_check::Project;
use ascribe_query::Section as QuerySection;
use clap::{Args as ClapArgs, ValueEnum};

use crate::answer::{self, Format};
use crate::cli::Global;
use crate::context::Failure;
use crate::exit;

/// How many characters the text summary may take before long lists are cut.
pub const SUMMARY_BUDGET: usize = 4_000;

/// Arguments of `ascribe model`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// A file or folder in the project, to find its `ascribe.toml` from.
    ///
    /// By default, the current directory.
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Show only this section, in full.
    #[arg(long, value_enum, value_name = "NAME")]
    pub section: Option<Section>,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// A section of the content model.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Section {
    /// Page types, with their files and frontmatter fields.
    Types,
    /// Dimensions and their values.
    Dimensions,
    /// Phrases, with their values.
    Phrases,
    /// Features, with their availability.
    Features,
    /// Glossary terms.
    Glossary,
    /// Project widgets, with their attributes.
    Widgets,
    /// Builds, with their variant and availability modes.
    Builds,
}

impl Section {
    fn query(self) -> QuerySection {
        match self {
            Section::Types => QuerySection::Types,
            Section::Dimensions => QuerySection::Dimensions,
            Section::Phrases => QuerySection::Phrases,
            Section::Features => QuerySection::Features,
            Section::Glossary => QuerySection::Glossary,
            Section::Widgets => QuerySection::Widgets,
            Section::Builds => QuerySection::Builds,
        }
    }
}

/// Runs the command. Exit codes: 0 with the answer, 2 when there's no
/// content model or it has errors.
pub fn run(global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = model(global, &args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn model(global: &Global, args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    // Only the content model: a project whose pages have errors still has
    // one to show.
    let loaded = answer::locate(global, args.path.as_deref())
        .map_err(Failure::Config)
        .and_then(|config| Project::load_model(&config).map_err(Failure::Load));
    let model = match loaded {
        Ok(file) => file.model,
        Err(failure) => return answer::report_failure(err, &failure),
    };
    let section = args.section.map(Section::query);
    let result = match args.format {
        Format::Json => answer::write_json(out, &ascribe_query::model(&model, section)),
        Format::Text => {
            out.write_all(ascribe_query::summary(&model, section, SUMMARY_BUDGET).as_bytes())
        }
    };
    answer::written(result, err).unwrap_or(exit::OK)
}
