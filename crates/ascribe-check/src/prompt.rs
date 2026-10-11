//! Agent prompts about problems: the text **Prompt agent** puts where a person
//! sends it to their agent, and `ascribe check --format prompt` writes.
//!
//! Three prompts, each built from what a check reported:
//!
//! - [`problem`]: one diagnostic, with its line, its fix advice, the
//!   evidence its registry entry names (such as the values the content model
//!   allows, when it's about one), and its fixes;
//! - [`file`]: a file's problems, one line each, at most [`MAX_LISTED`];
//! - [`project`]: how many problems each file has, at most [`MAX_LISTED`]
//!   files;
//! - [`batch`]: the findings of one of `ascribe report`'s sections, at most
//!   [`MAX_LISTED`], each with the evidence its registry entry names.
//!
//! They follow the agent prompt format (`project-docs/agents/README.md`):
//! the task, where, what Ascribe knows, how to finish; plain text with
//! Markdown, at most [`LIMIT`] characters, and the same prompt for the same
//! input. A prompt points rather than dumps: it names the file, the line,
//! and the command that gives the rest.

use std::path::{Path, PathBuf};

use ascribe_core::path::{normalize, relative_path};
use ascribe_core::{Applicability, FileId, LineIndex, Location, RelPath};

use crate::evidence;
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

/// What a prompt says besides its problems: which folder its paths and
/// commands are from, where the project is, which builds were checked, and
/// which files have unsaved changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Context {
    /// The folder every path and command in the prompt is from, which the
    /// prompt names once.
    pub base: Base,
    /// The project's folder from the base, `/`-separated, when it isn't the
    /// base: `docs`.
    pub folder: Option<String>,
    /// The `AGENTS.md` the agent follows, from the base: the project
    /// folder's, or else the repository's. `None` when neither has one.
    pub agents: Option<String>,
    /// The builds checked.
    pub builds: Builds,
    /// The files with unsaved changes, relative to the project root, as a
    /// diagnostic's file is shown.
    pub unsaved: Vec<String>,
    /// The command that runs Ascribe where the agent is, when `ascribe`
    /// isn't it: the path of the binary the editor runs, which the editor
    /// sends when `ascribe` on its path is another binary or none. `None`:
    /// `ascribe`.
    pub program: Option<String>,
    /// The diagnostic codes the check was narrowed to, which every
    /// `ascribe check` the prompt names keeps: `ASC036`. Empty: every code.
    pub codes: Vec<String>,
}

/// The folder a prompt's paths and commands are from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Base {
    /// The root of the repository the project is in.
    Repository,
    /// A folder in the repository, from its root, `/`-separated: where a
    /// command that wrote the prompt was run.
    InRepository(String),
    /// The project's folder, when there's no repository around it.
    Project,
}

impl Base {
    /// The line that names it.
    fn line(&self) -> String {
        let folder = match self {
            Base::Repository => "the repository's root".to_owned(),
            Base::InRepository(folder) => format!("`{folder}` in the repository"),
            Base::Project => "the project's folder, where `ascribe.toml` is".to_owned(),
        };
        format!("Paths and commands here are from {folder}.")
    }
}

impl Context {
    /// The context of a project whose folder (the directory of
    /// `ascribe.toml`) is `root`, for an agent started at the root of its
    /// repository: the nearest folder at or above it with a `.git`. With no
    /// repository, it's for an agent in the project's folder. Its
    /// `AGENTS.md` is looked for in the project's folder, then the
    /// repository's.
    pub fn of_project(root: &Path, builds: Builds) -> Context {
        Context::seen_from(root, None, builds)
    }

