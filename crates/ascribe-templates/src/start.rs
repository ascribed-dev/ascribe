//! Starting a page from its type: what `ascribe agents prompt new-page`
//! gives an agent to begin from.
//!
//! The page's text is the type's template, read through the project's
//! [`FileSystem`], with its frontmatter's `type` set to the type's name and
//! its `title` to the title given, and nothing else changed. A type with no
//! template gives its frontmatter: its type, the title, and each other field
//! it requires, empty. The implicit `page` type of a project that declares
//! none isn't named. The page goes where it's asked to, or, when it's asked
//! for a folder or nothing, in that folder or the content root, named for
//! the title slugged as a route's segments are.
//! Nothing here writes a file, and a page is never started where a file is.

use ascribe_check::Project;
use ascribe_core::{RelPath, Slugger as _};
use ascribe_model::{ContentModel, ContentType, TYPE_KEY};
use ascribe_resolve::slug::GithubSlugger;
use ascribe_resolve::{FileSystem, Probe};
use ascribe_syntax::frontmatter_lines;

/// A page to start: where it goes, and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPage {
    /// Its path, from the content root.
    pub path: RelPath,
    /// Its text: the type's template with the type and the title set, or
    /// the type's frontmatter when it has none.
    pub text: String,
    /// The type's `template`, from the folder `ascribe.toml` is in, when it
    /// has one.
    pub template: Option<String>,
    /// The type's `guide`, from the folder `ascribe.toml` is in, in the
    /// order to read it.
    pub guide: Vec<String>,
}

/// Why a page can't be started.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum StartError {
    /// The content model has no type of that name.
    #[error("the content model has no page type `{name}`; its types are {types}")]
    UnknownType {
        /// The name given.
        name: String,
        /// The model's types, each in backticks.
        types: String,
    },
    /// A file is where the page would go.
    #[error("`{path}` is there already, and a page is never started over a file")]
    FileInTheWay {
        /// The file, from the content root.
        path: String,
    },
    /// The path isn't one a page can have.
    #[error("`{path}` wouldn't be a page of type `{type_name}`: {reason}")]
    NotTheType {
        /// The path, from the content root.
        path: String,
        /// The type.
        type_name: String,
        /// Why.
        reason: String,
    },
    /// No path was given, and none comes from the type and the title.
    #[error("no path for a page of type `{type_name}` comes from its title: {reason}; give one")]
    NoPath {
        /// The type.
        type_name: String,
        /// Why.
        reason: String,
    },
    /// The type's template can't be read.
    #[error("can't read the template `{path}`: {reason}")]
    Template {
        /// The template, from the folder `ascribe.toml` is in.
        path: String,
        /// Why.
        reason: String,
    },
}

impl ascribe_core::Coded for StartError {
    fn code(&self) -> &'static str {
        match self {
            StartError::UnknownType { .. } => "unknown_type",
            StartError::FileInTheWay { .. } => "page_exists",
            StartError::NotTheType { .. } => "page_not_of_type",
            StartError::NoPath { .. } => "page_path_needed",
            StartError::Template { .. } => "template_unreadable",
        }
    }
}

/// The page of type `type_name` titled `title`, at `path` from the content
/// root: a page's path, ending in `.md`, or a folder, where the page is named
/// for the title. When it's `None`, the page goes in the content root. With
/// no title, the page keeps the template's, which is left to fill in as its
/// placeholders are, and needs a page's path.
///
/// # Errors
///
/// A type the model doesn't have, a file where the page would go, a path a
/// page can't have, no path coming from the title, or a template that can't
/// be read.
pub fn start(
    project: &Project,
    type_name: &str,
    title: Option<&str>,
    path: Option<&RelPath>,
) -> Result<NewPage, StartError> {
    let model = project.model();
    let Some(page_type) = model.types.iter().find(|t| t.name == type_name) else {
        return Err(StartError::UnknownType {
            name: type_name.to_owned(),
            types: model
                .types
                .iter()
                .map(|t| format!("`{}`", t.name))
                .collect::<Vec<_>>()
                .join(", "),
        });
    };
    let path = match path {
        // A folder: the page goes in it, named for the title.
        Some(folder) if folder.extension() != Some("md") => {
            let path = folder
                .join(&file_name(page_type, title)?)
                .unwrap_or_else(|_| folder.clone());
            of_type(model, page_type, path)?
        }
        Some(path) => of_type(model, page_type, path.clone())?,
        None => of_type(
            model,
            page_type,
            RelPath::root()
                .join(&file_name(page_type, title)?)
                .unwrap_or_else(|_| RelPath::root()),
        )?,
    };
    let fs = project.file_system();
    let in_project = project
        .content_root()
        .join(path.as_str())
        .unwrap_or_else(|_| path.clone());
    if is_there(fs, &in_project) {
        return Err(StartError::FileInTheWay {
            path: path.to_string(),
        });
    }
    let text = text(project, page_type, title)?;
    Ok(NewPage {
        path,
        text,
        template: page_type.template.as_ref().map(|t| t.path.clone()),
        guide: page_type.guide.iter().map(|g| g.path.clone()).collect(),
    })
}

