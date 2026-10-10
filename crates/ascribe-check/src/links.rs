//! External links, checked by a link checker the project installs: what
//! `ascribe report links` reports.
//!
//! Ascribe lists the project's external links with the places they're
//! written, hands their addresses to the checker, and maps what it says back
//! to those places as diagnostics (content checks decision 5): a link whose
//! address moved permanently is `link-external-moved`, with a fix that
//! writes the new address; one that answers with an error, doesn't answer in
//! time, or can't be reached is `link-external-broken`, a review check an
//! author can acknowledge. The checker is [lychee](https://lychee.cli.rs),
//! run as a program ([`Lychee`]); a test gives a stand-in ([`LinkChecker`]).
//! The binary itself makes no request.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use ascribe_core::{Applicability, Fix, Issue, Location, Span, TextEdit, diagnostics};
use ascribe_model::CheckLevel;
use ascribe_resolve::{RefKind, Target};
use serde::Deserialize;

use crate::tool::{self, ToolError};
use crate::{Diagnostic, Project};

/// How long the link checker may take, in all.
pub const TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// The most characters of the sentence around a link a prompt quotes.
const MAX_SENTENCE: usize = 300;

/// An external link, where it's written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    /// Its address, as written.
    pub url: String,
    /// The link: its file and span, from the `[` through its last character.
    pub location: Location,
    /// The address as written in the link, for an inline link; a reference
    /// link's address is in its definition.
    pub destination: Option<Span>,
}

/// What the link checker said about an address.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    /// It answers, or the checker left it out.
    Fine,
    /// It redirects permanently (`301` or `308`) to `to`, which answers.
    Moved {
        /// The address it redirects to: the last of its permanent
        /// redirects.
        to: String,
        /// The first redirect's status code.
        code: u16,
    },
    /// It answers with an error status.
    Broken {
        /// The status, `404 Not Found`.
        status: String,
    },
    /// It didn't answer in time.
    TimedOut {
        /// What the checker said.
        said: String,
    },
    /// It couldn't be reached at all: no such host, a refused connection, a
    /// certificate that isn't valid.
    Unreachable {
        /// What the checker said.
        reason: String,
    },
}

/// What to give the link checker.
#[derive(Clone, Debug)]
pub struct Request {
    /// The command, as `[checks.links] command` gives it. One with a path
    /// separator is relative to `root`; a bare name is looked up as the
    /// shell would.
    pub command: String,
    /// The project root, where it runs, so it finds its own configuration
    /// there (`lychee.toml`).
    pub root: PathBuf,
    /// The addresses, each once, in order.
    pub urls: Vec<String>,
    /// How long it may take.
    pub timeout: Duration,
}

/// What checks links: the program, or a stand-in in tests.
pub trait LinkChecker: Send + Sync {
    /// What it says about each address of the request, by address. An
    /// address it says nothing about is fine.
    ///
    /// # Errors
    ///
    /// It couldn't be run, failed, or ran out of time.
    fn check(&self, request: &Request) -> Result<BTreeMap<String, Answer>, ToolError>;
}

/// The project's external links, and what the checker said about them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checked {
    /// A diagnostic for each link that moved or is broken, in file order and
    /// source order, before `[checks]` levels and acknowledgements.
    pub diagnostics: Vec<Diagnostic>,
    /// How many different addresses were checked.
    pub checked: usize,
    /// How many different addresses `[checks.links] ignore` left out.
    pub ignored: usize,
}

/// Every external link in the project's pages and fragments (an `http` or
/// `https` address), in file order and source order.
pub fn external_links(project: &Project) -> Vec<Link> {
    let index = project.held_index();
    let mut out = Vec::new();
    for file in index.files() {
        let Some(source) = project.source_at(&file.path) else {
            continue;
        };
        for reference in &file.references {
            if reference.kind != RefKind::Link
                || reference.target != Target::External
                || host(&reference.destination).is_none()
            {
                continue;
            }
            out.push(Link {
                url: reference.destination.clone(),
                location: Location::new(source.id, reference.span),
                destination: reference.destination_span,
            });
        }
    }
    out.sort_by_key(|l| (l.location.file, l.location.span.start()));
    out
}

