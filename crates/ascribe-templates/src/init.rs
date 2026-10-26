//! Starting a project: `ascribe init`.
//!
//! It writes a project's first `ascribe.toml`, [`MODEL_TEXT`], in a folder
//! with none at or above it, and makes its content root. With the templates,
//! it calls [`eject()`] with that text, as `ascribe templates eject` would
//! in a project that had it, so the two can't differ; eject then writes the
//! file, with the six types added at its end. It asks nothing, and writes
//! over nothing: where there's a project already, it refuses.
//!
//! A content root that holds pages already is adopted: the file declares no
//! page types, so the pages are of the implicit type `page` and none of them
//! needs `type:` (SPEC §7.2), and the templates, which declare types, are
//! refused.

use std::path::{Path, PathBuf};

use ascribe_check::{MODEL_FILE, Project};
use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, FileSystem as _, Layout};
use serde::Serialize;

use crate::eject::{Eject, EjectAnswer, EjectError, TEMPLATES_DIR, eject};

/// The first `ascribe.toml`, as `ascribe init` writes it.
pub const MODEL_TEXT: &str = include_str!("../first.ascribe.toml");

/// The comment that opens [`MODEL_TEXT`]'s page type, its last section.
const PAGE_TYPE: &str = "# The one kind of page, `page`,";

/// What takes the place of the page type in the `ascribe.toml` of a folder of
/// pages already there.
const ADOPTED: &str = "\
# No page types: every page is of the one implicit kind, `page`, and names
# no type in its frontmatter. Each needs a `title`, and can have a
# `description`. Once a type is declared here, each page names its own, as
# `type: page`: https://ascribed-dev.com/go/content-model/
[frontmatter]
description = \"string?\"
";

/// Where `ascribe init` sends a person next: the Astro guide, by its
/// permalink.
pub const NEXT: &str = ascribe_core::docs_permalink!("astro-guide");

/// The version of [`InitAnswer`]'s schema.
pub const SCHEMA_VERSION: u32 = 2;

/// The first `ascribe.toml` for a content root: [`MODEL_TEXT`], with
/// `content_root` when one is given, and, for a folder of pages there
/// already (`adopted`), without its page type, so the pages are of the
/// implicit type `page` and name none.
#[must_use]
pub fn model_text(content_root: Option<&RelPath>, adopted: bool) -> String {
    let mut text = MODEL_TEXT.to_owned();
    if adopted && let Some(at) = text.find(PAGE_TYPE) {
        text.truncate(at);
        text.push_str(ADOPTED);
    }
    let Some(content_root) = content_root else {
        return text;
    };
    let Ok(mut doc) = text.parse::<toml_edit::DocumentMut>() else {
        return text;
    };
    let written = content_root.to_string();
    // The value only, so the comment above the key stays.
    if let Some(value) = doc
        .get_mut("project")
        .and_then(|project| project.get_mut(CONTENT_ROOT))
        .and_then(toml_edit::Item::as_value_mut)
    {
        let decor = value.decor().clone();
        *value = toml_edit::Value::from(written);
        *value.decor_mut() = decor;
    }
    doc.to_string()
}

/// The key of `[project]` that names the content root.
const CONTENT_ROOT: &str = "content-root";

/// What `ascribe init` did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct InitAnswer {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    pub schema_version: u32,
    /// The version of Ascribe that wrote it.
    pub ascribe_version: &'static str,
    /// The files written, from the new project's folder, `/`-separated, in
    /// the order they were written: with the templates, eject's files, then
    /// `ascribe.toml`. Empty when the templates were refused.
    pub written: Vec<String>,
    /// The content root, from the new project's folder.
    pub content_root: String,
    /// Whether the content root was made: false when the folder was there
    /// already, or nothing was written.
    pub content_root_made: bool,
    /// How many pages the content root held already. When it held any,
    /// `ascribe.toml` declares no page types, so none of them needs `type:`.
    pub pages: usize,
    /// What eject did, when the templates were asked for; otherwise null.
    /// When its `refused` isn't empty, nothing was written.
    pub templates: Option<EjectAnswer>,
    /// The page to read next: the Astro guide, by its permalink.
    pub next: &'static str,
}

/// Why `ascribe init` couldn't start a project. Nothing was written, except
/// after a [`InitError::Write`], or an [`EjectError::Write`]: the files
/// written before it stay.
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    /// There's an `ascribe.toml` at or above the folder.
    #[error(
        "there's a project here already, `{}`; to add the templates to it, run `ascribe templates eject` there",
        found.display()
    )]
    Project {
        /// The `ascribe.toml` found.
        found: PathBuf,
    },
    /// The folder named is a file.
    #[error("`{}` is a file, not a folder", folder.display())]
    NotAFolder {
        /// The folder named.
        folder: PathBuf,
    },
    /// The content root given isn't a relative path.
    #[error("--content-root takes a folder from the project's folder, and `{given}` {reason}")]
    ContentRoot {
        /// The content root given.
        given: String,
        /// Why it can't be one.
        reason: &'static str,
    },
    /// The templates were asked for in a folder of pages, which they'd make
    /// name a type each.
    #[error(
        "the templates declare page types, which every page in `{content_root}` ({pages} found) would then have to name as `type:`; run `ascribe init` without --templates, and `ascribe templates eject` once the pages check clean, then give each page `type:`"
    )]
    TemplatesWithPages {
        /// How many pages there are.
        pages: usize,
        /// The content root, from the new project's folder.
        content_root: String,
    },
    /// The templates couldn't be written.
    #[error(transparent)]
    Templates(#[from] EjectError),
    /// A file or the content root couldn't be written.
    #[error("can't write `{path}`: {reason}")]
    Write {
        /// The file or folder, from the new project's folder.
        path: String,
        /// Why.
        reason: String,
    },
}

