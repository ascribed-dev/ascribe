//! `ascribe report --format json`: one document, versioned as the other
//! commands' are ([`SCHEMA_VERSION`]), and capped: each list holds at most
//! `--limit` items, says how many there are, and the report names the
//! command that lists them all.

use std::io::{self, Write};

use ascribe_check::links::Answer;
use ascribe_check::{Diagnostic, Next};
use ascribe_model::{ListingFrom, ListingLevel};
use ascribe_query::report::{
    Agents, BuildPages, Builds, LeftContent, LeftPage, Links, NotRun, Pages, Problems, Ran, Report,
    Section,
};
use ascribe_resolve::pages::PageRecord;
use serde::Serialize;

use super::text::{by_check, by_file, by_next};
use crate::report::json::{AcknowledgedEntry, Entry, Range, acknowledged, entry, is_zero, range};
use crate::report::{Counts, FileTable};

/// The version of the JSON schema. It changes only when a field is removed or
/// changes meaning; fields may be added without a new version.
pub const SCHEMA_VERSION: u32 = 1;

/// What `ascribe report --format json` writes: one document, whatever the
/// outcome. A reader ignores fields it doesn't know.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
pub(crate) struct ReportJson {
    /// The version of this schema. It changes only when a field is removed
    /// or changes meaning.
    schema_version: u32,
    /// The version of Ascribe that wrote it.
    ascribe_version: &'static str,
    /// Why the report couldn't be made (exit code 2), or `null`. When it
    /// isn't `null`, no section is there.
    error: Option<String>,
    /// The sections asked for, in the order the report has them:
    /// `problems`, `inventory`, `builds`, `pages`, `links`, `agents`.
    sections: Vec<&'static str>,
    /// The builds `problems` checked and `builds` and `pages` report on, in
    /// `ascribe.toml`'s order.
    builds_checked: Vec<String>,
    /// The sections asked for that couldn't run. Each says why in its own
    /// `not_run`. With `--exit-code`, any is exit code 2.
    not_run: Vec<&'static str>,
    /// How many problems of each severity the sections that ran have, and
    /// of each kind of next step: the diagnostics of `problems`, `links`,
    /// and `agents`.
    summary: Summary,
    /// The command that lists everything a list here leaves out, when one
    /// does; `null` otherwise.
    next_command: Option<String>,
    /// `problems`, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    problems: Option<ProblemsJson>,
    /// `inventory`, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    inventory: Option<InventoryJson>,
    /// `builds`, when asked for: one for each build checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    builds: Option<Vec<BuildJson>>,
    /// `pages`, when asked for: one for each build checked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pages: Option<Vec<PagesJson>>,
    /// `links`, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    links: Option<LinksJson>,
    /// `agents`, when asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    agents: Option<AgentsJson>,
}

/// A list of at most `--limit` items.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
#[cfg_attr(test, schemars(rename = "Capped_{T}"))]
pub(crate) struct Capped<T> {
    /// The items, at most `--limit` of them.
    items: Vec<T>,
    /// How many are listed.
    shown: usize,
    /// How many there are.
    total: usize,
    /// Whether some are left out: `shown` is less than `total`. The
    /// report's `next_command` lists them all.
    truncated: bool,
}

/// How many problems of each severity, and of each kind of next step.
#[derive(Serialize, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Summary {
    /// How many errors.
    errors: usize,
    /// How many warnings.
    warnings: usize,
    /// How many advice.
    advice: usize,
    /// How many of each kind of next step, in the order `fix`, `choose`,
    /// `write`, `review`, `outside`; a kind with none is left out.
    by_next: Vec<NextCount>,
}

/// How many problems have one kind of next step.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct NextCount {
    /// `fix`, `choose`, `write`, `review`, or `outside`, as a diagnostic's
    /// `next` is.
    next: &'static str,
    /// How many.
    count: usize,
}

