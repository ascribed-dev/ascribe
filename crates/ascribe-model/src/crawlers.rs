//! `[crawlers]`: what a project says to the crawlers that read its site.
//! `lines` holds the project's own `robots.txt` lines, each one line and not
//! empty. What a line says isn't read here, apart from the crawlers its
//! `User-agent` lines name: a build writes the lines as they are, with the
//! base path put before each `Allow` and `Disallow` path, and `ascribe check`
//! reads the ones it can. `training`, `search`, and `on-request` say what
//! each kind of crawler may do, `allow` or `disallow`; a disallowed purpose
//! writes one group naming every token [`known_crawlers`] gives it.
//!
//! A crawler obeys only the one group that names it (RFC 9309), so a token in
//! two groups is an error: named by a group of `lines` and by a disallowed
//! purpose, or by two groups of `lines`.

use std::sync::OnceLock;

use ascribe_core::{Span, diagnostics};
use toml::de::{DeTable, DeValue};

use crate::loader::Loader;
use crate::model::{CrawlerLine, Crawlers, Permission, Purpose, PurposeChoice};
use crate::toml_util::{V, entries, sp};

/// The data file of the crawlers Ascribe knows, which the binary carries.
pub const KNOWN_CRAWLERS_FILE: &str = include_str!("../data/crawlers.toml");

/// A `robots.txt` token Ascribe knows, from its operator's own page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownCrawler {
    /// The name a `User-agent` line gives.
    pub token: String,
    /// Who runs it.
    pub operator: String,
    /// What it's for. `None` when the file names a purpose Ascribe doesn't
    /// have, which a test catches.
    pub purpose: Option<Purpose>,
    /// What the operator says of `robots.txt`: `follows` or `may-ignore`.
    pub robots: String,
    /// The address of the operator's page about it; empty when the file has
    /// none, which a test catches.
    pub source: String,
    /// The day the page was read, `YYYY-MM-DD`; empty when the file has none,
    /// which a test catches.
    pub read: String,
    /// Anything else the reference says about it.
    pub note: Option<String>,
}

/// Every crawler [`KNOWN_CRAWLERS_FILE`] lists, in its order. Empty if the
/// file doesn't parse, which a test catches.
pub fn known_crawlers() -> &'static [KnownCrawler] {
    static KNOWN: OnceLock<Vec<KnownCrawler>> = OnceLock::new();
    KNOWN.get_or_init(|| parse_known(KNOWN_CRAWLERS_FILE).unwrap_or_default())
}

/// The tokens of `purpose`, in the file's order.
pub fn crawler_tokens(purpose: Purpose) -> impl Iterator<Item = &'static str> {
    known_crawlers()
        .iter()
        .filter(move |c| c.purpose == Some(purpose))
        .map(|c| c.token.as_str())
}

/// The crawlers in a data file shaped as [`KNOWN_CRAWLERS_FILE`] is, or
/// `None` when it isn't TOML with a `crawler` array of tables. A missing or
/// mistyped field is left empty, for a test to name.
pub fn parse_known(text: &str) -> Option<Vec<KnownCrawler>> {
    let table = DeTable::parse(text).ok()?;
    let DeValue::Array(items) = table.get_ref().get("crawler")?.get_ref() else {
        return None;
    };
    let mut out = Vec::new();
    for item in items.iter() {
        let DeValue::Table(t) = item.get_ref() else {
            return None;
        };
        let field = |key: &str| match t.get(key).map(|v| v.get_ref()) {
            Some(DeValue::String(s)) => Some(s.to_string()),
            _ => None,
        };
        out.push(KnownCrawler {
            token: field("token").unwrap_or_default(),
            operator: field("operator").unwrap_or_default(),
            purpose: field("purpose").and_then(|p| Purpose::from_name(&p)),
            robots: field("robots").unwrap_or_default(),
            source: field("source").unwrap_or_default(),
            read: field("read").unwrap_or_default(),
            note: field("note"),
        });
    }
    Some(out)
}

/// The value of a `User-agent` line: the crawler it names, without a comment
/// or spaces. `None` for any other line.
pub fn user_agent(line: &str) -> Option<&str> {
    let text = line.split('#').next().unwrap_or_default();
    let (field, value) = text.split_once(':')?;
    field
        .trim()
        .eq_ignore_ascii_case("user-agent")
        .then(|| value.trim())
}

