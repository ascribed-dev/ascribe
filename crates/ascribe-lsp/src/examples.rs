//! Snippets and tested examples in the editor: where a `@snippet`'s code is,
//! for Go to Definition; its code, and what its file was tested with, on
//! hover; the command to run where it's out of date, to copy; which
//! pages to check again when a source's observations or config change; and
//! a source's `ascribe.examples.toml`, checked and completed as it's edited.
//!
//! Nothing here runs anything. The server compares files with what was
//! written down, as `ascribe check` does, and the only command it offers
//! puts `ascribe observe` on the clipboard, for a person to run in a
//! terminal, where they can see what it starts.

use std::collections::BTreeSet;
use std::path::Path;

use ascribe_core::{FileId, Issue, LineIndex, Location, RelPath, diagnostics};
use ascribe_model::examples::{EXAMPLES_FILE, OBSERVATIONS_FOLDER, Problem};
use ascribe_model::{ContentModel, Source};
use ascribe_resolve::snippet::SnippetError;
use ascribe_resolve::{
    Address, CodeFiles, Example, ExampleStatus, FileIndex, FileSystem, Snapshot, SnippetUse,
    resolve_snippet,
};
use lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CompletionItem, CompletionItemKind, Diagnostic,
};

use crate::nav::Ctx;
use crate::position::Encoding;
use crate::uri::path_to_uri;

/// The client's command that puts its one argument, a line of text, on the
/// clipboard. The extension registers it (`packages/vscode/src/extension.ts`);
/// the server never runs it.
pub(crate) const COPY_COMMAND: &str = "ascribe.copyCommand";

/// The title of the quick fix on `example-stale` and `example-unobserved`.
pub(crate) const COPY_TITLE: &str = "Copy the command";

/// The most lines of code a hover on a `@snippet` shows.
const HOVER_LINES: usize = 20;

/// Where a `@snippet`'s code is: its region's start tag, or the code file's
/// first line for a snippet of a whole file. `None` when the address names
/// no file, or no region in it, as a broken link goes nowhere.
pub(crate) fn snippet_definition(ctx: &Ctx, snippet: &SnippetUse) -> Option<lsp_types::Location> {
    let Some(Ok(address)) = &snippet.address else {
        return None;
    };
    let fs: &dyn FileSystem = &*ctx.fs;
    let code = CodeFiles::new();
    // A file whose tags have problems still has the regions it has.
    let file = match resolve_snippet(snippet, address, &ctx.model, fs, &code) {
        Ok(resolved) => code.read(fs, &resolved.path).ok()?,
        Err(SnippetError::Tags(file)) => file,
        Err(_) => return None,
    };
    let range = match &address.region {
        Some(name) => {
            let region = file.tags.region(name)?;
            ctx.encoding.range(&LineIndex::new(&file.text), region.span)
        }
        None => lsp_types::Range::default(),
    };
    let path = ascribe_core::path::normalize(&ctx.config.parent()?.join(file.path.as_str()));
    Some(lsp_types::Location {
        uri: path_to_uri(&path)?,
        range,
    })
}

/// What a hover on a `@snippet`'s address says: the file, as a link that
/// opens it, and its region; for a tested example, what it was tested with
/// or why it says nothing, with the observations behind that, and where to
/// run `ascribe observe`; then the code, as the page shows it. `None` when
/// the address names no snippet: its problem says why.
pub(crate) fn snippet_hover(ctx: &Ctx, snippet: &SnippetUse) -> Option<String> {
    let Some(Ok(address)) = &snippet.address else {
        return None;
    };
    let fs: &dyn FileSystem = &*ctx.fs;
    // Read now, not as the page last resolved it: an observation written
    // since counts.
    let code = CodeFiles::new();
    let resolved = resolve_snippet(snippet, address, &ctx.model, fs, &code).ok()?;
    let root = ctx.config.parent()?;
    let file = shown_path(&resolved.path);
    let mut out = match path_to_uri(&root.join(resolved.path.as_str())) {
        Some(uri) => {
            let line = resolved
                .lines
                .map(|(from, _)| format!("#L{from}"))
                .unwrap_or_default();
            format!("[{}](<{}{line}>)", escape(&file), uri.as_str())
        }
        None => code_span(&file),
    };
    if let Some(region) = &address.region {
        out.push_str(&format!(", region {}", code_span(region)));
    }
    if let Some((from, to)) = resolved.lines {
        if from == to {
            out.push_str(&format!(", line {from}"));
        } else {
            out.push_str(&format!(", lines {from} to {to}"));
        }
    }
    if let Some(example) = &resolved.example {
        out.push_str("\n\n");
        out.push_str(&tested(ctx, address, example, &code)?);
    }
    let lang = resolved.info.split(' ').next().unwrap_or_default();
    out.push_str(&format!("\n\n{}", code_block(lang, &resolved.code)));
    Some(out)
}