/// The text a page of `page_type` titled `title` starts with, wherever it
/// goes: the type's template with the type and the title set, or its
/// frontmatter when it has none. With no title, the template's stays, and a
/// type with none has an empty one.
///
/// # Errors
///
/// A template that can't be read.
pub fn text(
    project: &Project,
    page_type: &ContentType,
    title: Option<&str>,
) -> Result<String, StartError> {
    Ok(match &page_type.template {
        Some(template) => {
            let text = read_template(project.file_system(), &template.path)?;
            let text = match title {
                Some(title) => with_key(&text, "title", &yaml_string(title)),
                None => text,
            };
            if page_type.implicit {
                text
            } else {
                with_key(&text, TYPE_KEY, &type_value(&page_type.name))
            }
        }
        None => frontmatter_only(page_type, title),
    })
}

/// Whether a page of `page_type` can be started in `folder`, from the
/// content root, as [`start`] holds a page it's given a folder for: inside
/// the content root, and not a fragment's. A page is of the type it names,
/// wherever it is (SPEC §7.2), so every such folder takes every type.
pub fn takes_folder(model: &ContentModel, page_type: &ContentType, folder: &RelPath) -> bool {
    folder
        .join(TRIED_NAME)
        .is_ok_and(|path| not_of_type(model, page_type, &path).is_none())
}

/// The name [`takes_folder`] tries a folder with.
const TRIED_NAME: &str = "new-page.md";

/// Whether anything is at `path`, from the project's folder: a file, a
/// folder, a file whose name differs only in case, or a link.
fn is_there(fs: &dyn FileSystem, path: &RelPath) -> bool {
    fs.probe(path) != Probe::Missing || fs.real_path(path).is_some()
}

/// `path`, when it would be a page of `page_type`.
fn of_type(
    model: &ContentModel,
    page_type: &ContentType,
    path: RelPath,
) -> Result<RelPath, StartError> {
    match not_of_type(model, page_type, &path) {
        None => Ok(path),
        Some(reason) => Err(StartError::NotTheType {
            path: path.to_string(),
            type_name: page_type.name.clone(),
            reason,
        }),
    }
}

/// Why `path` wouldn't be a page of `page_type`, or `None` when it would:
/// a page names its type, so only a path that isn't a page's is refused.
fn not_of_type(model: &ContentModel, _page_type: &ContentType, path: &RelPath) -> Option<String> {
    if !path.is_inside() || path.is_root() {
        return Some("it isn't a file in the content root".to_owned());
    }
    if model.is_fragment(path.as_str()) {
        return Some("it's a fragment's path".to_owned());
    }
    None
}

/// The file a page titled `title` is named: the title slugged as a route's
/// segment is, with `.md`.
fn file_name(page_type: &ContentType, title: Option<&str>) -> Result<String, StartError> {
    let Some(title) = title else {
        return Err(StartError::NoPath {
            type_name: page_type.name.clone(),
            reason: "there's no title".to_owned(),
        });
    };
    // A fresh scope's first slug is the slug of the text alone, as a route's
    // segment is (`ascribe_resolve::astro`).
    let slug = GithubSlugger.new_scope().slug(title);
    let name = slug.trim_start_matches('_');
    if name.is_empty() {
        return Err(StartError::NoPath {
            type_name: page_type.name.clone(),
            reason: "it has no letter or digit".to_owned(),
        });
    }
    Ok(format!("{name}.md"))
}

fn read_template(fs: &dyn FileSystem, path: &str) -> Result<String, StartError> {
    let failed = |reason: String| StartError::Template {
        path: path.to_owned(),
        reason,
    };
    let rel = RelPath::parse(path).map_err(|e| failed(e.to_string()))?;
    let bytes = fs.read_file(&rel).map_err(|e| failed(e.to_string()))?;
    String::from_utf8(bytes).map_err(|_| failed("it isn't valid UTF-8".to_owned()))
}

