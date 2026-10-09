//! Agent prompts about problems: the text **Prompt agent** puts where a person
//! sends it to their agent, and `ascribe check --format prompt` writes.
//!
//! Three prompts, each built from what a check reported:
//!
//! - [`problem`]: one diagnostic, with its line, its fix advice, the values
//!   the content model allows when it's about one, and its fixes;
//! - [`file`]: a file's problems, one line each, at most [`MAX_LISTED`];
//! - [`project`]: how many problems each file has, at most [`MAX_LISTED`]
//!   files.
//!
//! They follow the agent prompt format (`project-docs/agents/README.md`):
//! the task, where, what Ascribe knows, how to finish; plain text with
//! Markdown, at most [`LIMIT`] characters, and the same prompt for the same
//! input. A prompt points rather than dumps: it names the file, the line,
//! and the command that gives the rest.

use std::path::{Path, PathBuf};

use ascribe_core::path::relative_path;
use ascribe_core::{Applicability, FileId, LineIndex, Location, RelPath};

use crate::{Diagnostic, MODEL_FILE, Project, Registry, Reported, Severity};

/// The most characters a prompt has: what the agents' own links take.
pub const LIMIT: usize = 5_000;

/// The most problems a file's prompt lists, and the most files a project's
/// prompt counts.
pub const MAX_LISTED: usize = 20;

/// The most pages a prompt names as showing a fragment.
pub const MAX_SHOWN_ON: usize = 10;

/// The most lines of a problem's text a prompt quotes.
const MAX_QUOTED_LINES: usize = 5;

/// The most related places a problem's prompt names.
const MAX_RELATED: usize = 5;

/// The builds whose problems a prompt is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Builds {
    /// Every build: what `ascribe check` checks.
    All,
    /// The builds named, in the order given.
    Named(Vec<String>),
    /// The editor's build alone, as the editor checks as you type.
    Editor(String),
}

/// What a prompt says besides its problems: where the project is, which
/// builds were checked, and which files have unsaved changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    /// The project's folder in its repository, `/`-separated, when it isn't
    /// the repository's root: `docs`. Commands in the prompt are written
    /// from the repository's root.
    pub folder: Option<String>,
    /// The `AGENTS.md` the agent follows, from the repository's root: the
    /// project folder's, or else the repository's. `None` when neither has
    /// one.
    pub agents: Option<String>,
    /// The builds checked.
    pub builds: Builds,
    /// The files with unsaved changes, relative to the project root, as a
    /// diagnostic's file is shown.
    pub unsaved: Vec<String>,
}

impl Context {
    /// The context of a project whose folder (the directory of
    /// `ascribe.toml`) is `root`: its repository is the nearest folder at or
    /// above it with a `.git`, and its `AGENTS.md` is looked for in the
    /// project's folder, then the repository's.
    pub fn of_project(root: &Path, builds: Builds) -> Context {
        let root = std::path::absolute(root).unwrap_or_else(|_| root.to_owned());
        let repository = repository_of(&root);
        let folder = repository
            .as_deref()
            .and_then(|repo| relative_path(repo, &root))
            .filter(|rel| !rel.is_root())
            .map(|rel| rel.to_string());
        let agents = if has_agents_md(&root) {
            Some(within(folder.as_deref(), "AGENTS.md"))
        } else if repository.as_deref().is_some_and(has_agents_md) {
            Some("AGENTS.md".to_owned())
        } else {
            None
        };
        Context {
            folder,
            agents,
            builds,
            unsaved: Vec::new(),
        }
    }

    /// The same context, with these files unsaved.
    #[must_use]
    pub fn with_unsaved(mut self, unsaved: Vec<String>) -> Context {
        self.unsaved = unsaved;
        self
    }

    /// A path relative to the project root, from the repository's root.
    pub fn in_repository(&self, path: &str) -> String {
        within(self.folder.as_deref(), path)
    }

    /// The `Project:` and `Build:` lines.
    pub fn project_lines(&self, lines: &mut Vec<String>) {
        if let Some(folder) = &self.folder {
            lines.push(format!("Project: {folder}/"));
        }
        match &self.builds {
            Builds::All => {}
            Builds::Named(names) => lines.push(format!(
                "{}: {}",
                if names.len() == 1 { "Build" } else { "Builds" },
                code_list(names)
            )),
            Builds::Editor(name) => lines.push(format!(
                "Build: `{name}`, the editor's. `ascribe check` checks every build."
            )),
        }
    }

