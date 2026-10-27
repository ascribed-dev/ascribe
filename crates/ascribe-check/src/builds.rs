//! What `ascribe check` and `ascribe build` share: choosing builds, and the
//! diagnostics reported for them. One implementation, so the two commands
//! report the same diagnostics, and `diff`, `drift`, and any other tool that
//! takes `--build` names choose builds the same way.

use std::collections::HashSet;

use ascribe_core::RelPath;
use ascribe_model::{Build, ContentModel};

use crate::baseline::Held;
use crate::identity::{Identity, identities, identities_in};
use crate::{
    Acknowledged, Acknowledgements, Baseline, BaselineError, Diagnostic, PageChecker, Project,
    Reported, Scope, apply_levels, check_all_builds, check_builds, check_file, check_files_with,
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
    /// The problems an acknowledgement covers (SPEC §4.9), which aren't in
    /// `diagnostics`.
    pub acknowledged: Vec<Acknowledged>,
    /// The builds checked, as [`select_builds`] chose them.
    pub builds: Vec<&'p Build>,
}

/// What a check found, before [`Request::settle`] applies what the project
/// asks of it: [`Diagnosed`], with the builds by name, so it can be kept
/// apart from the project.
#[derive(Clone, Debug, Default)]
pub struct Found {
    /// The diagnostics, in the order they're found.
    pub diagnostics: Vec<Diagnostic>,
    /// The problems an acknowledgement covers, which aren't in
    /// `diagnostics`.
    pub acknowledged: Vec<Acknowledged>,
    /// The builds whose page-level checks ran, by name.
    pub builds: Vec<String>,
}

impl From<Diagnosed<'_>> for Found {
    fn from(d: Diagnosed<'_>) -> Found {
        Found {
            builds: d.builds.iter().map(|b| b.name.clone()).collect(),
            diagnostics: d.diagnostics,
            acknowledged: d.acknowledged,
        }
    }
}

/// What a check reports, as a [`Request`] asks: the levels `[checks]` sets,
/// the scope, and the baseline applied.
#[derive(Clone, Debug, Default)]
pub struct Checked {
    /// The diagnostics that count for the scope, in the order they're found;
    /// all of them without one.
    pub reported: Vec<Reported>,
    /// The problems an acknowledgement covers in the scope, which aren't
    /// reported.
    pub acknowledged: Vec<Acknowledged>,
    /// The advice `ascribe.baseline` holds back in the scope, which isn't
    /// reported: none unless [`Request::baseline`] asks for it.
    pub baselined: Vec<Diagnostic>,
    /// The builds whose page-level checks ran, by name.
    pub builds: Vec<String>,
}

impl Checked {
    /// The diagnostics reported, without how often each repeats: all of
    /// them when there's no scope, which collapses none.
    pub fn into_diagnostics(self) -> Vec<Diagnostic> {
        self.reported.into_iter().map(|r| r.diagnostic).collect()
    }
}

/// What a check is asked for, besides the project: the settings every
/// surface that checks a project shares, so `ascribe check`, `ascribe
/// build`, `ascribe report`, the agents' hook and tools, and the editor
/// report the same diagnostics for the same project and builds.
#[derive(Clone, Copy, Debug, Default)]
pub struct Request<'a> {
    /// The builds to check, by name; every build when empty.
    pub builds: &'a [String],
    /// Whether to run the page-level checks of the editor's build only, as
    /// [`diagnose_editor_build`] does, instead of [`Request::builds`].
    pub editor_build: bool,
    /// What's reported: the diagnostics that count for these paths
    /// ([`Scope::report`]). The whole project when `None`.
    pub scope: Option<&'a Scope>,
    /// Whether `ascribe.baseline`, when the project has one, holds back the
    /// advice it lists. Without it, everything is reported.
    pub baseline: bool,
    /// Whether Vale ran, and its findings are among those settled: when it
    /// didn't, a baseline entry of Vale's matches nothing, and isn't stale.
    pub vale_ran: bool,
}

