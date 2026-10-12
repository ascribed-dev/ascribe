//! Starting a page from its type: what `ascribe agents prompt new-page`
//! gives an agent to begin from.
//!
//! The page's text is the type's template, read through the project's
//! [`FileSystem`], with its frontmatter's `title` set to the title given and
//! nothing else changed. A type with no template gives its frontmatter: the
//! title and each other field it requires, empty. The page goes where it's
//! asked to, or, when nothing is asked, in the first folder the type's
//! `files` covers, named for the title slugged as a route's segments are.
//! Nothing here writes a file, and a page is never started where a file is.

use ascribe_check::Project;
use ascribe_core::{RelPath, Slugger as _};
use ascribe_model::{ContentModel, ContentType, Pattern, TypeMatch};
use ascribe_resolve::slug::GithubSlugger;
use ascribe_resolve::{FileSystem, Probe};
use ascribe_syntax::frontmatter_lines;

/// A page to start: where it goes, and its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewPage {
    /// Its path, from the content root.
    pub path: RelPath,
    /// Its text: the type's template with the title set, or the type's
    /// frontmatter when it has none.
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
    /// The path isn't one a page of the type can have.
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
/// root, or where the type's pages go when it's `None`.
///
/// # Errors
///
/// A type the model doesn't have, a file where the page would go, a path a
/// page of the type can't have, no path coming from the title, or a
/// template that can't be read.
pub fn start(
    project: &Project,
    type_name: &str,
    title: &str,
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
        Some(path) => {
            if let Some(reason) = not_of_type(model, page_type, path) {
                return Err(StartError::NotTheType {
                    path: path.to_string(),
                    type_name: type_name.to_owned(),
                    reason,
                });
            }
            path.clone()
        }
        None => default_path(model, page_type, title)?,
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
    let text = match &page_type.template {
        Some(template) => with_title(&read_template(fs, &template.path)?, title),
        None => frontmatter_only(page_type, title),
    };
    Ok(NewPage {
        path,
        text,
        template: page_type.template.as_ref().map(|t| t.path.clone()),
        guide: page_type.guide.iter().map(|g| g.path.clone()).collect(),
    })
}

/// Whether anything is at `path`, from the project's folder: a file, a
/// folder, a file whose name differs only in case, or a link.
fn is_there(fs: &dyn FileSystem, path: &RelPath) -> bool {
    fs.probe(path) != Probe::Missing || fs.real_path(path).is_some()
}

/// Why `path` wouldn't be a page of `page_type`, or `None` when it would.
fn not_of_type(model: &ContentModel, page_type: &ContentType, path: &RelPath) -> Option<String> {
    if !path.is_inside() || path.is_root() {
        return Some("it isn't a file in the content root".to_owned());
    }
    if path.extension() != Some("md") {
        return Some("a page's name ends in `.md`".to_owned());
    }
    if model.is_fragment(path.as_str()) {
        return Some("it's a fragment's path".to_owned());
    }
    match model.type_for(path.as_str()) {
        TypeMatch::One(t) if t.name == page_type.name => None,
        TypeMatch::One(t) => Some(format!("it's a page of type `{}`", t.name)),
        TypeMatch::Ambiguous(_) => Some("several types' `files` match it".to_owned()),
        TypeMatch::None => Some("no type's `files` match it".to_owned()),
    }
}

/// The path a page of `page_type` titled `title` has when none is given:
/// the first folder the type's `files` covers, where a page named for the
/// title is of the type, and the title slugged as a route's segments are.
fn default_path(
    model: &ContentModel,
    page_type: &ContentType,
    title: &str,
) -> Result<RelPath, StartError> {
    let no_path = |reason: &str| StartError::NoPath {
        type_name: page_type.name.clone(),
        reason: reason.to_owned(),
    };
    // A fresh scope's first slug is the slug of the text alone, as a route's
    // segment is (`ascribe_resolve::astro`).
    let slug = GithubSlugger.new_scope().slug(title);
    let name = slug.trim_start_matches('_');
    if name.is_empty() {
        return Err(no_path("it has no letter or digit"));
    }
    let file = format!("{name}.md");
    let folders: Vec<RelPath> = if page_type.files.is_empty() {
        vec![RelPath::root()]
    } else {
        page_type.files.iter().map(folder_of).collect()
    };
    folders
        .iter()
        .filter_map(|folder| folder.join(&file).ok())
        .find(|path| not_of_type(model, page_type, path).is_none())
        .ok_or_else(|| no_path("no folder its `files` covers takes a page named for it"))
}