/// `template` with its frontmatter's top-level `key` set to `value`, as
/// YAML, and nothing else changed. A template with no `key` gets one, first;
/// one with no frontmatter gets frontmatter with only the key.
fn with_key(template: &str, key: &str, value: &str) -> String {
    let line = format!("{key}: {value}\n");
    let Some((start, end)) = frontmatter(template) else {
        return format!("---\n{line}---\n\n{template}");
    };
    let content = &template[start..end];
    let lines = frontmatter_lines(content);
    let Some(at) = lines
        .iter()
        .position(|l| l.key == Some(key) && l.indent == 0 && l.item.is_none())
    else {
        return format!("{}{line}{}", &template[..start], &template[start..]);
    };
    // The key's value runs on over the lines indented under it: a block
    // scalar, or a plain or quoted string over several lines.
    let from = start + lines[at].start;
    let to = lines[at + 1..]
        .iter()
        .find(|l| l.indent == 0 && !l.block)
        .map_or(end, |l| start + l.start);
    // Blank lines and comments before the next key stay.
    let kept = template[from..to]
        .split_inclusive('\n')
        .skip(1)
        .collect::<Vec<_>>();
    let trailing = kept
        .iter()
        .rev()
        .take_while(|l| {
            let t = l.trim();
            t.is_empty() || t.starts_with('#')
        })
        .count();
    let kept: String = kept[kept.len() - trailing..].concat();
    format!("{}{line}{kept}{}", &template[..from], &template[to..])
}

/// Where a page's frontmatter content starts and ends: after its opening
/// `---` line, and before its closing one.
fn frontmatter(text: &str) -> Option<(usize, usize)> {
    let start = if text.starts_with("---\n") {
        4
    } else if text.starts_with("---\r\n") {
        5
    } else {
        return None;
    };
    let mut at = start;
    for line in text[start..].split_inclusive('\n') {
        if line.trim_end_matches(['\n', '\r']) == "---" {
            return Some((start, at));
        }
        at += line.len();
    }
    None
}

/// The page a type with no template starts with: its type, its title, and
/// each other field it requires, empty, in its frontmatter. Every type has a
/// `title` field (SPEC §7.2).
fn frontmatter_only(page_type: &ContentType, title: Option<&str>) -> String {
    let mut lines = Vec::new();
    if !page_type.implicit {
        lines.push(format!("{TYPE_KEY}: {}", type_value(&page_type.name)));
    }
    lines.push(match title {
        Some(title) => format!("title: {}", yaml_string(title)),
        None => "title:".to_owned(),
    });
    for field in page_type
        .frontmatter
        .fields
        .iter()
        .filter(|f| f.required && f.name != "title")
    {
        lines.push(format!("{}:", field.name));
    }
    format!("---\n{}\n---\n", lines.join("\n"))
}

/// A type's name as the value of `type`: plain, as a page's is written,
/// unless YAML would read it as something other than that string.
fn type_value(name: &str) -> String {
    match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(name) {
        Ok(serde_yaml_ng::Value::String(s)) if s == name => name.to_owned(),
        _ => yaml_string(name),
    }
}

/// `text` as a YAML string in double quotes, its backslashes, quotes, and
/// control characters escaped.
fn yaml_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_is_set_and_nothing_else_changes() {
        let template =
            "---\ntitle: \"Title\"\ndescription: \"{One sentence.}\"\n---\n\n## Overview\n";
        assert_eq!(
            with_key(template, "title", &yaml_string("Rotate \"keys\"")),
            "---\ntitle: \"Rotate \\\"keys\\\"\"\ndescription: \"{One sentence.}\"\n---\n\n## Overview\n"
        );
    }

    #[test]
    fn a_title_over_several_lines_is_replaced_whole() {
        let template = "---\ntitle: >\n  Long\n  title\n# kept\ndescription: x\n---\nBody\n";
        assert_eq!(
            with_key(template, "title", &yaml_string("New")),
            "---\ntitle: \"New\"\n# kept\ndescription: x\n---\nBody\n"
        );
    }

    #[test]
    fn a_template_without_a_title_gets_one_first() {
        assert_eq!(
            with_key(
                "---\ndescription: x\n---\nBody\n",
                "title",
                &yaml_string("New")
            ),
            "---\ntitle: \"New\"\ndescription: x\n---\nBody\n"
        );
        assert_eq!(
            with_key("Body\n", "title", &yaml_string("New")),
            "---\ntitle: \"New\"\n---\n\nBody\n"
        );
    }

    #[test]
    fn a_type_goes_first_and_replaces_one_there() {
        let template = "---\ntitle: \"Title\"\n---\nBody\n";
        assert_eq!(
            with_key(template, "type", &type_value("how-to")),
            "---\ntype: how-to\ntitle: \"Title\"\n---\nBody\n"
        );
        assert_eq!(
            with_key(
                "---\ntype: concept\ntitle: T\n---\n",
                "type",
                &type_value("how-to")
            ),
            "---\ntype: how-to\ntitle: T\n---\n"
        );
        assert_eq!(type_value("true"), "\"true\"");
    }

    #[test]
    fn control_characters_are_escaped() {
        assert_eq!(yaml_string("a\\b\u{7}"), "\"a\\\\b\\u0007\"");
    }
}