impl ascribe_core::Coded for InitError {
    fn code(&self) -> &'static str {
        match self {
            InitError::Project { .. } => "project_exists",
            InitError::NotAFolder { .. } => "init_folder_refused",
            InitError::ContentRoot { .. } => "init_content_root_refused",
            InitError::TemplatesWithPages { .. } => "init_templates_refused",
            InitError::Templates(e) => e.code(),
            InitError::Write { .. } => "init_unwritable",
        }
    }
}

/// What `ascribe init` is asked to do.
#[derive(Clone, Debug)]
pub struct Init<'a> {
    /// The folder to start the project in, absolute. It need not exist.
    pub root: &'a Path,
    /// Whether to take the templates too, as `ascribe templates eject` with
    /// no name does.
    pub templates: bool,
    /// The content root, from the folder, `/`-separated: `docs` when it's
    /// `None`.
    pub content_root: Option<&'a str>,
    /// The version of Ascribe `templates.lock` records.
    pub ascribe_version: &'a str,
}

/// Starts a project in `request.root`: writes its first `ascribe.toml`, with
/// the templates when they're asked for, and makes its content root. A
/// content root with pages in it already gets an `ascribe.toml` that declares
/// no page types.
///
/// # Errors
///
/// There's an `ascribe.toml` at or above the folder, the folder is a file,
/// the content root given isn't a relative path, the templates are asked for
/// in a folder of pages, eject can't write the templates, or a file can't be
/// written. Templates refused aren't an error: the answer lists them, and
/// nothing is written.
pub fn init(request: &Init<'_>) -> Result<InitAnswer, InitError> {
    let root = request.root;
    if let Some(found) = Project::find_config(root) {
        return Err(InitError::Project { found });
    }
    // Outside FileSystem: the folder a project is to start in, before there's
    // one.
    if root.is_file() {
        return Err(InitError::NotAFolder {
            folder: root.to_path_buf(),
        });
    }
    let given = request
        .content_root
        .map(content_root)
        .transpose()
        .map_err(|reason| InitError::ContentRoot {
            given: request.content_root.unwrap_or_default().to_owned(),
            reason,
        })?;
    let refused = |reason| InitError::ContentRoot {
        given: request.content_root.unwrap_or_default().to_owned(),
        reason,
    };
    let unloaded = || refused("can't be written in `ascribe.toml`");
    let model = loaded(&model_text(given.as_ref(), false)).ok_or_else(unloaded)?;
    let layout = Layout::from_model(&model);
    let pages = DiskFs::new(root, &layout)
        .sources()
        .paths
        .iter()
        .filter(|path| !model.is_fragment(path.as_str()))
        .count();
    let text = model_text(given.as_ref(), pages > 0);
    let model = loaded(&text).ok_or_else(unloaded)?;
    let shown = layout.content_root.to_string();
    if pages > 0 && request.templates {
        return Err(InitError::TemplatesWithPages {
            pages,
            content_root: format!("{shown}/"),
        });
    }
    let mut answer = InitAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        written: Vec::new(),
        content_root: shown,
        content_root_made: false,
        pages,
        templates: None,
        next: NEXT,
    };
    if request.templates {
        let fs = DiskFs::new(root, &layout);
        let to = RelPath::parse(TEMPLATES_DIR).unwrap_or_default();
        let ejected = eject(
            &Eject {
                root,
                model: &model,
                model_text: &text,
                names: &[],
                to: &to,
                ascribe_version: request.ascribe_version,
                keep_type: false,
            },
            &fs,
        )?;
        let refused = !ejected.refused.is_empty();
        answer.written.clone_from(&ejected.written);
        answer.templates = Some(ejected);
        if refused {
            return Ok(answer);
        }
    } else {
        write(root, MODEL_FILE, |file| std::fs::write(file, &text))?;
        answer.written.push(MODEL_FILE.to_owned());
    }
    let content_root = layout.content_root.as_str();
    // Outside FileSystem: the content root of a project just written, which
    // must exist for the project to load.
    answer.content_root_made = !root.join(content_root).is_dir();
    write(root, content_root, |dir| std::fs::create_dir_all(dir))?;
    Ok(answer)
}

/// The first `ascribe.toml`, loaded: `None` when it has errors, which only a
/// content root given can give it, since a test loads [`MODEL_TEXT`] and its
/// adopted form with none.
fn loaded(text: &str) -> Option<ascribe_model::ContentModel> {
    ascribe_model::load_str(text, ascribe_core::FileId::new(0)).ok()
}

/// The content root given, as a path from the project's folder; or why it
/// can't be one. It's a folder inside the project's: the project's own
/// folder would hold the build's output, and a folder outside it isn't the
/// project's to start.
fn content_root(given: &str) -> Result<RelPath, &'static str> {
    let path = RelPath::parse(given).map_err(|_| "isn't a relative path")?;
    if path.is_root() || !path.is_inside() {
        return Err(
            "isn't a folder inside it; give the pages a folder of their own, such as `docs`",
        );
    }
    Ok(path)
}

/// Does `act` to `path`, from the project's folder, making the folders on the
/// way to it.
fn write(
    root: &Path,
    path: &str,
    act: impl FnOnce(&Path) -> std::io::Result<()>,
) -> Result<(), InitError> {
    let target = path.split('/').fold(root.to_path_buf(), |p, s| p.join(s));
    let failed = |e: std::io::Error| InitError::Write {
        path: path.to_owned(),
        reason: e.to_string(),
    };
    if let Some(dir) = target.parent() {
        std::fs::create_dir_all(dir).map_err(failed)?;
    }
    act(&target).map_err(failed)
}
