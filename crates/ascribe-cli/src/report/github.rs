//! The GitHub Actions report, `--format github`: one workflow command per
//! diagnostic, `::error file=…,line=…::message`, which a workflow run shows
//! on the pull request's lines, then the summary line.
//!
//! An error is `::error`, a warning `::warning`, and advice `::notice`. The
//! paths are from the repository's root, which is where GitHub looks for
//! them, and the message is one line, without the help.

use std::io::{self, Write};

use ascribe_check::{Reported, Severity};
use ascribe_core::RelPath;

use super::FileTable;
use super::concise::order;
use super::text::repeats_note;

/// Writes each diagnostic's workflow command, in the concise report's
/// [`order`], then `summary`. `prefix` is the path from the repository's
/// root to the project's (`git rev-parse --show-prefix`), empty outside a
/// repository.
pub fn write(
    out: &mut dyn Write,
    files: &FileTable,
    reported: &[Reported],
    summary: &str,
    prefix: &str,
) -> io::Result<()> {
    for i in order(files, reported) {
        writeln!(out, "{}", annotation(files, &reported[i], Some(prefix)))?;
    }
    writeln!(out, "{summary}")
}

/// One diagnostic's workflow command: at its place, with `prefix` before
/// the project's paths, or with none for a place whose repository path
/// isn't known.
pub fn annotation(files: &FileTable, r: &Reported, prefix: Option<&str>) -> String {
    let d = &r.diagnostic;
    let (start, end) = files.range(d.location);
    let line = ("line", start.line.to_string());
    let end_line = ("endLine", end.line.to_string());
    let file = prefix.map(|prefix| ("file", in_repository(prefix, &files.path(d.location.file))));
    // GitHub takes columns only for a range on one line.
    let mut properties = match file {
        None => Vec::new(),
        Some(file) if start.line == end.line => {
            vec![
                file,
                line,
                ("col", start.column.to_string()),
                end_line,
                ("endColumn", end.column.max(start.column).to_string()),
            ]
        }
        Some(file) => vec![file, line, end_line],
    };
    properties.push(("title", format!("{} ({})", d.slug, d.code)));
    let mut message = d.message.clone();
    if let Some(note) = repeats_note(r.repeats) {
        message.push_str(&format!(" ({note})"));
    }
    command(command_name(d.severity), &properties, &message)
}

/// The workflow command for a severity.
fn command_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Advice => "notice",
    }
}

/// `::name key=value,…::message`, each part escaped as the runner reads it.
pub fn command(name: &str, properties: &[(&str, String)], message: &str) -> String {
    let properties: Vec<String> = properties
        .iter()
        .map(|(key, value)| format!("{key}={}", escape_property(value)))
        .collect();
    let space = if properties.is_empty() { "" } else { " " };
    format!(
        "::{name}{space}{}::{}",
        properties.join(","),
        escape_data(message)
    )
}

/// A path shown from the project's root, from the repository's instead.
pub fn in_repository(prefix: &str, path: &str) -> String {
    if prefix.is_empty() {
        return path.to_owned();
    }
    RelPath::parse(&format!("{prefix}{path}")).map_or_else(|_| path.to_owned(), |p| p.to_string())
}

/// A message, with what would end the command or the line escaped.
fn escape_data(text: &str) -> String {
    text.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// A property's value, which a `,` or `:` would also end.
fn escape_property(text: &str) -> String {
    escape_data(text).replace(':', "%3A").replace(',', "%2C")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_would_end_a_command_is_escaped() {
        assert_eq!(
            command(
                "error",
                &[
                    ("file", "a,b:c.md".to_owned()),
                    ("title", "x (ASC1)".to_owned())
                ],
                "50% done\nnext"
            ),
            "::error file=a%2Cb%3Ac.md,title=x (ASC1)::50%25 done%0Anext"
        );
        assert_eq!(command("error", &[], "no"), "::error::no");
    }

    #[test]
    fn paths_are_from_the_repository_root() {
        assert_eq!(in_repository("", "docs/a.md"), "docs/a.md");
        assert_eq!(
            in_repository("handbook/", "docs/a.md"),
            "handbook/docs/a.md"
        );
        assert_eq!(in_repository("handbook/", "../src/a.rs"), "src/a.rs");
    }
}