/// `problems`: what `ascribe check` reports, counted.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct ProblemsJson {
    /// How many errors, warnings, and advice, and of each kind of next
    /// step.
    summary: Summary,
    /// How many diagnostics each check reports, most first.
    by_check: Capped<CheckCount>,
    /// How many diagnostics each file has, most first.
    by_file: Capped<FileCount>,
    /// The problems acknowledged as intended, with their reasons, in file
    /// order.
    acknowledged: Capped<AcknowledgedEntry>,
    /// How many advice `ascribe.baseline` holds back, which the counts
    /// leave out: the backlog the project has put off. Left out when there
    /// are none, and with `--no-baseline`.
    #[serde(skip_serializing_if = "is_zero")]
    baselined: usize,
    /// The command that lists each problem: `ascribe check`, as JSON.
    list_command: String,
    /// The day of the last report, `YYYY-MM-DD`, or `null` on a first run,
    /// with `--no-history`, or when the last report checked other builds.
    previous_run: Option<String>,
    /// How many problems the last report didn't list, or `null` when
    /// `previous_run` is. A problem is told from another by its check, its
    /// file, the text it's about, and its order among those alike.
    new: Option<usize>,
    /// How many problems the last report listed that this one doesn't,
    /// fixed or with the text they're about edited, or `null` when
    /// `previous_run` is.
    resolved: Option<usize>,
    /// The problems the last report didn't list, in the order `ascribe
    /// check` reports them; none when `previous_run` is `null`.
    new_items: Capped<Entry>,
}

/// How many diagnostics one check reports.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct CheckCount {
    /// The code, such as `ASC036`.
    code: &'static str,
    /// The check's name, such as `link-target-missing`.
    slug: String,
    /// `error`, `warning`, or `advice`.
    severity: &'static str,
    /// Its kind of next step, as a diagnostic's `next` is.
    next: &'static str,
    /// How many.
    count: usize,
}

/// How many diagnostics one file has.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct FileCount {
    /// The file, relative to the project root, as a diagnostic's `file` is.
    file: String,
    /// How many errors.
    errors: usize,
    /// How many warnings.
    warnings: usize,
    /// How many advice.
    advice: usize,
}

/// `inventory`: what the project holds.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct InventoryJson {
    /// How many pages.
    pages: usize,
    /// How many pages of each content type, most first.
    by_type: Vec<TypeCount>,
    /// How many pages each owner has, most first: the value of the
    /// frontmatter field the page's type marks `role = "owner"`.
    by_owner: Vec<OwnerCount>,
    /// The pages whose review date has passed (`review-overdue`).
    overdue: Capped<Entry>,
    /// The pages nothing links to (`page-orphan`).
    orphans: Capped<Entry>,
    /// The fragments, phrases, features, glossary terms, and images nothing
    /// uses.
    unused: Capped<Entry>,
    /// The tested examples the snippets show, for each source in the same
    /// repository with an `ascribe.examples.toml`, in the content model's
    /// order. Absent when no source has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    examples: Option<Vec<ExamplesCount>>,
    /// How many of each declared API's operations the pages cover, for each
    /// API whose spec could be read, in the content model's order. Absent
    /// when the project declares no API.
    #[serde(skip_serializing_if = "Option::is_none")]
    apis: Option<Vec<ApiCount>>,
}

/// How many of one API's operations the pages cover.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct ApiCount {
    /// The API's name, its key in `[apis]`.
    api: String,
    /// How many operations its spec has.
    operations: usize,
    /// How many of them a link names, by `operationId` or by method and
    /// path.
    linked: usize,
    /// How many of them a checked example claims as its request or
    /// response.
    examples: usize,
}

/// How many snippets of one source show a tested example, by status.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct ExamplesCount {
    /// The source's name.
    source: String,
    /// How many `@snippet` lines, in pages and fragments, show one of its
    /// examples.
    snippets: usize,
    /// How many of those show an example an observation is current for.
    current: usize,
    /// How many show one that changed after every observation listing it.
    stale: usize,
    /// How many show one no observation lists.
    unobserved: usize,
}

/// How many pages one content type has.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct TypeCount {
    /// The type's name; `null` for the pages no one type applies to.
    #[serde(rename = "type")]
    ty: Option<String>,
    /// How many pages.
    pages: usize,
}

/// How many pages one owner has.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct OwnerCount {
    /// The owner, as the frontmatter gives it; `null` for the pages without
    /// one.
    owner: Option<String>,
    /// How many pages.
    pages: usize,
}

