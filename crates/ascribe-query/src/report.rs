//! What state a project is in: `ascribe report`'s sections, each from what
//! the checks, the source index, the builds, and the tools a project
//! installs already compute.
//!
//! - **problems**: what `ascribe check` reports, and the problems
//!   acknowledged as intended;
//! - **inventory**: the pages by type and by owner, how many of each
//!   declared API's operations a link names, and the overdue rechecks, orphan
//!   pages, and unused entries the checks find, as lists;
//! - **builds**: the pages and the content each build leaves out that
//!   another keeps;
//! - **pages**: each build's page records ([`ascribe_resolve::pages`]),
//!   what every reader that isn't a person is told about each page, sorted
//!   by address;
//! - **links**: the external links a link checker finds broken or moved
//!   ([`ascribe_check::links`]);
//! - **delivery**: the delivery spec's checks on a built site
//!   ([`ascribe_check::site`]).
//!
//! The first four need nothing outside the project. Each section is
//! independent: one that can't run says why ([`NotRun`]), and the others
//! still run. The answer is typed; `ascribe report` writes it as text,
//! Markdown, JSON, or a prompt.

use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use ascribe_check::identity::{self, Identity};
use ascribe_check::links::{self, LinkChecker};
use ascribe_check::report_history::{self, ReportHistory, SiteChecks};
use ascribe_check::site::{self, CheckResult, SiteChecker};
use ascribe_check::tool::ToolError;
use ascribe_check::{
    Acknowledged, Acknowledgements, DiagnoseError, Diagnostic, Project, Request, apply_levels,
    diagnose_with, select_builds,
};
use ascribe_core::{DiagnosticSlug, FileId, Location, RelPath, diagnostics};
use ascribe_emit::labels::removal_detail;
use ascribe_model::{FieldRole, Listing};
use ascribe_resolve::pages::{self, PageRecord};
use ascribe_resolve::{AstroRouter, ExampleStatus, FileKind, Removal};

/// A section of the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// What `ascribe check` reports, and what's acknowledged.
    Problems,
    /// Pages by type and owner, overdue rechecks, orphans, unused entries.
    Inventory,
    /// What each build leaves out that another keeps.
    Builds,
    /// Each build's page records.
    Pages,
    /// External links that fail, redirect, or time out.
    Links,
    /// The delivery spec's checks on a built site.
    Delivery,
}

impl Section {
    /// Every section, in the order the report shows them.
    pub const ALL: [Section; 6] = [
        Section::Problems,
        Section::Inventory,
        Section::Builds,
        Section::Pages,
        Section::Links,
        Section::Delivery,
    ];

    /// The sections the report runs when none is named: those that need
    /// nothing outside the project, apart from `pages`, which lists every
    /// page rather than saying what state the project is in.
    pub const LOCAL: [Section; 3] = [Section::Problems, Section::Inventory, Section::Builds];

    /// Its name on the command line and in the JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Section::Problems => "problems",
            Section::Inventory => "inventory",
            Section::Builds => "builds",
            Section::Pages => "pages",
            Section::Links => "links",
            Section::Delivery => "delivery",
        }
    }
}

/// The programs the report runs, and how long each may take: the real
/// ones, or stand-ins in tests.
pub struct Tools<'a> {
    /// What checks external links.
    pub links: &'a dyn LinkChecker,
    /// What checks a built site.
    pub site: &'a dyn SiteChecker,
    /// How long the link checker may take.
    pub links_timeout: Duration,
    /// How long to wait before asking the link checker again about the
    /// addresses that answered with a server error.
    pub links_retry_wait: Duration,
    /// How long the site checker may take.
    pub site_timeout: Duration,
}

impl Tools<'static> {
    /// The programs a project installs: lychee and afdocs.
    pub fn installed() -> Tools<'static> {
        Tools {
            links: &links::Lychee,
            site: &site::Afdocs,
            links_timeout: links::TIMEOUT,
            links_retry_wait: links::RETRY_WAIT,
            site_timeout: site::TIMEOUT,
        }
    }
}

