//! What `ascribe check` and `ascribe build` share: choosing builds, and the
//! diagnostics reported for them. One implementation, so the two commands
//! report the same diagnostics, and `diff`, `drift`, and any other tool that
//! takes `--build` names choose builds the same way.

use ascribe_core::RelPath;
use ascribe_model::Build;

use crate::{
    Diagnostic, PageChecker, PageIndex, Project, apply_levels, check_all_builds, check_builds,
    check_file, check_files,
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
    let diagnostics = if names.is_empty() {
        check_all_builds(project)
    } else {
        check_builds(project, &builds)
    };
    let mut diagnostics = apply_levels(&project.model().checks, diagnostics);
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

/// How many errors [`diagnose`] would report for the builds named in
/// `names`, without the checks that can't report one and cost the most to
/// run ([`PageChecker::errors_only`]): for a command that only says whether
/// the project has errors, such as `ascribe diff`.
///
/// # Errors
///
/// The first name that isn't a build of the content model.
pub fn count_errors(project: &Project, names: &[String]) -> Result<usize, UnknownBuild> {
    let builds = select_builds(project, names)?;
    let checker = PageChecker::new(project).errors_only();
    let mut diagnostics = check_files(project);
    diagnostics.extend(if names.is_empty() {
        checker.check_all()
    } else {
        checker.check_builds(&builds)
    });
    Ok(apply_levels(&project.model().checks, diagnostics)
        .iter()
        .filter(|d| d.severity == crate::Severity::Error)
        .count())
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
    diagnose_editor_build_in(project, None, files)
}

/// [`diagnose_editor_build`], over `index` when it's given: the project's,
/// kept from an earlier check ([`PageIndex::new`]).
pub fn diagnose_editor_build_in<'p>(
    project: &'p Project,
    index: Option<&'p PageIndex>,
    files: Option<&[RelPath]>,
) -> Diagnosed<'p> {
    let build = project.model().editor_default_build();
    let checker = || match index {
        Some(index) => PageChecker::with_page_index(project, index),
        None => PageChecker::new(project),
    };
    let diagnostics = match files {
        None => {
            let mut out = check_files(project);
            out.extend(checker().check(build));
            out
        }
        Some(files) => {
            let mut out: Vec<Diagnostic> = files
                .iter()
                .filter_map(|path| project.source_at(path))
                .flat_map(|file| check_file(project, file))
                .collect();
            out.extend(checker().check_reaching(build, files));
            out
        }
    };
    Diagnosed {
        diagnostics: apply_levels(&project.model().checks, diagnostics),
        builds: vec![build],
    }
}