    /// The context of a project whose folder is `root`, for an agent in
    /// `here`, the folder the command that writes the prompt was run in.
    /// When `here` is inside the project's folder, the prompt is from the
    /// project's folder; when it's a folder beside the project's, outside
    /// the repository, or there's no repository, it's as
    /// [`Context::of_project`] has it.
    pub fn seen_from(root: &Path, here: Option<&Path>, builds: Builds) -> Context {
        let absolute =
            |p: &Path| normalize(&std::path::absolute(p).unwrap_or_else(|_| p.to_owned()));
        let root = absolute(root);
        let repository = repository_of(&root);
        let top = repository.clone().unwrap_or_else(|| root.clone());
        let inside = |from: &Path, to: &Path| relative_path(from, to).filter(RelPath::is_inside);
        let base_dir = here
            .map(absolute)
            .filter(|here| inside(&top, here).is_some())
            .and_then(|here| {
                if inside(&root, &here).is_some() {
                    Some(root.clone())
                } else if inside(&here, &root).is_some() {
                    Some(here)
                } else {
                    // A folder beside the project, such as `repo/other`
                    // for a project in `repo/docs`: its paths would climb
                    // out of it, so they're written from the repository's
                    // root instead.
                    None
                }
            })
            .unwrap_or_else(|| top.clone());
        let base = match inside(&top, &base_dir) {
            Some(rel) if !rel.is_root() => Base::InRepository(rel.to_string()),
            _ if repository.is_some() => Base::Repository,
            _ => Base::Project,
        };
        let folder = inside(&base_dir, &root)
            .filter(|rel| !rel.is_root())
            .map(|rel| rel.to_string());
        let agents = if has_agents_md(&root) {
            Some(within(folder.as_deref(), "AGENTS.md"))
        } else if repository.as_deref().is_some_and(has_agents_md) {
            let up = match &base {
                Base::InRepository(folder) => "../".repeat(folder.split('/').count()),
                _ => String::new(),
            };
            Some(format!("{up}AGENTS.md"))
        } else {
            None
        };
        Context {
            base,
            folder,
            agents,
            builds,
            unsaved: Vec::new(),
            program: None,
            codes: Vec::new(),
        }
    }

    /// The same context, with these files unsaved.
    #[must_use]
    pub fn with_unsaved(mut self, unsaved: Vec<String>) -> Context {
        self.unsaved = unsaved;
        self
    }

    /// The same context, with Ascribe run as `program`; `None` for
    /// `ascribe`.
    #[must_use]
    pub fn with_program(mut self, program: Option<String>) -> Context {
        self.program = program;
        self
    }

    /// The same context, for a check narrowed to `codes`.
    #[must_use]
    pub fn with_codes(mut self, codes: Vec<String>) -> Context {
        self.codes = codes;
        self
    }

    /// The first word of every command in the prompt: `ascribe`, or the
    /// program as a shell reads it.
    pub fn ascribe(&self) -> String {
        self.program
            .as_deref()
            .map_or_else(|| "ascribe".to_owned(), shell_word)
    }

    /// A path relative to the project root, as the prompt writes it: from
    /// its base.
    pub fn shown(&self, path: &str) -> String {
        within(self.folder.as_deref(), path)
    }

