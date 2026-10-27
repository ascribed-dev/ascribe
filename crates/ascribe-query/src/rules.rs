//! A project's rules for an agent: what the instruction files agents read on
//! their own (`AGENTS.md` and the like) say about one Ascribe project. The
//! check to run, the page types and phrases from the content model, the
//! directives the project's pages use, and where to find more, in at most
//! [`RULES_BUDGET`] characters.
//!
//! [`rules`] gives the whole text, and [`pointer`] a short one for a file
//! above the project's folder that sends an agent to it. Both write their
//! commands to run from the repository's root, where agents run them.

use ascribe_check::prompt::shell_word;
use ascribe_core::RelPath;
use ascribe_core::schema::{Attributes, Builtin, DirectiveSchema, Primary};
use ascribe_model::ContentModel;
use ascribe_resolve::{Committed, FileSystem, Layout, MemoStore, Project};

use crate::model::{Section, entries, fit, type_entries};

/// The most characters [`rules`] takes.
pub const RULES_BUDGET: usize = 4_000;

/// What the rules of a project with `ascribe.addresses` say about moving a
/// page, in a section of their own after the links': `{config}` is where
/// `--config` goes.
pub const MOVING_SENTENCES: &str = "This project keeps the addresses it has published in \
     `ascribe.addresses`, so a reader's old link still leads somewhere. Move or rename a page \
     with `ascribe rename <page> <new>`, which records the move. After moving pages with `git \
     mv`, run `ascribe addresses move{config} --from-git` to record the moves git sees. For a \
     page you delete on purpose, forward its address to the page that covers it now, with \
     `ascribe addresses move{config} <old> <page>`, or retire it, with `ascribe addresses \
     retire{config} <old> --reason \"<why>\"`. `ascribe check` reports `address-lost` for a \
     page that's gone without either.";

/// The most characters the phrases take within the rules.
pub const MODEL_BUDGET: usize = 2_000;

/// What the rules say of memos, in a project whose pages have one.
pub const MEMO_SENTENCE: &str = "Read a block's `@memo` before changing the block, and keep \
     the two together: a memo says why the page is written the way it is, and is never \
     published.";

/// What the rules say of memo files, in a project with one in `memos/`.
pub const MEMO_FILE_SENTENCE: &str = "`ascribe outline <page>` lists the memo files about a \
     page and each of its headings: read a memo before changing the block it's about, and \
     keep it.";

/// What the rules say of permalinks, in a project that declares one: the
/// command that shows a permalink's help, which `{config}` is replaced in
/// with the project's `--config`.
pub const PERMALINK_SENTENCE: &str = "A product's message that links to these docs by a \
     permalink can read what the docs will show for it first: `ascribe resolve \
     go:<name>{config}` prints the help a product shows, from the build's `help.json`.";

/// What the rules say of summaries, in a project with a heading's
/// permalink: that it gets a `@summary`, the text a product shows with its
/// help, and how to choose between the two bindings (SPEC §4.12).
pub const SUMMARY_SENTENCE: &str = "Give a heading with a permalink a summary, a sentence or \
     two a product shows with its help: `@summary` on the line above a paragraph of the \
     section that reads well alone, or `@summary: <text>` under the heading and its `@id` \
     when none does, as for a lead-in to a list. Keep it under 300 characters.";

/// What the rules say of the site form: how to see a page as a reader of the
/// site gets it, and where the elements it holds are explained.
pub const SITE_FORM_SENTENCE: &str = concat!(
    "To see what a reader of the site gets, not only a reader of the Markdown, run `ascribe \
     resolve <page> --build <name> --emit site`, or call `ascribe_resolve` with `emit: \
     \"site\"`: the page as the site output gives it to the site, with Ascribe's elements. \
     How to read them: ",
    ascribe_core::docs_permalink!("reading-site-output"),
);