/// What a tested example was tested with, or why it says nothing, with the
/// observations behind that, and where to run `ascribe observe`.
fn tested(ctx: &Ctx, address: &Address, example: &Example, code: &CodeFiles) -> Option<String> {
    let fs: &dyn FileSystem = &*ctx.fs;
    let mut out = String::new();
    match example.status {
        ExampleStatus::Current => {
            out.push_str(&format!(
                "**Tested with** {}",
                escape(example.tested.as_deref().unwrap_or_default())
            ));
            let names: Vec<&str> = example.current.iter().map(|o| o.name.as_str()).collect();
            out.push_str(&format!("\n\n{}", observations("current", &names)));
        }
        ExampleStatus::Stale => {
            out.push_str("**Changed after its last test run**");
            let source = ctx.model.source(&address.source)?;
            let examples = code.examples(fs, &example.folder);
            let names: Vec<&str> = examples
                .readable()
                .filter(|(_, o)| o.files.contains_key(&address.path))
                .map(|(name, _)| name)
                .collect();
            out.push_str(&format!("\n\n{}", observations("that list it", &names)));
            out.push_str(&format!("\n\n{}", what_to_run(source, &example.folder)));
        }
        ExampleStatus::Unobserved => {
            out.push_str("**Not tested:** no observation lists it.");
            let source = ctx.model.source(&address.source)?;
            out.push_str(&format!("\n\n{}", what_to_run(source, &example.folder)));
        }
    }
    Some(out)
}