/// What to report.
pub struct Options<'a> {
    /// The sections, each once, in the order [`Section::ALL`] has them.
    pub sections: &'a [Section],
    /// The builds `problems` checks and `builds` reports on; every build
    /// when empty.
    pub builds: &'a [String],
    /// Whether `ascribe.baseline` holds back the advice it lists from
    /// `problems` and `inventory`, as it does from `ascribe check`.
    pub baseline: bool,
    /// For `pages`, only the pages that aren't `listed`.
    pub not_listed: bool,
    /// The built site `delivery` checks.
    pub site: Option<&'a str>,
    /// The programs to run.
    pub tools: &'a Tools<'a>,
    /// The answers `links` reuses, and where it keeps those it gets; none
    /// to ask about every address and keep nothing.
    pub link_cache: Option<&'a links::Cache>,
    /// What `links` remembers from earlier runs, and where it keeps what
    /// this run sees; none to hold nothing and keep nothing.
    pub link_history: Option<&'a links::History>,
    /// What the report remembers of its other sections from the last run,
    /// and where it keeps what this run finds; none to say nothing about
    /// what's new and keep nothing.
    pub history: Option<&'a ReportHistory>,
}

/// Why a section didn't run, and what would let it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotRun {
    /// Why, as a sentence without its full stop.
    pub reason: String,
    /// What to do so that it runs, as a sentence without its full stop.
    pub how: String,
}

/// A section that runs a program: what it found, or why it didn't run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ran<T> {
    /// It ran.
    Done(T),
    /// It didn't.
    NotRun(NotRun),
}

impl<T> Ran<T> {
    /// What it found, when it ran.
    pub fn done(&self) -> Option<&T> {
        match self {
            Ran::Done(found) => Some(found),
            Ran::NotRun(_) => None,
        }
    }
}

/// `problems`: what `ascribe check` reports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Problems {
    /// The diagnostics, in the order `ascribe check` reports them.
    pub diagnostics: Vec<Diagnostic>,
    /// The problems acknowledged as intended, with their reasons.
    pub acknowledged: Vec<Acknowledged>,
    /// How many advice `ascribe.baseline` holds back, which aren't in
    /// `diagnostics`: the backlog the project has put off.
    pub baselined: usize,
    /// The day of the run the history is from, when there's one to go by.
    pub previous_run: Option<ascribe_core::Date>,
    /// The diagnostics the previous run didn't report, in the order of
    /// `diagnostics`, when there's one to go by.
    pub new: Option<Vec<Diagnostic>>,
    /// How many findings the previous run reported that this one doesn't,
    /// when there's one to go by.
    pub resolved: Option<usize>,
}

/// `inventory`: what the project holds, and what the checks across it find.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inventory {
    /// How many pages, fragments left out.
    pub pages: usize,
    /// How many pages of each content type, most first; `None` for pages no
    /// one type applies to.
    pub by_type: Vec<(Option<String>, usize)>,
    /// How many pages each owner has, most first: the value of the field
    /// the page's type marks `role = "owner"`; `None` for pages without
    /// one.
    pub by_owner: Vec<(Option<String>, usize)>,
    /// The pages whose recheck date has passed (`recheck-overdue`).
    pub overdue: Vec<Diagnostic>,
    /// The pages nothing links to (`page-orphan`).
    pub orphans: Vec<Diagnostic>,
    /// The fragments, phrases, features, glossary terms, and images nothing
    /// uses.
    pub unused: Vec<Diagnostic>,
    /// The tested examples the snippets show, for each source in the same
    /// repository with an `ascribe.examples.toml`, in the content model's
    /// order; `None` when no source has one.
    pub examples: Option<Vec<SourceExamples>>,
    /// How much of each declared API whose spec was read the pages cover, in
    /// the content model's order; `None` when the project declares none.
    pub apis: Option<Vec<ApiOperations>>,
}