/// A project's rules, as Markdown, for a file in the folder `dir` is relative
/// to: `dir` is the project's folder (the one holding `ascribe.toml`) as seen
/// from there, and empty when the file is beside `ascribe.toml`. `root` is
/// the project's folder as seen from the repository's root, where the
/// commands are run from. `fs` is the project's [`FileSystem`], which the
/// summary of each type ejected from the set is read through, from its
/// `templates.lock`.
///
/// In a project whose pages have a `@memo`, the rules say to read a block's
/// memo before changing it, in [`MEMO_SENTENCE`]; in one with a memo file
/// in `memos/`, where to find the memos about a page, in
/// [`MEMO_FILE_SENTENCE`], after it. Personal memos don't count: the rules
/// are the same on every machine.
///
/// In a project that declares a permalink, the links section ends with
/// [`PERMALINK_SENTENCE`], and in one that declares a heading's, with
/// [`SUMMARY_SENTENCE`] after it.
///
/// When the whole is longer than [`RULES_BUDGET`], the memo sentences are
/// cut first; then the page types' lines, and the cut says where the full list
/// is; then the directives'.
pub fn rules(project: &Project, fs: &dyn FileSystem, dir: &RelPath, root: &RelPath) -> String {
    let model = project.model();
    let at = At::new(project, dir, root);
    let mut head = String::from("## Ascribe documentation\n\n");
    head.push_str(&at.about());
    head.push_str("\n\n");
    head.push_str(&at.check());
    head.push_str("\n\n");
    head.push_str(&at.search());
    head.push('\n');

    let types = type_entries(model, &ascribe_templates::summaries(model, fs));
    let phrases = phrases_section(model, &at);
    let declared = project.declared_permalinks();
    let mut permalinks = if declared.is_empty() {
        String::new()
    } else {
        format!(" {}", PERMALINK_SENTENCE.replace("{config}", &at.config()))
    };
    if declared.iter().any(|p| p.id.is_some()) {
        permalinks.push(' ');
        permalinks.push_str(SUMMARY_SENTENCE);
    }
    let moving = if project.addresses_file().is_some() {
        format!(
            "\n### Moving pages\n\n{}\n",
            MOVING_SENTENCES.replace("{config}", &at.config())
        )
    } else {
        String::new()
    };
    let tail = format!(
        "\n### Links\n\nLink to the source file, with a relative path such as \
         `../guides/install.md#configure`, not to the site's address. `ascribe outline \
         <page>` lists a page's headings with their ids, and `ascribe link <target> --from \
         <page>` says whether a link works and what to write.{permalinks}\n{moving}\n### More\n\n\
         {SITE_FORM_SENTENCE}\n\n`ascribe model{}` shows everything the content model allows. Every directive: {}\n",
        at.path_arg(),
        directive_reference_url(),
    );
    let used = project.directive_names();
    let mut memo = String::new();
    if used.contains(Builtin::Memo.name()) {
        memo.push_str(&format!("\n{MEMO_SENTENCE}\n"));
    }
    if !Committed::new(fs, project.layout()).memos().is_empty() {
        memo.push_str(&format!("\n{MEMO_FILE_SENTENCE}\n"));
    }
    let lines = directive_lines(model, &used);
    let directives = |n: usize| -> String {
        if lines.is_empty() {
            return String::new();
        }
        let mut out = String::from("\n### Directives this project uses\n\n");
        for line in &lines[..n] {
            out.push_str(&format!("- {line}\n"));
        }
        if n < lines.len() {
            out.push_str(&format!(
                "- …and {} more, in the directive reference below\n",
                lines.len() - n
            ));
        }
        out
    };
    let render = |memo: &str, types_taken: usize, directives_taken: usize| -> String {
        format!(
            "{head}{memo}{}{phrases}{}{tail}",
            section(Section::Types, types_intro(model), &types, types_taken, &at),
            directives(directives_taken)
        )
    };
    let every_directive = lines.len();
    let whole = render(&memo, types.len(), every_directive);
    if whole.chars().count() <= RULES_BUDGET {
        return whole;
    }
    let fits = |t: usize, d: usize| render("", t, d).chars().count() <= RULES_BUDGET;
    let types_taken = fit(&[types.len()], |taken| fits(taken[0], every_directive))[0];
    let directives_taken = if fits(types_taken, every_directive) {
        every_directive
    } else {
        fit(&[lines.len()], |taken| fits(0, taken[0]))[0]
    };
    render("", types_taken, directives_taken)
}