/// The code as a fenced block in `lang`, its fence longer than any run of
/// backticks in it, cut at [`HOVER_LINES`] lines with how many more there
/// are after it.
fn code_block(lang: &str, code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let shown = &lines[..lines.len().min(HOVER_LINES)];
    let longest = shown
        .iter()
        .flat_map(|line| line.split(|c| c != '`'))
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat((longest + 1).max(3));
    let mut out = format!("{fence}{lang}\n");
    for line in shown {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str(&fence);
    match lines.len() - shown.len() {
        0 => {}
        1 => out.push_str("\n\nand 1 more line"),
        more => out.push_str(&format!("\n\nand {more} more lines")),
    }
    out
}

/// `Observations current: `default`, `node-22``.
fn observations(which: &str, names: &[&str]) -> String {
    let listed: Vec<String> = names.iter().map(|n| code_span(n)).collect();
    format!("Observations {which}: {}", listed.join(", "))
}

/// Where an example's observation comes from: `ascribe observe`, run in its
/// folder, or the next update of a source in another repository.
fn what_to_run(source: &Source, folder: &RelPath) -> String {
    if source.git.is_some() {
        format!(
            "Its observation is written in the source's own repository, and arrives with the next {}.",
            code_span("ascribe sources update")
        )
    } else {
        format!(
            "Run {} in {} to test it as it is now.",
            code_span("ascribe observe"),
            code_span(&shown_path(folder))
        )
    }
}

/// "Copy the command", on `example-stale` or `example-unobserved` at the
/// address of a `@snippet` of `file`: `cd` to the example's folder, then
/// `ascribe observe`, for a person to paste in a terminal. A source in
/// another repository is tested there, so it gets none.
pub(crate) fn copy_command(
    ctx: &Ctx,
    file: &FileIndex,
    at: usize,
    diagnostic: Diagnostic,
) -> Option<CodeActionOrCommand> {
    let snippet = file
        .snippets
        .iter()
        .find(|s| s.primary.is_some_and(|p| p.start() == at))?;
    let Some(Ok(address)) = &snippet.address else {
        return None;
    };
    let source = ctx.model.source(&address.source)?;
    if source.git.is_some() {
        return None;
    }
    let folder = ctx.config.parent()?.join(source.path.as_str());
    let folder = ascribe_core::path::normalize(&folder);
    let line = observe_line(&folder, cfg!(windows));
    Some(CodeActionOrCommand::CodeAction(CodeAction {
        title: COPY_TITLE.to_owned(),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: None,
        command: Some(lsp_types::Command {
            title: COPY_TITLE.to_owned(),
            command: COPY_COMMAND.to_owned(),
            arguments: Some(vec![serde_json::Value::String(line)]),
        }),
        is_preferred: Some(false),
        disabled: None,
        data: None,
    }))
}

/// The line that runs `ascribe observe` in `folder`, for the terminal a
/// person is likely to paste it in: PowerShell on Windows, whose default
/// version, 5.1, has no `&&`, and `sh` elsewhere. The folder is in single
/// quotes, which both read as it's written, `$` and backticks included; a
/// single quote in it is doubled for PowerShell, and closed, escaped, and
/// reopened for `sh`.
fn observe_line(folder: &Path, windows: bool) -> String {
    let text = folder.to_string_lossy();
    if windows {
        format!(
            "Set-Location -LiteralPath '{}'; ascribe observe",
            text.replace('\'', "''")
        )
    } else {
        format!("cd '{}' && ascribe observe", text.replace('\'', "'\\''"))
    }
}

/// The pages and fragments whose `@snippet`s name a source whose
/// `ascribe.examples.toml` or observations are among `changed` (paths from
/// the project root): what to check again so their advice is right.
pub(crate) fn showing_changed(
    snapshot: &Snapshot,
    model: &ContentModel,
    changed: &[RelPath],
) -> BTreeSet<RelPath> {
    let sources: Vec<&str> = model
        .sources
        .iter()
        .filter(|source| {
            RelPath::parse(&source.path)
                .is_ok_and(|folder| changed.iter().any(|path| is_examples_path(&folder, path)))
        })
        .map(|source| source.name.as_str())
        .collect();
    if sources.is_empty() {
        return BTreeSet::new();
    }
    snapshot
        .files()
        .filter(|file| {
            file.snippets.iter().any(|s| {
                matches!(&s.address, Some(Ok(address)) if sources.contains(&address.source.as_str()))
            })
        })
        .map(|file| file.path.clone())
        .collect()
}

/// Whether `path` is `folder`'s `ascribe.examples.toml`, its observations'
/// folder, or a file in it.
fn is_examples_path(folder: &RelPath, path: &RelPath) -> bool {
    let Some(rest) = path.relative_to(folder) else {
        return false;
    };
    let rest = rest.as_str();
    rest == EXAMPLES_FILE
        || rest == OBSERVATIONS_FOLDER
        || rest
            .strip_prefix(OBSERVATIONS_FOLDER)
            .is_some_and(|after| after.starts_with('/'))
}

/// The source whose `ascribe.examples.toml` is at `path`, from the project
/// root.
pub(crate) fn config_source<'a>(model: &'a ContentModel, path: &RelPath) -> Option<&'a Source> {
    model.sources.iter().find(|source| {
        RelPath::parse(&source.path)
            .ok()
            .and_then(|folder| folder.join(EXAMPLES_FILE).ok())
            .is_some_and(|config| config == *path)
    })
}

