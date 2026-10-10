//! The adapter for the checks only `ascribe report` runs: external links
//! (tag `links`) and the delivery spec's checks on a built site (tag
//! `site`).
//!
//! Its diagnostics are the case's file-level diagnostics with those the
//! report's `links` or `agents` section gives, through
//! `ascribe_query::report`, the call `ascribe report` makes. Neither lychee
//! nor afdocs is run, and nothing reaches the network: stand-ins answer from
//! the address alone, so a case shows where a finding lands and what's
//! acknowledged, on every platform.
//!
//! The link checker's stand-in answers by host: `gone.test` and its subdomains are a 404,
//! `moved.test` redirects permanently to the same path on `new.test`,
//! `landing.test` sends its root to `/docs/intro`,
//! `slow.test` doesn't answer in time, `down.test` can't be reached, and
//! every other host answers. A `[checks.links] command` of `not-installed`
//! is a checker that can't be run.
//!
//! The site checker's stand-in checks `[consumer] site`: on a host
//! `failing.test`, three checks fail, one each that Ascribe, the hosting,
//! and the pages own (`llms-txt-exists`, `content-negotiation`,
//! `page-size-markdown`); every other check passes.

use std::collections::BTreeMap;
use std::time::Duration;

use ascribe_check::Project;
use ascribe_check::links::{self, Answer, LinkChecker};
use ascribe_check::site::{self, CheckResult, SiteChecker, Status};
use ascribe_check::tool::ToolError;
use ascribe_conformance::{
    AdapterError, AdapterResult, BuildResult, Case, ConformanceAdapter, Diagnostic,
};
use ascribe_query::report::{Options, Ran, Section, Tools};

/// Handles the cases that expect the report's findings.
pub struct ReportAdapter;

impl ConformanceAdapter for ReportAdapter {
    fn name(&self) -> &str {
        "report"
    }

    fn handles_tag(&self, tag: &str) -> bool {
        tag == "links" || tag == "site"
    }

    fn diagnostics(&self, case: &Case) -> AdapterResult<Vec<Diagnostic>> {
        let (project, found) = found(case)?;
        let mut all = ascribe_check::check_files(&project);
        all.extend(found.into_iter().filter(|d| !is_page_level(d)));
        super::to_conformance(&project, all).map(Some)
    }

    /// The build as `ascribe check` sees it, with the acknowledgements the
    /// report finds unused, which are page-level.
    fn build(&self, case: &Case, build: &str) -> AdapterResult<BuildResult> {
        let mut out = super::resolve::resolve_build(case, build)?;
        let (project, found) = found(case)?;
        let unused: Vec<_> = found.into_iter().filter(is_page_level).collect();
        out.diagnostics
            .extend(super::to_conformance(&project, unused)?);
        Ok(Some(out))
    }
}

fn is_page_level(d: &ascribe_check::Diagnostic) -> bool {
    d.slug == ascribe_core::diagnostics::INTENDED_UNUSED
}

/// The case's project, and what the report's `links` or `agents` section
/// finds in it.
fn found(case: &Case) -> Result<(Project, Vec<ascribe_check::Diagnostic>), AdapterError> {
    let project = super::check::project(case)?;
    let section = if case.expect.tags.iter().any(|t| t == "site") {
        Section::Agents
    } else {
        Section::Links
    };
    let tools = Tools {
        links: &StandInLinks,
        site: &StandInSite,
        links_timeout: Duration::from_secs(5),
        site_timeout: Duration::from_secs(5),
    };
    let site = project.model().consumer.site.clone();
    let options = Options {
        sections: &[section],
        builds: &[],
        site: site.as_deref(),
        tools: &tools,
    };
    let report =
        ascribe_query::report(&project, &options).map_err(|e| AdapterError(e.to_string()))?;
    let found = match (report.links, report.agents) {
        (Some(Ran::Done(links)), _) => links.diagnostics,
        (_, Some(Ran::Done(agents))) => agents.diagnostics,
        (Some(Ran::NotRun(why)), _) | (_, Some(Ran::NotRun(why))) => {
            return Err(AdapterError(format!(
                "the section didn't run: {}",
                why.reason
            )));
        }
        (None, None) => Vec::new(),
    };
    Ok((project, found))
}

/// A link checker that answers from the host alone.
struct StandInLinks;

impl LinkChecker for StandInLinks {
    fn check(&self, request: &links::Request) -> Result<BTreeMap<String, Answer>, ToolError> {
        if request.command == "not-installed" {
            return Err(ToolError::NotRun {
                command: request.command.clone(),
                reason: "it isn't installed, or isn't on the PATH".to_owned(),
            });
        }
        Ok(request
            .urls
            .iter()
            .map(|url| {
                let host = links::host(url).unwrap_or_default();
                let answer = match host {
                    h if h == "gone.test" || h.ends_with(".gone.test") => Answer::Broken {
                        status: "404 Not Found".to_owned(),
                    },
                    "moved.test" => Answer::Moved {
                        to: url.replacen("moved.test", "new.test", 1),
                        code: 301,
                    },
                    "landing.test" if url.trim_end_matches('/').ends_with("landing.test") => {
                        Answer::Moved {
                            to: "https://landing.test/docs/intro".to_owned(),
                            code: 301,
                        }
                    }
                    "slow.test" => Answer::TimedOut {
                        said: "Timeout".to_owned(),
                    },
                    "down.test" => Answer::Unreachable {
                        reason: "Connection failed".to_owned(),
                    },
                    _ => Answer::Fine,
                };
                (url.clone(), answer)
            })
            .collect())
    }
}

/// A site checker that fails three checks on `failing.test`.
struct StandInSite;

impl SiteChecker for StandInSite {
    fn check(&self, request: &site::Request) -> Result<Vec<CheckResult>, ToolError> {
        let failing = links::host(&request.site) == Some("failing.test");
        Ok(site::CHECKS
            .iter()
            .map(|(id, _)| {
                let fails = failing
                    && matches!(
                        *id,
                        "llms-txt-exists" | "content-negotiation" | "page-size-markdown"
                    );
                CheckResult {
                    id: (*id).to_owned(),
                    category: "test".to_owned(),
                    status: if fails { Status::Fail } else { Status::Pass },
                    message: if fails {
                        format!("{id} failed")
                    } else {
                        "passed".to_owned()
                    },
                }
            })
            .collect())
    }
}
