//! Conformance cases and their discovery.

use std::path::{Path, PathBuf};

use crate::expect::{Expect, OutputSlot};

/// Whether a case is one file or a whole project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseKind {
    /// The case has `input.md`.
    SingleFile,
    /// The case has a `files/` tree, which is its content root.
    Project,
}

/// A problem loading a case.
#[derive(Debug, thiserror::Error)]
pub enum CaseError {
    /// A file couldn't be read.
    #[error("couldn't read {path}: {source}")]
    Io {
        /// The file or directory.
        path: PathBuf,
        /// The underlying error.
        source: std::io::Error,
    },
    /// `expect.yaml` isn't valid.
    #[error("invalid expect.yaml: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// The case breaks a rule of the case format.
    #[error("{0}")]
    Format(String),
}

/// One conformance case.
#[derive(Debug, Clone)]
pub struct Case {
    /// The case's id: its directory relative to `cases/`, with `/` separators.
    pub id: String,
    /// The case directory.
    pub dir: PathBuf,
    /// Single file or project.
    pub kind: CaseKind,
    /// The content model the case uses: its own `ascribe.toml`, or the shared
    /// fixture model.
    pub model: PathBuf,
    /// Whether [`Case::model`] is the shared fixture model.
    pub model_is_shared: bool,
    /// The parsed `expect.yaml`.
    pub expect: Expect,
}

impl Case {
    /// Loads the case in `dir`, which contains `expect.yaml`.
    pub fn load(id: String, dir: &Path, shared_model: &Path) -> Result<Case, CaseError> {
        let expect_path = dir.join("expect.yaml");
        let text = read(&expect_path)?;
        let expect: Expect = serde_yaml::from_str(&text)?;

        let has_input = dir.join("input.md").is_file();
        let has_files = dir.join("files").is_dir();
        let kind = match (has_input, has_files) {
            (true, false) => CaseKind::SingleFile,
            (false, true) => CaseKind::Project,
            (true, true) => {
                return Err(CaseError::Format(
                    "a case has either input.md or files/, not both".into(),
                ));
            }
            (false, false) => {
                return Err(CaseError::Format("a case needs input.md or files/".into()));
            }
        };

        let own_model = dir.join("ascribe.toml");
        let (model, model_is_shared) = if own_model.is_file() {
            (own_model, false)
        } else {
            (shared_model.to_owned(), true)
        };

        let case = Case {
            id,
            dir: dir.to_owned(),
            kind,
            model,
            model_is_shared,
            expect,
        };
        case.validate()?;
        Ok(case)
    }