/// Why [`diagnose_with`] couldn't check the project.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum DiagnoseError {
    /// A build named isn't the content model's.
    #[error(transparent)]
    UnknownBuild(#[from] UnknownBuild),
    /// `ascribe.baseline` can't be read, so nothing is checked with it.
    #[error("{0}; fix the line, or pass --no-baseline to check without the baseline")]
    Baseline(BaselineError),
}

impl ascribe_core::Coded for DiagnoseError {
    fn code(&self) -> &'static str {
        match self {
            DiagnoseError::UnknownBuild(e) => e.code(),
            DiagnoseError::Baseline(e) => e.code(),
        }
    }
}

impl Request<'_> {
    /// Whether the check covers every build and the whole project, and so
    /// would find everything a baseline entry lists: only then is an entry
    /// that matches nothing stale.
    pub fn whole(&self) -> bool {
        self.scope.is_none() && self.builds.is_empty() && !self.editor_build
    }

    /// The checks whose findings this check doesn't have, by slug, so an
    /// entry of theirs isn't stale: Vale's unless it ran, and in the
    /// editor's build, the checks the editor doesn't run
    /// ([`NOT_IN_EDITOR`](crate::NOT_IN_EDITOR)).
    pub fn not_run(&self) -> Vec<&'static str> {
        let prose = (!self.vale_ran).then_some(ascribe_core::diagnostics::PROSE);
        let editor = if self.editor_build {
            crate::NOT_IN_EDITOR
        } else {
            &[]
        };
        prose
            .into_iter()
            .chain(editor.iter().copied())
            .map(|slug| slug.as_str())
            .collect()
    }

    /// The baseline this request checks with: `None` without one, or when
    /// it doesn't ask for it.
    ///
    /// # Errors
    ///
    /// It asks for the baseline, and `ascribe.baseline` can't be read.
    pub fn baseline_of<'p>(
        &self,
        project: &'p Project,
    ) -> Result<Option<&'p Baseline>, BaselineError> {
        match project.baseline() {
            Some(Err(e)) if self.baseline => Err(e.clone()),
            Some(Ok(baseline)) if self.baseline => Ok(Some(baseline)),
            _ => Ok(None),
        }
    }

    /// What the checks find, before [`settle`](Request::settle): of the
    /// builds named, as [`diagnose`] finds it, or of the editor's build, as
    /// [`diagnose_editor_build`] finds it, only what can count for the
    /// scope's files when it names some.
    ///
    /// # Errors
    ///
    /// The first name that isn't a build of the content model.
    pub fn find<'p>(&self, project: &'p Project) -> Result<Diagnosed<'p>, UnknownBuild> {
        if self.editor_build {
            let files = self.scope.and_then(|s| s.named_sources(project));
            Ok(diagnose_editor_build(project, files.as_deref()))
        } else {
            diagnose(project, self.builds)
        }
    }

    /// `found`, as the project asks: at the levels `[checks]` sets, with
    /// the advice `ascribe.baseline` lists held back when this request asks
    /// for the baseline, and what counts for the scope kept. An entry that
    /// matches nothing is reported, as `baseline-entry-unmatched`, when the
    /// check is [`whole`](Request::whole), apart from the entries of the
    /// checks [`not_run`](Request::not_run) names.
    ///
    /// What [`find`](Request::find) found is at those levels already; what
    /// a caller adds to it, such as Vale's findings or the addresses
    /// compared with a base, is set at them here.
    ///
    /// # Errors
    ///
    /// This request asks for the baseline, and `ascribe.baseline` can't be
    /// read.
    pub fn settle(&self, project: &Project, found: Found) -> Result<Checked, BaselineError> {
        let baseline = self.baseline_of(project)?;
        let Found {
            diagnostics,
            mut acknowledged,
            builds,
        } = found;
        let diagnostics = apply_levels(&project.model().checks, diagnostics);
        let (diagnostics, mut baselined) = match baseline {
            Some(baseline) => {
                let held = self.hold(project, baseline, diagnostics);
                let mut diagnostics = held.diagnostics;
                if self.whole() {
                    let matched: HashSet<&Identity> = held.matched.iter().collect();
                    diagnostics.extend(self.unmatched(project.model(), baseline, &matched));
                }
                (diagnostics, held.held)
            }
            None => (diagnostics, Vec::new()),
        };
        let reported = match self.scope {
            Some(scope) => {
                acknowledged.retain(|a| scope.contains_file(project, a.problem.location.file));
                baselined.retain(|d| scope.contains_file(project, d.location.file));
                scope.report(project, diagnostics)
            }
            None => Reported::all(diagnostics),
        };
        Ok(Checked {
            reported,
            acknowledged,
            baselined,
            builds,
        })
    }

    /// `diagnostics`, of `project`, with the advice `baseline` lists set
    /// aside, without the entries that match nothing: the whole project's,
    /// or all of some files', since an identity's order is among the
    /// findings of one file and one check. The editor holds back a file's
    /// diagnostics at a time with it (#790).
    pub fn hold(
        &self,
        project: &Project,
        baseline: &Baseline,
        diagnostics: Vec<Diagnostic>,
    ) -> Held {
        let found = identities(project, &diagnostics);
        crate::baseline::hold(baseline, diagnostics, found)
    }

    /// [`hold`](Request::hold), for `diagnostics` all located in `file`,
    /// from the project's root, whose text is `text`: Vale's, which the
    /// editor checks without a project.
    pub fn hold_in(
        &self,
        baseline: &Baseline,
        file: &str,
        text: &str,
        diagnostics: Vec<Diagnostic>,
    ) -> Held {
        let found = identities_in(file, text, &diagnostics);
        crate::baseline::hold(baseline, diagnostics, found)
    }

    /// A `baseline-entry-unmatched` at each entry of `baseline` that isn't
    /// `matched`, at the level `model` sets for it, apart from the entries
    /// of the checks [`not_run`](Request::not_run) names.
    pub fn unmatched(
        &self,
        model: &ContentModel,
        baseline: &Baseline,
        matched: &HashSet<&Identity>,
    ) -> Vec<Diagnostic> {
        crate::baseline::stale(&model.checks, baseline, matched, &self.not_run())
    }
}

