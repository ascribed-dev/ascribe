//! What state a project is in: `ascribe report`'s sections, each from what
//! the checks, the source index, the builds, and the tools a project
//! installs already compute.
//!
//! - **problems**: what `ascribe check` reports, and the problems
//!   acknowledged as intended;
//! - **inventory**: the pages by type and by owner, and the overdue reviews,
//!   orphan pages, and unused entries the checks find, as lists;
//! - **builds**: the pages and the content each build leaves out that
//!   another keeps;
//! - **links**: the external links a link checker finds broken or moved
//!   ([`ascribe_check::links`]);
//! - **agents**: the delivery spec's checks on a built site
//!   ([`ascribe_check::site`]).
//!
//! The first three need nothing outside the project. Each section is
//! independent: one that can't run says why ([`NotRun`]), and the others
//! still run. The answer is typed; `ascribe report` writes it as text,
//! Markdown, JSON, or a prompt.

use std::collections::BTreeMap;
use std::time::Duration;

use ascribe_check::links::{self, LinkChecker};
use ascribe_check::site::{self, CheckResult, SiteChecker};
use ascribe_check::tool::ToolError;
use ascribe_check::{
    Acknowledged, Acknowledgements, Diagnostic, Project, UnknownBuild, apply_levels, diagnose,
    select_builds,
};
use ascribe_core::{DiagnosticSlug, FileId, Location, RelPath, diagnostics};
use ascribe_emit::labels::removal_detail;
use ascribe_model::{FieldRole, TypeMatch};
use ascribe_resolve::{FileKind, Removal};

/// A section of the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Section {
    /// What `ascribe check` reports, and what's acknowledged.
    Problems,
    /// Pages by type and owner, overdue reviews, orphans, unused entries.
    Inventory,
    /// What each build leaves out that another keeps.
    Builds,
    /// External links that fail, redirect, or time out.
    Links,
    /// The delivery spec's checks on a built site.
    Agents,
}

impl Section {
    /// Every section, in the order the report shows them.
    pub const ALL: [Section; 5] = [
        Section::Problems,
        Section::Inventory,
        Section::Builds,
        Section::Links,
        Section::Agents,
    ];

    /// The sections that need nothing outside the project: those the report
    /// runs when none is named.
    pub const LOCAL: [Section; 3] = [Section::Problems, Section::Inventory, Section::Builds];

    /// Its name on the command line and in the JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Section::Problems => "problems",
            Section::Inventory => "inventory",
            Section::Builds => "builds",
            Section::Links => "links",
            Section::Agents => "agents",
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
    /// The built site `agents` checks.
    pub site: Option<&'a str>,
    /// The programs to run.
    pub tools: &'a Tools<'a>,
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
    /// The pages whose review date has passed (`review-overdue`).
    pub overdue: Vec<Diagnostic>,
    /// The pages nothing links to (`page-orphan`).
    pub orphans: Vec<Diagnostic>,
    /// The fragments, phrases, features, glossary terms, and images nothing
    /// uses.
    pub unused: Vec<Diagnostic>,
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

/// `links`: what the link checker said about the external links.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Links {
    /// How many different addresses were checked.
    pub checked: usize,
    /// How many `[checks.links] ignore` left out.
    pub ignored: usize,
    /// A diagnostic for each link that moved or is broken, at its place, at
    /// its `[checks]` level, and the acknowledgements that cover nothing.
    pub diagnostics: Vec<Diagnostic>,
    /// The broken links acknowledged as intended.
    pub acknowledged: Vec<Acknowledged>,
}

/// `agents`: the delivery spec's checks on a built site.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Agents {
    /// The site's address.
    pub site: String,
    /// Each check's result, in the checker's order.
    pub results: Vec<CheckResult>,
    /// A diagnostic for each check that failed or warned that Ascribe or the
    /// hosting owns, at its `[checks]` level.
    pub diagnostics: Vec<Diagnostic>,
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
    /// `links`, when asked for.
    pub links: Option<Ran<Links>>,
    /// `agents`, when asked for.
    pub agents: Option<Ran<Agents>>,
}