    /// The two lines that end every prompt: the agent's instructions, and
    /// checking `targets` (relative to the repository's root).
    fn finish(&self, targets: &[String]) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(agents) = &self.agents {
            lines.push(format!("Follow the project's rules in `{agents}`."));
        }
        lines.push(format!(
            "When you're done, run `{}` and fix what it reports.",
            check_command(targets, &[])
        ));
        lines
    }

    /// The lines that end a prompt that asks for a report, not edits: the
    /// agent's instructions, and what to do instead of editing.
    pub fn report_finish(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(agents) = &self.agents {
            lines.push(format!("Follow the project's rules in `{agents}`."));
        }
        lines.push("Report what reads wrongly; don't edit.".to_owned());
        lines
    }
}

/// `path` inside `folder`, `/`-separated.
fn within(folder: Option<&str>, path: &str) -> String {
    match folder {
        Some(folder) if path.is_empty() => folder.to_owned(),
        Some(folder) => format!("{folder}/{path}"),
        None => path.to_owned(),
    }
}

/// The nearest folder at or above `root` with a `.git` (a directory, or the
/// file a worktree has).
fn repository_of(root: &Path) -> Option<PathBuf> {
    // Outside FileSystem: finding the repository around the project, which
    // isn't one of the project's files.
    root.ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_owned)
}

fn has_agents_md(dir: &Path) -> bool {
    // Outside FileSystem: the agents' instructions file sits beside the
    // project or at the repository's root, not among the project's sources.
    dir.join("AGENTS.md").is_file()
}

/// `ascribe check` on `targets` (relative to the repository's root), with
/// `options` after them.
fn check_command(targets: &[String], options: &[&str]) -> String {
    let mut words = vec!["ascribe".to_owned(), "check".to_owned()];
    words.extend(targets.iter().map(|t| shell_word(t)));
    words.extend(options.iter().map(|o| (*o).to_owned()));
    words.join(" ")
}

/// A word as a POSIX shell reads it: as it is when it's plain, else in single
/// quotes.
pub fn shell_word(word: &str) -> String {
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_-./:@%+=,".contains(c));
    if plain {
        word.to_owned()
    } else {
        format!("'{}'", word.replace('\'', r"'\''"))
    }
}

/// The prompt about one problem: "Fix this problem in `<file>`."
///
/// `shown_on` gives the pages that include a fragment, by content path,
/// directly or through other fragments.
pub fn problem(
    project: &Project,
    context: &Context,
    reported: &Reported,
    shown_on: &dyn Fn(&RelPath) -> Vec<RelPath>,
) -> String {
    let d = &reported.diagnostic;
    let file = path_of(project, d.location.file);
    let lines = lines_of(project, d.location);

    let mut place = vec![format!("Where: {file}:{}", line_range(lines.as_ref()))];
    unsaved_line(context, &file, &mut place);
    context.project_lines(&mut place);
    shown_on_line(project, context, d.location.file, shown_on, &mut place);

    let mut known = vec![
        format!(
            "Problem: {} `{}`, {}",
            d.code,
            d.slug.as_str(),
            match d.severity {
                Severity::Error => "an error",
                Severity::Warning => "a warning",
            }
        ),
        format!("Message: {}", d.message),
    ];
    if reported.repeats > 0 {
        known.push(format!(
            "It's reported at {} other {} too.",
            reported.repeats,
            if reported.repeats == 1 {
                "include"
            } else {
                "includes"
            }
        ));
    }
    if let Some(allowed) = &d.allowed {
        known.push(format!("Allowed values: {allowed}"));
    }
    let registered = Registry::global().get(d.slug);
    if let Some(help) = registered.and_then(|e| e.fix.as_deref()) {
        known.push(format!("How to fix it: {help}"));
    }
    for fix in &d.fixes {
        known.push(match fix.applicability {
            Applicability::Safe => format!("Ascribe has a safe automatic fix: `{}`.", fix.title),
            Applicability::Unsafe => format!("Ascribe has a fix to review: `{}`.", fix.title),
        });
    }
    for related in d.related.iter().take(MAX_RELATED) {
        let at = path_of(project, related.location.file);
        let line = lines_of(project, related.location).map_or(1, |l| l.first);
        known.push(format!("Related: {at}:{line}: {}", related.message));
    }
    if let Some(docs) = registered.map(crate::Entry::docs) {
        known.push(format!("Reference: {docs}"));
    }
    if let Some(lines) = &lines {
        known.push(quoted(lines, &file));
    }

    let target = context.in_repository(&file);
    assemble(
        &format!("Fix this problem in `{file}`."),
        &place,
        &known,
        &format!("(cut: read the rest in `{file}`)"),
        &context.finish(&[target]),
    )
}