/// The problems of a source's `ascribe.examples.toml` whose text is `text`,
/// in the file itself, at what each is about: `examples-config-invalid`, as
/// `ascribe check` reports it at the source, at the levels `[checks]` sets.
pub(crate) fn config_diagnostics(
    model: &ContentModel,
    source: &Source,
    shown: &str,
    text: &str,
    encoding: Encoding,
) -> Vec<Diagnostic> {
    let problems: Vec<Problem> = match ascribe_model::examples::ExamplesConfig::parse(text) {
        Ok(_) => return Vec::new(),
        Err(problems) => problems,
    };
    let found: Vec<ascribe_check::Diagnostic> = problems
        .into_iter()
        .map(|problem| {
            let issue = Issue::new(
                diagnostics::EXAMPLES_CONFIG_INVALID,
                Location::new(FileId::new(0), problem.span),
            )
            .with_arg("source", source.name.clone())
            .with_arg("file", shown.to_owned())
            .with_arg("detail", problem.detail);
            ascribe_check::Diagnostic::from_issue(&issue)
        })
        .collect();
    let index = LineIndex::new(text);
    ascribe_check::apply_levels(&model.checks, found)
        .iter()
        .map(|d| crate::compute::to_lsp(d, &index, encoding, &|_| None))
        .collect()
}

/// The keys of `ascribe.examples.toml` that can be written at `offset` of
/// `text`, the config's: `include`, `ignore`, `test`, and the
/// `[environment]` table at the top, or `label` and `command` in an
/// environment entry. Keys already written aren't offered again.
pub(crate) fn complete_config(text: &str, offset: usize) -> Vec<CompletionItem> {
    let offset = offset.min(text.len());
    if !text.is_char_boundary(offset) {
        return Vec::new();
    }
    let line_start = text[..offset].rfind('\n').map_or(0, |i| i + 1);
    let before = &text[line_start..offset];
    let key_chars = |s: &str| {
        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    };
    // The table the line is in: the last header above it.
    let table = text[..line_start].lines().rev().find_map(|line| {
        let line = line.trim();
        (line.starts_with('[') && !line.starts_with("[[")).then(|| {
            line.trim_start_matches('[')
                .split(']')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned()
        })
    });
    let line_end = text[offset..].find('\n').map_or(text.len(), |i| offset + i);
    let line = &text[line_start..line_end];
    let item = |key: &str, detail: &str| CompletionItem {
        label: key.to_owned(),
        kind: Some(CompletionItemKind::PROPERTY),
        detail: Some(detail.to_owned()),
        ..CompletionItem::default()
    };
    // In an entry's inline table, after `{` or a `,`.
    if let Some(open) = before.rfind('{') {
        let inside = &before[open + 1..];
        let typed = inside.rsplit(',').next().unwrap_or_default().trim_start();
        if table.as_deref() == Some("environment") && key_chars(typed) {
            let written = &line[open + 1..];
            return entry_keys(written)
                .into_iter()
                .map(|(k, d)| item(k, d))
                .collect();
        }
        return Vec::new();
    }
    let typed = before.trim_start();
    if !key_chars(typed) {
        return Vec::new();
    }
    match table.as_deref() {
        None => {
            let written: Vec<&str> = text
                .lines()
                .take_while(|l| !l.trim_start().starts_with('['))
                .filter_map(|l| l.split('=').next())
                .map(str::trim)
                .collect();
            let mut out: Vec<CompletionItem> = [
                (
                    "include",
                    "The files that are examples: patterns from this folder",
                ),
                ("ignore", "Files `include` matches that aren't examples"),
                (
                    "test",
                    "The line of shell that tests the examples, run by `ascribe observe`",
                ),
            ]
            .into_iter()
            .filter(|(key, _)| !written.contains(key))
            .map(|(k, d)| item(k, d))
            .collect();
            if !text.lines().any(|l| l.trim() == "[environment]") {
                let mut table = item(
                    "[environment]",
                    "What to note about where the test ran: a `label` and a `command` for each entry",
                );
                table.kind = Some(CompletionItemKind::MODULE);
                table.filter_text = Some("environment".to_owned());
                table.insert_text = Some("[environment]".to_owned());
                out.push(table);
            }
            out
        }
        Some(name) if name.starts_with("environment.") => {
            // The keys written in this entry's table so far.
            let rest = &text[line_start..];
            let end = rest.find("\n[").unwrap_or(rest.len());
            let start = text[..line_start]
                .rfind(&format!("[{name}]"))
                .unwrap_or(line_start);
            let written = format!("{}{}", &text[start..line_start], &rest[..end]);
            entry_keys(&written)
                .into_iter()
                .map(|(k, d)| item(k, d))
                .collect()
        }
        _ => Vec::new(),
    }
}