impl Report {
    /// The sections that were asked for and didn't run, with why.
    pub fn not_run(&self) -> Vec<(Section, &NotRun)> {
        let mut out = Vec::new();
        if let Some(Ran::NotRun(why)) = &self.links {
            out.push((Section::Links, why));
        }
        if let Some(Ran::NotRun(why)) = &self.agents {
            out.push((Section::Agents, why));
        }
        out
    }

    /// Every finding: the diagnostics of `problems`, `links`, and `agents`.
    pub fn findings(&self) -> impl Iterator<Item = &Diagnostic> {
        let problems = self.problems.iter().flat_map(|p| &p.diagnostics);
        let links = self
            .links
            .iter()
            .filter_map(Ran::done)
            .flat_map(|l| &l.diagnostics);
        let agents = self
            .agents
            .iter()
            .filter_map(Ran::done)
            .flat_map(|a| &a.diagnostics);
        problems.chain(links).chain(agents)
    }
}

/// The report on `project`.
///
/// # Errors
///
/// A build named isn't the content model's.
pub fn report(project: &Project, options: &Options<'_>) -> Result<Report, UnknownBuild> {
    let asked = |s: Section| options.sections.contains(&s);
    let selected = select_builds(project, options.builds)?;
    let mut out = Report {
        builds_checked: selected.iter().map(|b| b.name.clone()).collect(),
        ..Report::default()
    };
    if asked(Section::Problems) || asked(Section::Inventory) {
        let diagnosed = diagnose(project, options.builds)?;
        if asked(Section::Inventory) {
            out.inventory = Some(inventory(project, &diagnosed.diagnostics));
        }
        if asked(Section::Problems) {
            out.problems = Some(Problems {
                diagnostics: diagnosed.diagnostics,
                acknowledged: diagnosed.acknowledged,
            });
        }
    }
    if asked(Section::Builds) {
        out.builds = Some(builds(project, &out.builds_checked));
    }
    if asked(Section::Links) {
        out.links = Some(links(project, options.tools));
    }
    if asked(Section::Agents) {
        out.agents = Some(agents(project, options.site, options.tools));
    }
    Ok(out)
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
        let ty = match model.type_for(file.path.as_str()) {
            TypeMatch::One(ty) => Some(ty),
            TypeMatch::Ambiguous(_) | TypeMatch::None => None,
        };
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
        overdue: listed(&[diagnostics::REVIEW_OVERDUE]),
        orphans: listed(&[diagnostics::PAGE_ORPHAN]),
        unused: listed(UNUSED),
    }
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

/// `links`: runs the link checker, and applies `[checks]` levels and the
/// acknowledgements, reporting those of `link-external-broken` that cover
/// nothing.
fn links(project: &Project, tools: &Tools<'_>) -> Ran<Links> {
    match links::check(project, tools.links, tools.links_timeout) {
        Ok(found) => {
            let model = project.model();
            let leveled = apply_levels(&model.checks, found.diagnostics);
            let applied = Acknowledgements::of(project).apply_ran(
                model,
                leveled,
                &[diagnostics::LINK_EXTERNAL_BROKEN],
            );
            Ran::Done(Links {
                checked: found.checked,
                ignored: found.ignored,
                diagnostics: applied.diagnostics,
                acknowledged: applied.acknowledged,
            })
        }
        Err(e) => Ran::NotRun(not_run(&e, LYCHEE_HOW)),
    }
}

/// What gets the link checker: installing it, or saying where it is.
const LYCHEE_HOW: &str = "Install lychee (`cargo install lychee`, or a release from https://github.com/lycheeverse/lychee/releases), or set `[checks.links] command` to where it is";

/// `agents`: runs the site checker on `site`, and applies `[checks]` levels.
fn agents(project: &Project, site: Option<&str>, tools: &Tools<'_>) -> Ran<Agents> {
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
        Ok(found) => Ran::Done(Agents {
            site: site.to_owned(),
            results: found.results,
            diagnostics: apply_levels(&project.model().checks, found.diagnostics),
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