/// The prompt about one file's problems: "Fix the N problems `ascribe
/// check` reports in `<file>`." `file` is the file's id; `reported` is what
/// counts for it, in the report's order. `None` when there are none.
pub fn file(
    project: &Project,
    context: &Context,
    file: FileId,
    reported: &[Reported],
    shown_on: &dyn Fn(&RelPath) -> Vec<RelPath>,
) -> Option<String> {
    if reported.is_empty() {
        return None;
    }
    let path = path_of(project, file);
    let mut place = vec![format!("Where: {path}")];
    unsaved_line(context, &path, &mut place);
    context.project_lines(&mut place);
    shown_on_line(project, context, file, shown_on, &mut place);

    let target = context.in_repository(&path);
    let rest = check_command(std::slice::from_ref(&target), &["--format", "concise"]);
    let order = in_order(reported);
    let mut listed: Vec<String> = order
        .iter()
        .take(MAX_LISTED)
        .map(|r| {
            let d = &r.diagnostic;
            let line = lines_of(project, d.location).map_or(1, |l| l.first);
            let at = if d.location.file == file {
                line.to_string()
            } else {
                format!("{}:{line}", path_of(project, d.location.file))
            };
            format!("- {at}: [{}] {}", d.code, d.message)
        })
        .collect();
    if order.len() > MAX_LISTED {
        listed.push(format!("and {} more: `{rest}`", order.len() - MAX_LISTED));
    }
    Some(assemble(
        &format!(
            "Fix {} `ascribe check` reports in `{path}`.",
            the_problems(reported.len())
        ),
        &place,
        &listed,
        &format!("(cut: run `{rest}` for the rest)"),
        &context.finish(&[target]),
    ))
}

/// The prompt about a project's problems, or those of the paths named
/// (relative to the project root; none for the whole project): "Fix the
/// problems `ascribe check` reports in this project." `None` when there are
/// none.
pub fn project(
    project: &Project,
    context: &Context,
    paths: &[RelPath],
    reported: &[Reported],
) -> Option<String> {
    if reported.is_empty() {
        return None;
    }
    let targets: Vec<String> = if paths.is_empty() {
        context.folder.iter().cloned().collect()
    } else {
        paths
            .iter()
            .map(|p| context.in_repository(p.as_str()))
            .collect()
    };
    let where_ = if paths.is_empty() {
        "this project".to_owned()
    } else {
        code_list(&paths.iter().map(ToString::to_string).collect::<Vec<_>>())
    };

    let mut place = Vec::new();
    let unsaved: Vec<String> = files_of(project, reported)
        .iter()
        .map(|(path, _)| path.clone())
        .filter(|p| context.unsaved.contains(p))
        .collect();
    if !unsaved.is_empty() {
        place.push(format!(
            "Some files have unsaved changes; save them before you start: {}",
            code_list(&unsaved)
        ));
    }
    context.project_lines(&mut place);

    let concise = check_command(&targets, &["--format", "concise"]);
    let files = files_of(project, reported);
    let mut listed = vec!["Problems by file:".to_owned()];
    listed.extend(
        files
            .iter()
            .take(MAX_LISTED)
            .map(|(path, counts)| format!("- {path}: {}", counts.words())),
    );
    if files.len() > MAX_LISTED {
        listed.push(format!("and {} more files.", files.len() - MAX_LISTED));
    }
    listed.push(format!("To list each problem: `{concise}`"));

    Some(assemble(
        &format!(
            "Fix {} `ascribe check` reports in {where_}.",
            the_problems(reported.len())
        ),
        &place,
        &listed,
        &format!("(cut: run `{concise}` for the rest)"),
        &context.finish(&targets),
    ))
}