/// How many of one API's operations the pages cover.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ApiOperations {
    /// The API's name.
    pub api: String,
    /// How many operations its spec has.
    pub operations: usize,
    /// How many of them a link names, by id or by method and path.
    pub linked: usize,
    /// How many of them a checked example claims as its request or
    /// response.
    pub examples: usize,
}

/// How many snippets of one source show a tested example, by status.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceExamples {
    /// The source's name.
    pub source: String,
    /// How many `@snippet` lines show one of its examples.
    pub snippets: usize,
    /// How many of those show an example an observation is current for.
    pub current: usize,
    /// How many show one that changed after every observation listing it.
    pub stale: usize,
    /// How many show one no observation lists.
    pub unobserved: usize,
}

/// The checks whose problems `inventory` lists as unused entries.
pub const UNUSED: &[DiagnosticSlug] = &[
    diagnostics::FRAGMENT_UNUSED,
    diagnostics::PHRASE_UNUSED,
    diagnostics::FEATURE_UNUSED,
    diagnostics::GLOSSARY_TERM_UNUSED,
    diagnostics::IMAGE_UNUSED,
];

/// `builds`: for each build, what it leaves out that another keeps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Builds {
    /// One for each build reported on, in the content model's order.
    pub builds: Vec<LeftOut>,
}

/// What one build leaves out that another keeps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LeftOut {
    /// The build.
    pub build: String,
    /// The pages it doesn't publish and another does, in path order.
    pub pages: Vec<LeftPage>,
    /// The content it takes out of a page it publishes, and another build
    /// keeps, in file and source order.
    pub content: Vec<LeftContent>,
}

/// A page a build doesn't publish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftPage {
    /// The page's content path.
    pub path: RelPath,
    /// Why, as the end of a sentence: `its variant frontmatter names …`.
    pub why: String,
    /// The builds that publish it.
    pub kept_by: Vec<String>,
}

/// Content a build takes out of a page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeftContent {
    /// Where it's written: a block, a variant arm, or a table row, in the
    /// page or a fragment it includes.
    pub location: Location,
    /// The page it's taken out of: the first, when it's in a fragment
    /// several pages include.
    pub page: RelPath,
    /// `variant` or `availability`.
    pub kind: &'static str,
    /// Why, with the content model's labels: `Shows only os=linux`.
    pub why: String,
    /// The builds that keep it.
    pub kept_by: Vec<String>,
}

/// `pages`: each build's page records.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pages {
    /// One for each build reported on, in the content model's order.
    pub builds: Vec<BuildPages>,
}

/// One build's page records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildPages {
    /// The build.
    pub build: String,
    /// The project's address, `[consumer] site`, without a final `/`.
    pub site: Option<String>,
    /// How many pages the build publishes.
    pub published: usize,
    /// How many of them aren't `listed`.
    pub not_listed: usize,
    /// The records, sorted by address: every page's, or with
    /// [`Options::not_listed`] those that aren't `listed`. Two pages at one
    /// address, which the site output refuses, are listed with the first,
    /// as `_ascribe/pages.json` has them.
    pub records: Vec<PageRecord>,
}

/// `links`: what the link checker said about the external links.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Links {
    /// How many different addresses were checked: `asked` and `reused`.
    pub checked: usize,
    /// How many different addresses the link checker was asked about.
    pub asked: usize,
    /// How many different addresses' answers were reused from an earlier
    /// run.
    pub reused: usize,
    /// The day of the oldest answer reused, when one was.
    pub oldest_answer: Option<ascribe_core::Date>,
    /// How many `[checks.links] ignore` left out.
    pub ignored: usize,
    /// A diagnostic for each link that moved or is broken, at its place, at
    /// its `[checks]` level, and the acknowledgements that cover nothing.
    /// None is unconfirmed.
    pub diagnostics: Vec<Diagnostic>,
    /// The broken links acknowledged as intended, confirmed or not.
    pub acknowledged: Vec<Acknowledged>,
    /// The broken links that answered before and aren't reported until
    /// they've kept failing, in file order and source order.
    pub unconfirmed: Vec<links::Unconfirmed>,
    /// The day of the run the history is from, when there's one to go by.
    pub previous_run: Option<ascribe_core::Date>,
    /// How many of `diagnostics` the previous run didn't report, when
    /// there's one to go by.
    pub new: Option<usize>,
    /// How many addresses the previous run reported that this one doesn't,
    /// when there's one to go by.
    pub resolved: Option<usize>,
}

