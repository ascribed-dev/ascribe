//! What `ascribe report` remembers of its sections from one run to the
//! next, so each can say what's new since the last report and what's gone
//! (decisions #787 and #788). `links` keeps its own,
//! [`crate::links::History`].
//!
//! - **The history is one file,** `.ascribe/history/report.json`, with a
//!   part for each section it remembers, each with its own run's day:
//!   `problems`, the builds it checked and its findings by
//!   [identity](crate::identity), sorted, so a diff of it reads as a list
//!   of findings; and `agents`, the site it checked and each of the
//!   delivery spec's checks by id, with how it came out. It's written whole
//!   through a rename after each run that reports either; a section the run
//!   didn't report keeps what the file held.
//! - **A first run says nothing about what's new,** and neither does one
//!   on other builds than the history's, compared as a set, for `problems`,
//!   or on another site, for `agents`: either changes what's found.
//!   `--no-history` doesn't open the history at all, so it neither reads
//!   nor writes it.
//! - **A reader trusts nothing:** a file that doesn't parse or has another
//!   version is no history. Deleting it costs one report that can't say
//!   what changed.
//! - **Two runs that finish together** keep the one that writes last: the
//!   file is the last run, not a log.
//!
//! The file isn't the project's sources, so it's read and written through
//! the [`Store`] the caller hands it, not `ascribe_resolve::FileSystem`.
//! Only the report's code opens it (`tests/all/link_cache.rs` holds that).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use ascribe_core::Date;
use serde::{Deserialize, Serialize};

use crate::identity::Identity;
use crate::site::Status;
use crate::store::Store;

/// The history's file, relative to the project's folder.
pub const FILE: &str = ".ascribe/history/report.json";

/// The version of the file's format.
const VERSION: u32 = 2;

/// What `ascribe report` remembers from its last run, as on one day.
#[derive(Clone, Debug)]
pub struct ReportHistory {
    /// Where the file is kept.
    store: Arc<dyn Store>,
    /// The day the run is on.
    today: Date,
    /// The builds the run checks, sorted.
    builds: Vec<String>,
    /// What the file held when it was opened.
    read: FileJson,
}

/// What `agents` found on a site: each check's result, by its id. A
/// check's identity is the site's address and its id.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SiteChecks {
    /// The site's address, as `--site` gives it.
    pub site: String,
    /// How each check came out, by its id.
    pub checks: BTreeMap<String, Status>,
}

/// What a run found, to keep for the next: a section's `None` when the run
/// didn't report it, so the history keeps what it held.
#[derive(Clone, Debug, Default)]
pub struct Found {
    /// The findings `problems` reported.
    pub problems: Option<BTreeSet<Identity>>,
    /// What `agents` found on the site.
    pub agents: Option<SiteChecks>,
}

/// The file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct FileJson {
    version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    problems: Option<ProblemsJson>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agents: Option<AgentsJson>,
}

/// What `problems` reported on a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct ProblemsJson {
    /// The day of the run.
    run: String,
    /// The builds it checked, sorted.
    builds: Vec<String>,
    /// Its findings, sorted.
    findings: BTreeSet<Identity>,
}

/// What `agents` found on a run.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct AgentsJson {
    /// The day of the run.
    run: String,
    /// The site it checked.
    site: String,
    /// Each check, sorted by its id.
    checks: Vec<CheckJson>,
}

/// One of the delivery spec's checks, and how it came out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct CheckJson {
    check: String,
    status: Status,
}

impl ReportHistory {
    /// The history kept in `store`, as on `today`, for a run that checks
    /// `builds`, in any order.
    pub fn open(store: Arc<dyn Store>, today: Date, builds: &[String]) -> ReportHistory {
        let mut builds = builds.to_vec();
        builds.sort();
        builds.dedup();
        let mut history = ReportHistory {
            store,
            today,
            builds,
            read: FileJson::default(),
        };
        history.read = history.load().unwrap_or_default();
        history
    }