    /// The line that names the base, then the `Project:` and `Build:` lines.
    pub fn project_lines(&self, lines: &mut Vec<String>) {
        lines.push(self.base.line());
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
                "Build: `{name}`, the editor's. `{} check` checks every build.",
                self.ascribe()
            )),
        }
    }

    /// The two lines that end every prompt: the agent's instructions, and
    /// checking `targets` (from the base).
    fn finish(&self, targets: &[String]) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(agents) = &self.agents {
            lines.push(format!("Follow the project's rules in `{agents}`."));
        }
        lines.push(format!(
            "When you're done, run `{}` and fix what it reports.",
            self.check_command(targets, &[])
        ));
        lines
    }

    /// The lines that end a prompt whose work `command` verifies, from the
    /// base.
    fn finish_with(&self, command: &str) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(agents) = &self.agents {
            lines.push(format!("Follow the project's rules in `{agents}`."));
        }
        lines.push(format!(
            "When you're done, run `{command}` and fix what it reports."
        ));
        lines
    }

    /// `ascribe check` on `targets` (from the base), narrowed to the
    /// context's codes, with `options` after them.
    fn check_command(&self, targets: &[String], options: &[&str]) -> String {
        let mut words = vec![self.ascribe(), "check".to_owned()];
        words.extend(targets.iter().map(|t| shell_word(t)));
        for code in &self.codes {
            words.push("--code".to_owned());
            words.push(shell_word(code));
        }
        words.extend(options.iter().map(|o| (*o).to_owned()));
        words.join(" ")
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
    let in_project = path_of(project, d.location.file);
    let file = context.shown(&in_project);
    let lines = lines_of(project, d.location);

    let mut place = vec![format!("Where: {file}:{}", line_range(lines.as_ref()))];
    unsaved_line(context, &in_project, &mut place);
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
                Severity::Advice => "advice",
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
    for evidence in &d.evidence {
        if evidence.text.contains('\n') {
            known.push(format!("{}:\n{}", evidence.label, evidence.text));
        } else {
            known.push(format!("{}: {}", evidence.label, evidence.text));
        }
    }
    let registered = Registry::global().get(d.slug);
    // What only a prompt needs is gathered now, not when the check ran.
    for name in registered
        .map(|e| e.evidence.as_slice())
        .unwrap_or_default()
    {
        known.extend(evidence::lines(project, d, name));
    }
    if let Some(help) = registered.and_then(|e| e.fix.as_deref()) {
        known.push(format!("How to fix it: {help}"));
    }
    if let Some(place) = registered.and_then(|e| e.place) {
        known.push(format!(
            "It may be intended. Only when the user says it is, acknowledge it {} with the reason they give, instead of fixing it.",
            place.written()
        ));
    }
    for fix in &d.fixes {
        known.push(match fix.applicability {
            Applicability::Safe => format!("Ascribe has a safe automatic fix: `{}`.", fix.title),
            Applicability::Unsafe => format!("Ascribe has a fix to review: `{}`.", fix.title),
        });
    }
    for related in d.related.iter().take(MAX_RELATED) {
        let at = context.shown(&path_of(project, related.location.file));
        let line = lines_of(project, related.location).map_or(1, |l| l.first);
        known.push(format!("Related: {at}:{line}: {}", related.message));
    }
    if let Some(docs) = registered.map(crate::Entry::docs) {
        known.push(format!("Reference: {docs}"));
    }
    if let Some(lines) = &lines {
        known.push(quoted(lines, &file));
    }

    let target = file.clone();
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
    let in_project = path_of(project, file);
    let path = context.shown(&in_project);
    let mut place = vec![format!("Where: {path}")];
    unsaved_line(context, &in_project, &mut place);
    context.project_lines(&mut place);
    shown_on_line(project, context, file, shown_on, &mut place);

    let target = path.clone();
    let rest = context.check_command(std::slice::from_ref(&target), &["--format", "concise"]);
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
                format!(
                    "{}:{line}",
                    context.shown(&path_of(project, d.location.file))
                )
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
        paths.iter().map(|p| context.shown(p.as_str())).collect()
    };
    let where_ = if paths.is_empty() {
        "this project".to_owned()
    } else {
        code_list(&targets)
    };

    let mut place = Vec::new();
    let unsaved: Vec<String> = files_of(project, reported)
        .iter()
        .map(|(path, _)| path.clone())
        .filter(|p| context.unsaved.contains(p))
        .map(|p| context.shown(&p))
        .collect();
    if !unsaved.is_empty() {
        place.push(format!(
            "Some files have unsaved changes; save them before you start: {}",
            code_list(&unsaved)
        ));
    }
    context.project_lines(&mut place);

    let concise = context.check_command(&targets, &["--format", "concise"]);
    let files = files_of(project, reported);
    let mut listed = vec!["Problems by file:".to_owned()];
    listed.extend(
        files
            .iter()
            .take(MAX_LISTED)
            .map(|(path, counts)| format!("- {}: {}", context.shown(path), counts.words())),
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

/// A prompt of another kind in the same format, for the named prompts of
/// `ascribe agents prompt`: `task`; then `place`, followed by the
/// `Project:` and `Build:` lines; then what Ascribe knows, `known`, cut from
/// the end with the line `cut` when the prompt would be too long; then the
/// two lines that end every prompt, checking `targets` (from the base).
pub fn compose(
    context: &Context,
    task: &str,
    mut place: Vec<String>,
    known: &[String],
    cut: &str,
    targets: &[String],
) -> String {
    context.project_lines(&mut place);
    assemble(task, &place, known, cut, &context.finish(targets))
}

/// The prompt about a batch of findings that aren't `ascribe check`'s,
/// such as the broken links `ascribe report links` finds: `task`; then
/// `place`, followed by the `Project:` and `Build:` lines; then each
/// finding, at most [`MAX_LISTED`], with the evidence it carries, and how to
/// fix each kind; then the lines that end every prompt, with `verify`, the
/// command that checks the work. A finding about no place in the source
/// (the published site's) is listed without one. `None` when there are no
/// findings.
pub fn batch(
    project: &Project,
    context: &Context,
    task: &str,
    mut place: Vec<String>,
    findings: &[&Diagnostic],
    verify: &str,
) -> Option<String> {
    if findings.is_empty() {
        return None;
    }
    context.project_lines(&mut place);
    let mut known = Vec::new();
    let mut kinds: Vec<ascribe_core::DiagnosticSlug> = Vec::new();
    for d in findings.iter().take(MAX_LISTED) {
        let at = if d.location.file == FileId::new(0) && d.location.span.is_empty() {
            String::new()
        } else {
            let file = context.shown(&path_of(project, d.location.file));
            let line = lines_of(project, d.location).map_or(1, |l| l.first);
            format!("{file}:{line}: ")
        };
        let mut item = format!("- {at}[{}] {}", d.code, d.message);
        for evidence in &d.evidence {
            item.push_str(&format!(
                "\n  {}: {}",
                evidence.label,
                evidence.text.replace('\n', "\n  ")
            ));
        }
        known.push(item);
        if !kinds.contains(&d.slug) {
            kinds.push(d.slug);
        }
    }
    if findings.len() > MAX_LISTED {
        known.push(format!(
            "and {} more: `{verify}` lists them.",
            findings.len() - MAX_LISTED
        ));
    }
    for slug in kinds {
        let Some(entry) = Registry::global().get(slug) else {
            continue;
        };
        if let Some(fix) = &entry.fix {
            known.push(format!("How to fix `{slug}`: {fix}"));
        }
        if let Some(place) = entry.place {
            known.push(format!(
                "A `{slug}` finding may be intended. Only when the user says it is, acknowledge it {} with the reason they give, instead of fixing it.",
                place.written()
            ));
        }
    }
    Some(assemble(
        task,
        &place,
        &known,
        &format!("(cut: run `{verify}` for the rest)"),
        &context.finish_with(verify),
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
            let in_project = project
                .content_root()
                .join(p.as_str())
                .map_or_else(|_| p.to_string(), |p| p.to_string());
            context.shown(&in_project)
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
            "{} refs {}",
            context.ascribe(),
            shell_word(&context.shown(&path_of(project, file)))
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
        // A file without text, such as an image, has no lines to quote.
        Some(entry) if entry.text.is_empty() => return None,
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

/// How many errors, warnings, and advice.
#[derive(Clone, Copy, Default)]
struct Counts {
    errors: usize,
    warnings: usize,
    advice: usize,
}

impl Counts {
    fn add(&mut self, d: &Diagnostic) {
        match d.severity {
            Severity::Error => self.errors += 1,
            Severity::Warning => self.warnings += 1,
            Severity::Advice => self.advice += 1,
        }
    }

    fn total(self) -> usize {
        self.errors + self.warnings + self.advice
    }

    /// "3 errors, 1 warning, 2 advice".
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
        if self.advice > 0 {
            parts.push(plural(self.advice, "advice", "advice"));
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
