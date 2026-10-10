//! `ascribe diff` and `ascribe drift` on a project loaded from disk: the
//! working tree against a base revision, from choosing the builds to the
//! finished report.

use ascribe_check::{count_errors, select_builds};
use ascribe_core::RelPath;
use ascribe_resolve::Project;

use crate::drift::{BaseProject, drift_with, read_base};
use crate::html::{
    AssetFiles, DiskAssets, GitAssets, MAX_PAGES, ReportOptions, Version, write_report,
};
use crate::prompt::{self, Review};
use crate::{Base, DiffError, DriftReport, Report, Repository, Side, compare_builds};
use crate::{BuildDiff, PageDiff};
use ascribe_check::prompt::{Builds, Context};

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
    before: BaseProject,
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
        let page_prompt = |build: &str, page: &PageDiff| self.page_prompt(build, page);
        write_report(
            &self.report,
            base,
            now,
            &ReportOptions {
                max_pages: MAX_PAGES,
                prompt: Some(&page_prompt),
            },
        )
    }

    /// What the prompts are compared with, and where the content is.
    fn review(&self) -> Review<'_> {
        Review {
            base: &self.report.base,
            content_root: &self.now.layout().content_root,
        }
    }

    /// The agent prompt about a changed page of `build`, as the report's Copy
    /// prompt button and `ascribe diff --format prompt <PAGE>` give it.
    pub fn page_prompt(&self, build: &str, page: &PageDiff) -> String {
        let context = Context::of_project(&self.root, Builds::Named(vec![build.to_owned()]));
        prompt::page(&context, &self.review(), build, page)
    }

    /// The agent prompt about the page or fragment at a content path: the
    /// page's, in the first build it changed in; else the fragment's reach,
    /// in the first build with a page that changed through it. `None` when
    /// neither changed.
    pub fn prompt_about(&self, path: &RelPath) -> Option<String> {
        let builds = &self.report.builds;
        if let Some((build, page)) = builds.iter().find_map(|b| {
            b.pages
                .iter()
                .find(|p| p.path == path.as_str())
                .map(|p| (b, p))
        }) {
            return Some(self.page_prompt(&build.build, page));
        }
        builds.iter().find_map(|build: &BuildDiff| {
            let context = Context::of_project(&self.root, Builds::Named(vec![build.build.clone()]));
            prompt::fragment_reach(&context, &self.review(), build, path)
        })
    }

    /// The agent prompt about every changed page; `builds` are the builds
    /// compared, as the prompt names them. `None` when nothing changed.
    pub fn pages_prompt(&self, builds: Builds) -> Option<String> {
        let context = Context::of_project(&self.root, builds);
        prompt::pages(&context, &self.review(), &self.report.builds)
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
    project: &ascribe_check::Project,
    options: &DiffOptions<'_>,
) -> Result<ProjectDiff, DiffError> {
    let builds = select_builds(project, options.builds)?;
    let repo = Repository::discover(project.root())?;
    let base = repo.base(options.base, options.base_exact)?;
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    // Counting the errors is a full check of the working tree, the longest
    // part: it runs beside everything else, as reading and indexing the base
    // does beside indexing the working tree.
    let (before, now, diffs, errors) = std::thread::scope(|scope| {
        let errors = scope.spawn(|| count_errors(project, options.builds).unwrap_or(0));
        let before = scope.spawn(|| read_base(&repo, base.compared()));
        let now = project.index();
        let before = joined(before)?;
        let before_side = before.as_ref().map(|(revision, project)| Side {
            project,
            model_text: &revision.model_text,
        });
        let now_side = Side {
            project: &now,
            model_text: project.model_text(),
        };
        let diffs = compare_builds(before_side, now_side, &names);
        Ok::<_, DiffError>((before, now, diffs, joined(errors)))
    })?;
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

/// What a scoped thread returned, or its panic, again.
fn joined<T>(thread: std::thread::ScopedJoinHandle<'_, T>) -> T {
    thread
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
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
    project: &ascribe_check::Project,
    options: &DriftOptions<'_>,
) -> Result<DriftReport, DiffError> {
    let builds = select_builds(project, options.builds)?;
    let repo = Repository::discover(project.root())?;
    let base: Base = repo.base(options.base, options.base_exact)?;
    let names: Vec<&str> = builds.iter().map(|b| b.name.as_str()).collect();
    // The base's pages are read only when an example changed, which takes
    // the working tree's index to know. When `git diff` lists a file in a
    // source's folder, one likely did: the base is read beside the index
    // then, instead of after it. A listing that fails is listed again, and
    // reported, only when it's needed, as `drift` does.
    let changes = repo.changed_files(base.compared()).ok();
    let likely = changes
        .as_ref()
        .is_some_and(|changes| in_sources(project, &repo, changes.keys()));
    std::thread::scope(|scope| {
        let early = likely.then(|| scope.spawn(|| read_base(&repo, base.compared())));
        let now_project = project.index();
        let now = Side {
            project: &now_project,
            model_text: project.model_text(),
        };
        drift_with(
            &repo,
            &base,
            now,
            project.file_system(),
            &names,
            changes,
            || match early {
                Some(thread) => joined(thread),
                None => read_base(&repo, base.compared()),
            },
        )
    })
}

/// Whether any of these files, by path from the repository's root, is in
/// the folder of one of the project's sources (where code files are).
fn in_sources<'a>(
    project: &ascribe_check::Project,
    repo: &Repository,
    mut files: impl Iterator<Item = &'a RelPath>,
) -> bool {
    let project_dir = repo.project_dir();
    let folders: Vec<RelPath> = project
        .model()
        .sources
        .iter()
        .filter_map(|source| project_dir.join(&source.path).ok())
        .collect();
    files.any(|file| folders.iter().any(|folder| file.starts_with(folder)))
}
