//! A build's page records: one for each page it publishes, saying what every
//! reader that isn't a person is told about the page, and what the project
//! says to crawlers. The site output writes them as `_ascribe/pages.json`
//! (output-layout contract §1.3), and `llms.txt` is built from them
//! ([`crate::llms::llms_files`]), so the two can't disagree.
//!
//! A record holds the page's route, its source path, its title and
//! description as plain text, its canonical address, its `listing` and
//! where that comes from, and its Markdown version's address when the build
//! publishes one. The project's
//! `[crawlers] lines` are held as a host reads them: a path in an `Allow` or
//! `Disallow` line is from the project's base path in `ascribe.toml`, and
//! from the host's root here ([`host_line`]).
//!
//! [`disallowing`] reads the lines as RFC 9309 says a crawler does, for the
//! check that a page that isn't listed can be fetched, so that the crawler
//! reads that.

use ascribe_core::{RelPath, Router};
use ascribe_model::{ContentModel, Listing, ListingFrom};

use crate::AstroRouter;
use crate::build::ResolvedPage;
use crate::llms;

/// The records of the pages a build publishes, and what the project says to
/// crawlers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRecords {
    /// The project's address, `[consumer] site`, without a final `/`.
    pub site: Option<String>,
    /// One record per page, in the order the pages were given.
    pub pages: Vec<PageRecord>,
    /// The project's `[crawlers] lines`, with paths from the host's root.
    pub robots_lines: Vec<String>,
}

/// What a build says about one page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRecord {
    /// The page's route, from the host's root, as the site output's links
    /// write it.
    pub route: String,
    /// The page's source path, from the content root.
    pub source: RelPath,
    /// Its title, as plain text: the site output's `title`.
    pub title: Option<String>,
    /// Its description, as plain text on one line: what `llms.txt` shows.
    pub description: Option<String>,
    /// Its own address, absolute: the site and the route. `None` without a
    /// site.
    pub canonical: Option<String>,
    /// Its `listing` (SPEC §2.1), after the rules of `[listing]`.
    pub listing: Listing,
    /// Where its `listing` comes from: the page, a rule of `[listing]`, or
    /// nothing.
    pub listing_from: ListingFrom,
    /// Its Markdown version's address, from the host's root, when the build
    /// publishes one (`[consumer] agents = true`).
    pub markdown: Option<String>,
}

impl PageRecord {
    /// Whether the page is offered: in the sitemap and `llms.txt`.
    pub fn offered(&self) -> bool {
        self.listing != Listing::Unlisted
    }
}

/// The records of `pages`, the pages a build publishes, in their order.
pub fn records<'a>(
    model: &ContentModel,
    pages: impl IntoIterator<Item = &'a ResolvedPage>,
) -> PageRecords {
    let router = AstroRouter::from_consumer(&model.consumer);
    let site = model
        .consumer
        .site
        .as_deref()
        .map(|s| s.trim_end_matches('/').to_owned());
    let pages = pages
        .into_iter()
        .map(|page| {
            let route = router.route(&page.path);
            PageRecord {
                canonical: site.as_ref().map(|site| format!("{site}{route}")),
                source: page.path.clone(),
                title: page.title.clone(),
                description: llms::description(model, page),
                listing: page.listing,
                listing_from: page.listing_from.clone(),
                markdown: model
                    .consumer
                    .agents
                    .then(|| router.markdown_url(&page.path)),
                route,
            }
        })
        .collect();
    PageRecords {
        site,
        pages,
        robots_lines: robots_lines(model),
    }
}

/// The project's `[crawlers] lines`, as a host reads them ([`host_line`]).
pub fn robots_lines(model: &ContentModel) -> Vec<String> {
    let base = AstroRouter::from_consumer(&model.consumer);
    model
        .crawlers
        .lines
        .iter()
        .map(|line| host_line(base.base(), &line.text))
        .collect()
}

/// A line of `[crawlers] lines` from the host's root: an `Allow` or
/// `Disallow` path that starts with `/` gets the base path before it, and
/// anything else is as written. `Disallow: /drafts/` under `/docs/` is
/// `Disallow: /docs/drafts/`.
pub fn host_line(base: &str, line: &str) -> String {
    let base = base.trim_end_matches('/');
    let Some((field, value)) = line.split_once(':') else {
        return line.to_owned();
    };
    let field = field.trim();
    let is_rule = field.eq_ignore_ascii_case("allow") || field.eq_ignore_ascii_case("disallow");
    let path = value.trim_start();
    if !is_rule || base.is_empty() || !path.starts_with('/') {
        return line.to_owned();
    }
    let at = line.len() - path.len();
    format!("{}{base}{}", &line[..at], &line[at..])
}