/// What one build leaves out that another build keeps.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct BuildJson {
    /// The build.
    build: String,
    /// The pages it doesn't publish, in path order.
    pages: Capped<LeftPageJson>,
    /// The content it takes out of the pages it publishes, in file and
    /// source order.
    content: Capped<LeftContentJson>,
}

/// A page a build doesn't publish.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct LeftPageJson {
    /// The page, relative to the project root, as a diagnostic's `file` is.
    file: String,
    /// Why, as the end of a sentence: `its variant frontmatter names …`.
    why: String,
    /// The builds that publish it.
    kept_by: Vec<String>,
}

/// Content a build takes out of a page it publishes.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct LeftContentJson {
    /// The file it's written in: the page, or a fragment it includes.
    file: String,
    /// Where in the file.
    range: Range,
    /// The page it's taken out of, as `file` is.
    page: String,
    /// `variant` for a variant arm, or `availability` for content a
    /// feature's availability takes out.
    kind: &'static str,
    /// Why, with the content model's labels: `Shows only os=linux`.
    why: String,
    /// The builds that keep it.
    kept_by: Vec<String>,
}

/// One build's page records.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct PagesJson {
    /// The build.
    build: String,
    /// The project's address, `[consumer] site`, as the build's
    /// `_ascribe/pages.json` has it; absent when it isn't set.
    #[serde(skip_serializing_if = "Option::is_none")]
    site: Option<String>,
    /// How many pages the build publishes.
    published: usize,
    /// How many of them aren't `listed`: `unquoted` or `unlisted`.
    not_listed: usize,
    /// The records, sorted by `route`: every page's, or with `--not-listed`
    /// those that aren't `listed`. Each has the fields of its entry in the
    /// build's `_ascribe/pages.json` (output-layout contract §1.3), by the
    /// same names, with `route`, the key it's under there.
    records: Capped<PageRecordJson>,
}

/// What a build says about one page: its entry in `_ascribe/pages.json`,
/// with its route.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct PageRecordJson {
    /// The page's address, from the host's root, as the site output's links
    /// write it.
    route: String,
    /// The page's source path, from the content root.
    source: String,
    /// Its title, as plain text; absent when it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    /// Its description, as plain text on one line; absent when it has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Its own address, absolute; absent without `[consumer] site`.
    #[serde(skip_serializing_if = "Option::is_none")]
    canonical: Option<String>,
    /// `listed`, `unquoted`, or `unlisted`.
    listing: &'static str,
    /// Where `listing` comes from.
    listing_from: ListingFromJson,
    /// Its Markdown version's address, from the host's root; absent when the
    /// build publishes none.
    #[serde(skip_serializing_if = "Option::is_none")]
    markdown: Option<String>,
    /// Its place in the site, from the top down to the page itself.
    breadcrumbs: Vec<StepJson>,
}

/// Where a page's listing comes from.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct ListingFromJson {
    /// `default` when nothing sets it, `page` for the page's own `listing`,
    /// or the level of the rule of `[listing]` it's under: `site`,
    /// `older-lines`, `folder`, or `type`.
    level: &'static str,
    /// For a folder's rule, the folder, as `[listing.folders]` names it.
    #[serde(skip_serializing_if = "Option::is_none")]
    folder: Option<String>,
    /// For a type's rule, the type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_name: Option<String>,
    /// For a rule, the rule as written in `ascribe.toml`.
    #[serde(skip_serializing_if = "Option::is_none")]
    rule: Option<String>,
    /// For a rule, its line in `ascribe.toml`.
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<usize>,
}

/// A step of a page's breadcrumbs.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct StepJson {
    /// The step's page's address, as `route` is.
    route: String,
    /// The step's page's title, as plain text.
    title: String,
}

/// Why a section couldn't run, and what would let it.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct NotRunJson {
    /// Why, as a sentence.
    reason: String,
    /// What to do so that it runs, as a sentence.
    how: String,
}

