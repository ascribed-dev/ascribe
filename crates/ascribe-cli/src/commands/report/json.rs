//! `ascribe report --format json`: one document, versioned as the other
//! commands' are ([`SCHEMA_VERSION`]), and capped: each list holds at most
//! `--limit` items, says how many there are, and the report names the
//! command that lists them all.

use std::io::{self, Write};

use ascribe_check::{Diagnostic, Next};
use ascribe_query::report::{
    Agents, Builds, LeftContent, LeftPage, Links, NotRun, Problems, Ran, Report, Section,
};
use serde::Serialize;

use super::text::{by_check, by_file, by_next};
use crate::report::json::{AcknowledgedEntry, Entry, Range, acknowledged, entry, range};
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
    /// `problems`, `inventory`, `builds`, `links`, `agents`.
    sections: Vec<&'static str>,
    /// The builds `problems` checked and `builds` reports on, in
    /// `ascribe.toml`'s order.
    builds_checked: Vec<String>,
    /// The sections asked for that couldn't run. Each says why in its own
    /// `not_run`. With `--exit-code`, any is exit code 2.
    not_run: Vec<&'static str>,
    /// How many findings of each severity the sections that ran have, and
    /// of each kind of next step: the diagnostics of `problems`, `links`,
    /// and `agents`.
    findings: Findings,
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

/// How many findings of each severity, and of each kind of next step.
#[derive(Serialize, Default)]
#[cfg_attr(test, derive(schemars::JsonSchema))]
struct Findings {
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

/// How many findings have one kind of next step.
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
    findings: Findings,
    /// How many diagnostics each check reports, most first.
    by_check: Capped<CheckCount>,
    /// How many diagnostics each file has, most first.
    by_file: Capped<FileCount>,
    /// The problems acknowledged as intended, with their reasons, in file
    /// order.
    acknowledged: Capped<AcknowledgedEntry>,
    /// The command that lists each problem: `ascribe check`, as JSON.
    list_command: String,
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
    /// How many different addresses were checked.
    checked: usize,
    /// How many `[checks.links] ignore` left out.
    ignored: usize,
    /// A diagnostic for each link that moved or is broken, at its place,
    /// and for each acknowledgement of a broken link that covers nothing.
    diagnostics: Capped<Entry>,
    /// The broken links acknowledged as intended.
    acknowledged: Capped<AcknowledgedEntry>,
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

fn findings<'a>(diagnostics: impl Iterator<Item = &'a Diagnostic>) -> Findings {
    let diagnostics: Vec<&Diagnostic> = diagnostics.collect();
    let mut counts = Counts::default();
    for d in &diagnostics {
        counts.add(d.severity);
    }
    Findings {
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
        findings: findings(report.findings()),
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
        }),
        builds: report.builds.as_ref().map(|b| builds(b, files, limit)),
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
        findings: Findings::default(),
        next_command: None,
        problems: None,
        inventory: None,
        builds: None,
        links: None,
        agents: None,
    }
}

fn problems(p: &Problems, files: &FileTable, limit: usize, list_command: &str) -> ProblemsJson {
    let by_check = by_check(&p.diagnostics);
    let by_file = by_file(files, &p.diagnostics);
    ProblemsJson {
        findings: findings(p.diagnostics.iter()),
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
        list_command: list_command.to_owned(),
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
        ignored: links.ignored,
        diagnostics: Capped::of(links.diagnostics.iter(), limit, |d| entry(files, d, 0)),
        acknowledged: Capped::of(links.acknowledged.iter(), limit, |a| acknowledged(files, a)),
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
    if let Some(Ran::Done(l)) = &report.links {
        see(l.diagnostics.len());
        see(l.acknowledged.len());
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