/// A short text for a file above a project's folder (the repository's root
/// `AGENTS.md`, for an agent that reads no file below where it starts): where
/// the project's pages and its full rules are, and the check to run. `dir`
/// is as for [`rules`], and isn't empty; the file is at the repository's
/// root.
pub fn pointer(project: &Project, dir: &RelPath) -> String {
    let at = At::new(project, dir, dir);
    format!(
        "## Ascribe documentation in `{}`\n\n{} This project's rules are in `{}`: read them before \
         editing a page there.\n\n{}\n",
        at.folder(&RelPath::root()),
        at.about(),
        at.path("AGENTS.md"),
        at.check(),
    )
}

/// The glob of the project's pages, as seen from the folder `dir` is
/// relative to, as for [`rules`]: `docs/**/*.md`.
pub fn content_glob(project: &Project, dir: &RelPath) -> String {
    let content = At::new(project, dir, dir).content;
    if content.is_root() {
        "**/*.md".to_owned()
    } else {
        format!("{}/**/*.md", content.as_str())
    }
}

/// The address of the directive reference on the docs site.
pub fn directive_reference_url() -> String {
    ascribe_core::docs_permalink!("directives").to_owned()
}

/// A line of Markdown for each built-in directive, in SPEC §4 order: its
/// syntax and what it's for.
pub fn builtin_lines() -> Vec<String> {
    Builtin::ALL.into_iter().map(builtin_line).collect()
}

/// Where the reader of the rules is, relative to the project.
struct At {
    /// The project's folder.
    dir: RelPath,
    /// The content root.
    content: RelPath,
    /// The project's folder from the repository's root, where commands run.
    root: RelPath,
    /// The folders of other projects nested in the content root, as the
    /// reader writes them.
    nested: Vec<RelPath>,
}

impl At {
    fn new(project: &Project, dir: &RelPath, root: &RelPath) -> At {
        let layout: &Layout = project.layout();
        let content = dir
            .join(layout.content_root.as_str())
            .unwrap_or_else(|_| dir.clone());
        let nested = project
            .nested_projects()
            .iter()
            .map(|n| content.join(n.as_str()).unwrap_or_else(|_| n.clone()))
            .collect();
        At {
            dir: dir.clone(),
            content,
            root: root.clone(),
            nested,
        }
    }

    /// A path in the project's folder, as the reader writes it.
    fn path(&self, name: &str) -> String {
        self.dir
            .join(name)
            .map(|p| p.as_str().to_owned())
            .unwrap_or_else(|_| name.to_owned())
    }

    /// A folder as the reader writes it, with a trailing `/`: `./` for the
    /// reader's own.
    fn folder(&self, path: &RelPath) -> String {
        let path = self
            .dir
            .join(path.as_str())
            .unwrap_or_else(|_| path.clone());
        if path.is_root() {
            "./".to_owned()
        } else {
            format!("{}/", path.as_str())
        }
    }

    /// The project's folder as a command's argument, with a leading space,
    /// or nothing when it's the repository's root.
    fn path_arg(&self) -> String {
        if self.root.is_root() {
            String::new()
        } else {
            format!(" {}", shell_word(self.root.as_str()))
        }
    }