impl Loader<'_> {
    /// `[crawlers]`: `lines`, an array of single lines, and the purposes.
    pub(crate) fn crawlers(&mut self, v: Option<&V<'_>>) -> Crawlers {
        let mut out = Crawlers::default();
        let Some(v) = v else { return out };
        let Some(t) = self.as_table("crawlers", v) else {
            return out;
        };
        self.check_table("crawlers", t, "crawlers");
        for (key, key_span, value) in entries(t) {
            let Some(purpose) = Purpose::from_name(key) else {
                continue;
            };
            let names = Permission::ALL.map(Permission::name);
            let Some(name) = self.choice(&format!("crawlers.{key}"), value, &names) else {
                continue;
            };
            if let Some(permission) = Permission::from_name(&name) {
                out.purposes.push(PurposeChoice {
                    purpose,
                    permission,
                    span: Span::new(key_span.start(), sp(value).end()),
                });
            }
        }
        out.purposes.sort_by_key(|c| c.purpose);
        if let Some(lines) = t.get("lines")
            && let Some(lines) = self.strings("crawlers.lines", lines)
        {
            for (n, (text, span)) in lines.into_iter().enumerate() {
                let key = format!("crawlers.lines[{n}]");
                if text.trim().is_empty() {
                    self.push(
                        self.issue(diagnostics::MODEL_EMPTY_TEXT, span)
                            .with_arg("key", key),
                    );
                } else if text.contains(['\n', '\r']) {
                    self.push(
                        self.issue(diagnostics::MODEL_CRAWLERS_LINE, span)
                            .with_arg("key", key),
                    );
                } else {
                    out.lines.push(CrawlerLine { text, span });
                }
            }
        }
        self.crawlers_named_twice(&out);
        out
    }

    /// `model-crawlers-named-twice`: a `User-agent` line of `lines` naming a
    /// token a disallowed purpose's group names, or one an earlier group of
    /// `lines` names. Consecutive `User-agent` lines open one group, and any
    /// other line ends the run. `*` is left alone: its groups are combined.
    fn crawlers_named_twice(&mut self, crawlers: &Crawlers) {
        // Each token named so far, its group, and its line.
        let mut named: Vec<(String, usize, usize)> = Vec::new();
        let mut group = 0;
        let mut after_agent = false;
        for (at, line) in crawlers.lines.iter().enumerate() {
            let Some(token) = user_agent(&line.text) else {
                after_agent = false;
                continue;
            };
            if !after_agent {
                group += 1;
            }
            after_agent = true;
            if token == "*" {
                continue;
            }
            let disallowed = Purpose::ALL.into_iter().find(|&p| {
                crawlers.disallows(p)
                    && crawler_tokens(p).any(|known| known.eq_ignore_ascii_case(token))
            });
            let earlier = named
                .iter()
                .find(|(t, g, _)| *g != group && t.eq_ignore_ascii_case(token));
            let issue = if let Some(purpose) = disallowed {
                self.issue(diagnostics::MODEL_CRAWLERS_NAMED_TWICE, line.span)
                    .with_variant("purpose")
                    .with_arg("purpose", purpose.name())
            } else if let Some((_, _, first)) = earlier {
                let first = crawlers.lines.get(*first).map_or(line.span, |l| l.span);
                self.issue(diagnostics::MODEL_CRAWLERS_NAMED_TWICE, line.span)
                    .with_arg("line", self.line_of(first).to_string())
            } else {
                named.push((token.to_owned(), group, at));
                continue;
            };
            self.push(
                issue
                    .with_arg("key", format!("crawlers.lines[{at}]"))
                    .with_arg("token", token),
            );
        }
    }

    /// The line `span` starts on in `ascribe.toml`, from 1.
    fn line_of(&self, span: Span) -> usize {
        self.src
            .get(..span.start())
            .unwrap_or_default()
            .matches('\n')
            .count()
            + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_user_agent_line_names_its_crawler() {
        assert_eq!(user_agent("User-agent: GPTBot"), Some("GPTBot"));
        assert_eq!(user_agent("user-agent:*  # all"), Some("*"));
        assert_eq!(user_agent("Disallow: /"), None);
        assert_eq!(user_agent("# User-agent: x"), None);
    }

    #[test]
    fn a_data_file_without_crawlers_doesnt_parse() {
        assert_eq!(parse_known("crawler = 1"), None);
        assert_eq!(parse_known("not toml ="), None);
        let known = parse_known("[[crawler]]\ntoken = \"Bot\"\npurpose = \"other\"\n");
        let known = known.unwrap_or_default();
        assert_eq!(known.len(), 1);
        assert_eq!(known.first().map(|c| c.purpose), Some(None));
        assert_eq!(known.first().map(|c| c.read.as_str()), Some(""));
    }

    /// What a commit can break: every token Ascribe knows has an operator,
    /// one of the three purposes, what the operator says of `robots.txt`, a
    /// source, and the day it was read, and is listed once. How old a date is
    /// isn't a test's to judge: the daily Report workflow says that.
    #[test]
    fn every_known_crawler_has_a_source_and_a_date() {
        let known = parse_known(KNOWN_CRAWLERS_FILE).unwrap_or_default();
        assert!(!known.is_empty(), "data/crawlers.toml has no crawlers");
        assert_eq!(known_crawlers(), known.as_slice());
        let mut tokens = std::collections::BTreeSet::new();
        for c in &known {
            let name = &c.token;
            assert!(!name.is_empty(), "a crawler has no token");
            assert!(!name.contains([' ', ':', '#', '*']), "{name}: not a token");
            assert!(
                tokens.insert(name.to_ascii_lowercase()),
                "{name}: listed twice"
            );
            assert!(!c.operator.is_empty(), "{name}: no operator");
            assert!(
                c.purpose.is_some(),
                "{name}: no purpose, or not one of the three"
            );
            assert!(
                ["follows", "may-ignore"].contains(&c.robots.as_str()),
                "{name}: robots"
            );
            assert!(c.source.starts_with("https://"), "{name}: no source");
            let date: Vec<&str> = c.read.split('-').collect();
            let digits = |s: &&str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_digit());
            assert!(
                matches!(date.as_slice(), [y, m, d] if digits(y, 4) && digits(m, 2) && digits(d, 2)),
                "{name}: no date it was read, as YYYY-MM-DD"
            );
        }
        for purpose in Purpose::ALL {
            assert!(
                crawler_tokens(purpose).next().is_some(),
                "{purpose:?}: no tokens"
            );
        }
    }
}