/// Checks the project's external links with `checker`, leaving out the
/// hosts `[checks.links] ignore` names. The checker isn't run when there's
/// nothing to check, or when `[checks]` turns both link checks off.
///
/// # Errors
///
/// The link checker couldn't be run, failed, or ran out of time.
pub fn check(
    project: &Project,
    checker: &dyn LinkChecker,
    timeout: Duration,
) -> Result<Checked, ToolError> {
    let checks = &project.model().checks;
    let links = external_links(project);
    let mut checked: Vec<String> = Vec::new();
    let mut ignored: Vec<String> = Vec::new();
    for link in &links {
        let list = if host(&link.url).is_some_and(|h| checks.links.ignores(h)) {
            &mut ignored
        } else {
            &mut checked
        };
        if !list.contains(&link.url) {
            list.push(link.url.clone());
        }
    }
    let off = |slug| checks.level(slug) == Some(CheckLevel::Off);
    let mut out = Checked {
        diagnostics: Vec::new(),
        checked: checked.len(),
        ignored: ignored.len(),
    };
    if checked.is_empty()
        || (off(diagnostics::LINK_EXTERNAL_BROKEN) && off(diagnostics::LINK_EXTERNAL_MOVED))
    {
        return Ok(out);
    }
    let answers = checker.check(&Request {
        command: checks.links.command.clone(),
        root: project.root().to_owned(),
        urls: checked,
        timeout,
    })?;
    for link in &links {
        let Some(answer) = answers.get(&link.url) else {
            continue;
        };
        if let Some(d) = diagnostic(project, link, answer) {
            out.diagnostics.push(d);
        }
    }
    Ok(out)
}

/// The diagnostic about a link the checker said `answer` about, if it's one
/// to report.
fn diagnostic(project: &Project, link: &Link, answer: &Answer) -> Option<Diagnostic> {
    let sentence = || sentence_around(project, link.location);
    let issue = match answer {
        Answer::Fine => return None,
        Answer::Moved { to, .. } if is_landing_page(&link.url, to) => return None,
        Answer::Moved { to, .. } => {
            let mut issue = Issue::new(diagnostics::LINK_EXTERNAL_MOVED, link.location)
                .with_arg("url", link.url.clone())
                .with_arg("to", to.clone());
            if let Some(span) = link.destination {
                issue = issue.with_fix(Fix {
                    title: "Link to the new address".to_owned(),
                    file: link.location.file,
                    edits: vec![TextEdit::replace(span, destination(to))],
                    // A site may send every old address to its home page.
                    applicability: Applicability::Unsafe,
                });
            }
            issue
        }
        Answer::Broken { status } => Issue::new(diagnostics::LINK_EXTERNAL_BROKEN, link.location)
            .with_arg("url", link.url.clone())
            .with_arg("status", status.clone())
            .with_arg("sentence", sentence()),
        Answer::TimedOut { said } => Issue::new(diagnostics::LINK_EXTERNAL_BROKEN, link.location)
            .with_variant("timeout")
            .with_arg("url", link.url.clone())
            .with_arg("said", said.clone())
            .with_arg("sentence", sentence()),
        Answer::Unreachable { reason } => {
            Issue::new(diagnostics::LINK_EXTERNAL_BROKEN, link.location)
                .with_variant("error")
                .with_arg("url", link.url.clone())
                .with_arg("reason", reason.clone())
                .with_arg("sentence", sentence())
        }
    };
    Some(Diagnostic::from_issue(&issue))
}

/// Whether `to` is where the site at `from` sends a visitor to its home
/// page: `from` is a site's root, and `to` another page of the same site,
/// `www.` and the scheme aside. A site that does this, such as to its latest version's
/// introduction, still answers at its root, and linking to the landing page
/// would pin what the site may change.
fn is_landing_page(from: &str, to: &str) -> bool {
    let site =
        |url: &str| host(url).map(|h| h.strip_prefix("www.").unwrap_or(h).to_ascii_lowercase());
    fn path(url: &str) -> &str {
        let after = url.split_once("://").map_or("", |(_, rest)| rest);
        after.find(['/', '?', '#']).map_or("", |i| &after[i..])
    }
    matches!(path(from), "" | "/") && !matches!(path(to), "" | "/") && site(from) == site(to)
}