/// "the problem" or "the N problems".
fn the_problems(n: usize) -> String {
    if n == 1 {
        "the problem".to_owned()
    } else {
        format!("the {n} problems")
    }
}

/// `a`, `b`, as code.
pub fn code_list(items: &[String]) -> String {
    items
        .iter()
        .map(|i| format!("`{i}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn unsaved_line(context: &Context, file: &str, lines: &mut Vec<String>) {
    if context.unsaved.iter().any(|u| u == file) {
        lines.push("The file has unsaved changes; save it before you start.".to_owned());
    }
}

/// `Shown on:` with the pages that include `file`, when it's a fragment;
/// past [`MAX_SHOWN_ON`], the `ascribe refs` command that lists them all.
fn shown_on_line(
    project: &Project,
    context: &Context,
    file: FileId,
    shown_on: &dyn Fn(&RelPath) -> Vec<RelPath>,
    lines: &mut Vec<String>,
) {
    let Some(content) = project.file(file).and_then(|f| f.content_path.cloned()) else {
        return;
    };
    if !project.model().is_fragment(content.as_str()) {
        return;
    }
    let mut pages: Vec<String> = shown_on(&content)
        .into_iter()
        .filter(|p| !project.model().is_fragment(p.as_str()))
        .map(|p| {
            project
                .content_root()
                .join(p.as_str())
                .map_or_else(|_| p.to_string(), |p| p.to_string())
        })
        .collect();
    pages.sort();
    pages.dedup();
    if pages.is_empty() {
        return;
    }
    let more = pages.len().saturating_sub(MAX_SHOWN_ON);
    pages.truncate(MAX_SHOWN_ON);
    let mut line = format!("Shown on: {}", pages.join(", "));
    if more > 0 {
        let refs = format!(
            "ascribe refs {}",
            shell_word(&context.in_repository(&path_of(project, file)))
        );
        line.push_str(&format!(", and {more} more: `{refs}`"));
    }
    lines.push(line);
}

/// The path a prompt shows for a file: relative to the project root.
fn path_of(project: &Project, file: FileId) -> String {
    project
        .display_path(file)
        .unwrap_or_else(|| MODEL_FILE.to_owned())
}

/// The lines a location covers, from 1, and their text.
struct Lines {
    first: usize,
    last: usize,
    text: Vec<String>,
}

fn lines_of(project: &Project, at: Location) -> Option<Lines> {
    let text: String = match project.file(at.file) {
        Some(entry) => entry.text.to_owned(),
        None => project.code_file(at.file)?.text.clone(),
    };
    let index = LineIndex::new(&text);
    let start = index.line_col(at.span.start())?.line;
    // An end just past a line break is still on the line before it.
    let end_offset = at.span.end().max(at.span.start());
    let end_offset = if end_offset > at.span.start() && text[..end_offset].ends_with('\n') {
        end_offset - 1
    } else {
        end_offset
    };
    let end = index
        .line_col(end_offset)
        .map_or(start, |p| p.line)
        .max(start);
    let quoted = (start..=end)
        .take(MAX_QUOTED_LINES)
        .filter_map(|line| index.line_span(line))
        .map(|span| {
            text.get(span.start()..span.end())
                .unwrap_or_default()
                .trim_end_matches(['\r', '\n'])
                .to_owned()
        })
        .collect();
    Some(Lines {
        first: start as usize + 1,
        last: end as usize + 1,
        text: quoted,
    })
}

fn line_range(lines: Option<&Lines>) -> String {
    match lines {
        Some(l) if l.last > l.first => format!("{}-{}", l.first, l.last),
        Some(l) => l.first.to_string(),
        None => "1".to_owned(),
    }
}

/// A problem's lines, fenced, in a fence longer than any run of backticks in
/// them.
fn quoted(lines: &Lines, file: &str) -> String {
    let label = if lines.last > lines.first {
        let shown = lines.first + lines.text.len().saturating_sub(1);
        if shown < lines.last {
            format!(
                "Lines {}-{} (the first {}):",
                lines.first,
                lines.last,
                lines.text.len()
            )
        } else {
            format!("Lines {}-{}:", lines.first, lines.last)
        }
    } else {
        format!("Line {}:", lines.first)
    };
    let body = lines.text.join("\n");
    let mut longest = 0;
    let mut run = 0;
    for c in body.chars() {
        if c == '`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat(longest.max(2) + 1);
    let language = if file.ends_with(".md") {
        "markdown"
    } else if file.ends_with(".toml") {
        "toml"
    } else {
        ""
    };
    format!("{label}\n{fence}{language}\n{body}\n{fence}")
}

/// A report in the order a prompt lists it: by file, then by place.
fn in_order(reported: &[Reported]) -> Vec<&Reported> {
    let mut order: Vec<&Reported> = reported.iter().collect();
    order.sort_by_key(|r| {
        let at = r.diagnostic.location;
        (at.file, at.span.start())
    });
    order
}

/// How many errors and warnings.
#[derive(Clone, Copy, Default)]
struct Counts {
    errors: usize,
    warnings: usize,
}

impl Counts {
    fn add(&mut self, d: &Diagnostic) {
        match d.severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
        }
    }

    fn total(self) -> usize {
        self.errors + self.warnings
    }

    /// "3 errors, 1 warning".
    fn words(self) -> String {
        let plural =
            |n: usize, one: &str, many: &str| format!("{n} {}", if n == 1 { one } else { many });
        let mut parts = Vec::new();
        if self.errors > 0 {
            parts.push(plural(self.errors, "error", "errors"));
        }
        if self.warnings > 0 {
            parts.push(plural(self.warnings, "warning", "warnings"));
        }
        parts.join(", ")
    }
}

/// Each file's counts, most problems first; a tie keeps file order.
fn files_of(project: &Project, reported: &[Reported]) -> Vec<(String, Counts)> {
    let mut by_file: Vec<(FileId, Counts)> = Vec::new();
    for r in in_order(reported) {
        let file = r.diagnostic.location.file;
        match by_file.iter_mut().find(|(f, _)| *f == file) {
            Some((_, counts)) => counts.add(&r.diagnostic),
            None => {
                let mut counts = Counts::default();
                counts.add(&r.diagnostic);
                by_file.push((file, counts));
            }
        }
    }
    by_file.sort_by_key(|(_, counts)| std::cmp::Reverse(counts.total()));
    by_file
        .into_iter()
        .map(|(file, counts)| (path_of(project, file), counts))
        .collect()
}

/// The prompt from its parts, with a blank line between them: the task,
/// where, what Ascribe knows (each item a line, or lines that go together),
/// and how to finish. When it would pass [`LIMIT`], items are left out from
/// the end of what Ascribe knows, and `cut` says so.
pub fn assemble(
    task: &str,
    place: &[String],
    known: &[String],
    cut: &str,
    finish: &[String],
) -> String {
    let build = |known: &[String], cut: Option<&str>| {
        let mut parts = vec![task.to_owned()];
        if !place.is_empty() {
            parts.push(place.join("\n"));
        }
        let mut body: Vec<&str> = known.iter().map(String::as_str).collect();
        body.extend(cut);
        if !body.is_empty() {
            parts.push(body.join("\n"));
        }
        parts.push(finish.join("\n"));
        let mut text = parts.join("\n\n");
        text.push('\n');
        text
    };
    let whole = build(known, None);
    if whole.chars().count() <= LIMIT {
        return whole;
    }
    for kept in (0..known.len()).rev() {
        let text = build(&known[..kept], Some(cut));
        if text.chars().count() <= LIMIT {
            return text;
        }
    }
    // Even the parts that are always there are too long: a path or a
    // message of thousands of characters.
    let mut text: String = build(&[], Some(cut)).chars().take(LIMIT - 1).collect();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::shell_word;

    #[test]
    fn words_are_quoted_only_when_a_shell_would_split_or_expand_them() {
        assert_eq!(
            shell_word("docs/guides/install.md"),
            "docs/guides/install.md"
        );
        assert_eq!(shell_word("my guide.md"), "'my guide.md'");
        assert_eq!(shell_word("it's.md"), r"'it'\''s.md'");
        assert_eq!(shell_word(""), "''");
    }
}