/// What a check reports, as `request` asks: what [`Request::find`] finds,
/// [settled](Request::settle).
///
/// # Errors
///
/// A baseline that can't be read, when `request` asks for it, or else the
/// first name that isn't a build of the content model.
pub fn diagnose_with(project: &Project, request: &Request<'_>) -> Result<Checked, DiagnoseError> {
    // A baseline that can't be read stops the check before it starts.
    request
        .baseline_of(project)
        .map_err(DiagnoseError::Baseline)?;
    let found = request.find(project)?;
    request
        .settle(project, found.into())
        .map_err(DiagnoseError::Baseline)
}

/// The diagnostics of the builds named in `names`, and the builds: what
/// `ascribe check` reports, and what `ascribe build` checks before it writes
/// anything. With none named, that's every build and also the content no
/// build publishes; each problem is listed once, and the builds it appears
/// in are added to its message when they aren't all of the selected ones.
/// A problem an acknowledgement covers is set aside in
/// [`Diagnosed::acknowledged`]; with every build checked, an acknowledgement
/// that covers nothing is reported.
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
    let diagnostics = apply_levels(&project.model().checks, diagnostics);
    let diagnostics = crate::generated::mark(project, diagnostics);
    let applied =
        Acknowledgements::of(project).apply(project.model(), diagnostics, names.is_empty());
    let mut diagnostics = applied.diagnostics;
    for d in &mut diagnostics {
        if let Some(note) = d.builds_note_in(&builds) {
            d.message = format!("{} ({note})", d.message);
        }
    }
    Ok(Diagnosed {
        diagnostics,
        acknowledged: applied.acknowledged,
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
    let mut diagnostics = check_files_with(project, Some(&checker));
    diagnostics.extend(if names.is_empty() {
        checker.check_all()
    } else {
        checker.check_builds(&builds)
    });
    let diagnostics = apply_levels(&project.model().checks, diagnostics);
    let applied =
        Acknowledgements::of(project).apply(project.model(), diagnostics, names.is_empty());
    Ok(applied
        .diagnostics
        .iter()
        .filter(|d| d.severity == crate::Severity::Error)
        .count())
}

/// How many errors [`diagnose`] would report for one build, which needn't be
/// one the content model declares: `ascribe versions export` checks a
/// planned line's build, made for it by
/// [`ContentModel::line_build`](ascribe_model::ContentModel::line_build).
/// As [`count_errors`], without the checks that can't report one.
pub fn count_errors_in(project: &Project, build: &Build) -> usize {
    let checker = PageChecker::new(project).errors_only();
    let mut diagnostics = check_files_with(project, Some(&checker));
    diagnostics.extend(checker.check_builds(&[build]));
    let diagnostics = apply_levels(&project.model().checks, diagnostics);
    Acknowledgements::of(project)
        .apply(project.model(), diagnostics, false)
        .diagnostics
        .iter()
        .filter(|d| d.severity == crate::Severity::Error)
        .count()
}

/// The diagnostics located in `files` (content paths of source files) of
/// the checks across the project that hold for every build: the
/// navigation's, the permalinks list's, the glossary's, `fragment-unused`,
/// and the content model's unused entries, with levels and acknowledgements
/// applied. The language server reports them in a file on save, besides
/// what [`diagnose_editor_build`] reports for it; the agents' hook adds them
/// after an edit, for the advice with a safe fix it passes on.
pub fn diagnose_every_build_in(project: &Project, files: &[RelPath]) -> Vec<Diagnostic> {
    let ids: Vec<ascribe_core::FileId> = files
        .iter()
        .filter_map(|path| project.source_at(path))
        .map(|file| file.id)
        .collect();
    let diagnostics = PageChecker::new(project)
        .for_every_build(false)
        .into_iter()
        .filter(|(id, _)| ids.contains(id))
        .flat_map(|(_, found)| found)
        .collect();
    let diagnostics =
        crate::generated::mark(project, apply_levels(&project.model().checks, diagnostics));
    Acknowledgements::of(project)
        .apply(project.model(), diagnostics, false)
        .diagnostics
}

/// What the language server reports, for `ascribe check --editor-build`:
/// the file-level diagnostics, and the page-level ones of the editor's build
/// alone (`[editor] build`; `ContentModel::editor_default_build`), without
/// the pass over content no build publishes, and without the image checks
/// ([`NOT_IN_EDITOR`](crate::NOT_IN_EDITOR)).
///
/// With `files` (content paths of source files), only what can count for
/// them is checked, for speed: the file-level checks of those files, and the
/// page-level checks of the pages that are them or include them. That's every
/// diagnostic located in them or with a related place in them, apart from a
/// file-level one of another file whose related place is in them, which no
/// file-level check makes. The content model's warnings and the checks of
/// `ascribe.lock` aren't run then, since they're located in neither, and
/// neither are the content checks across the project, which need every page
/// (the server runs them on save).
pub fn diagnose_editor_build<'p>(project: &'p Project, files: Option<&[RelPath]>) -> Diagnosed<'p> {
    let build = project.model().editor_default_build();
    let checker = || PageChecker::new(project);
    let diagnostics = match files {
        None => {
            let checker = checker();
            let mut out = check_files_with(project, Some(&checker));
            out.extend(checker.check(build));
            // What the editor doesn't report.
            out.retain(|d| !crate::NOT_IN_EDITOR.contains(&d.slug));
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
    // One build: whether an acknowledgement covers nothing in the others
    // can't be told.
    let diagnostics =
        crate::generated::mark(project, apply_levels(&project.model().checks, diagnostics));
    let applied = Acknowledgements::of(project).apply(project.model(), diagnostics, false);
    Diagnosed {
        diagnostics: applied.diagnostics,
        acknowledged: applied.acknowledged,
        builds: vec![build],
    }
}