/// `links`: what the link checker said about the external links.
#[derive(Serialize, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct LinksJson {
    /// Why it couldn't run, or `null` when it ran. When it isn't `null`,
    /// the rest is empty.
    not_run: Option<NotRunJson>,
    /// How many different addresses were checked: `asked` and `reused`.
    checked: usize,
    /// How many different addresses the link checker was asked about in
    /// this run.
    asked: usize,
    /// How many different addresses' answers were reused from an earlier
    /// run, within `[checks.links] reuse-days`. None of `diagnostics` is
    /// from a reused answer: only an answer with nothing to report is kept.
    reused: usize,
    /// The day of the oldest answer reused, `YYYY-MM-DD`, or `null` when
    /// none was.
    oldest_answer: Option<String>,
    /// How many `[checks.links] ignore` left out.
    ignored: usize,
    /// A diagnostic for each link that moved or is broken, at its place,
    /// and for each acknowledgement of a broken link that covers nothing.
    diagnostics: Capped<Entry>,
    /// The broken links acknowledged as intended, confirmed or not.
    acknowledged: Capped<AcknowledgedEntry>,
    /// The day of the run the link history is from, `YYYY-MM-DD`, or `null`
    /// on a first run, or with `--no-history`.
    previous_run: Option<String>,
    /// How many of `diagnostics` the previous run didn't report, or `null`
    /// when `previous_run` is.
    new: Option<usize>,
    /// How many addresses the previous run reported that this one doesn't,
    /// fixed or no longer linked to, or `null` when `previous_run` is.
    resolved: Option<usize>,
    /// The broken links that answered before and aren't reported until
    /// they've kept failing for `[checks.links] confirm-days`. They aren't
    /// in `diagnostics` or the summary, and don't fail `--exit-code`.
    unconfirmed: Capped<UnconfirmedEntry>,
}

/// A broken link that answered before, not reported yet.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct UnconfirmedEntry {
    /// The file, as a diagnostic's `file` is.
    file: String,
    /// Where in the file: the link.
    range: Range,
    /// The link's address.
    url: String,
    /// What the link checker said: a status, such as `503 Service
    /// Unavailable`, or why it didn't answer.
    status: String,
    /// The last day it answered, `YYYY-MM-DD`.
    last_answered: String,
    /// The day it started failing, `YYYY-MM-DD`.
    failing_since: String,
    /// The day it's reported from if it's still failing, `YYYY-MM-DD`.
    reported_from: String,
}

/// `agents`: the delivery spec's checks on a built site.
#[derive(Serialize, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct AgentsJson {
    /// Why it couldn't run, or `null` when it ran. When it isn't `null`,
    /// the rest is empty.
    not_run: Option<NotRunJson>,
    /// The site checked, as `--site` gives it; empty when none was given.
    site: String,
    /// Each check's result, in the checker's order.
    results: Vec<CheckResultJson>,
    /// A diagnostic for each check that failed or warned that the hosting
    /// or Ascribe owns. One a page owns is `ascribe check`'s, and isn't
    /// repeated.
    diagnostics: Capped<Entry>,
    /// The day of the last report that checked the same site,
    /// `YYYY-MM-DD`, or `null` on a first run, with `--no-history`, or when
    /// the last report checked another site or none.
    previous_run: Option<String>,
    /// How many checks fail or warn now that passed on the last report or
    /// weren't in it, or `null` when `previous_run` is. A check is told from
    /// another by the site's address and its id.
    new: Option<usize>,
    /// How many checks that failed or warned on the last report pass now or
    /// aren't in this one, or `null` when `previous_run` is.
    resolved: Option<usize>,
    /// The ids of the checks `new` counts, in the checker's order; empty
    /// when `previous_run` is `null`.
    new_checks: Vec<String>,
}

/// One check of the delivery spec.
#[derive(Serialize)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct CheckResultJson {
    /// The check's id, such as `llms-txt-exists`.
    id: String,
    /// Its category in the spec.
    category: String,
    /// `pass`, `warn`, `fail`, `skip`, or `error`.
    status: &'static str,
    /// What the checker said.
    message: String,
    /// Who changes what it checks: `ascribe` (what Ascribe writes),
    /// `hosting` (where the site is hosted), or `pages` (the pages, which
    /// `ascribe check` reports on).
    owner: &'static str,
}