/// The line of `lines`, as [`robots_lines`] gives them, that keeps every
/// crawler from fetching `path`, a path from the host's root, by its index.
///
/// The lines are read as RFC 9309 says a crawler reads them: the groups for
/// `*`, combined, with the lines before any `User-agent` line among them;
/// the rule whose path matches the most characters wins, `*` and `$` read as
/// patterns, and an `Allow` wins a tie. A group for a named crawler isn't
/// read, and a line that isn't a `User-agent`, `Allow`, or `Disallow` line is
/// skipped. `None` when the path may be fetched.
pub fn disallowing(lines: &[String], path: &str) -> Option<usize> {
    // Lines before any `User-agent` belong to `*`.
    let mut for_every = true;
    let mut after_agent = false;
    // The longest match so far: its length, whether it allows, and its line.
    let mut best: Option<(usize, bool, usize)> = None;
    for (at, line) in lines.iter().enumerate() {
        let text = line.split('#').next().unwrap_or_default();
        let Some((field, value)) = text.split_once(':') else {
            continue;
        };
        let field = field.trim();
        let value = value.trim();
        if field.eq_ignore_ascii_case("user-agent") {
            // Consecutive `User-agent` lines open one group.
            if !after_agent {
                for_every = false;
            }
            for_every |= value == "*";
            after_agent = true;
            continue;
        }
        // Any other line, such as `Sitemap` or `Crawl-delay`, ends a run of
        // `User-agent` lines, so one after it opens a group of its own.
        after_agent = false;
        let allow = field.eq_ignore_ascii_case("allow");
        if !allow && !field.eq_ignore_ascii_case("disallow") {
            continue;
        }
        if !for_every || value.is_empty() || !matches(value, path) {
            continue;
        }
        let length = value.len();
        let wins = match best {
            None => true,
            Some((longest, allowed, _)) => {
                length > longest || (length == longest && allow && !allowed)
            }
        };
        if wins {
            best = Some((length, allow, at));
        }
    }
    best.and_then(|(_, allow, at)| (!allow).then_some(at))
}

/// Whether a rule's path matches `path` from its start: `*` is any run of
/// characters, and a final `$` is the path's end.
fn matches(rule: &str, path: &str) -> bool {
    let (rule, anchored) = match rule.strip_suffix('$') {
        Some(rule) => (rule, true),
        None => (rule, false),
    };
    let mut parts = rule.split('*');
    let first = parts.next().unwrap_or_default();
    let Some(mut rest) = path.strip_prefix(first) else {
        return false;
    };
    let parts: Vec<&str> = parts.collect();
    let Some((last, middle)) = parts.split_last() else {
        return !anchored || rest.is_empty();
    };
    for part in middle {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    if anchored {
        rest.ends_with(last)
    } else {
        rest.contains(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|&l| l.to_owned()).collect()
    }

    #[test]
    fn a_rule_path_gets_the_base_path() {
        assert_eq!(
            host_line("/docs/", "Disallow: /drafts/"),
            "Disallow: /docs/drafts/"
        );
        assert_eq!(host_line("/docs/", "allow:/a"), "allow:/docs/a");
        assert_eq!(host_line("/", "Disallow: /drafts/"), "Disallow: /drafts/");
        assert_eq!(host_line("/docs/", "Disallow:"), "Disallow:");
        assert_eq!(host_line("/docs/", "Disallow: *.pdf"), "Disallow: *.pdf");
        assert_eq!(host_line("/docs/", "User-agent: *"), "User-agent: *");
        assert_eq!(host_line("/docs/", "Sitemap: /a.xml"), "Sitemap: /a.xml");
    }

    #[test]
    fn the_longest_match_wins_and_allow_wins_a_tie() {
        let l = lines(&["Disallow: /a/", "Allow: /a/b/"]);
        assert_eq!(disallowing(&l, "/a/c/"), Some(0));
        assert_eq!(disallowing(&l, "/a/b/c/"), None);
        let l = lines(&["Disallow: /a/", "Allow: /a/"]);
        assert_eq!(disallowing(&l, "/a/"), None);
        assert_eq!(disallowing(&lines(&["Disallow:"]), "/a/"), None);
    }

    #[test]
    fn only_the_groups_for_every_crawler_are_read() {
        let l = lines(&["User-agent: Googlebot", "Disallow: /a/"]);
        assert_eq!(disallowing(&l, "/a/"), None);
        let l = lines(&[
            "Disallow: /a/",
            "User-agent: Googlebot",
            "Disallow: /b/",
            "User-agent: *",
            "Disallow: /c/",
        ]);
        assert_eq!(disallowing(&l, "/a/"), Some(0));
        assert_eq!(disallowing(&l, "/b/"), None);
        assert_eq!(disallowing(&l, "/c/"), Some(4));
        let l = lines(&["User-agent: Googlebot", "User-agent: *", "Disallow: /a/"]);
        assert_eq!(disallowing(&l, "/a/"), Some(2));
        // Another line ends the run of `User-agent` lines.
        for between in ["Crawl-delay: 5", "Sitemap: https://example.com/s.xml"] {
            let l = lines(&["User-agent: *", between, "User-agent: Bot", "Disallow: /x/"]);
            assert_eq!(disallowing(&l, "/x/"), None, "{between}");
        }
    }

    #[test]
    fn patterns_and_comments() {
        let l = lines(&["Disallow: /*.pdf$ # files"]);
        assert_eq!(disallowing(&l, "/a/b.pdf"), Some(0));
        assert_eq!(disallowing(&l, "/a/b.pdf/"), None);
        let l = lines(&["Disallow: /a*/c"]);
        assert_eq!(disallowing(&l, "/ab/cd/"), Some(0));
        assert_eq!(disallowing(&l, "/b/c"), None);
        assert_eq!(disallowing(&lines(&["Disallow: /a$"]), "/a/"), None);
        assert_eq!(disallowing(&lines(&["not a rule"]), "/a/"), None);
    }
}
