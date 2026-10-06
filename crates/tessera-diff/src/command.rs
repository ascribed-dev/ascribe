//! `ascribe diff` and `ascribe drift` on a project loaded from disk: the
//! working tree against a base revision, from choosing the builds to the
//! finished report.

use tessera_check::{diagnose, select_builds};
use tessera_resolve::Project;

use crate::html::{AssetFiles, DiskAssets, GitAssets, Version, write_html};
use crate::{
    Base, DiffError, DriftReport, Report, Repository, Revision, Side, compare_builds, drift,
};

/// What [`diff_project`] compares.
#[derive(Clone, Copy, Debug, Default)]
pub struct DiffOptions<'a> {
    /// The revision to compare with; the repository's default branch when
    /// `None`.
    pub base: Option<&'a str>,
    /// Compare with the revision itself instead of its merge base with
    /// `HEAD`.
    pub base_exact: bool,
    /// The builds to compare, as [`select_builds`] takes them: every build
    /// when empty.
    pub builds: &'a [String],
}

/// What [`diff_project`] found: the report, and what's needed to render it
/// as HTML.
pub struct ProjectDiff {
    /// The report, with the working tree's error count set.
    pub report: Report,
    repo: Repository,
    /// The base revision and its source index, when the project exists there.
    before: Option<(Revision, Project)>,
    now: Project,
    root: std::path::PathBuf,
}

impl std::fmt::Debug for ProjectDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProjectDiff")
            .field("report", &self.report)
            .finish_non_exhaustive()
    }
}

impl ProjectDiff {
    /// The report as one self-contained HTML file, every changed page
    /// rendered with its changes marked.
    pub fn html(&self) -> String {
        let now_files = DiskAssets::new(&self.root, &self.now);
        let base_files = self
            .before
            .as_ref()
            .map(|(revision, _)| GitAssets::new(&self.repo, &revision.fs));
        let base = self
            .before
            .as_ref()
            .zip(base_files.as_ref())
            .map(|((_, project), files)| Version {
                project,
                files: files as &dyn AssetFiles,
            });
        let now = Version {
            project: &self.now,
            files: &now_files,
        };
        write_html(&self.report, base, now)
    }
}

/// `ascribe diff`: what changed in each build between a base revision and
/// the working tree, which `project` was loaded from.
///
/// The report also counts the errors `ascribe check` finds in the working
/// tree for the same builds ([`Report::working_tree_errors`]): they don't
/// stop the comparison, since work in progress is worth comparing, but a
/// page with an error may not render as it will once it's fixed.
///
/// # Errors
///
/// A build named isn't the content model's, or the comparison can't run: not
/// a git repository, an unknown revision, no `git`, or a content model at the
/// base with errors.
pub fn diff_project(
    project: &tessera_check::Project,
    options: &DiffOptions<'_>,
) -> Result<ProjectDiff, DiffError> {
    let builds = select_builds(project, options.builds)?;
    let repo = Repository::discover(project.root())?;
    let base = repo.base(options.base, options.base_exact)?;
    let before = Revision::read(&repo, base.compared())?.map(|revision| {
        let index = revision.project();
        (revision, index)
    });
    let now = project.index();
    let before_side = before.as_ref().map(|(revision, project)| Side {
        project,
        model_text: &revision.model_text,
    });
    let now_side = Side {
        project: &now,
        model_text: project.model_text(),
    };
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    // Counting the errors is a full check of the working tree: it runs beside
    // the comparison, so the two take about as long as the slower one.
    let (diffs, errors) = std::thread::scope(|scope| {
        let errors = scope.spawn(|| {
            diagnose(project, options.builds).map_or(0, |found| {
                found
                    .diagnostics
                    .iter()
                    .filter(|d| d.severity == tessera_check::Severity::Error)
                    .count()
            })
        });
        let diffs = compare_builds(before_side, now_side, &names);
        (
            diffs,
            errors
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
        )
    });
    let mut report = Report::new(&repo, &base, diffs);
    report.working_tree_errors = errors;
    Ok(ProjectDiff {
        report,
        repo,
        before,
        now,
        root: project.root().to_owned(),
    })
}

/// What [`drift_project`] compares.
#[derive(Clone, Copy, Debug, Default)]
pub struct DriftOptions<'a> {
    /// The revision to compare with; the repository's default branch when
    /// `None`.
    pub base: Option<&'a str>,
    /// Compare with the revision itself instead of its merge base with
    /// `HEAD`.
    pub base_exact: bool,
    /// The builds to look at, as [`select_builds`] takes them: every build
    /// when empty.
    pub builds: &'a [String],
}

/// `ascribe drift`: the pages of the working tree, which `project` was
/// loaded from, whose examples changed since a base revision, and whether
/// the words around them did. [`drift`] does the work.
///
/// # Errors
///
/// A build named isn't the content model's, or the comparison can't run: not
/// a git repository, an unknown revision, a shallow clone, or no `git`.
pub fn drift_project(
    project: &tessera_check::Project,
    options: &DriftOptions<'_>,
) -> Result<DriftReport, DiffError> {
    let builds = select_builds(project, options.builds)?;
    let repo = Repository::discover(project.root())?;
    let base: Base = repo.base(options.base, options.base_exact)?;
    let now_project = project.index();
    let now = Side {
        project: &now_project,
        model_text: project.model_text(),
    };
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    drift(&repo, &base, now, project.file_system(), &names)
}
