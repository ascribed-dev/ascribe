//! `ascribe explain`: what a diagnostic means, how to fix it, and a short
//! wrong-and-right example. It needs no project.

use std::io::{self, Write};
use std::process::ExitCode;

use ascribe_query::{ExampleText, Explanation};
use clap::Args as ClapArgs;

use crate::answer::{self, Format, fence};
use crate::cli::Global;
use crate::exit;

/// Arguments of `ascribe explain`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    /// The diagnostic's code (`ASC036`) or name (`link-target-missing`).
    #[arg(value_name = "CODE", required_unless_present = "list")]
    pub code: Option<String>,

    /// List every diagnostic's code and name, one per line, instead.
    #[arg(long, conflicts_with = "code")]
    pub list: bool,

    /// How to show the answer.
    #[arg(long, value_enum, default_value_t = Format::Text, value_name = "FORMAT")]
    pub format: Format,
}

/// Runs the command. Exit codes: 0 with the answer, 2 when no diagnostic
/// has that code or name.
pub fn run(_global: &Global, args: Args) -> ExitCode {
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    let stderr = io::stderr();
    let mut err = stderr.lock();
    let code = explain(&args, &mut out, &mut err);
    let _ = out.flush();
    exit::code(code)
}

fn explain(args: &Args, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let result = if args.list {
        let list = ascribe_query::list();
        match args.format {
            Format::Json => answer::write_json(out, &list),
            Format::Text => list
                .diagnostics
                .iter()
                .try_for_each(|d| writeln!(out, "{} {}", d.code, d.slug)),
        }
    } else {
        let given = args.code.as_deref().unwrap_or_default();
        let explanation = match ascribe_query::explain(given) {
            Ok(explanation) => explanation,
            Err(e) => return answer::fail(err, &e),
        };
        match args.format {
            Format::Json => answer::write_json(out, &explanation),
            Format::Text => write_text(out, &explanation),
        }
    };
    answer::written(result, err).unwrap_or(exit::OK)
}

fn write_text(out: &mut dyn Write, e: &Explanation) -> io::Result<()> {
    let level = match e.level {
        "page" => "found in each page, per build",
        _ => "found in each file",
    };
    writeln!(out, "{} {}: {}, {level}", e.code, e.slug, e.severity)?;
    writeln!(out)?;
    writeln!(out, "Message: {}", e.message)?;
    for variant in &e.variants {
        writeln!(out, "  or: {}", variant.message)?;
    }
    if let Some(fix) = &e.fix {
        writeln!(out)?;
        writeln!(out, "Fix: {fix}")?;
    }
    if let Some(example) = &e.example {
        write_example(out, example)?;
    }
    writeln!(out)?;
    writeln!(out, "Docs: {}", e.docs)
}

fn write_example(out: &mut dyn Write, example: &ExampleText) -> io::Result<()> {
    let block = |out: &mut dyn Write, lang: &str, text: &str| -> io::Result<()> {
        let fence = fence(text);
        writeln!(out, "{fence}{lang}")?;
        write!(out, "{text}")?;
        if !text.ends_with('\n') {
            writeln!(out)?;
        }
        writeln!(out, "{fence}")
    };
    writeln!(out)?;
    writeln!(out, "Wrong, in page.md:")?;
    block(out, "md", &example.wrong)?;
    writeln!(out)?;
    writeln!(out, "Right:")?;
    block(out, "md", &example.right)?;
    if let Some(model) = &example.model {
        writeln!(out)?;
        writeln!(out, "With this in ascribe.toml:")?;
        block(out, "toml", model)?;
    }
    for file in &example.files {
        writeln!(out)?;
        let extension = file.path.rsplit_once('.').map_or("", |(_, e)| e);
        if ascribe_core::image_media_type(extension).is_some() {
            writeln!(out, "And an image, {}.", file.path)?;
            continue;
        }
        writeln!(out, "And {}:", file.path)?;
        let lang = if file.path.ends_with(".md") { "md" } else { "" };
        block(out, lang, &file.text)?;
    }
    Ok(())
}
