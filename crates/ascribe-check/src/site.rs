//! A published site, checked against the Web Documentation Delivery Spec
//! with its own checker, [`afdocs`](https://www.npmjs.com/package/afdocs):
//! what `ascribe report agents` reports.
//!
//! The checker is a program the project installs (content checks decision
//! 5), run on the site's address ([`Afdocs`]); a test gives a stand-in
//! ([`SiteChecker`]). Each of its checks is known here by who can make it
//! pass ([`Owner`]): one that `llms.txt`, the Markdown pages, or the pointer
//! on each page should pass fails because of a bug in Ascribe
//! (`delivery-output`, with the issue tracker's address); one the hosting
//! decides is `delivery-hosting`, which says what to set and where; and one
//! about a page's own size is already `ascribe check`'s, at the page. None
//! asks the author to edit a page.

use std::path::PathBuf;
use std::time::Duration;

use ascribe_core::{DiagnosticSlug, FileId, Issue, Location, Span, diagnostics};
use ascribe_model::CheckLevel;
use serde::Deserialize;

use crate::tool::{self, ToolError};
use crate::{Diagnostic, Project};

/// The command that runs the checker.
pub const COMMAND: &str = "afdocs";

/// The version of the checker Ascribe's output is tested with, and which
/// `ascribe report agents` asks for when it isn't installed: the one
/// `examples/astro-site` pins.
pub const VERSION: &str = "0.22.2";

/// How long the checker may take, in all.
pub const TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Where a bug in Ascribe is reported.
pub const ISSUES: &str = concat!(env!("CARGO_PKG_REPOSITORY"), "/issues");

/// Who can make one of the checker's checks pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// What Ascribe writes with `[consumer] agents = true`: a failure is a
    /// bug in Ascribe.
    Ascribe,
    /// The site's host, or the site's own templates: what to set, and where.
    Hosting(&'static str),
    /// A page's own size, which a check of `ascribe check` reports at the
    /// page.
    Pages(DiagnosticSlug),
}

impl Owner {
    /// `ascribe`, `hosting`, or `pages`.
    pub fn as_str(self) -> &'static str {
        match self {
            Owner::Ascribe => "ascribe",
            Owner::Hosting(_) => "hosting",
            Owner::Pages(_) => "pages",
        }
    }
}

/// The checker's checks, by id, and who can make each pass. A check that
/// isn't listed, from a later version of the checker, is the hosting's,
/// with the checker's own message.
pub const CHECKS: &[(&str, Owner)] = &[
    ("llms-txt-exists", Owner::Ascribe),
    ("llms-txt-valid", Owner::Ascribe),
    (
        "llms-txt-size",
        Owner::Pages(diagnostics::LLMS_SECTION_LARGE),
    ),
    ("llms-txt-links-resolve", Owner::Ascribe),
    ("llms-txt-links-markdown", Owner::Ascribe),
    ("llms-txt-directive-html", Owner::Ascribe),
    ("llms-txt-directive-md", Owner::Ascribe),
    ("llms-txt-coverage", Owner::Ascribe),
    ("markdown-url-support", Owner::Ascribe),
    ("markdown-code-fence-validity", Owner::Ascribe),
    ("markdown-link-portability", Owner::Ascribe),
    ("markdown-content-parity", Owner::Ascribe),
    ("section-header-quality", Owner::Ascribe),
    (
        "content-negotiation",
        Owner::Hosting(
            "Answer a request with `Accept: text/markdown` with the page's `.md`, with a rewrite rule or an edge function on the host",
        ),
    ),
    (
        "cache-header-hygiene",
        Owner::Hosting(
            "Set `Cache-Control` on the host so that `llms.txt` and the `.md` files are cached no longer than the pages",
        ),
    ),
    (
        "http-status-codes",
        Owner::Hosting(
            "Answer an address with no page with status 404, not 200: set the host's not-found page",
        ),
    ),
    (
        "redirect-behavior",
        Owner::Hosting(
            "Redirect with an HTTP status on the same host, in the host's redirect rules, not with a script or another host",
        ),
    ),
    (
        "auth-gate-detection",
        Owner::Hosting(
            "Let agents read the docs without signing in, in the host's access settings",
        ),
    ),
    (
        "auth-alternative-access",
        Owner::Hosting(
            "Serve `llms.txt` publicly, in the host's access settings, when the pages need a sign-in",
        ),
    ),
    (
        "bot-protection-interference",
        Owner::Hosting(
            "Let agents through the host's bot protection, at least for `llms.txt` and the `.md` files",
        ),
    ),
    (
        "rendering-strategy",
        Owner::Hosting(
            "Render each page's content into its HTML when the site is built, not only in the browser: the site framework's settings",
        ),
    ),
    (
        "content-start-position",
        Owner::Hosting(
            "Put each page's content earlier in its HTML than the navigation, or in `<main>`: the site's page template",
        ),
    ),
    (
        "page-size-html",
        Owner::Hosting(
            "Trim what the site's page template adds around the content, such as navigation repeated on every page",
        ),
    ),
    (
        "page-size-transfer",
        Owner::Hosting(
            "Compress responses on the host (gzip or Brotli), and trim what the page template adds",
        ),
    ),
    ("page-size-markdown", Owner::Pages(diagnostics::PAGE_SIZE)),
    (
        "single-fetch-completeness",
        Owner::Pages(diagnostics::PAGE_SIZE),
    ),
    (
        "embedded-data-serialization",
        Owner::Pages(diagnostics::PAGE_SIZE),
    ),
    (
        "tabbed-content-serialization",
        Owner::Pages(diagnostics::PAGE_SIZE),
    ),
];