    /// The first paragraph: what the pages are and where, and the folders in
    /// it that are other projects', whose rules these aren't.
    fn about(&self) -> String {
        let content = if self.content.is_root() {
            "./".to_owned()
        } else {
            format!("{}/", self.content.as_str())
        };
        let mut out = format!(
            "The Markdown pages under `{content}` are Ascribe documentation: pages with YAML \
             frontmatter, `@` directives, and `{{key}}` phrases, checked against the content \
             model in `{}`.",
            self.path("ascribe.toml")
        );
        let folders: Vec<String> = self
            .nested
            .iter()
            .map(|n| format!("`{}/`", n.as_str()))
            .collect();
        if let Some((last, rest)) = folders.split_last() {
            let (list, which) = if rest.is_empty() {
                (last.clone(), "that folder is")
            } else {
                (format!("{} and {last}", rest.join(", ")), "each folder is")
            };
            out.push_str(&format!(
                " The pages under {list} aren't: {which} another project's, with its own \
                 `ascribe.toml` and rules."
            ));
        }
        out
    }

    /// The loop, as a check to run.
    fn check(&self) -> String {
        let whole = format!("ascribe check{}", self.config());

        format!(
            "After editing a page, run `ascribe check <file> --format \
             concise` and fix every error. Before you finish, run `{whole}`. `ascribe explain \
             <code>` says what a problem means and how to fix it."
        )
    }

    /// Searching before writing a page.
    fn search(&self) -> String {
        format!(
            "Before writing a page about a subject, run `ascribe search \"<words>\"{}` to find \
             where the pages already say it, as readers read them, and link to what's there or \
             extend it instead of saying it again.",
            self.config()
        )
    }

    /// ` --config <folder>` for a command run from the repository's root,
    /// when the project isn't there; empty when it is.
    fn config(&self) -> String {
        if self.root.is_root() {
            String::new()
        } else {
            format!(" --config {}", shell_word(self.root.as_str()))
        }
    }
}

/// What the page types' section says before its lines: in a project that
/// declares types, that a page names its own (SPEC §7.2).
fn types_intro(model: &ContentModel) -> &'static str {
    if model.declares_types() {
        "A page says what type it is with `type:` in its frontmatter, first, and its other \
         fields are its type's. A file or folder whose name starts with `_` is a fragment, \
         which pages `@include`, not a page, and names no type."
    } else {
        "A page's frontmatter is its type's. A file or folder whose name starts with `_` is a \
         fragment, which pages `@include`, not a page."
    }
}

/// The phrases, cut to [`MODEL_BUDGET`].
fn phrases_section(model: &ContentModel, at: &At) -> String {
    let lines = entries(model, Section::Phrases);
    let intro = model
        .phrases
        .first()
        .map(|p| {
            format!(
                "Write the phrase (`{{{}}}`), not the text it stands for.",
                p.key
            )
        })
        .unwrap_or_default();
    let render = |n: usize| section(Section::Phrases, &intro, &lines, n, at);
    let taken = fit(&[lines.len()], |taken| {
        render(taken[0]).chars().count() <= MODEL_BUDGET
    });
    render(taken[0])
}

/// A section of the content model: its heading, `intro`, and its first `n`
/// lines, then where the rest are when that isn't all of them. Nothing when
/// it has no lines.
fn section(section: Section, intro: &str, lines: &[String], n: usize, at: &At) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let mut out = format!("\n### {}\n\n", section.heading());
    if !intro.is_empty() {
        out.push_str(&format!("{intro}\n\n"));
    }
    for line in &lines[..n] {
        out.push_str(&format!("- {line}\n"));
    }
    if n < lines.len() {
        out.push_str(&format!(
            "- …and {} more: `ascribe model{} --section {}`\n",
            lines.len() - n,
            at.path_arg(),
            section.name()
        ));
    }
    out
}

