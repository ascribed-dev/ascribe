//! `ascribe report` as text, for people, and as Markdown (`--format
//! markdown`), for a CI job's summary or an issue's body. Both say the same
//! things in the same order; a section that couldn't run says why and what
//! would let it, first.

use std::io::{self, Write};

use ascribe_check::links::{Answer, Unconfirmed};
use ascribe_check::{Acknowledged, Diagnostic, Next};
use ascribe_core::Date;
use ascribe_model::{ListingFrom, ListingLevel};
use ascribe_query::report::{
    Agents, Builds, Inventory, Links, NotRun, Pages, Problems, Ran, Report,
};
use ascribe_resolve::pages::PageRecord;

use crate::report::{Counts, FileTable};

/// How the report is written.
#[derive(Clone, Copy)]
pub(crate) struct Style {
    /// Markdown, for `--format markdown`; plain text otherwise.
    pub markdown: bool,
    /// The most items a list shows.
    pub limit: usize,
}

/// What the report needs besides the report.
pub(crate) struct About<'a> {
    /// The command that lists everything a list leaves out.
    pub next_command: Option<&'a str>,
    /// The command that lists each problem.
    pub list_command: &'a str,
}

struct Writer<'a> {
    out: &'a mut dyn Write,
    files: &'a FileTable,
    style: Style,
    about: &'a About<'a>,
}

/// Writes the report.
pub(crate) fn write(
    out: &mut dyn Write,
    files: &FileTable,
    report: &Report,
    style: Style,
    about: &About<'_>,
) -> io::Result<()> {
    // Written whole, then without the blank lines Markdown's paragraphs
    // leave at the end.
    let mut text = Vec::new();
    write_sections(&mut text, files, report, style, about)?;
    while text.ends_with(b"\n\n") {
        text.pop();
    }
    out.write_all(&text)
}

fn write_sections(
    out: &mut dyn Write,
    files: &FileTable,
    report: &Report,
    style: Style,
    about: &About<'_>,
) -> io::Result<()> {
    let mut w = Writer {
        out,
        files,
        style,
        about,
    };
    w.title(report)?;
    let mut first = true;
    let mut gap = |w: &mut Writer<'_>| -> io::Result<()> {
        if !std::mem::take(&mut first) && !w.style.markdown {
            writeln!(w.out)?;
        }
        Ok(())
    };
    if let Some(p) = &report.problems {
        gap(&mut w)?;
        w.problems(p)?;
    }
    if let Some(i) = &report.inventory {
        gap(&mut w)?;
        w.inventory(i)?;
    }
    if let Some(b) = &report.builds {
        gap(&mut w)?;
        w.builds(b)?;
    }
    if let Some(p) = &report.pages {
        gap(&mut w)?;
        w.pages(p)?;
    }
    if let Some(l) = &report.links {
        gap(&mut w)?;
        w.links(l)?;
    }
    if let Some(a) = &report.agents {
        gap(&mut w)?;
        w.agents(a)?;
    }
    Ok(())
}