/// Who can make the check with this id pass.
pub fn owner(id: &str) -> Owner {
    CHECKS
        .iter()
        .find(|(known, _)| *known == id)
        .map_or(Owner::Hosting(""), |(_, owner)| *owner)
}

/// How one check came out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// It passed.
    Pass,
    /// It passed with a warning.
    Warn,
    /// It failed.
    Fail,
    /// It wasn't run: a check it depends on didn't pass, or there was
    /// nothing for it to look at.
    Skip,
    /// It couldn't be run.
    Error,
}

impl Status {
    /// `pass`, `warn`, `fail`, `skip`, or `error`.
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Warn => "warn",
            Status::Fail => "fail",
            Status::Skip => "skip",
            Status::Error => "error",
        }
    }

    /// Whether it's a finding: a failure, or a warning.
    pub fn is_finding(self) -> bool {
        matches!(self, Status::Warn | Status::Fail)
    }
}

/// One check's result, as the checker gives it.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CheckResult {
    /// The check's id, such as `llms-txt-exists`.
    pub id: String,
    /// Its category, such as `content-discoverability`.
    #[serde(default)]
    pub category: String,
    /// How it came out.
    pub status: Status,
    /// What the checker said.
    #[serde(default)]
    pub message: String,
}

/// What to give the checker.
#[derive(Clone, Debug)]
pub struct Request {
    /// The command, [`COMMAND`].
    pub command: String,
    /// The project root, where it runs.
    pub root: PathBuf,
    /// The site's address.
    pub site: String,
    /// How long it may take.
    pub timeout: Duration,
}

/// What checks a site: the program, or a stand-in in tests.
pub trait SiteChecker: Send + Sync {
    /// Each check's result, in the checker's order.
    ///
    /// # Errors
    ///
    /// The checker couldn't be run, failed, or ran out of time.
    fn check(&self, request: &Request) -> Result<Vec<CheckResult>, ToolError>;
}

/// A site's checks, and the diagnostics for those that didn't pass.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checked {
    /// Each check's result, in the checker's order.
    pub results: Vec<CheckResult>,
    /// A diagnostic for each check that failed or warned and that Ascribe
    /// or the hosting owns, before `[checks]` levels, located at the start
    /// of `ascribe.toml`.
    pub diagnostics: Vec<Diagnostic>,
}