/// The folder a pattern covers: its segments before the first with a glob
/// in it, `how-to` for `how-to/**`.
fn folder_of(pattern: &Pattern) -> RelPath {
    let source = pattern.as_str();
    let mut segments: Vec<&str> = source.split('/').collect();
    // The last segment names files, not a folder.
    segments.pop();
    let literal: Vec<&str> = segments
        .into_iter()
        .take_while(|s| !s.contains(['*', '?', '[', '{', '\\']))
        .collect();
    RelPath::parse(&literal.join("/")).unwrap_or_else(|_| RelPath::root())
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

/// `template` with its frontmatter's `title` set to `title`, and nothing
/// else changed. A template with no `title` gets one, first; one with no
/// frontmatter gets frontmatter with only the title.
fn with_title(template: &str, title: &str) -> String {
    let line = format!("title: {}\n", yaml_string(title));
    let Some((start, end)) = frontmatter(template) else {
        return format!("---\n{line}---\n\n{template}");
    };
    let content = &template[start..end];
    let lines = frontmatter_lines(content);
    let Some(at) = lines
        .iter()
        .position(|l| l.key == Some("title") && l.indent == 0 && l.item.is_none())
    else {
        return format!("{}{line}{}", &template[..start], &template[start..]);
    };
    // The title's value runs on over the lines indented under it: a block
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

/// The page a type with no template starts with: its title, and each other
/// field it requires, empty, in its frontmatter. Every type has a `title`
/// field (`model-type-title`).
fn frontmatter_only(page_type: &ContentType, title: &str) -> String {
    let mut lines = vec![format!("title: {}", yaml_string(title))];
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
        let template = "---\ntitle: \"Title\"\ndescription: \"{One sentence.}\"\n---\n\n## Overview\n";
        assert_eq!(
            with_title(template, "Rotate \"keys\""),
            "---\ntitle: \"Rotate \\\"keys\\\"\"\ndescription: \"{One sentence.}\"\n---\n\n## Overview\n"
        );
    }

    #[test]
    fn a_title_over_several_lines_is_replaced_whole() {
        let template = "---\ntitle: >\n  Long\n  title\n# kept\ndescription: x\n---\nBody\n";
        assert_eq!(
            with_title(template, "New"),
            "---\ntitle: \"New\"\n# kept\ndescription: x\n---\nBody\n"
        );
    }

    #[test]
    fn a_template_without_a_title_gets_one_first() {
        assert_eq!(
            with_title("---\ndescription: x\n---\nBody\n", "New"),
            "---\ntitle: \"New\"\ndescription: x\n---\nBody\n"
        );
        assert_eq!(
            with_title("Body\n", "New"),
            "---\ntitle: \"New\"\n---\n\nBody\n"
        );
    }

    #[test]
    fn a_pattern_covers_its_literal_folders() {
        let folder = |p: &str| folder_of(&Pattern::new(p).unwrap()).to_string();
        assert_eq!(folder("how-to/**"), "how-to");
        assert_eq!(folder("guides/*.md"), "guides");
        assert_eq!(folder("api/v1/{a,b}/**"), "api/v1");
        assert_eq!(folder("**/*.md"), "");
        assert_eq!(folder("intro.md"), "");
    }

    #[test]
    fn control_characters_are_escaped() {
        assert_eq!(yaml_string("a\\b\u{7}"), "\"a\\\\b\\u0007\"");
    }
}