/// A line for each directive the project's pages use: the built-ins in
/// SPEC §4 order, with what this project declares for them, then its
/// widgets in declaration order. A name that's neither is left out.
fn directive_lines(model: &ContentModel, used: &std::collections::BTreeSet<String>) -> Vec<String> {
    let code = |names: Vec<&str>| -> String {
        names
            .into_iter()
            .map(|n| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut lines: Vec<String> = Builtin::ALL
        .into_iter()
        .filter(|b| used.contains(b.name()))
        .map(|b| {
            let line = match b {
                Builtin::Available => available_line(Some(&available_example(model))),
                _ => builtin_line(b),
            };
            let declared = match b {
                Builtin::Variant if !model.dimensions.is_empty() => Some(format!(
                    "dimensions {}",
                    model
                        .dimensions
                        .iter()
                        .map(dimension_values)
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                Builtin::Note => {
                    let types: Vec<&str> = model.notes.iter().map(|n| n.name.as_str()).collect();
                    (!types.is_empty()).then(|| format!("types {}", code(types)))
                }
                Builtin::Snippet if !model.sources.is_empty() => Some(format!(
                    "sources {}",
                    code(model.sources.iter().map(|s| s.name.as_str()).collect())
                )),
                Builtin::Operation if !model.apis.is_empty() => Some(format!(
                    "APIs {}",
                    code(model.apis.iter().map(|a| a.name.as_str()).collect())
                )),
                _ => None,
            };
            match declared {
                Some(declared) => format!("{line}. Here: {declared}"),
                None => line,
            }
        })
        .collect();
    lines.extend(
        model
            .widgets
            .iter()
            .filter(|w| used.contains(&w.schema.name))
            .map(|w| widget_line(&w.schema)),
    );
    lines
}

fn builtin_line(builtin: Builtin) -> String {
    match builtin {
        Builtin::Id => "`@id: <id>` on the line under a heading: an id for links to it that \
                        doesn't change with its text"
            .to_owned(),
        Builtin::Include => "`@include: <file>.md`, or `@include: <file>.md#<id>` for one \
                             section: another file's content, in place"
            .to_owned(),
        Builtin::Variant => "`@variant {<dimension>=<value>}:` before each arm, and one `@end` \
                             after the last: content that differs by a dimension"
            .to_owned(),
        Builtin::Available => available_line(None),
        Builtin::Note => "`@note {type=<type>}: <text>`, or `@note:` … `@end` around several \
                          blocks: a callout"
            .to_owned(),
        Builtin::Steps => {
            "`@steps` on the line above an ordered list: the list is a procedure".to_owned()
        }
        Builtin::Details => "`.<Title>` on its own line, then `@details:` … `@end`: collapsible \
                             content; the title is required"
            .to_owned(),
        Builtin::Snippet => "`@snippet: <source>:<path>#<region>`: a code example from a file \
                             outside the pages"
            .to_owned(),
        Builtin::Intended => "`@intended {check=<check>}: <reason>` above a block: a review \
                               check's problem in it is intended; write one only when the \
                               user says the problem is intended"
            .to_owned(),
        Builtin::Memo => "`@memo: <text>` under a heading or above a block, or `@memo:` … \
                          `@end` around several lines: a note for whoever maintains the page, \
                          never published"
            .to_owned(),
        Builtin::Term => "`@term` on the line under a heading of a glossary page, with \
                          `{aliases=\"<a>|<b>\"}` for other forms: the heading is a glossary \
                          term, defined by the paragraph under it"
            .to_owned(),
        Builtin::Summary => "`@summary: <text>` under a heading, or `@summary` on the line \
                             above a paragraph: the section's summary, its own text, which \
                             the page doesn't show, or the paragraph, which it does"
            .to_owned(),
        Builtin::Operation => "`@operation: <spec>#<operationId>`, a link to an operation of an \
                               API `[apis]` declares: its method, path, and summary, linked to \
                               its reference"
            .to_owned(),
    }
}

/// The line for `@available`, with `example`, a spec, after its targets; or,
/// with none, the language's own example.
fn available_line(example: Option<&str>) -> String {
    let example = match example {
        Some("") => String::new(),
        Some(spec) => format!(" (`{spec}`)"),
        None => " (`cloud, self-managed preview 3.4`)".to_owned(),
    };
    format!(
        "`@available: <spec>` under a heading or above a block: where it applies, as targets \
         with an optional state and version{example}, or a feature's key"
    )
}

/// A spec of the project's own targets: two values of the dimension
/// availability is most likely about, the first with a value that has
/// release lines or is versionless, or else the first dimension. Empty when
/// it has no dimension values.
fn available_example(model: &ContentModel) -> String {
    let versioned = |d: &&ascribe_model::Dimension| {
        d.implicit
            || d.values
                .iter()
                .any(|v| v.versionless || model.releases.iter().any(|r| r.target == v.value))
    };
    let dimension = model
        .dimensions
        .iter()
        .find(versioned)
        .or_else(|| model.dimensions.first());
    dimension.map_or_else(String::new, |d| {
        d.values
            .iter()
            .take(2)
            .map(|v| v.value.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    })
}

/// The most values a dimension's line names.
const VALUES_SHOWN: usize = 6;

/// A dimension and its values, as the directives' lines name them:
/// `` `pm` (`npm`, `pnpm`) ``, its first [`VALUES_SHOWN`] values and `…` when
/// there are more.
fn dimension_values(dimension: &ascribe_model::Dimension) -> String {
    let mut values: Vec<String> = dimension
        .values
        .iter()
        .take(VALUES_SHOWN)
        .map(|v| format!("`{}`", v.value))
        .collect();
    if dimension.values.len() > VALUES_SHOWN {
        values.push("…".to_owned());
    }
    if values.is_empty() {
        return format!("`{}`", dimension.name);
    }
    format!("`{}` ({})", dimension.name, values.join(", "))
}

fn widget_line(schema: &DirectiveSchema) -> String {
    let mut syntax = format!("@{}", schema.name);
    if let Attributes::Declared(attributes) = &schema.attributes
        && !attributes.is_empty()
    {
        let keys: Vec<String> = attributes.iter().map(|a| format!("{}=…", a.key)).collect();
        syntax.push_str(&format!(" {{{}}}", keys.join(", ")));
    }
    let primary = match schema.primary {
        Primary::None => "",
        Primary::Identifier { .. } => ": <id>",
        Primary::Text { .. } => ": <text>",
        Primary::Availability { .. } => ": <spec>",
        Primary::Reason { .. } => ": <reason>",
    };
    let mut line = match (schema.forms.line, schema.forms.container) {
        (_, false) => format!("`{syntax}{primary}`"),
        (false, true) => format!("`{syntax}:` … `@end`"),
        (true, true) => format!("`{syntax}{primary}`, or `{syntax}:` … `@end`"),
    };
    if let Some(description) = &schema.description {
        line.push_str(&format!(": {}", description.trim_end_matches('.')));
    }
    line
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::sync::Arc;

    use ascribe_core::FileId;
    use ascribe_resolve::MemoryFs;

    use super::*;

    const SIX: &str = include_str!("../../../examples/templates/ascribe.toml");
    const SIX_LOCK: &str = include_str!("../../../examples/templates/templates/templates.lock");

    /// The rules for a project with the content model `model`, and the
    /// files `files` beside it, read from the repository's root.
    fn rules_of(model: &str, files: &[(&str, &str)]) -> String {
        let mut model = ascribe_model::load_str(model, FileId::new(0)).unwrap();
        let layout = Layout::from_model(&model);
        let fs = files
            .iter()
            .fold(MemoryFs::new(&layout), |fs, (path, text)| {
                fs.with_file(path, text)
            });
        // The types take their templates from the lock, as when the project
        // is loaded from its folder.
        ascribe_check::type_files::from_lock(&mut model, &fs);
        let project = Project::load(Arc::new(model), layout, &fs);
        rules(&project, &fs, &RelPath::root(), &RelPath::root())
    }

    /// The line of the rules for the type `name`.
    fn line<'a>(rules: &'a str, name: &str) -> &'a str {
        let start = format!("- `{name}`");
        rules
            .lines()
            .find(|l| {
                l.strip_prefix(&start)
                    .is_some_and(|rest| rest.starts_with([' ', ':']))
            })
            .unwrap_or_else(|| panic!("no line for `{name}`:\n{rules}"))
    }

    #[test]
    fn a_project_with_the_six_types_says_what_each_is_for_within_the_budget() {
        let rules = rules_of(SIX, &[("templates/templates.lock", SIX_LOCK)]);
        assert!(rules.chars().count() <= RULES_BUDGET, "{rules}");
        assert!(
            !rules.contains("more: `ascribe model --section types`"),
            "{rules}"
        );
        assert!(
            line(&rules, "how-to").ends_with(
                ". A how-to is a concise set of numbered steps to do one task with the product"
            ),
            "{rules}"
        );
        for name in [
            "concept",
            "tutorial",
            "reference",
            "troubleshooting",
            "release-notes",
        ] {
            let summary = SIX_LOCK
                .split("[[template]]")
                .find(|entry| entry.contains(&format!("name = \"{name}\"")))
                .and_then(|entry| entry.lines().find_map(|l| l.strip_prefix("summary = \"")))
                .map(|s| s.trim_end_matches('"').trim_end_matches('.'))
                .unwrap();
            assert!(line(&rules, name).ends_with(summary), "{name}:\n{rules}");
        }
        assert_eq!(
            line(&rules, "page"),
            "- `page`: frontmatter `title` (string, required)"
        );
    }

    #[test]
    fn a_type_s_purpose_comes_before_the_summary() {
        let model = SIX.replacen(
            "[types.how-to]\n",
            "[types.how-to]\npurpose = \"One task, step by step.\"\n",
            1,
        );
        let rules = rules_of(&model, &[("templates/templates.lock", SIX_LOCK)]);
        let how_to = line(&rules, "how-to");
        assert!(how_to.ends_with(". One task, step by step"), "{rules}");
        assert!(!how_to.contains("concise set"), "{rules}");
    }

    #[test]
    fn without_a_lock_a_type_from_the_set_says_nothing_of_what_it_is_for() {
        let rules = rules_of(SIX, &[]);
        assert!(
            line(&rules, "how-to").ends_with("`description` (string, required)"),
            "{rules}"
        );
    }

    #[test]
    fn a_purpose_over_several_lines_is_one_line_of_the_rules() {
        let model = SIX.replacen(
            "[types.how-to]\n",
            "[types.how-to]\npurpose = \"\"\"\nOne task,\n- step by step.\n\"\"\"\n",
            1,
        );
        let rules = rules_of(&model, &[]);
        assert!(
            line(&rules, "how-to").ends_with(". One task, - step by step"),
            "{rules}"
        );
        assert!(!rules.contains("\n- step by step"), "{rules}");
    }

    #[test]
    fn a_project_with_an_addresses_file_is_told_how_to_move_pages() {
        let model = "spec = \"0.1\"\n";
        let page = "---\ntitle: Keys\n---\n\nRotate the key.\n";
        let rules = rules_of(
            model,
            &[
                ("docs/keys.md", page),
                ("ascribe.addresses", "version = 1\n"),
            ],
        );
        assert!(rules.contains("\n### Moving pages\n"), "{rules}");
        assert!(
            rules.contains("`ascribe addresses move --from-git`"),
            "{rules}"
        );
        let rules = rules_of(model, &[("docs/keys.md", page)]);
        assert!(!rules.contains("Moving pages"), "{rules}");
    }

    #[test]
    fn a_project_with_a_memo_is_told_to_keep_it() {
        let model = "spec = \"0.1\"\n";
        let page = "---\ntitle: Keys\n---\n\n@memo: Kept short.\nRotate the key.\n";
        let rules = rules_of(model, &[("docs/keys.md", page)]);
        assert!(rules.contains(MEMO_SENTENCE), "{rules}");
        let plain = "---\ntitle: Keys\n---\n\nRotate the key.\n";
        let rules = rules_of(model, &[("docs/keys.md", plain)]);
        assert!(!rules.contains("@memo"), "{rules}");
    }

    #[test]
    fn a_project_with_a_memo_file_is_told_where_a_pages_memos_are() {
        let model = "spec = \"0.1\"\n";
        let plain = "---\ntitle: Keys\n---\n\nRotate the key.\n";
        let memo = "---\nabout: keys.md\n---\n\nKept short.\n";
        let rules = rules_of(model, &[("docs/keys.md", plain), ("memos/keys.md", memo)]);
        assert!(rules.contains(MEMO_FILE_SENTENCE), "{rules}");
        assert!(!rules.contains(MEMO_SENTENCE), "{rules}");
        // A memo kept on one machine doesn't change the rules.
        let rules = rules_of(
            model,
            &[("docs/keys.md", plain), (".ascribe/memos/keys.md", memo)],
        );
        assert!(!rules.contains("memo"), "{rules}");
    }

    #[test]
    fn the_memo_sentences_are_cut_before_the_type_lines() {
        // Types whose lines take the budget but for a few characters more
        // than the memo sentences leave.
        let mut model = String::from("spec = \"0.1\"\n");
        let page = "---\ntype: kind-0\ntitle: Keys\n---\n\n@memo: Kept short.\nRotate the key.\n";
        let mut i = 0;
        loop {
            let with = format!(
                "{model}\n[types.kind-{i}]\npurpose = \"A page about one kind of thing.\"\n\n\
                 [types.kind-{i}.frontmatter]\ntitle = \"string\"\n"
            );
            let memo = "---\nabout: keys.md\n---\n\nKept short.\n";
            let rules = rules_of(&with, &[("docs/keys.md", page), ("memos/keys.md", memo)]);
            assert_eq!(
                rules.contains(MEMO_SENTENCE),
                rules.contains(MEMO_FILE_SENTENCE),
                "{rules}"
            );
            if !rules.contains(MEMO_SENTENCE) {
                // The sentences went, and every type line stayed.
                assert!(rules.chars().count() <= RULES_BUDGET, "{rules}");
                assert!(
                    !rules.contains("more: `ascribe model --section types`"),
                    "{rules}"
                );
                assert_eq!(
                    rules.lines().filter(|l| l.starts_with("- `kind-")).count(),
                    i + 1,
                    "{rules}"
                );
                return;
            }
            model = with;
            i += 1;
            assert!(i < 100, "the rules never ran over the budget");
        }
    }

    #[test]
    fn the_type_lines_are_cut_first_and_say_where_the_rest_are() {
        let mut model = String::from("spec = \"0.1\"\n");
        for i in 0..30 {
            model.push_str(&format!(
                "\n[types.kind-{i}]\npurpose = \"A page about the \
                 {i}th kind of thing this product does, with every detail a reader asks for.\"\n\n\
                 [types.kind-{i}.frontmatter]\ntitle = \"string\"\n"
            ));
        }
        model.push_str("\n[phrases]\nproduct = \"Lantern\"\n");
        let page = "---\ntype: kind-0\ntitle: Steps\n---\n\n@steps\n1. One.\n2. Two.\n";
        let rules = rules_of(&model, &[("docs/kind-0/steps.md", page)]);
        assert!(rules.chars().count() <= RULES_BUDGET, "{rules}");
        let kept = rules.lines().filter(|l| l.starts_with("- `kind-")).count();
        assert!(kept > 0 && kept < 30, "{kept} kept:\n{rules}");
        assert!(
            rules.contains(&format!(
                "- …and {} more: `ascribe model --section types`",
                30 - kept
            )),
            "{rules}"
        );
        // What's after the types is whole.
        assert!(rules.contains("- `{product}`: Lantern"), "{rules}");
        assert!(rules.contains("- `@steps` on the line above"), "{rules}");
    }
}
