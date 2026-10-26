//! The site's checks reported as problems, from a stand-in checker: what
//! `delivery-output` says of a check whose `details` name the addresses it
//! didn't get.

#![allow(clippy::expect_used)]

use std::fs;
use std::time::Duration;

use ascribe_check::site::{self, CheckResult, Request, SiteChecker, Status};
use ascribe_check::tool::ToolError;
use ascribe_check::{MODEL_FILE, Project};
use serde_json::json;

/// A checker whose one result is `llms-txt-links-resolve`, warning, with
/// `broken` as its same-origin links that didn't come back OK.
struct LinksResolve {
    broken: serde_json::Value,
}

impl SiteChecker for LinksResolve {
    fn check(&self, _: &Request) -> Result<Vec<CheckResult>, ToolError> {
        Ok(vec![CheckResult {
            id: "llms-txt-links-resolve".to_owned(),
            category: "content-discoverability".to_owned(),
            status: Status::Warn,
            message: "48/50 same-origin sampled links resolve (96%); 2 broken; 1 failed to fetch"
                .to_owned(),
            details: json!({
                "sameOrigin": {"broken": self.broken, "fetchErrors": 1},
                "crossOrigin": {"broken": []},
                "broken": self.broken,
            }),
        }])
    }
}

/// The messages of the problems the report finds with `checker`.
fn messages(checker: &dyn SiteChecker) -> Vec<String> {
    let dir = tempfile::tempdir().expect("a temporary directory");
    fs::write(
        dir.path().join(MODEL_FILE),
        "spec = \"0.1\"\n[consumer]\nsite = \"https://docs.example.com\"\n",
    )
    .expect("write the model");
    fs::create_dir(dir.path().join("docs")).expect("create docs");
    let project = Project::load(&dir.path().join(MODEL_FILE)).expect("the project loads");
    site::check(
        &project,
        checker,
        "https://docs.example.com",
        Duration::from_secs(1),
    )
    .expect("the stand-in checks")
    .diagnostics
    .into_iter()
    .map(|d| d.message)
    .collect()
}

#[test]
fn names_each_link_with_its_status_or_why_it_wasn_t_fetched() {
    let checker = LinksResolve {
        broken: json!([
            {"url": "https://docs.example.com/gone.md", "status": 404},
            {"url": "https://docs.example.com/slow.md", "status": 0, "error": "fetch failed"},
        ]),
    };
    assert_eq!(
        messages(&checker),
        [format!(
            "the site passes the delivery spec's `llms-txt-links-resolve` check with a warning, which what Ascribe writes should pass without one: 48/50 same-origin sampled links resolve (96%); 2 broken; 1 failed to fetch. Not reached: `https://docs.example.com/gone.md` (HTTP 404), `https://docs.example.com/slow.md` (couldn't be fetched: fetch failed). Report it at {}",
            site::ISSUES
        )]
    );
}

#[test]
fn links_that_only_failed_to_fetch_aren_t_a_bug_to_report() {
    let checker = LinksResolve {
        broken: json!([
            {"url": "https://docs.example.com/slow.md", "status": 0, "error": "fetch failed"},
        ]),
    };
    let found = messages(&checker);
    assert_eq!(
        found,
        [
            "the delivery spec's `llms-txt-links-resolve` check couldn't fetch every address it tried, so it doesn't say yet whether what Ascribe writes passes: 48/50 same-origin sampled links resolve (96%); 2 broken; 1 failed to fetch. Not reached: `https://docs.example.com/slow.md` (couldn't be fetched: fetch failed). Run the report again"
        ]
    );
    assert!(!found[0].contains(site::ISSUES), "{}", found[0]);
}