impl<T> Default for Capped<T> {
    fn default() -> Capped<T> {
        Capped {
            items: Vec::new(),
            shown: 0,
            total: 0,
            truncated: false,
        }
    }
}

impl<T> Capped<T> {
    /// At most `limit` of `all`, made into items with `item`.
    fn of<'a, S: 'a>(
        all: impl ExactSizeIterator<Item = &'a S>,
        limit: usize,
        item: impl Fn(&S) -> T,
    ) -> Capped<T> {
        let total = all.len();
        let items: Vec<T> = all.take(limit).map(item).collect();
        Capped {
            shown: items.len(),
            truncated: items.len() < total,
            total,
            items,
        }
    }
}

fn summary<'a>(diagnostics: impl Iterator<Item = &'a Diagnostic>) -> Summary {
    let diagnostics: Vec<&Diagnostic> = diagnostics.collect();
    let mut counts = Counts::default();
    for d in &diagnostics {
        counts.add(d.severity);
    }
    Summary {
        errors: counts.errors,
        warnings: counts.warnings,
        advice: counts.advice,
        by_next: by_next(diagnostics.into_iter())
            .into_iter()
            .map(|(next, count)| NextCount {
                next: next.as_str(),
                count,
            })
            .collect(),
    }
}

/// What the report needs to write its JSON besides the report.
pub(crate) struct About<'a> {
    pub sections: &'a [Section],
    pub limit: usize,
    pub next_command: Option<String>,
    pub list_command: String,
}

/// The report as its JSON.
pub(crate) fn of(report: &Report, files: &FileTable, about: &About<'_>) -> ReportJson {
    let limit = about.limit;
    let entries = |list: &[Diagnostic]| Capped::of(list.iter(), limit, |d| entry(files, d, 0));
    ReportJson {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        error: None,
        sections: about.sections.iter().map(|s| s.as_str()).collect(),
        builds_checked: report.builds_checked.clone(),
        not_run: report.not_run().iter().map(|(s, _)| s.as_str()).collect(),
        summary: summary(report.all_problems()),
        next_command: about.next_command.clone(),
        problems: report
            .problems
            .as_ref()
            .map(|p| problems(p, files, limit, &about.list_command)),
        inventory: report.inventory.as_ref().map(|i| InventoryJson {
            pages: i.pages,
            by_type: i
                .by_type
                .iter()
                .map(|(ty, pages)| TypeCount {
                    ty: ty.clone(),
                    pages: *pages,
                })
                .collect(),
            by_owner: i
                .by_owner
                .iter()
                .map(|(owner, pages)| OwnerCount {
                    owner: owner.clone(),
                    pages: *pages,
                })
                .collect(),
            overdue: entries(&i.overdue),
            orphans: entries(&i.orphans),
            unused: entries(&i.unused),
            examples: i.examples.as_ref().map(|list| {
                list.iter()
                    .map(|e| ExamplesCount {
                        source: e.source.clone(),
                        snippets: e.snippets,
                        current: e.current,
                        stale: e.stale,
                        unobserved: e.unobserved,
                    })
                    .collect()
            }),
            apis: i.apis.as_ref().map(|list| {
                list.iter()
                    .map(|a| ApiCount {
                        api: a.api.clone(),
                        operations: a.operations,
                        linked: a.linked,
                        examples: a.examples,
                    })
                    .collect()
            }),
        }),
        builds: report.builds.as_ref().map(|b| builds(b, files, limit)),
        pages: report.pages.as_ref().map(|p| pages(p, limit)),
        links: report.links.as_ref().map(|l| match l {
            Ran::NotRun(why) => LinksJson {
                not_run: Some(not_run(why)),
                ..LinksJson::default()
            },
            Ran::Done(links) => links_json(links, files, limit),
        }),
        agents: report.agents.as_ref().map(|a| match a {
            Ran::NotRun(why) => AgentsJson {
                not_run: Some(not_run(why)),
                ..AgentsJson::default()
            },
            Ran::Done(agents) => agents_json(agents, files, limit),
        }),
    }
}