/// Checks the site at `site` with `checker`.
///
/// # Errors
///
/// The checker couldn't be run, failed, or ran out of time.
pub fn check(
    project: &Project,
    checker: &dyn SiteChecker,
    site: &str,
    timeout: Duration,
) -> Result<Checked, ToolError> {
    let checks = &project.model().checks;
    let off = |slug| checks.level(slug) == Some(CheckLevel::Off);
    let results = checker.check(&Request {
        command: COMMAND.to_owned(),
        root: project.root().to_owned(),
        site: site.to_owned(),
        timeout,
    })?;
    let at = Location::new(FileId::new(0), Span::new(0, 0));
    let diagnostics = results
        .iter()
        .filter(|r| r.status.is_finding())
        .filter_map(|r| {
            let result = if r.status == Status::Warn {
                format!("{} (a warning)", r.message.trim_end_matches('.'))
            } else {
                r.message.trim_end_matches('.').to_owned()
            };
            let issue = match owner(&r.id) {
                Owner::Ascribe if !off(diagnostics::DELIVERY_OUTPUT) => {
                    Issue::new(diagnostics::DELIVERY_OUTPUT, at)
                        .with_arg("check", r.id.clone())
                        .with_arg("result", result)
                        .with_arg("issues", ISSUES)
                }
                Owner::Hosting(setting) if !off(diagnostics::DELIVERY_HOSTING) => {
                    let setting = if setting.is_empty() {
                        "See the check at https://afdocs.dev/checks/ for what to set on the host"
                            .to_owned()
                    } else {
                        setting.to_owned()
                    };
                    Issue::new(diagnostics::DELIVERY_HOSTING, at)
                        .with_arg("check", r.id.clone())
                        .with_arg("result", result)
                        .with_arg("setting", format!("{setting}."))
                }
                _ => return None,
            };
            Some(Diagnostic::from_issue(&issue))
        })
        .collect();
    Ok(Checked {
        results,
        diagnostics,
    })
}

/// Runs `afdocs check <site> --format json --quiet`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Afdocs;

impl SiteChecker for Afdocs {
    fn check(&self, request: &Request) -> Result<Vec<CheckResult>, ToolError> {
        let program = tool::program(&request.command, &request.root);
        let args: Vec<String> = ["check", &request.site, "--format", "json", "--quiet"]
            .iter()
            .map(|a| (*a).to_owned())
            .collect();
        let output = tool::run(
            &request.command,
            &program,
            &args,
            &request.root,
            b"",
            request.timeout,
        )?;
        read_afdocs(&request.command, &output)
    }
}

/// The checker's JSON: its results, each with its id and status.
#[derive(Deserialize)]
struct Report {
    results: Vec<CheckResult>,
}

/// The results in what the checker wrote. It exits with 1 when a check
/// fails, which isn't a failure to run: whether it wrote its JSON tells.
fn read_afdocs(command: &str, output: &tool::Output) -> Result<Vec<CheckResult>, ToolError> {
    serde_json::from_slice::<Report>(&output.stdout)
        .map(|r| r.results)
        .map_err(|_| ToolError::Failed {
            command: command.to_owned(),
            reason: tool::said(output),
        })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// The version is written where the checker is installed too: the Astro
    /// example's end-to-end job, and the weekly report on the docs.
    #[test]
    fn the_checker_s_version_is_the_one_installed() {
        let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let read = |path: &str| std::fs::read_to_string(repo.join(path)).unwrap();
        assert!(
            read("examples/astro-site/package.json")
                .contains(&format!("\"afdocs\": \"{VERSION}\"")),
            "examples/astro-site/package.json pins another afdocs than {VERSION}"
        );
        assert!(
            read(".github/workflows/report.yml").contains(&format!("AFDOCS: \"{VERSION}\"")),
            ".github/workflows/report.yml installs another afdocs than {VERSION}"
        );
    }

    #[test]
    fn reads_what_the_checker_writes() {
        let json = br#"{"url": "https://docs.example.com", "results": [
            {"id": "llms-txt-exists", "category": "content-discoverability", "status": "pass", "message": "llms.txt found", "details": {}},
            {"id": "content-negotiation", "category": "markdown-availability", "status": "fail", "message": "Server ignores Accept: text/markdown"},
            {"id": "llms-txt-valid", "category": "content-discoverability", "status": "skip", "message": "Skipped", "dependsOn": ["llms-txt-exists"]}
        ], "summary": {"total": 3}}"#;
        let output = tool::Output {
            stdout: json.to_vec(),
            stderr: Vec::new(),
        };
        let results = read_afdocs("afdocs", &output).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[1].id, "content-negotiation");
        assert_eq!(results[1].status, Status::Fail);
        assert_eq!(results[2].status, Status::Skip);

        let error = read_afdocs(
            "afdocs",
            &tool::Output {
                stdout: Vec::new(),
                stderr: b"Invalid URL\n".to_vec(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "`afdocs` failed: Invalid URL");
    }

    #[test]
    fn every_known_check_once() {
        let mut ids: Vec<&str> = CHECKS.iter().map(|(id, _)| *id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "a check is listed twice");
        assert_eq!(owner("content-negotiation").as_str(), "hosting");
        assert_eq!(owner("llms-txt-exists"), Owner::Ascribe);
        assert_eq!(owner("a-later-check"), Owner::Hosting(""));
    }
}