/// `delivery`: the delivery spec's checks on a built site.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Delivery {
    /// The site's address.
    pub site: String,
    /// Each check's result, in the checker's order.
    pub results: Vec<CheckResult>,
    /// A diagnostic for each check that failed or warned that Ascribe or the
    /// hosting owns, at its `[checks]` level.
    pub diagnostics: Vec<Diagnostic>,
    /// The day of the run the history is from, when there's one to go by:
    /// one on the same site.
    pub previous_run: Option<ascribe_core::Date>,
    /// The ids of the checks failing now, failed or warned, that passed or
    /// weren't run on the previous run, in the checker's order, when
    /// there's one to go by.
    pub new: Option<Vec<String>>,
    /// How many checks failing on the previous run pass now or aren't run,
    /// when there's one to go by.
    pub resolved: Option<usize>,
}

/// The report: each section asked for, and the builds it's about.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The builds `problems` checked and `builds` reports on, in the content
    /// model's order.
    pub builds_checked: Vec<String>,
    /// `problems`, when asked for.
    pub problems: Option<Problems>,
    /// `inventory`, when asked for.
    pub inventory: Option<Inventory>,
    /// `builds`, when asked for.
    pub builds: Option<Builds>,
    /// `pages`, when asked for.
    pub pages: Option<Pages>,
    /// `links`, when asked for.
    pub links: Option<Ran<Links>>,
    /// `delivery`, when asked for.
    pub delivery: Option<Ran<Delivery>>,
}

impl Report {
    /// The sections that were asked for and didn't run, with why.
    pub fn not_run(&self) -> Vec<(Section, &NotRun)> {
        let mut out = Vec::new();
        if let Some(Ran::NotRun(why)) = &self.links {
            out.push((Section::Links, why));
        }
        if let Some(Ran::NotRun(why)) = &self.delivery {
            out.push((Section::Delivery, why));
        }
        out
    }

    /// Every problem the report found: the diagnostics of `problems`,
    /// `links`, and `delivery`.
    pub fn all_problems(&self) -> impl Iterator<Item = &Diagnostic> {
        let problems = self.problems.iter().flat_map(|p| &p.diagnostics);
        let links = self
            .links
            .iter()
            .filter_map(Ran::done)
            .flat_map(|l| &l.diagnostics);
        let delivery = self
            .delivery
            .iter()
            .filter_map(Ran::done)
            .flat_map(|a| &a.diagnostics);
        problems.chain(links).chain(delivery)
    }
}

/// The report on `project`.
///
/// # Errors
///
/// A build named isn't the content model's, or, with
/// [`Options::baseline`], `ascribe.baseline` can't be read.
pub fn report(project: &Project, options: &Options<'_>) -> Result<Report, DiagnoseError> {
    let asked = |s: Section| options.sections.contains(&s);
    let selected = select_builds(project, options.builds)?;
    let mut out = Report {
        builds_checked: selected.iter().map(|b| b.name.clone()).collect(),
        ..Report::default()
    };
    // What this run found, for the history to keep.
    let mut kept = report_history::Found::default();
    if asked(Section::Problems) || asked(Section::Inventory) {
        let mut checked = diagnose_with(
            project,
            &Request {
                builds: options.builds,
                baseline: options.baseline,
                ..Request::default()
            },
        )?;
        let baselined = checked.baselined.len();
        let acknowledged = std::mem::take(&mut checked.acknowledged);
        let diagnostics = checked.into_diagnostics();
        if asked(Section::Inventory) {
            out.inventory = Some(inventory(project, &diagnostics));
        }
        if asked(Section::Problems) {
            let (mut problems, found) =
                problems(project, diagnostics, acknowledged, options.history);
            problems.baselined = baselined;
            out.problems = Some(problems);
            kept.problems = found;
        }
    }
    if asked(Section::Builds) {
        out.builds = Some(builds(project, &out.builds_checked));
    }
    if asked(Section::Pages) {
        out.pages = Some(page_records(
            project,
            &out.builds_checked,
            options.not_listed,
        ));
    }
    if asked(Section::Links) {
        out.links = Some(links(project, options));
    }
    if asked(Section::Delivery) {
        let mut ran = delivery(project, options.site, options.tools);
        if let (Ran::Done(delivery), Some(history)) = (&mut ran, options.history) {
            kept.delivery = Some(delivery_since(delivery, history));
        }
        out.delivery = Some(ran);
    }
    if let Some(history) = options.history
        && (kept.problems.is_some() || kept.delivery.is_some())
    {
        history.save(kept);
    }
    Ok(out)
}

