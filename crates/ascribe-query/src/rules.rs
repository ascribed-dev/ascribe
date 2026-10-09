//! A project's rules for an agent: what the instruction files agents read on
//! their own (`AGENTS.md` and the like) say about one Ascribe project. The
//! check to run, the page types and phrases from the content model, the
//! directives the project's pages use, and where to find more, in at most
//! [`RULES_BUDGET`] characters.
//!
//! [`rules`] gives the whole text, and [`pointer`] a short one for a file
//! above the project's folder that sends an agent to it. Both say which
//! commands to run from the folder of the file they're written in.

use ascribe_core::RelPath;
use ascribe_core::schema::{Attributes, Builtin, DirectiveSchema, Primary};
use ascribe_model::ContentModel;
use ascribe_resolve::{Layout, Project};

use crate::model::{Section, entries, fit};

/// The most characters [`rules`] takes.
pub const RULES_BUDGET: usize = 4_000;

/// The most characters the page types and phrases take within the rules.
pub const MODEL_BUDGET: usize = 2_000;

/// A project's rules, as Markdown, for a file in the folder `dir` is relative
/// to: `dir` is the project's folder (the one holding `ascribe.toml`) as seen
/// from there, and empty when the file is beside `ascribe.toml`.
pub fn rules(project: &Project, dir: &RelPath) -> String {
    let model = project.model();
    let at = At::new(project, dir);
    let mut head = String::from("## Ascribe documentation\n\n");
    head.push_str(&at.about());
    head.push_str("\n\n");
    head.push_str(&at.check());
    head.push('\n');

    let model_part = model_sections(model, &at);
    let tail = format!(
        "\n### Links\n\nLink to the source file, with a relative path such as \
         `../guides/install.md#configure`, not to the site's address. `ascribe outline \
         <page>` lists a page's headings with their ids, and `ascribe link <target> --from \
         <page>` says whether a link works and what to write.\n\n### More\n\n`ascribe \
         model{}` shows everything the content model allows. Every directive: {}\n",
        at.path_arg(),
        directive_reference_url(),
    );
    let lines = directive_lines(model, &project.directive_names());
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
    let fixed = head.chars().count() + model_part.chars().count() + tail.chars().count();
    let taken = fit(&[lines.len()], |taken| {
        fixed + directives(taken[0]).chars().count() <= RULES_BUDGET
    });
    format!("{head}{model_part}{}{tail}", directives(taken[0]))
}

/// A short text for a file above a project's folder (the repository's root
/// `AGENTS.md`, for an agent that reads no file below where it starts): where
/// the project's pages and its full rules are, and the check to run. `dir`
/// is as for [`rules`], and isn't empty.
pub fn pointer(project: &Project, dir: &RelPath) -> String {
    let at = At::new(project, dir);
    format!(
        "## Ascribe documentation in `{}`\n\n{} Its rules are in `{}`: read them before \
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
    let content = At::new(project, dir).content;
    if content.is_root() {
        "**/*.md".to_owned()
    } else {
        format!("{}/**/*.md", content.as_str())
    }
}

/// The address of the directive reference on the docs site.
pub fn directive_reference_url() -> String {
    concat!(ascribe_core::docs_site!(), "/reference/directives/").to_owned()
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
}

impl At {
    fn new(project: &Project, dir: &RelPath) -> At {
        let layout: &Layout = project.layout();
        At {
            dir: dir.clone(),
            content: dir
                .join(layout.content_root.as_str())
                .unwrap_or_else(|_| dir.clone()),
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
    /// or nothing for the reader's own.
    fn path_arg(&self) -> String {
        if self.dir.is_root() {
            String::new()
        } else {
            format!(" {}", quote(self.dir.as_str()))
        }
    }

    /// The first paragraph: what the pages are and where.
    fn about(&self) -> String {
        let content = if self.content.is_root() {
            "./".to_owned()
        } else {
            format!("{}/", self.content.as_str())
        };
        format!(
            "The Markdown pages under `{content}` are Ascribe documentation: pages with YAML \
             frontmatter, `@` directives, and `{{key}}` phrases, checked against the content \
             model in `{}`.",
            self.path("ascribe.toml")
        )
    }

    /// The loop, as a check to run.
    fn check(&self) -> String {
        let whole = if self.dir.is_root() {
            "ascribe check".to_owned()
        } else {
            format!("ascribe check --config {}", quote(self.dir.as_str()))
        };
        format!(
            "Programmatic check: after editing a page, run `ascribe check <file> --format \
             concise` and fix every error. Before you finish, run `{whole}`. `ascribe explain \
             <code>` says what a problem means and how to fix it."
        )
    }
}

/// The page types and phrases, cut together to [`MODEL_BUDGET`].
fn model_sections(model: &ContentModel, at: &At) -> String {
    let sections: Vec<(Section, Vec<String>)> = [Section::Types, Section::Phrases]
        .into_iter()
        .map(|s| (s, entries(model, s)))
        .filter(|(_, lines)| !lines.is_empty())
        .collect();
    let first_phrase = model.phrases.first().map(|p| p.key.as_str());
    let render = |taken: &[usize]| -> String {
        let mut out = String::new();
        for ((section, lines), &n) in sections.iter().zip(taken) {
            out.push_str(&format!("\n### {}\n\n", section.heading()));
            match section {
                Section::Types => out.push_str(
                    "A page's frontmatter is its type's. A file or folder whose name starts with \
                     `_` is a fragment, which pages `@include`, not a page.\n\n",
                ),
                Section::Phrases => {
                    if let Some(key) = first_phrase {
                        out.push_str(&format!(
                            "Write the phrase (`{{{key}}}`), not the text it stands for.\n\n"
                        ));
                    }
                }
                _ => {}
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
        }
        out
    };
    let all: Vec<usize> = sections.iter().map(|(_, lines)| lines.len()).collect();
    if render(&all).chars().count() <= MODEL_BUDGET {
        return render(&all);
    }
    render(&fit(&all, |taken| {
        render(taken).chars().count() <= MODEL_BUDGET
    }))
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
            let line = builtin_line(b);
            let declared = match b {
                Builtin::Variant if !model.dimensions.is_empty() => Some(format!(
                    "dimensions {}",
                    code(model.dimensions.iter().map(|d| d.name.as_str()).collect())
                )),
                Builtin::Note => {
                    let types: Vec<&str> = model.notes.iter().map(|n| n.name.as_str()).collect();
                    (!types.is_empty()).then(|| format!("types {}", code(types)))
                }
                Builtin::Snippet if !model.sources.is_empty() => Some(format!(
                    "sources {}",
                    code(model.sources.iter().map(|s| s.name.as_str()).collect())
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
        Builtin::Available => "`@available: <spec>` under a heading or above a block: where it \
                               applies, as targets with an optional state and version \
                               (`cloud, self-managed preview 3.4`), or a feature's key"
            .to_owned(),
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
    }
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

/// A path as a shell reads it: as it is when that's safe, else in single
/// quotes.
fn quote(path: &str) -> String {
    let plain = path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-'));
    if plain {
        path.to_owned()
    } else {
        format!("'{}'", path.replace('\'', r"'\''"))
    }
}