/// `label` and `command`, those not already a key in `written`.
fn entry_keys(written: &str) -> Vec<(&'static str, &'static str)> {
    let has = |key: &str| {
        written.split([',', '{', '\n']).any(|part| {
            part.split('=').next().is_some_and(|k| k.trim() == key) && part.contains('=')
        })
    };
    [
        (
            "label",
            "What a reader sees beside the value; the entry's name when it has none",
        ),
        (
            "command",
            "The line of shell that prints the value; its first line is kept",
        ),
    ]
    .into_iter()
    .filter(|(key, _)| !has(key))
    .collect()
}

/// A path as written with `/`, on every platform.
fn shown_path(path: &RelPath) -> String {
    path.as_str().to_owned()
}

/// Text from a config, an observation, or a page, as Markdown that shows it
/// as it is: every ASCII punctuation character escaped, so none of it is
/// read as a link, an image, HTML, or emphasis.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if c.is_ascii_punctuation() {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Text as a Markdown code span, its backticks fenced by a longer run.
fn code_span(text: &str) -> String {
    let longest = text
        .split(|c| c != '`')
        .map(str::len)
        .max()
        .unwrap_or_default();
    let fence = "`".repeat(longest + 1);
    let pad = if text.starts_with('`') || text.ends_with('`') {
        " "
    } else {
        ""
    };
    format!("{fence}{pad}{text}{pad}{fence}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_everything_markdown_would_read() {
        assert_eq!(
            escape("<img src=x>[a](javascript:x) *b*"),
            "\\<img src\\=x\\>\\[a\\]\\(javascript\\:x\\) \\*b\\*"
        );
    }

    #[test]
    fn a_code_span_fences_its_backticks() {
        assert_eq!(code_span("a"), "`a`");
        assert_eq!(code_span("a`b"), "``a`b``");
        assert_eq!(code_span("`a"), "`` `a ``");
    }

    #[test]
    fn the_copied_line_is_for_each_platforms_terminal() {
        assert_eq!(
            observe_line(Path::new("/home/a/it's $HOME/code"), false),
            "cd '/home/a/it'\\''s $HOME/code' && ascribe observe"
        );
        assert_eq!(
            observe_line(Path::new("D:/it's `x`/code"), true),
            "Set-Location -LiteralPath 'D:/it''s `x`/code'; ascribe observe"
        );
    }

    #[test]
    fn knows_the_files_that_change_an_examples_status() {
        let folder = RelPath::parse("code").unwrap_or_else(|_| unreachable!());
        let is = |p: &str| {
            is_examples_path(
                &folder,
                &RelPath::parse(p).unwrap_or_else(|_| unreachable!()),
            )
        };
        assert!(is("code/ascribe.examples.toml"));
        assert!(is("code/examples.observations"));
        assert!(is("code/examples.observations/default.toml"));
        assert!(!is("code/examples/greet.ts"));
        assert!(!is("code/examples.observationsx"));
        assert!(!is("other/ascribe.examples.toml"));
    }

    fn labels(text: &str) -> Vec<String> {
        let offset = text.find('|').unwrap_or(text.len());
        let text = text.replacen('|', "", 1);
        complete_config(&text, offset)
            .into_iter()
            .map(|i| i.label)
            .collect()
    }

    #[test]
    fn completes_the_keys_at_the_top() {
        assert_eq!(labels("|"), ["include", "ignore", "test", "[environment]"]);
        assert_eq!(
            labels("include = [\"a/**\"]\nte|\n\n[environment]\n"),
            ["ignore", "test"]
        );
        // Not in a value.
        assert!(labels("test = \"no|").is_empty());
    }

    #[test]
    fn completes_an_entrys_keys() {
        assert_eq!(
            labels("include = []\n[environment]\nnode = { |"),
            ["label", "command"]
        );
        assert_eq!(
            labels("include = []\n[environment]\nnode = { label = \"Node\", c| }"),
            ["command"]
        );
        assert_eq!(
            labels("include = []\n[environment.node]\nlabel = \"Node\"\n|\n"),
            ["command"]
        );
        // An entry's own name is the writer's.
        assert!(labels("include = []\n[environment]\n|").is_empty());
    }
}