/// `problems`: what `ascribe check` found, and, from the history, which of
/// it is new since the last run and how many of that run's findings are
/// gone; with what this run found, for the history to keep.
fn problems(
    project: &Project,
    diagnostics: Vec<Diagnostic>,
    acknowledged: Vec<Acknowledged>,
    history: Option<&ReportHistory>,
) -> (Problems, Option<BTreeSet<Identity>>) {
    let mut out = Problems {
        diagnostics,
        acknowledged,
        ..Problems::default()
    };
    let Some(history) = history else {
        return (out, None);
    };
    let found = identity::identities(project, &out.diagnostics);
    if let Some((run, before)) = history.problems_before() {
        out.previous_run = Some(run);
        out.new = Some(
            out.diagnostics
                .iter()
                .zip(&found)
                .filter(|(_, id)| !before.contains(*id))
                .map(|(d, _)| d.clone())
                .collect(),
        );
        let now: BTreeSet<&Identity> = found.iter().collect();
        out.resolved = Some(before.iter().filter(|id| !now.contains(id)).count());
    }
    (out, Some(found.into_iter().collect()))
}

/// Which of `delivery`'s checks are failing since the last run on its site,
/// and how many failing then aren't now, from the history; and what this
/// run found, for the history to keep. A check's identity is the site's
/// address and its id. Failing is failed or warned; one that was skipped
/// or couldn't run, then or now, counts as neither new nor resolved.
fn delivery_since(delivery: &mut Delivery, history: &ReportHistory) -> SiteChecks {
    let found = SiteChecks {
        site: delivery.site.clone(),
        checks: delivery
            .results
            .iter()
            .map(|r| (r.id.clone(), r.status))
            .collect(),
    };
    if let Some((run, before)) = history.delivery_before(&delivery.site) {
        let passing = |id: &str| before.get(id).is_none_or(|s| *s == site::Status::Pass);
        delivery.previous_run = Some(run);
        delivery.new = Some(
            delivery
                .results
                .iter()
                .filter(|r| r.status.is_finding() && passing(&r.id))
                .map(|r| r.id.clone())
                .collect(),
        );
        delivery.resolved = Some(
            before
                .iter()
                .filter(|(id, status)| {
                    status.is_finding()
                        && found
                            .checks
                            .get(**id)
                            .is_none_or(|s| *s == site::Status::Pass)
                })
                .count(),
        );
    }
    found
}