/// A report that couldn't be made, because of `error`.
pub(crate) fn failed(error: &str, sections: &[Section]) -> ReportJson {
    ReportJson {
        schema_version: SCHEMA_VERSION,
        ascribe_version: env!("CARGO_PKG_VERSION"),
        error: Some(error.to_owned()),
        sections: sections.iter().map(|s| s.as_str()).collect(),
        builds_checked: Vec::new(),
        not_run: Vec::new(),
        summary: Summary::default(),
        next_command: None,
        problems: None,
        inventory: None,
        builds: None,
        pages: None,
        links: None,
        agents: None,
    }
}

fn problems(p: &Problems, files: &FileTable, limit: usize, list_command: &str) -> ProblemsJson {
    let by_check = by_check(&p.diagnostics);
    let by_file = by_file(files, &p.diagnostics);
    ProblemsJson {
        summary: summary(p.diagnostics.iter()),
        by_check: Capped::of(by_check.iter(), limit, |(d, count)| CheckCount {
            code: d.code,
            slug: d.slug.to_string(),
            severity: d.severity.as_str(),
            next: d.next().map_or("write", Next::as_str),
            count: *count,
        }),
        by_file: Capped::of(by_file.iter(), limit, |(file, counts)| FileCount {
            file: file.clone(),
            errors: counts.errors,
            warnings: counts.warnings,
            advice: counts.advice,
        }),
        acknowledged: Capped::of(p.acknowledged.iter(), limit, |a| acknowledged(files, a)),
        baselined: p.baselined,
        list_command: list_command.to_owned(),
        previous_run: p.previous_run.map(|d| d.to_string()),
        new: p.new.as_ref().map(Vec::len),
        resolved: p.resolved,
        new_items: Capped::of(p.new.as_deref().unwrap_or_default().iter(), limit, |d| {
            entry(files, d, 0)
        }),
    }
}

fn builds(b: &Builds, files: &FileTable, limit: usize) -> Vec<BuildJson> {
    b.builds
        .iter()
        .map(|left| BuildJson {
            build: left.build.clone(),
            pages: Capped::of(left.pages.iter(), limit, |p: &LeftPage| LeftPageJson {
                file: p.path.to_string(),
                why: p.why.clone(),
                kept_by: p.kept_by.clone(),
            }),
            content: Capped::of(left.content.iter(), limit, |c: &LeftContent| {
                LeftContentJson {
                    file: files.path(c.location.file),
                    range: range(files, c.location),
                    page: c.page.to_string(),
                    kind: c.kind,
                    why: c.why.clone(),
                    kept_by: c.kept_by.clone(),
                }
            }),
        })
        .collect()
}

fn pages(p: &Pages, limit: usize) -> Vec<PagesJson> {
    p.builds
        .iter()
        .map(|b: &BuildPages| PagesJson {
            build: b.build.clone(),
            site: b.site.clone(),
            published: b.published,
            not_listed: b.not_listed,
            records: Capped::of(b.records.iter(), limit, record),
        })
        .collect()
}

fn record(r: &PageRecord) -> PageRecordJson {
    let rule = match &r.listing_from {
        ListingFrom::Rule(rule) => Some(rule),
        ListingFrom::Default | ListingFrom::Page => None,
    };
    PageRecordJson {
        route: r.route.clone(),
        source: r.source.to_string(),
        title: r.title.clone(),
        description: r.description.clone(),
        canonical: r.canonical.clone(),
        listing: r.listing.name(),
        listing_from: ListingFromJson {
            level: r.listing_from.level_name(),
            folder: rule.and_then(|rule| match &rule.level {
                ListingLevel::Folder(folder) => Some(folder.clone()),
                _ => None,
            }),
            type_name: rule.and_then(|rule| match &rule.level {
                ListingLevel::Type(name) => Some(name.clone()),
                _ => None,
            }),
            rule: rule.map(|rule| rule.text.clone()),
            line: rule.map(|rule| rule.line),
        },
        markdown: r.markdown.clone(),
        breadcrumbs: r
            .breadcrumbs
            .iter()
            .map(|step| StepJson {
                route: step.route.clone(),
                title: step.title.clone(),
            })
            .collect(),
    }
}

fn not_run(why: &NotRun) -> NotRunJson {
    NotRunJson {
        reason: why.reason.clone(),
        how: why.how.clone(),
    }
}

