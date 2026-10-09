//! A page as a reader of one build sees it: the build's plain output for that
//! page alone, rendered as `ascribe build` renders it ([`emit_page`] with the
//! [`PlainEmitter`]), without writing anything.

use std::path::Path;

use ascribe_core::RelPath;
use ascribe_emit::{EmitContext, PlainEmitter, emit_page};
use ascribe_model::Build;
use ascribe_resolve::{AstroRouter, FileKind, Project};
use serde::Serialize;

use crate::{ASCRIBE_VERSION, QueryError, SCHEMA_VERSION};

/// What `ascribe render <PAGE> --format json` answers.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Rendered {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The page, as a path from the content root.
    pub page: String,
    /// The build.
    pub build: String,
    /// Why the build doesn't publish the page, when it doesn't; `text` is
    /// empty then.
    pub not_published: Option<String>,
    /// The page's route in the build's site.
    pub route: Option<String>,
    /// The page as plain Markdown, as the build's `plain` output writes it:
    /// with its frontmatter first when asked for.
    pub text: String,
}

/// The page at content path `path`, as `build` renders it in its plain
/// output; with `frontmatter`, its resolved frontmatter first, between `---`
/// lines. `project_root` is the directory of `ascribe.toml`.
///
/// # Errors
///
/// `path` isn't a page of the project, or the page can't be rendered.
pub fn render(
    project: &Project,
    project_root: &Path,
    path: &RelPath,
    build: &Build,
    frontmatter: bool,
) -> Result<Rendered, QueryError> {
    let file = project.file(path).ok_or_else(|| QueryError::NotASource {
        path: path.to_string(),
    })?;
    if file.kind == FileKind::Fragment {
        return Err(QueryError::NotAPage {
            path: path.to_string(),
        });
    }
    let mut answer = Rendered {
        schema_version: SCHEMA_VERSION,
        ascribe_version: ASCRIBE_VERSION,
        page: path.to_string(),
        build: build.name.clone(),
        not_published: None,
        route: None,
        text: String::new(),
    };
    if let Some(reason) = project.dropped(path, build) {
        answer.not_published = Some(format!(
            "build `{}` doesn't publish `{path}`: {}",
            build.name,
            reason.explanation()
        ));
        return Ok(answer);
    }
    let router = AstroRouter::from_consumer(&project.model().consumer);
    let Some(page) = project.resolve_page(path, build, &router) else {
        return Err(QueryError::NotAPage {
            path: path.to_string(),
        });
    };
    let cx = EmitContext::new(project, project_root, build);
    let emitted = emit_page(&PlainEmitter, &cx, &page)?;
    if frontmatter && let Some(value) = page.frontmatter.as_ref().filter(|v| !v.is_null()) {
        // A value parsed from YAML always writes back as YAML; were it not
        // to, the page is still worth showing without its frontmatter.
        let yaml = serde_yaml_ng::to_string(value).unwrap_or_default();
        answer.text.push_str("---\n");
        answer.text.push_str(&yaml);
        if !yaml.ends_with('\n') {
            answer.text.push('\n');
        }
        answer.text.push_str("---\n\n");
    }
    answer.text.push_str(&emitted.text);
    answer.route = Some(page.route.clone());
    Ok(answer)
}