/// `inventory`, from the project's pages and what `ascribe check` found.
fn inventory(project: &Project, found: &[Diagnostic]) -> Inventory {
    let model = project.model();
    let index = project.held_index();
    let mut by_type: BTreeMap<Option<String>, usize> = BTreeMap::new();
    let mut by_owner: BTreeMap<Option<String>, usize> = BTreeMap::new();
    let mut pages = 0;
    for file in index.files().filter(|f| f.kind == FileKind::Page) {
        pages += 1;
        let ty = file.content_type(model);
        *by_type.entry(ty.map(|t| t.name.clone())).or_default() += 1;
        let owner = ty
            .and_then(|t| t.frontmatter.field_with_role(FieldRole::Owner))
            .and_then(|field| file.frontmatter.as_ref()?.get(field.name.as_str()))
            .and_then(|value| match value {
                serde_yaml_ng::Value::String(s) if !s.trim().is_empty() => {
                    Some(s.trim().to_owned())
                }
                _ => None,
            });
        *by_owner.entry(owner).or_default() += 1;
    }
    let listed = |slugs: &[DiagnosticSlug]| -> Vec<Diagnostic> {
        found
            .iter()
            .filter(|d| slugs.contains(&d.slug))
            .cloned()
            .collect()
    };
    Inventory {
        pages,
        by_type: most_first(by_type),
        by_owner: most_first(by_owner),
        overdue: listed(&[diagnostics::RECHECK_OVERDUE]),
        orphans: listed(&[diagnostics::PAGE_ORPHAN]),
        unused: listed(UNUSED),
        examples: examples(project, &index),
        apis: apis(&index),
    }
}

/// How many of each declared API's operations the links name: `None` when
/// the project declares no API.
fn apis(index: &ascribe_resolve::Project) -> Option<Vec<ApiOperations>> {
    if index.model().apis.is_empty() {
        return None;
    }
    let counts = index
        .api_coverage()
        .into_iter()
        .map(|c| ApiOperations {
            operations: c.operations(),
            linked: c.linked(),
            examples: c.examples,
            api: c.api,
        })
        .collect();
    Some(counts)
}

/// The tested examples each source's snippets show, counted once for each
/// `@snippet` line, in pages and fragments alike: `None` when no source has
/// an `ascribe.examples.toml`. A source in another repository's is the copy
/// `ascribe sources fetch` puts beside its files.
fn examples(project: &Project, index: &ascribe_resolve::Project) -> Option<Vec<SourceExamples>> {
    let model = project.model();
    let (fs, code) = (project.file_system(), project.code_files());
    let mut counts: Vec<SourceExamples> = model
        .sources
        .iter()
        .filter(|s| {
            RelPath::parse(&s.path).is_ok_and(|folder| code.examples(fs, &folder).config.is_some())
        })
        .map(|s| SourceExamples {
            source: s.name.clone(),
            ..SourceExamples::default()
        })
        .collect();
    if counts.is_empty() {
        return None;
    }
    for file in index.files() {
        for snippet in &file.snippets {
            let Some(Ok(address)) = &snippet.address else {
                continue;
            };
            let Some(example) = index
                .snippet_at(&file.path, snippet.span)
                .and_then(|s| s.example.as_ref())
            else {
                continue;
            };
            let Some(count) = counts.iter_mut().find(|c| c.source == address.source) else {
                continue;
            };
            count.snippets += 1;
            match example.status {
                ExampleStatus::Current => count.current += 1,
                ExampleStatus::Stale => count.stale += 1,
                ExampleStatus::Unobserved => count.unobserved += 1,
            }
        }
    }
    Some(counts)
}

/// Counts, most first; a tie in key order, with `None` last.
fn most_first(counts: BTreeMap<Option<String>, usize>) -> Vec<(Option<String>, usize)> {
    let mut list: Vec<(Option<String>, usize)> = counts.into_iter().collect();
    list.sort_by(|(a, m), (b, n)| {
        n.cmp(m)
            .then_with(|| a.is_none().cmp(&b.is_none()))
            .then_with(|| a.cmp(b))
    });
    list
}