    /// The day of the run `problems` is compared with, and the findings it
    /// reported, unless there's none or it checked other builds.
    pub fn problems_before(&self) -> Option<(Date, &BTreeSet<Identity>)> {
        let saved = self.read.problems.as_ref()?;
        let mut builds = saved.builds.clone();
        builds.sort();
        builds.dedup();
        (builds == self.builds).then_some(())?;
        Some((Date::parse(&saved.run)?, &saved.findings))
    }

    /// The day of the run `agents` is compared with, and how each check
    /// came out on it, unless there's none or it checked another site
    /// than `site`.
    pub fn agents_before(&self, site: &str) -> Option<(Date, BTreeMap<&str, Status>)> {
        let saved = self.read.agents.as_ref().filter(|a| a.site == site)?;
        let checks = saved
            .checks
            .iter()
            .map(|c| (c.check.as_str(), c.status))
            .collect();
        Some((Date::parse(&saved.run)?, checks))
    }

    /// Writes what this run `found` as the history, after the run, keeping
    /// what the file held for a section it didn't report. A write that
    /// fails is skipped.
    pub fn save(&self, found: Found) {
        let run = self.today.to_string();
        let file = FileJson {
            version: VERSION,
            problems: match found.problems {
                Some(findings) => Some(ProblemsJson {
                    run: run.clone(),
                    builds: self.builds.clone(),
                    findings,
                }),
                None => self.read.problems.clone(),
            },
            agents: match found.agents {
                Some(found) => Some(AgentsJson {
                    run,
                    site: found.site,
                    checks: found
                        .checks
                        .into_iter()
                        .map(|(check, status)| CheckJson { check, status })
                        .collect(),
                }),
                None => self.read.agents.clone(),
            },
        };
        let Ok(mut text) = serde_json::to_string_pretty(&file) else {
            return;
        };
        text.push('\n');
        let _ = self.store.write(FILE, &text);
        if let Some((folder, _)) = FILE.rsplit_once('/') {
            self.store.tidy(folder, self.today);
        }
    }