impl Writer<'_> {
    /// Text as code: in backticks in Markdown, as it is in text.
    fn code(&self, text: &str) -> String {
        if self.style.markdown {
            format!("`{text}`")
        } else {
            text.to_owned()
        }
    }

    fn title(&mut self, report: &Report) -> io::Result<()> {
        if !self.style.markdown {
            return Ok(());
        }
        let mut problems = Counts::default();
        for d in report.all_problems() {
            problems.add(d.severity);
        }
        writeln!(self.out, "## Ascribe report")?;
        writeln!(self.out)?;
        let mut line = counts_words(problems);
        let not_run = report.not_run();
        if !not_run.is_empty() {
            let names: Vec<String> = not_run.iter().map(|(s, _)| self.code(s.as_str())).collect();
            line.push_str(&format!("; {} couldn't run", names.join(" and ")));
        }
        writeln!(self.out, "{}.", capitalized(&line))?;
        writeln!(self.out)
    }

    fn heading(&mut self, text: &str) -> io::Result<()> {
        if self.style.markdown {
            writeln!(self.out, "### {text}")?;
            writeln!(self.out)
        } else {
            writeln!(self.out, "{text}")
        }
    }

    /// A line of a section: indented in text, a paragraph in Markdown.
    fn line(&mut self, text: &str) -> io::Result<()> {
        if self.style.markdown {
            writeln!(self.out, "{text}")?;
            writeln!(self.out)
        } else {
            writeln!(self.out, "  {text}")
        }
    }

    /// An item of a list under a line.
    fn item(&mut self, text: &str) -> io::Result<()> {
        if self.style.markdown {
            writeln!(self.out, "- {text}")
        } else {
            writeln!(self.out, "    {text}")
        }
    }

    /// The end of a list of Markdown items, before the next line.
    fn end_list(&mut self) -> io::Result<()> {
        if self.style.markdown {
            writeln!(self.out)?;
        }
        Ok(())
    }

    /// Up to the limit of `items`, each written with `text`, then how many
    /// are left out.
    fn items<T>(&mut self, items: &[T], text: impl Fn(&Self, &T) -> String) -> io::Result<()> {
        for item in items.iter().take(self.style.limit) {
            let line = text(self, item);
            self.item(&line)?;
        }
        if items.len() > self.style.limit {
            let more = items.len() - self.style.limit;
            let line = match self.about.next_command {
                Some(command) => format!("and {more} more: {}", self.code(command)),
                None => format!("and {more} more"),
            };
            self.item(&line)?;
        }
        self.end_list()
    }

    fn not_run(&mut self, why: &NotRun) -> io::Result<()> {
        self.line(&format!("Not run: {}.", why.reason))?;
        self.line(&format!("{}.", why.how))
    }

    /// `file:line: [code] message`, or `[code] message` for a problem about
    /// the published site, which has no place in the source.
    fn diagnostic(&self, d: &Diagnostic) -> String {
        let code = format!("[{}]", d.code);
        if d.location.file == ascribe_core::FileId::new(0) && d.location.span.is_empty() {
            return format!("{code} {}", d.message);
        }
        let start = self
            .files
            .position(d.location.file, d.location.span.start());
        let at = format!("{}:{}", self.files.path(d.location.file), start.line);
        format!("{}: {code} {}", self.code(&at), d.message)
    }

    fn acknowledged(&self, a: &Acknowledged) -> String {
        format!("{} Intended: {}", self.diagnostic(&a.problem), a.reason)
    }

    fn problems(&mut self, p: &Problems) -> io::Result<()> {
        self.heading("Problems")?;
        let counts = Counts::of(&p.diagnostics);
        let mut line = counts_words(counts);
        if !p.acknowledged.is_empty() {
            line.push_str(&format!(
                ", and {} acknowledged as intended",
                p.acknowledged.len()
            ));
        }
        self.line(&format!("{}.", capitalized(&line)))?;
        if p.diagnostics.is_empty() && p.acknowledged.is_empty() {
            return Ok(());
        }
        if !p.diagnostics.is_empty() {
            let nexts: Vec<String> = by_next(p.diagnostics.iter())
                .iter()
                .map(|(next, n)| format!("{n} {}", next.as_str()))
                .collect();
            self.line(&format!("By next step: {}.", nexts.join(", ")))?;
            self.line("By check:")?;
            let checks = by_check(&p.diagnostics);
            self.items(&checks, |w, (d, count)| {
                format!("{count} {}", w.code(&format!("{} {}", d.code, d.slug)))
            })?;
            self.line("By file:")?;
            let files = by_file(self.files, &p.diagnostics);
            self.items(&files, |w, (file, counts)| {
                format!("{}: {}", w.code(file), counts_words(*counts))
            })?;
        }
        if !p.acknowledged.is_empty() {
            self.line("Acknowledged as intended:")?;
            self.items(&p.acknowledged, Self::acknowledged)?;
        }
        let list = self.code(self.about.list_command);
        self.line(&format!("To list each problem: {list}"))
    }

    fn inventory(&mut self, i: &Inventory) -> io::Result<()> {
        self.heading("Inventory")?;
        self.line(&format!(
            "{} {}.",
            i.pages,
            plural(i.pages, "page", "pages")
        ))?;
        let counted = |list: &[(Option<String>, usize)], none: &str| -> String {
            list.iter()
                .map(|(name, n)| format!("{} {n}", name.as_deref().unwrap_or(none)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        if !i.by_type.is_empty() {
            self.line(&format!("By type: {}.", counted(&i.by_type, "no type")))?;
        }
        if i.by_owner.iter().any(|(owner, _)| owner.is_some()) {
            self.line(&format!("By owner: {}.", counted(&i.by_owner, "no owner")))?;
        }
        for e in i.examples.iter().flatten() {
            let source = self.code(&e.source);
            self.line(&format!(
                "Examples from {source}: {} {}, {} current, {} stale, {} unobserved.",
                e.snippets,
                plural(e.snippets, "snippet", "snippets"),
                e.current,
                e.stale,
                e.unobserved,
            ))?;
        }
        for (label, list) in [
            ("Overdue reviews", &i.overdue),
            ("Pages nothing links to", &i.orphans),
            ("Entries nothing uses", &i.unused),
        ] {
            if list.is_empty() {
                self.line(&format!("{label}: none."))?;
            } else {
                self.line(&format!("{label} ({}):", list.len()))?;
                self.items(list, |w, d| w.diagnostic(d))?;
            }
        }
        Ok(())
    }

    fn builds(&mut self, b: &Builds) -> io::Result<()> {
        self.heading("Builds")?;
        for left in &b.builds {
            let build = self.code(&left.build);
            if left.pages.is_empty() && left.content.is_empty() {
                self.line(&format!("{build} leaves out nothing another build keeps."))?;
                continue;
            }
            self.line(&format!(
                "{build} leaves out {} and {} that another build keeps:",
                counted(left.pages.len(), "page", "pages"),
                counted(left.content.len(), "piece of content", "pieces of content"),
            ))?;
            let mut lines: Vec<String> = left
                .pages
                .iter()
                .map(|p| {
                    format!(
                        "{}: {}; {} publishes it",
                        self.code(p.path.as_str()),
                        p.why,
                        names(&p.kept_by, |n| self.code(n))
                    )
                })
                .collect();
            lines.extend(left.content.iter().map(|c| {
                let start = self
                    .files
                    .position(c.location.file, c.location.span.start());
                let at = format!("{}:{}", self.files.path(c.location.file), start.line);
                let on = if self.files.path(c.location.file) == c.page.as_str() {
                    String::new()
                } else {
                    format!(", on {}", self.code(c.page.as_str()))
                };
                format!(
                    "{} ({}{on}): {}; {} keeps it",
                    self.code(&at),
                    c.kind,
                    c.why,
                    names(&c.kept_by, |n| self.code(n))
                )
            }));
            self.items(&lines, |_, line| line.clone())?;
        }
        Ok(())
    }

    fn pages(&mut self, p: &Pages) -> io::Result<()> {
        self.heading("Pages")?;
        for build in &p.builds {
            let name = self.code(&build.build);
            let on = build
                .site
                .as_deref()
                .map(|site| format!(" on {}", self.code(site)))
                .unwrap_or_default();
            let shown_all = build.records.len() == build.published;
            let line = if shown_all {
                let hidden = match build.not_listed {
                    0 => "every one listed".to_owned(),
                    n => format!("{n} of them not listed"),
                };
                format!(
                    "{name} publishes {}{on}, {hidden}",
                    counted(build.published, "page", "pages")
                )
            } else {
                let hidden = match build.not_listed {
                    0 => "every one listed".to_owned(),
                    1 => "and 1 isn't listed".to_owned(),
                    n => format!("and {n} aren't listed"),
                };
                format!(
                    "{name} publishes {}{on}, {hidden}",
                    counted(build.published, "page", "pages"),
                )
            };
            if build.records.is_empty() {
                self.line(&format!("{line}."))?;
                continue;
            }
            self.line(&format!("{line}:"))?;
            let rows: Vec<[String; 5]> = build.records.iter().map(row).collect();
            if self.style.markdown {
                self.table(&rows)?;
            } else {
                self.columns(&rows)?;
            }
        }
        Ok(())
    }

    /// The page table as text: a heading, then up to the limit of rows,
    /// each column as wide as its widest cell, then how many are left out.
    fn columns(&mut self, rows: &[[String; 5]]) -> io::Result<()> {
        let shown = &rows[..rows.len().min(self.style.limit)];
        let mut widths = HEADINGS.map(|h| h.chars().count());
        for row in shown {
            for (width, cell) in widths.iter_mut().zip(row) {
                *width = (*width).max(cell.chars().count());
            }
        }
        let line = |cells: [&str; 5]| -> String {
            let mut text = String::new();
            for (i, (cell, width)) in cells.iter().zip(widths).enumerate() {
                if i + 1 == cells.len() {
                    text.push_str(cell);
                } else {
                    text.push_str(&format!("{cell:<width$}  "));
                }
            }
            text.trim_end().to_owned()
        };
        let heading = line(
            HEADINGS
                .map(|h| h.to_uppercase())
                .each_ref()
                .map(String::as_str),
        );
        self.item(&heading)?;
        let lines: Vec<String> = rows
            .iter()
            .map(|r| line(r.each_ref().map(String::as_str)))
            .collect();
        self.items(&lines, |_, l| l.clone())
    }

    /// The page table as Markdown: a table of up to the limit of rows, then
    /// how many are left out.
    fn table(&mut self, rows: &[[String; 5]]) -> io::Result<()> {
        let cell = |text: &str| text.replace('|', "\\|");
        writeln!(self.out, "| {} |", HEADINGS.join(" | "))?;
        writeln!(self.out, "|{}", "---|".repeat(HEADINGS.len()))?;
        for r in rows.iter().take(self.style.limit) {
            let [route, listing, from, title, description] = r;
            writeln!(
                self.out,
                "| `{}` | {} | {} | {} | {} |",
                route,
                cell(listing),
                cell(from),
                cell(title),
                cell(description)
            )?;
        }
        writeln!(self.out)?;
        if rows.len() > self.style.limit {
            let more = rows.len() - self.style.limit;
            let line = match self.about.next_command {
                Some(command) => format!("And {more} more: {}", self.code(command)),
                None => format!("And {more} more."),
            };
            self.line(&line)?;
        }
        Ok(())
    }

    fn links(&mut self, l: &Ran<Links>) -> io::Result<()> {
        self.heading("External links")?;
        let links = match l {
            Ran::NotRun(why) => return self.not_run(why),
            Ran::Done(links) => links,
        };
        let mut line = format!("Checked {}", counted(links.checked, "address", "addresses"));
        let ignored = counted(links.ignored, "address", "addresses");
        if links.asked == 0 && links.reused == 0 {
            // Nothing was asked about: there was nothing to check, or
            // `[checks]` turns both link checks off.
            if links.ignored > 0 {
                line.push_str(&format!(
                    ", and left out {ignored} `[checks.links] ignore` names"
                ));
            }
            line.push('.');
        } else {
            match links.oldest_answer.filter(|_| links.reused > 0) {
                None if links.previous_run.is_none() => line.push_str(
                    ", asking about each. Nothing is remembered from an earlier run, so every failure is reported.",
                ),
                None => {
                    line.push_str(", asking about each: no answer from an earlier run was reused.")
                }
                Some(oldest) => {
                    let asked = match links.asked {
                        0 => "none".to_owned(),
                        n => n.to_string(),
                    };
                    line.push_str(&format!(
                        ": asked about {asked}, and reused {}, the oldest from {}.",
                        counted(links.reused, "answer", "answers"),
                        day(oldest)
                    ));
                }
            }
            if links.ignored > 0 {
                line.push_str(&format!(
                    " Left out {ignored} `[checks.links] ignore` names."
                ));
            }
        }
        if !self.style.markdown {
            line = line.replace('`', "");
        }
        self.line(&line)?;
        if links.diagnostics.is_empty() {
            self.line("None is broken or moved.")?;
        } else {
            let total = links.diagnostics.len();
            let mut line = capitalized(&counted(total, "link", "links"));
            line.push_str(" to look at");
            if let (Some(new), Some(since)) = (links.new, links.previous_run) {
                let since = day(since);
                line.push_str(&match (total, new) {
                    (1, 1) => format!(", new since {since}"),
                    (1, _) => format!(", reported on {since} too"),
                    (_, 0) => format!(", none of them new since {since}"),
                    (_, n) => format!(", {n} of them new since {since}"),
                });
            }
            line.push(':');
            self.line(&line)?;
            self.items(&links.diagnostics, |w, d| w.diagnostic(d))?;
        }
        if !links.unconfirmed.is_empty() {
            let n = links.unconfirmed.len();
            self.line(&format!(
                "{} not confirmed. {} answered before and fails now, and is reported if it's still failing on its day:",
                counted(n, "link", "links"),
                plural(n, "It", "Each")
            ))?;
            self.items(&links.unconfirmed, Self::unconfirmed)?;
        }
        if let Some(resolved) = links.resolved.filter(|n| *n > 0) {
            self.line(&format!(
                "{} the last run reported {} fine now, or no longer in the pages.",
                counted(resolved, "link", "links"),
                plural(resolved, "is", "are")
            ))?;
        }
        if !links.acknowledged.is_empty() {
            self.line("Acknowledged as intended:")?;
            self.items(&links.acknowledged, Self::acknowledged)?;
        }
        Ok(())
    }

    /// `file:line: url answered 503 Service Unavailable. Last answered 8
    /// October; reported from 13 October`
    fn unconfirmed(&self, u: &Unconfirmed) -> String {
        let start = self
            .files
            .position(u.location.file, u.location.span.start());
        let at = format!("{}:{}", self.files.path(u.location.file), start.line);
        let said = match &u.answer {
            Answer::Broken { status } => format!("answered {status}"),
            Answer::TimedOut { .. } => "didn't answer in time".to_owned(),
            Answer::Unreachable { reason } => format!("couldn't be reached: {reason}"),
            Answer::Fine | Answer::Moved { .. } => "answered".to_owned(),
        };
        format!(
            "{}: {} {}. Last answered {}; reported from {}",
            self.code(&at),
            self.code(&u.url),
            said.trim_end_matches('.'),
            day(u.last_answered),
            day(u.reported_from)
        )
    }

    fn agents(&mut self, a: &Ran<Agents>) -> io::Result<()> {
        self.heading("Agents")?;
        let agents = match a {
            Ran::NotRun(why) => return self.not_run(why),
            Ran::Done(agents) => agents,
        };
        let mut counts: Vec<(&str, usize)> = Vec::new();
        for r in &agents.results {
            let status = r.status.as_str();
            match counts.iter_mut().find(|(s, _)| *s == status) {
                Some((_, n)) => *n += 1,
                None => counts.push((status, 1)),
            }
        }
        let said: Vec<String> = counts.iter().map(|(s, n)| format!("{n} {s}")).collect();
        self.line(&format!(
            "The delivery spec's checks on {}: {}.",
            self.code(&agents.site),
            said.join(", ")
        ))?;
        if agents.diagnostics.is_empty() {
            self.line("Nothing for the hosting or Ascribe to change.")?;
        } else {
            self.line("To change:")?;
            self.items(&agents.diagnostics, |w, d| w.diagnostic(d))?;
        }
        let pages: Vec<String> = agents
            .results
            .iter()
            .filter(|r| {
                r.status.is_finding()
                    && matches!(
                        ascribe_check::site::owner(&r.id),
                        ascribe_check::site::Owner::Pages(_)
                    )
            })
            .map(|r| self.code(&r.id))
            .collect();
        if !pages.is_empty() {
            self.line(&format!(
                "Also failing, about the pages: {}. `ascribe check` reports what to change in them.",
                pages.join(", ")
            ))?;
        }
        Ok(())
    }
}

/// The page table's columns.
const HEADINGS: [&str; 5] = ["Address", "Listing", "From", "Title", "Description"];

/// A page's row of the table: its address, its listing, where that comes
/// from, its title, and its description.
fn row(r: &PageRecord) -> [String; 5] {
    [
        r.route.clone(),
        r.listing.name().to_owned(),
        listing_from(&r.listing_from),
        r.title.clone().unwrap_or_default(),
        r.description.clone().unwrap_or_default(),
    ]
}

/// Where a listing comes from, in a few words: `default`, `page`, or a rule
/// of `[listing]` and its line in `ascribe.toml`, such as `folder internal/,
/// line 12`.
fn listing_from(from: &ListingFrom) -> String {
    match from {
        ListingFrom::Default => "default".to_owned(),
        ListingFrom::Page => "page".to_owned(),
        ListingFrom::Rule(rule) => {
            let level = match &rule.level {
                ListingLevel::Site => "site".to_owned(),
                ListingLevel::OlderLines => "older lines".to_owned(),
                ListingLevel::Folder(folder) => format!("folder {folder}"),
                ListingLevel::Type(name) => format!("type {name}"),
            };
            format!("{level}, line {}", rule.line)
        }
    }
}

/// `3 errors, 1 warning, 2 advice`.
fn counts_words(counts: Counts) -> String {
    format!(
        "{}, {}, {} advice",
        counted(counts.errors, "error", "errors"),
        counted(counts.warnings, "warning", "warnings"),
        counts.advice
    )
}

/// A day as a person reads it in a report: `8 October`.
fn day(date: Date) -> String {
    format!("{} {}", date.day(), date.month_name())
}

fn plural<'a>(n: usize, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 { one } else { many }
}

fn counted(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", plural(n, one, many))
}

fn capitalized(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// `a`, `a and b`, `a, b, and c`, each written with `name`.
fn names(list: &[String], name: impl Fn(&str) -> String) -> String {
    let named: Vec<String> = list.iter().map(|n| name(n)).collect();
    match named.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// The kinds of next step, in the order they're counted in.
pub(super) const NEXTS: [Next; 5] = [
    Next::Fix,
    Next::Choose,
    Next::Write,
    Next::Review,
    Next::Outside,
];

/// How many diagnostics have each kind of next step, in [`NEXTS`]' order;
/// a kind with none is left out.
pub(super) fn by_next<'a>(diagnostics: impl Iterator<Item = &'a Diagnostic>) -> Vec<(Next, usize)> {
    let mut counts = [0; NEXTS.len()];
    for d in diagnostics {
        let next = d.next().unwrap_or(Next::Write);
        if let Some(i) = NEXTS.iter().position(|n| *n == next) {
            counts[i] += 1;
        }
    }
    NEXTS
        .into_iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// How many diagnostics each check reports, most first, each with the
/// first it reports; a tie keeps the order the check reports them in.
pub(super) fn by_check(diagnostics: &[Diagnostic]) -> Vec<(&Diagnostic, usize)> {
    let mut list: Vec<(&Diagnostic, usize)> = Vec::new();
    for d in diagnostics {
        match list.iter_mut().find(|(first, _)| first.code == d.code) {
            Some((_, n)) => *n += 1,
            None => list.push((d, 1)),
        }
    }
    list.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    list
}

/// How many diagnostics of each severity each file has, most first.
pub(super) fn by_file(files: &FileTable, diagnostics: &[Diagnostic]) -> Vec<(String, Counts)> {
    let mut list: Vec<(String, Counts)> = Vec::new();
    for d in diagnostics {
        let file = files.path(d.location.file);
        match list.iter_mut().find(|(f, _)| *f == file) {
            Some((_, counts)) => counts.add(d.severity),
            None => {
                let mut counts = Counts::default();
                counts.add(d.severity);
                list.push((file, counts));
            }
        }
    }
    list.sort_by_key(|(_, counts)| std::cmp::Reverse(counts.total()));
    list
}
