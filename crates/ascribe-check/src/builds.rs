//! What `ascribe check` and `ascribe build` share: choosing builds, and the
//! diagnostics reported for them. One implementation, so the two commands
//! report the same diagnostics, and `diff`, `drift`, and any other tool that
//! takes `--build` names choose builds the same way.

use ascribe_core::RelPath;
use ascribe_model::Build;

use crate::{
    Diagnostic, PageChecker, Project, check_all_builds, check_builds, check_file, check_files,
};

/// A build name that isn't a build of the content model.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error(
    "the content model has no build `{name}`; its builds are {}",
    known.join(", ")
)]
pub struct UnknownBuild {
    /// The name asked for.
    pub name: String,
    /// The content model's builds, in its order.
    pub known: Vec<String>,
}

impl ascribe_core::Coded for UnknownBuild {
    fn code(&self) -> &'static str {
        "unknown_build"
    }
}

/// The builds named in `names`, in the order given and each once; every
/// build of the content model when none is named.
///
/// # Errors
///
/// The first name that isn't a build of the content model.
pub fn select_builds<'p>(
    project: &'p Project,
    names: &[String],
) -> Result<Vec<&'p Build>, UnknownBuild> {
    let all = &project.model().builds;
    if names.is_empty() {
        return Ok(all.iter().collect());
    }
    let mut out: Vec<&Build> = Vec::new();
    for name in names {
        let build = all
            .iter()
            .find(|b| &b.name == name)
            .ok_or_else(|| UnknownBuild {
                name: name.clone(),
                known: all.iter().map(|b| b.name.clone()).collect(),
            })?;
        if !out.iter().any(|b| b.name == build.name) {
            out.push(build);
        }
    }
    Ok(out)
}

/// What [`diagnose`] found.
#[derive(Debug)]
pub struct Diagnosed<'p> {
    /// The diagnostics, in the order they're reported.
    pub diagnostics: Vec<Diagnostic>,
    /// The builds checked, as [`select_builds`] chose them.
    pub builds: Vec<&'p Build>,
}

/// The diagnostics of the builds named in `names`, and the builds: what
/// `ascribe check` reports, and what `ascribe build` checks before it writes
/// anything. With none named, that's every build and also the content no
/// build publishes; each problem is listed once, and the builds it appears
/// in are added to its message when they aren't all of the selected ones.
///
/// # Errors
///
/// The first name that isn't a build of the content model.
pub fn diagnose<'p>(project: &'p Project, names: &[String]) -> Result<Diagnosed<'p>, UnknownBuild> {
    let builds = select_builds(project, names)?;
    let mut diagnostics = if names.is_empty() {
        check_all_builds(project)
    } else {
        check_builds(project, &builds)
    };
    for d in &mut diagnostics {
        if let Some(note) = d.builds_note(builds.len()) {
            d.message = format!("{} ({note})", d.message);
        }
    }
    Ok(Diagnosed {
        diagnostics,
        builds,
    })
}

/// What the language server reports as you type, for `ascribe check
/// --editor-build`: the file-level diagnostics, and the page-level ones of
/// the editor's build alone (`[editor] build`;
/// `ContentModel::editor_default_build`), without the pass over content no
/// build publishes.
///
/// With `files` (content paths of source files), only what can count for
/// them is checked, for speed: the file-level checks of those files, and the
/// page-level checks of the pages that are them or include them. That's every
/// diagnostic located in them or with a related place in them, apart from a
/// file-level one of another file whose related place is in them, which no
/// file-level check makes. The content model's warnings and the checks of
/// `ascribe.lock` aren't run then, since they're located in neither.
pub fn diagnose_editor_build<'p>(project: &'p Project, files: Option<&[RelPath]>) -> Diagnosed<'p> {
    let build = project.model().editor_default_build();
    let diagnostics = match files {
        None => {
            let mut out = check_files(project);
            out.extend(PageChecker::new(project).check(build));
            out
        }
        Some(files) => {
            let mut out: Vec<Diagnostic> = files
                .iter()
                .filter_map(|path| project.source_at(path))
                .flat_map(|file| check_file(project, file))
                .collect();
            out.extend(PageChecker::new(project).check_reaching(build, files));
            out
        }
    };
    Diagnosed {
        diagnostics,
        builds: vec![build],
    }
}