/// `builds`: for each build in `names`, the pages it doesn't publish and the
/// content it takes out that another build of the project keeps.
fn builds(project: &Project, names: &[String]) -> Builds {
    let model = project.model();
    let index = project.held_index();
    let all = &model.builds;
    let pages: Vec<&RelPath> = index
        .files()
        .filter(|f| f.kind == FileKind::Page)
        .map(|f| &f.path)
        .collect();
    let id = |file: FileId| {
        index
            .path_of(file)
            .and_then(|p| project.source_at(p))
            .map(|s| s.id)
    };
    let mut out = Builds::default();
    for build in all.iter().filter(|b| names.contains(&b.name)) {
        let others: Vec<_> = all.iter().filter(|b| b.name != build.name).collect();
        let mut left = LeftOut {
            build: build.name.clone(),
            ..LeftOut::default()
        };
        let mut seen: Vec<Location> = Vec::new();
        for page in &pages {
            if let Some(reason) = index.dropped(page, build) {
                let kept_by: Vec<String> = others
                    .iter()
                    .filter(|b| index.dropped(page, b).is_none())
                    .map(|b| b.name.clone())
                    .collect();
                if !kept_by.is_empty() {
                    left.pages.push(LeftPage {
                        path: (*page).clone(),
                        why: reason.explanation().to_owned(),
                        kept_by,
                    });
                }
                continue;
            }
            let Some(removed) = index.removed(page, build) else {
                continue;
            };
            // What each other build that publishes the page removes from it.
            let elsewhere: Vec<(&str, Vec<ascribe_resolve::Removed>)> = others
                .iter()
                .filter_map(|b| Some((b.name.as_str(), index.removed(page, b)?)))
                .collect();
            for r in &removed {
                let Some(file) = id(r.file) else {
                    continue;
                };
                let location = Location::new(file, r.span);
                if seen.contains(&location) {
                    continue;
                }
                let overlaps = |o: &ascribe_resolve::Removed| {
                    o.file == r.file
                        && o.span.start() < r.span.end()
                        && r.span.start() < o.span.end()
                };
                let kept_by: Vec<String> = elsewhere
                    .iter()
                    .filter(|(_, theirs)| !theirs.iter().any(overlaps))
                    .map(|(name, _)| (*name).to_owned())
                    .collect();
                if kept_by.is_empty() {
                    continue;
                }
                seen.push(location);
                left.content.push(LeftContent {
                    location,
                    page: (*page).clone(),
                    kind: match r.cause {
                        Removal::Variant { .. } => "variant",
                        Removal::Availability(_) => "availability",
                    },
                    why: removal_detail(model, build, r),
                    kept_by,
                });
            }
        }
        left.content
            .sort_by_key(|c| (c.location.file, c.location.span.start()));
        out.builds.push(left);
    }
    out
}

/// `pages`: the records of each build in `names`, sorted by address, as the
/// site output's `_ascribe/pages.json` holds them; with `not_listed`, only
/// the pages that aren't `listed`.
fn page_records(project: &Project, names: &[String], not_listed: bool) -> Pages {
    let model = project.model();
    let index = project.held_index();
    let router = AstroRouter::from_consumer(&model.consumer);
    let builds = model
        .builds
        .iter()
        .filter(|b| names.contains(&b.name))
        .map(|build| {
            let resolved = index.resolve_build(build, &router);
            let records = pages::records(model, &resolved.pages);
            let mut by_route: BTreeMap<String, PageRecord> = BTreeMap::new();
            for record in records.pages {
                by_route.entry(record.route.clone()).or_insert(record);
            }
            let published = by_route.len();
            let hidden = by_route
                .values()
                .filter(|r| r.listing != Listing::Listed)
                .count();
            BuildPages {
                build: build.name.clone(),
                site: records.site,
                published,
                not_listed: hidden,
                records: by_route
                    .into_values()
                    .filter(|r| !not_listed || r.listing != Listing::Listed)
                    .collect(),
            }
        })
        .collect();
    Pages { builds }
}