/// An address as a link's destination: in angle brackets when it has a
/// space or a parenthesis, which would end it otherwise.
fn destination(url: &str) -> String {
    if url.contains([' ', '(', ')', '<', '>']) {
        format!("<{}>", url.replace('<', "%3C").replace('>', "%3E"))
    } else {
        url.to_owned()
    }
}

/// The host of an `http` or `https` address, without a port or user; `None`
/// for any other address.
pub fn host(url: &str) -> Option<&str> {
    let lower = url.get(..8).unwrap_or(url).to_ascii_lowercase();
    let rest = if lower.starts_with("https://") {
        &url[8..]
    } else if lower.starts_with("http://") {
        &url[7..]
    } else {
        return None;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let authority = authority.rsplit('@').next().unwrap_or_default();
    let host = if let Some(bracketed) = authority.strip_prefix('[') {
        bracketed.split(']').next().unwrap_or_default()
    } else {
        authority.split(':').next().unwrap_or_default()
    };
    (!host.is_empty()).then_some(host)
}

/// The sentence a link is in: its paragraph's text from the end of the
/// sentence before it to the end of its own, on one line, at most
/// [`MAX_SENTENCE`] characters.
fn sentence_around(project: &Project, at: Location) -> String {
    let Some(text) = project.file(at.file).map(|f| f.text) else {
        return String::new();
    };
    let (start, end) = (at.span.start(), at.span.end());
    let (Some(before), Some(after)) = (text.get(..start), text.get(end..)) else {
        return String::new();
    };
    // The paragraph: back to the blank line before, on to the one after.
    let from = before.rfind("\n\n").map_or(0, |i| i + 2);
    let to = after.find("\n\n").map_or(text.len(), |i| end + i);
    // The sentence: from the end of the one before to the end of its own.
    let ends = |s: &str| {
        s.char_indices()
            .filter(|(i, c)| {
                matches!(c, '.' | '!' | '?') && s[i + 1..].starts_with(|n: char| n.is_whitespace())
            })
            .map(|(i, _)| i + 1)
            .collect::<Vec<_>>()
    };
    let head = &text[from..start];
    let from = ends(head).last().map_or(from, |i| from + i);
    let tail = &text[end..to];
    let to = ends(tail).first().map_or(to, |i| end + i);
    let line = text[from..to]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut short: String = line.chars().take(MAX_SENTENCE).collect();
    if short.len() < line.len() {
        short.push('…');
    }
    short
}

/// Runs [lychee](https://lychee.cli.rs): the addresses go in on standard
/// input, one a line, and its JSON comes out, with every answer
/// (`--verbose`) so that redirects are listed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Lychee;

impl LinkChecker for Lychee {
    fn check(&self, request: &Request) -> Result<BTreeMap<String, Answer>, ToolError> {
        let program = tool::program(&request.command, &request.root);
        let args: Vec<String> = ["--format", "json", "--verbose", "--no-progress", "-"]
            .iter()
            .map(|a| (*a).to_owned())
            .collect();
        let mut input = request.urls.join("\n");
        input.push('\n');
        let output = tool::run(
            &request.command,
            &program,
            &args,
            &request.root,
            input.as_bytes(),
            request.timeout,
        )?;
        read_lychee(&request.command, &request.urls, &output)
    }
}

/// lychee's JSON: each kind of answer by input, and each answer with the
/// line of its address.
#[derive(Deserialize)]
struct LycheeStats {
    #[serde(default)]
    success_map: BTreeMap<String, Vec<LycheeAnswer>>,
    #[serde(default)]
    error_map: BTreeMap<String, Vec<LycheeAnswer>>,
    #[serde(default)]
    timeout_map: BTreeMap<String, Vec<LycheeAnswer>>,
}

#[derive(Deserialize)]
struct LycheeAnswer {
    url: String,
    status: LycheeStatus,
    #[serde(default)]
    redirects: Option<LycheeRedirects>,
    #[serde(default)]
    span: Option<LycheeSpan>,
}

#[derive(Deserialize)]
struct LycheeStatus {
    #[serde(default)]
    text: String,
    #[serde(default)]
    code: Option<u16>,
    #[serde(default)]
    details: Option<String>,
}

#[derive(Deserialize)]
struct LycheeRedirects {
    #[serde(default)]
    redirects: Vec<LycheeRedirect>,
}

#[derive(Deserialize)]
struct LycheeRedirect {
    url: String,
    code: u16,
}

#[derive(Deserialize)]
struct LycheeSpan {
    line: usize,
}

/// What lychee said about each of `urls`, from what it wrote. It reads the
/// addresses as text, one a line, so an answer's line says which address
/// it's about whatever lychee made of it (a slash added, say).
fn read_lychee(
    command: &str,
    urls: &[String],
    output: &tool::Output,
) -> Result<BTreeMap<String, Answer>, ToolError> {
    let stats: LycheeStats =
        serde_json::from_slice(&output.stdout).map_err(|_| ToolError::Failed {
            command: command.to_owned(),
            reason: tool::said(output),
        })?;
    let address = |a: &LycheeAnswer| {
        a.span
            .as_ref()
            .and_then(|s| urls.get(s.line.checked_sub(1)?))
            .cloned()
            .unwrap_or_else(|| a.url.clone())
    };
    let mut out = BTreeMap::new();
    for answer in stats.success_map.values().flatten() {
        if let Some(moved) = moved(answer) {
            out.insert(address(answer), moved);
        }
    }
    for answer in stats.timeout_map.values().flatten() {
        out.insert(
            address(answer),
            Answer::TimedOut {
                said: said(&answer.status),
            },
        );
    }
    for answer in stats.error_map.values().flatten() {
        let found = match answer.status.code {
            Some(_) => Answer::Broken {
                status: status(&answer.status),
            },
            None => Answer::Unreachable {
                reason: said(&answer.status),
            },
        };
        out.insert(address(answer), found);
    }
    Ok(out)
}

/// A permanent redirect: the address after the permanent redirects the
/// chain starts with, when it starts with one.
fn moved(answer: &LycheeAnswer) -> Option<Answer> {
    let chain = &answer.redirects.as_ref()?.redirects;
    let code = chain.first()?.code;
    let to = chain
        .iter()
        .take_while(|r| matches!(r.code, 301 | 308))
        .last()?;
    Some(Answer::Moved {
        to: to.url.clone(),
        code,
    })
}

/// A status as a person reads it: `404 Not Found`, without lychee's
/// `Rejected status code: `.
fn status(status: &LycheeStatus) -> String {
    let text = status.text.trim();
    let text = text
        .strip_prefix("Rejected status code:")
        .map_or(text, str::trim);
    match status.code {
        Some(code) if !text.starts_with(&code.to_string()) => format!("{code} {text}"),
        _ => text.to_owned(),
    }
}

/// What lychee said: the status's text, and its details when they say more.
fn said(status: &LycheeStatus) -> String {
    let text = status.text.trim();
    match status.details.as_deref().map(str::trim) {
        Some(details) if !details.is_empty() && !text.contains(details) => {
            format!("{text}: {details}")
        }
        _ => text.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn hosts_come_from_http_addresses_only() {
        assert_eq!(host("https://example.com/a?b#c"), Some("example.com"));
        assert_eq!(host("HTTP://user@Example.com:8080"), Some("Example.com"));
        assert_eq!(host("https://[::1]:80/"), Some("::1"));
        assert_eq!(host("mailto:someone@example.com"), None);
        assert_eq!(host("ftp://example.com"), None);
        assert_eq!(host("https:///nothing"), None);
    }

    #[test]
    fn a_destination_with_a_parenthesis_is_bracketed() {
        assert_eq!(destination("https://a.example/b"), "https://a.example/b");
        assert_eq!(
            destination("https://a.example/b_(c)"),
            "<https://a.example/b_(c)>"
        );
    }

    #[test]
    fn reads_what_lychee_writes() {
        let urls: Vec<String> = [
            "https://ok.example",
            "https://moved.example/a",
            "https://temp.example",
            "https://gone.example",
            "https://slow.example",
            "https://nohost.example",
        ]
        .iter()
        .map(|u| (*u).to_owned())
        .collect();
        let json = r#"{
          "success_map": {"stdin": [
            {"url": "https://ok.example/", "status": {"text": "200 OK", "code": 200}, "span": {"line": 1, "column": 1}},
            {"url": "https://moved.example/a", "status": {"text": "200 OK", "code": 200},
             "redirects": {"origin": "https://moved.example/a", "redirects": [
               {"url": "https://moved.example/a/", "code": 308},
               {"url": "https://new.example/a/", "code": 301},
               {"url": "https://new.example/a/?session=1", "code": 302}]},
             "span": {"line": 2, "column": 1}},
            {"url": "https://temp.example/", "status": {"text": "200 OK", "code": 200},
             "redirects": {"origin": "https://temp.example/", "redirects": [{"url": "https://temp.example/x", "code": 302}]},
             "span": {"line": 3, "column": 1}}]},
          "error_map": {"stdin": [
            {"url": "https://gone.example/", "status": {"text": "Rejected status code: 404 Not Found", "code": 404}, "span": {"line": 4, "column": 1}},
            {"url": "https://nohost.example/", "status": {"text": "Network error", "details": "dns error"}, "span": {"line": 6, "column": 1}}]},
          "timeout_map": {"stdin": [
            {"url": "https://slow.example/", "status": {"text": "Timeout", "details": "Request timed out"}, "span": {"line": 5, "column": 1}}]}
        }"#;
        let output = tool::Output {
            stdout: json.as_bytes().to_vec(),
            stderr: Vec::new(),
        };
        let answers = read_lychee("lychee", &urls, &output).unwrap();
        assert_eq!(answers.get("https://ok.example"), None);
        assert_eq!(answers.get("https://temp.example"), None);
        assert_eq!(
            answers["https://moved.example/a"],
            Answer::Moved {
                to: "https://new.example/a/".to_owned(),
                code: 308
            }
        );
        assert_eq!(
            answers["https://gone.example"],
            Answer::Broken {
                status: "404 Not Found".to_owned()
            }
        );
        assert_eq!(
            answers["https://slow.example"],
            Answer::TimedOut {
                said: "Timeout: Request timed out".to_owned()
            }
        );
        assert_eq!(
            answers["https://nohost.example"],
            Answer::Unreachable {
                reason: "Network error: dns error".to_owned()
            }
        );
    }

    #[test]
    fn output_that_isn_t_lychee_s_json_is_a_failure() {
        let output = tool::Output {
            stdout: Vec::new(),
            stderr: b"error: unexpected argument '--verbose'\n".to_vec(),
        };
        let error = read_lychee("lychee", &[], &output).unwrap_err();
        assert_eq!(
            error.to_string(),
            "`lychee` failed: error: unexpected argument '--verbose'"
        );
    }

    #[test]
    fn a_site_s_root_sending_readers_to_a_landing_page_hasn_t_moved() {
        let landing = |from, to| is_landing_page(from, to);
        assert!(landing(
            "https://modelcontextprotocol.io",
            "https://modelcontextprotocol.io/docs/2026-07-28/getting-started/intro"
        ));
        assert!(landing(
            "https://example.com/",
            "https://www.example.com/en/"
        ));
        // The scheme aside too: the root still answers, upgraded.
        assert!(landing("http://example.com", "https://example.com/docs"));
        // To another site, to the root, or from a page: it moved.
        assert!(!landing("https://example.com/", "https://example.org/docs"));
        assert!(!landing("http://example.com", "https://example.com/"));
        assert!(!landing("https://example.com/a", "https://example.com/b"));
        assert!(!landing(
            "https://example.com/?q=1",
            "https://example.com/docs"
        ));
    }
}