    /// What the file holds, when it's a history of this version whose days
    /// are days.
    fn load(&self) -> Option<FileJson> {
        let text = self.store.read(FILE)?;
        let file: FileJson = serde_json::from_str(&text).ok()?;
        let days_parse = file
            .problems
            .iter()
            .map(|p| &p.run)
            .chain(file.agents.iter().map(|a| &a.run))
            .all(|d| Date::parse(d).is_some());
        (file.version == VERSION && days_parse).then_some(file)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::store::MemoryStore;

    fn day(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    fn found(fingerprint: &str) -> Identity {
        Identity {
            check: "link-target-missing".to_owned(),
            file: "docs/index.md".to_owned(),
            fingerprint: fingerprint.to_owned(),
            order: 0,
        }
    }

    const BUILDS: &[&str] = &["site"];

    fn builds() -> Vec<String> {
        BUILDS.iter().map(|b| (*b).to_owned()).collect()
    }

    fn problems(found: BTreeSet<Identity>) -> Found {
        Found {
            problems: Some(found),
            agents: None,
        }
    }

    fn agents(site: &str, checks: &[(&str, Status)]) -> Found {
        Found {
            problems: None,
            agents: Some(SiteChecks {
                site: site.to_owned(),
                checks: checks
                    .iter()
                    .map(|(id, status)| ((*id).to_owned(), *status))
                    .collect(),
            }),
        }
    }

    #[test]
    fn a_run_reads_what_the_last_one_wrote() {
        let store: Arc<dyn Store> = Arc::new(MemoryStore::new());
        let first = ReportHistory::open(store.clone(), day("2026-10-09"), &builds());
        assert_eq!(first.problems_before(), None);
        first.save(problems(BTreeSet::from([found("b"), found("a")])));
        assert_eq!(
            store.read(FILE).unwrap(),
            r#"{
  "version": 2,
  "problems": {
    "run": "2026-10-09",
    "builds": [
      "site"
    ],
    "findings": [
      {
        "check": "link-target-missing",
        "file": "docs/index.md",
        "fingerprint": "a",
        "order": 0
      },
      {
        "check": "link-target-missing",
        "file": "docs/index.md",
        "fingerprint": "b",
        "order": 0
      }
    ]
  }
}
"#
        );
        let next = ReportHistory::open(store.clone(), day("2026-10-10"), &builds());
        assert_eq!(
            next.problems_before(),
            Some((day("2026-10-09"), &BTreeSet::from([found("a"), found("b")])))
        );
    }

    #[test]
    fn other_builds_have_nothing_to_go_by_and_the_same_in_another_order_do() {
        let store: Arc<dyn Store> = Arc::new(MemoryStore::new());
        let two = |a: &str, b: &str| vec![a.to_owned(), b.to_owned()];
        ReportHistory::open(store.clone(), day("2026-10-09"), &two("one", "two"))
            .save(problems(BTreeSet::from([found("a")])));
        let other = ReportHistory::open(store.clone(), day("2026-10-10"), &builds());
        assert_eq!(other.problems_before(), None);
        let swapped = ReportHistory::open(store.clone(), day("2026-10-10"), &two("two", "one"));
        assert_eq!(
            swapped.problems_before().map(|(run, _)| run),
            Some(day("2026-10-09"))
        );
    }

    #[test]
    fn each_section_keeps_its_own_run_and_one_not_reported_keeps_what_it_held() {
        let store: Arc<dyn Store> = Arc::new(MemoryStore::new());
        let site = "https://docs.example.com";
        ReportHistory::open(store.clone(), day("2026-10-09"), &builds())
            .save(problems(BTreeSet::from([found("a")])));
        ReportHistory::open(store.clone(), day("2026-10-10"), &builds()).save(agents(
            site,
            &[
                ("llms-txt-exists", Status::Pass),
                ("content-negotiation", Status::Fail),
            ],
        ));
        let next = ReportHistory::open(store.clone(), day("2026-10-11"), &builds());
        assert_eq!(
            next.problems_before(),
            Some((day("2026-10-09"), &BTreeSet::from([found("a")])))
        );
        assert_eq!(
            next.agents_before(site),
            Some((
                day("2026-10-10"),
                BTreeMap::from([
                    ("content-negotiation", Status::Fail),
                    ("llms-txt-exists", Status::Pass)
                ])
            ))
        );
        // Another site has nothing to go by.
        assert_eq!(next.agents_before("http://localhost:4321"), None);
        let text = store.read(FILE).unwrap();
        assert!(
            text.contains(
                r#""agents": {
    "run": "2026-10-10",
    "site": "https://docs.example.com",
    "checks": [
      {
        "check": "content-negotiation",
        "status": "fail"
      },"#
            ),
            "{text}"
        );
    }

    #[test]
    fn a_reader_trusts_nothing() {
        let store: Arc<dyn Store> = Arc::new(MemoryStore::new());
        for text in [
            "not json",
            r#"{"version":1,"run":"2026-10-09","builds":["site"],"problems":[]}"#,
            r#"{"version":2,"problems":{"run":"someday","builds":["site"],"findings":[]}}"#,
            r#"{"version":2,"problems":{"run":"2026-10-09","builds":["site"],"findings":[{"check":"x"}]}}"#,
            r#"{"version":2,"agents":{"run":"2026-10-09","site":"s","checks":[{"check":"x","status":"maybe"}]}}"#,
        ] {
            store.write(FILE, text).unwrap();
            let read = ReportHistory::open(store.clone(), day("2026-10-10"), &builds());
            assert_eq!(read.problems_before(), None, "{text}");
            assert_eq!(read.agents_before("s"), None, "{text}");
        }
    }
}