/// `links`: runs the link checker, and applies `[checks]` levels and the
/// acknowledgements, reporting those of `link-external-broken` that cover
/// nothing. A broken link the history holds is listed as unconfirmed, not
/// reported, and what this run reported is kept in the history.
fn links(project: &Project, options: &Options<'_>) -> Ran<Links> {
    let tools = options.tools;
    let history = options.link_history;
    let found = match links::check(
        project,
        tools.links,
        tools.links_timeout,
        tools.links_retry_wait,
        options.link_cache,
        history,
    ) {
        Ok(found) => found,
        Err(e) => return Ran::NotRun(not_run(&e, LYCHEE_HOW)),
    };
    let model = project.model();
    let leveled = apply_levels(&model.checks, found.diagnostics);
    let applied = Acknowledgements::of(project).apply_ran(
        model,
        leveled,
        &[diagnostics::LINK_EXTERNAL_BROKEN],
    );
    let held: BTreeMap<ascribe_core::Location, &links::Unconfirmed> =
        found.held.iter().map(|u| (u.location, u)).collect();
    let mut unconfirmed = Vec::new();
    let mut reported = Vec::new();
    for d in applied.diagnostics {
        match held.get(&d.location) {
            Some(u) if d.slug == diagnostics::LINK_EXTERNAL_BROKEN => {
                unconfirmed.push((*u).clone());
            }
            _ => reported.push(d),
        }
    }
    let urls: BTreeSet<&str> = reported
        .iter()
        .filter_map(|d| found.urls.get(&d.location).map(String::as_str))
        .collect();
    let (mut previous_run, mut new, mut resolved) = (None, None, None);
    if let (Some(history), Some(mut observed)) = (history, found.observed) {
        if let Some(run) = history.previous_run() {
            let before = history.reported_before();
            previous_run = Some(run);
            new = Some(
                reported
                    .iter()
                    .filter_map(|d| found.urls.get(&d.location))
                    .filter(|url| !before.contains(url.as_str()))
                    .count(),
            );
            resolved = Some(before.iter().filter(|url| !urls.contains(*url)).count());
        }
        for url in &urls {
            if let Some(record) = observed.get_mut(*url) {
                record.reported = true;
            }
        }
        history.save(observed);
    }
    Ran::Done(Links {
        checked: found.checked,
        asked: found.asked,
        reused: found.reused,
        oldest_answer: found.oldest_answer,
        ignored: found.ignored,
        diagnostics: reported,
        acknowledged: applied.acknowledged,
        unconfirmed,
        previous_run,
        new,
        resolved,
    })
}

/// What gets the link checker: installing it, or saying where it is.
const LYCHEE_HOW: &str = "Install lychee (`cargo install lychee`, or a release from https://github.com/lycheeverse/lychee/releases), or set `[checks.links] command` to where it is";

/// `delivery`: runs the site checker on `site`, and applies `[checks]` levels.
fn delivery(project: &Project, site: Option<&str>, tools: &Tools<'_>) -> Ran<Delivery> {
    let Some(site) = site else {
        let how = match &project.model().consumer.site {
            Some(published) => format!(
                "Give the address of a built site with `--site`, such as the published one, `--site {published}`"
            ),
            None => "Give the address of a built site with `--site <URL>`".to_owned(),
        };
        return Ran::NotRun(NotRun {
            reason: "it checks a built site, and no `--site` was given".to_owned(),
            how,
        });
    };
    match site::check(project, tools.site, site, tools.site_timeout) {
        Ok(found) => Ran::Done(Delivery {
            site: site.to_owned(),
            results: found.results,
            diagnostics: apply_levels(&project.model().checks, found.diagnostics),
            ..Delivery::default()
        }),
        Err(e) => {
            let how = format!(
                "Install the delivery spec's checker, `npm install -g afdocs@{}` (it needs Node.js 22 or later)",
                site::VERSION
            );
            Ran::NotRun(not_run(&e, &how))
        }
    }
}

/// Why a section's program said nothing, and what to do: `install` when it
/// couldn't be run at all.
fn not_run(e: &ToolError, install: &str) -> NotRun {
    let how = match e {
        ToolError::NotRun { .. } => install.to_owned(),
        ToolError::Failed { .. } => {
            "Fix what it says, then run the report again; run it on its own to see more".to_owned()
        }
        ToolError::TimedOut { .. } => "Run the report again when the network is quicker".to_owned(),
    };
    NotRun {
        reason: e.to_string(),
        how,
    }
}