fn links_json(links: &Links, files: &FileTable, limit: usize) -> LinksJson {
    LinksJson {
        not_run: None,
        checked: links.checked,
        asked: links.asked,
        reused: links.reused,
        oldest_answer: links.oldest_answer.map(|d| d.to_string()),
        ignored: links.ignored,
        diagnostics: Capped::of(links.diagnostics.iter(), limit, |d| entry(files, d, 0)),
        acknowledged: Capped::of(links.acknowledged.iter(), limit, |a| acknowledged(files, a)),
        previous_run: links.previous_run.map(|d| d.to_string()),
        new: links.new,
        resolved: links.resolved,
        unconfirmed: Capped::of(links.unconfirmed.iter(), limit, |u| UnconfirmedEntry {
            file: files.path(u.location.file),
            range: range(files, u.location),
            url: u.url.clone(),
            status: match &u.answer {
                Answer::Broken { status } => status.clone(),
                Answer::TimedOut { said } => said.clone(),
                Answer::Unreachable { reason } => reason.clone(),
                Answer::Fine | Answer::Moved { .. } => String::new(),
            },
            last_answered: u.last_answered.to_string(),
            failing_since: u.failing_since.to_string(),
            reported_from: u.reported_from.to_string(),
        }),
    }
}

fn agents_json(agents: &Agents, files: &FileTable, limit: usize) -> AgentsJson {
    AgentsJson {
        not_run: None,
        site: agents.site.clone(),
        results: agents
            .results
            .iter()
            .map(|r| CheckResultJson {
                id: r.id.clone(),
                category: r.category.clone(),
                status: r.status.as_str(),
                message: r.message.clone(),
                owner: ascribe_check::site::owner(&r.id).as_str(),
            })
            .collect(),
        diagnostics: Capped::of(agents.diagnostics.iter(), limit, |d| entry(files, d, 0)),
        previous_run: agents.previous_run.map(|d| d.to_string()),
        new: agents.new.as_ref().map(Vec::len),
        resolved: agents.resolved,
        new_checks: agents.new.clone().unwrap_or_default(),
    }
}

/// Writes `json` and a newline.
pub(crate) fn write(out: &mut dyn Write, json: &ReportJson) -> io::Result<()> {
    serde_json::to_writer_pretty(&mut *out, json).map_err(io::Error::other)?;
    writeln!(out)
}

/// Whether any list of `report` would leave items out at `limit`, and the
/// most any list holds: the `--limit` that lists them all.
pub(crate) fn longest(report: &Report) -> usize {
    let mut most = 0;
    let mut see = |n: usize| most = most.max(n);
    if let Some(p) = &report.problems {
        see(p.acknowledged.len());
        let mut codes: Vec<&str> = p.diagnostics.iter().map(|d| d.code).collect();
        codes.sort_unstable();
        codes.dedup();
        see(codes.len());
        let mut files: Vec<ascribe_core::FileId> =
            p.diagnostics.iter().map(|d| d.location.file).collect();
        files.sort_unstable();
        files.dedup();
        see(files.len());
        see(p.new.as_ref().map_or(0, Vec::len));
    }
    if let Some(i) = &report.inventory {
        see(i.overdue.len());
        see(i.orphans.len());
        see(i.unused.len());
    }
    if let Some(b) = &report.builds {
        for left in &b.builds {
            see(left.pages.len());
            see(left.content.len());
        }
    }
    if let Some(p) = &report.pages {
        for build in &p.builds {
            see(build.records.len());
        }
    }
    if let Some(Ran::Done(l)) = &report.links {
        see(l.diagnostics.len());
        see(l.acknowledged.len());
        see(l.unconfirmed.len());
    }
    if let Some(Ran::Done(a)) = &report.agents {
        see(a.diagnostics.len());
    }
    most
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn a_list_says_what_it_leaves_out() {
        let all = [1, 2, 3];
        let capped = Capped::of(all.iter(), 2, |n| n * 10);
        assert_eq!(capped.items, [10, 20]);
        assert_eq!((capped.shown, capped.total, capped.truncated), (2, 3, true));
        let whole = Capped::of(all.iter(), 5, |n| *n);
        assert!(!whole.truncated);
    }
}