    fn validate(&self) -> Result<(), CaseError> {
        let e = &self.expect;
        let fail = |m: String| Err(CaseError::Format(m));
        if e.area_tags().next().is_none() {
            return fail("a case needs at least one area tag".into());
        }
        if e.is_provisional() && e.questions.is_empty() {
            return fail("a provisional case must list the `questions` it depends on".into());
        }
        if e.outline.is_none()
            && e.diagnostics.is_none()
            && e.builds.is_empty()
            && e.formatted.is_none()
        {
            return fail(
                "a case must expect at least one of outline, diagnostics, builds, or formatted"
                    .into(),
            );
        }
        if let Some(file) = &e.formatted {
            if self.kind == CaseKind::Project {
                return fail("project cases have no `formatted`; it formats `input.md`".into());
            }
            if !self.dir.join(file).is_file() {
                return fail(format!("formatted: {file} doesn't exist"));
            }
        }
        if self.kind == CaseKind::Project {
            if e.outline.is_some() {
                return fail(
                    "project cases have no top-level outline; use builds.<name>.pages".into(),
                );
            }
            let file_diags = e.diagnostics.iter().flatten();
            let build_diags = e
                .builds
                .values()
                .flat_map(|b| b.diagnostics.iter().flatten());
            if let Some(d) = file_diags.chain(build_diags).find(|d| d.file.is_none()) {
                return fail(format!(
                    "diagnostic `{}` needs a `file` in a project case",
                    d.slug
                ));
            }
        }
        for (build, b) in &e.builds {
            for (page, expect) in b.pages.iter().flatten() {
                for (kind, slot) in expect.iter().flat_map(|p| &p.outputs) {
                    if let OutputSlot::File { file } = slot
                        && !self.dir.join(file).is_file()
                    {
                        return fail(format!(
                            "builds.{build}.pages.{page}.outputs.{kind}: {file} doesn't exist"
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// The content root: the case directory for single-file cases, `files/`
    /// for project cases.
    pub fn content_root(&self) -> PathBuf {
        match self.kind {
            CaseKind::SingleFile => self.dir.clone(),
            CaseKind::Project => self.dir.join("files"),
        }
    }

    /// `input.md`, for single-file cases.
    pub fn input_path(&self) -> Option<PathBuf> {
        match self.kind {
            CaseKind::SingleFile => Some(self.dir.join("input.md")),
            CaseKind::Project => None,
        }
    }

    /// The source text of `input.md`, for single-file cases.
    pub fn input(&self) -> Result<Option<String>, CaseError> {
        self.input_path().map(|p| read(&p)).transpose()
    }

    /// Every file under the content root, as sorted `/`-separated paths
    /// relative to it. For a single-file case, that's `input.md` alone.
    pub fn source_files(&self) -> Result<Vec<String>, CaseError> {
        match self.kind {
            CaseKind::SingleFile => Ok(vec!["input.md".into()]),
            CaseKind::Project => {
                let root = self.content_root();
                let mut out = Vec::new();
                walk_files(&root, &root, &mut out)?;
                out.sort();
                Ok(out)
            }
        }
    }
}

fn read(path: &Path) -> Result<String, CaseError> {
    std::fs::read_to_string(path).map_err(|source| CaseError::Io {
        path: path.to_owned(),
        source,
    })
}

fn read_dir_sorted(dir: &Path) -> Result<Vec<PathBuf>, CaseError> {
    let io = |source| CaseError::Io {
        path: dir.to_owned(),
        source,
    };
    let mut entries = std::fs::read_dir(dir)
        .map_err(io)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(io)?;
    entries.sort();
    Ok(entries)
}

fn rel(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), CaseError> {
    for path in read_dir_sorted(dir)? {
        if path.is_dir() {
            walk_files(root, &path, out)?;
        } else {
            out.push(rel(root, &path));
        }
    }
    Ok(())
}

/// A directory found during discovery: a loaded case, or one that failed to load.
#[derive(Debug)]
pub struct Discovered {
    /// The case id (its directory relative to `cases/`).
    pub id: String,
    /// The case, or why it couldn't be loaded.
    pub case: Result<Case, CaseError>,
}

/// Finds every case under `cases_dir`, in path order.
///
/// A directory containing `expect.yaml` is a case, and discovery doesn't look
/// inside it. Directories whose names start with `_` or `.` are skipped, so
/// shared fixtures can live next to cases. A directory that has `input.md` or
/// `files/` but no `expect.yaml` is reported as a broken case.
pub fn discover(cases_dir: &Path, shared_model: &Path) -> Result<Vec<Discovered>, CaseError> {
    let mut out = Vec::new();
    if cases_dir.is_dir() {
        discover_in(cases_dir, cases_dir, shared_model, &mut out)?;
    }
    Ok(out)
}

fn discover_in(
    root: &Path,
    dir: &Path,
    shared_model: &Path,
    out: &mut Vec<Discovered>,
) -> Result<(), CaseError> {
    for path in read_dir_sorted(dir)? {
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        if name.is_some_and(|n| n.starts_with('_') || n.starts_with('.')) {
            continue;
        }
        let id = rel(root, &path);
        if path.join("expect.yaml").is_file() {
            let case = Case::load(id.clone(), &path, shared_model);
            out.push(Discovered { id, case });
        } else if path.join("input.md").is_file() || path.join("files").is_dir() {
            out.push(Discovered {
                id,
                case: Err(CaseError::Format(
                    "has input.md or files/ but no expect.yaml".into(),
                )),
            });
        } else {
            discover_in(root, &path, shared_model, out)?;
        }
    }
    Ok(())
}
