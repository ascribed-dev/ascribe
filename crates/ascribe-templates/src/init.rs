//! Starting a project: `ascribe init`.
//!
//! It writes a project's first `ascribe.toml`, [`MODEL_TEXT`], in a folder
//! with none at or above it, and makes its content root. With the templates,
//! it calls [`eject()`] with that text, as `ascribe templates eject` would
//! in a project that had it, so the two can't differ; eject then writes the
//! file, with the six types added at its end. It asks nothing, and writes
//! over nothing: where there's a project already, it refuses.

use std::path::{Path, PathBuf};

use ascribe_check::{MODEL_FILE, Project};
use ascribe_core::RelPath;
use ascribe_resolve::{DiskFs, Layout};
use serde::Serialize;

use crate::eject::{Eject, EjectAnswer, EjectError, TEMPLATES_DIR, eject};

/// The first `ascribe.toml`, as `ascribe init` writes it.
pub const MODEL_TEXT: &str = include_str!("../first.ascribe.toml");

/// Where `ascribe init` sends a person next: the Astro guide, by its
/// permalink.
pub const NEXT: &str = ascribe_core::docs_permalink!("astro-guide");

/// The version of [`InitAnswer`]'s schema.
pub const SCHEMA_VERSION: u32 = 1;

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
    /// The version of Ascribe `templates.lock` records.
    pub ascribe_version: &'a str,
}

/// Starts a project in `request.root`: writes its first `ascribe.toml`, with
/// the templates when they're asked for, and makes its content root.
///
/// # Errors
///
/// There's an `ascribe.toml` at or above the folder, the folder is a file,
/// eject can't write the templates, or a file can't be written. Templates
/// refused aren't an error: the answer lists them, and nothing is written.
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
    // The text is compiled in, and a test loads it with no diagnostics.
    #[allow(clippy::expect_used)]
    let model = ascribe_model::load_str(MODEL_TEXT, ascribe_core::FileId::new(0))
        .expect("the first ascribe.toml loads");
    let layout = Layout::from_model(&model);
    let mut answer = InitAnswer {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        written: Vec::new(),
        content_root: layout.content_root.to_string(),
        content_root_made: false,
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
                model_text: MODEL_TEXT,
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
        write(root, MODEL_FILE, |file| std::fs::write(file, MODEL_TEXT))?;
        answer.written.push(MODEL_FILE.to_owned());
    }
    let content_root = layout.content_root.as_str();
    // Outside FileSystem: the content root of a project just written, which
    // must exist for the project to load.
    answer.content_root_made = !root.join(content_root).is_dir();
    write(root, content_root, |dir| std::fs::create_dir_all(dir))?;
    Ok(answer)
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
