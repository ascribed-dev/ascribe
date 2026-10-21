//! What `ascribe report` remembers of its sections from one run to the
//! next, so each can say what's new since the last report and what's gone
//! (decision #787). `links` keeps its own, [`crate::links::History`].
//!
//! - **The history is one file,** `.ascribe/history/report.json`: the last
//!   run's day, the builds it checked, and the `problems` section's
//!   findings by [identity](crate::identity), sorted, so a diff of it reads
//!   as a list of findings. It's written whole through a rename after each
//!   run that reports `problems`.
//! - **A first run says nothing about what's new,** and neither does one
//!   on other builds than the history's, compared as a set: `--build`
//!   changes what's found. `--no-history` doesn't open the history at all,
//!   so it neither reads nor writes it.
//! - **A reader trusts nothing:** a file that doesn't parse or has another
//!   version is no history. Deleting it costs one report that can't say
//!   what changed.
//! - **Two runs that finish together** keep the one that writes last: the
//!   file is the last run, not a log.
//!
//! The file isn't the project's sources, so it's read outside
//! `ascribe_resolve::FileSystem`. Only the report's code opens it
//! (`tests/all/link_cache.rs` holds that).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ascribe_core::Date;
use serde::{Deserialize, Serialize};

use crate::identity::Identity;
use crate::links::cache::{clean_beside, write_whole};

/// The history's file, relative to the project's folder.
pub const FILE: &str = ".ascribe/history/report.json";

/// The version of the file's format.
const VERSION: u32 = 1;

/// What `ascribe report` remembers from its last run, as on one day.
#[derive(Clone, Debug)]
pub struct ReportHistory {
    /// The folder `.ascribe/` is in: the project's folder.
    root: PathBuf,
    /// The day the run is on.
    today: Date,
    /// The builds the run checks, sorted.
    builds: Vec<String>,
    /// What the file held when it was opened.
    read: Option<Saved>,
}

/// The file, read.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Saved {
    /// The day of the run that wrote it.
    run: Date,
    /// The builds it checked, sorted.
    builds: Vec<String>,
    /// The findings `problems` reported.
    problems: BTreeSet<Identity>,
}

/// The file, as written.
#[derive(Serialize, Deserialize)]
struct FileJson {
    version: u32,
    run: String,
    builds: Vec<String>,
    problems: BTreeSet<Identity>,
}

impl ReportHistory {
    /// The history of the project under `root` (`root` is the project's
    /// folder, but for a test), as on `today`, for a run that checks
    /// `builds`, in any order.
    pub fn open(root: &Path, today: Date, builds: &[String]) -> ReportHistory {
        let mut builds = builds.to_vec();
        builds.sort();
        builds.dedup();
        let mut history = ReportHistory {
            root: root.to_owned(),
            today,
            builds,
            read: None,
        };
        history.read = history.load();
        history
    }

    /// The day of the run the history is from, when this one has one to go
    /// by.
    pub fn previous_run(&self) -> Option<Date> {
        self.earlier().map(|saved| saved.run)
    }

    /// The findings the previous run's `problems` reported, when this run
    /// has one to go by.
    pub fn problems_before(&self) -> Option<&BTreeSet<Identity>> {
        self.earlier().map(|saved| &saved.problems)
    }

    /// What the history held, unless this run checks other builds.
    fn earlier(&self) -> Option<&Saved> {
        self.read
            .as_ref()
            .filter(|saved| saved.builds == self.builds)
    }

    /// Writes `problems` as the history, after the run. A write that fails
    /// is skipped.
    pub fn save(&self, problems: BTreeSet<Identity>) {
        let file = FileJson {
            version: VERSION,
            run: self.today.to_string(),
            builds: self.builds.clone(),
            problems,
        };
        let Ok(mut text) = serde_json::to_string_pretty(&file) else {
            return;
        };
        text.push('\n');
        let path = self.path();
        let _ = write_whole(&self.root, &path, &text);
        clean_beside(&path, self.today);
    }

    /// The file's path.
    fn path(&self) -> PathBuf {
        FILE.split('/').fold(self.root.clone(), |p, s| p.join(s))
    }

    /// What the file holds, when it's a history of this version.
    fn load(&self) -> Option<Saved> {
        // Outside FileSystem: the history Ascribe writes under `.ascribe/`,
        // not the project's sources.
        let text = std::fs::read_to_string(self.path()).ok()?;
        let file: FileJson = serde_json::from_str(&text).ok()?;
        if file.version != VERSION {
            return None;
        }
        let mut builds = file.builds;
        builds.sort();
        builds.dedup();
        Some(Saved {
            run: Date::parse(&file.run)?,
            builds,
            problems: file.problems,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

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

    #[test]
    fn a_run_reads_what_the_last_one_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let first = ReportHistory::open(dir.path(), day("2026-10-09"), &builds());
        assert_eq!(first.previous_run(), None);
        assert_eq!(first.problems_before(), None);
        first.save(BTreeSet::from([found("b"), found("a")]));
        assert_eq!(
            std::fs::read_to_string(first.path()).unwrap(),
            r#"{
  "version": 1,
  "run": "2026-10-09",
  "builds": [
    "site"
  ],
  "problems": [
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
"#
        );
        let next = ReportHistory::open(dir.path(), day("2026-10-10"), &builds());
        assert_eq!(next.previous_run(), Some(day("2026-10-09")));
        assert_eq!(
            next.problems_before(),
            Some(&BTreeSet::from([found("a"), found("b")]))
        );
    }

    #[test]
    fn other_builds_have_nothing_to_go_by_and_the_same_in_another_order_do() {
        let dir = tempfile::tempdir().unwrap();
        let two = |a: &str, b: &str| vec![a.to_owned(), b.to_owned()];
        ReportHistory::open(dir.path(), day("2026-10-09"), &two("one", "two"))
            .save(BTreeSet::from([found("a")]));
        let other = ReportHistory::open(dir.path(), day("2026-10-10"), &builds());
        assert_eq!(other.previous_run(), None);
        assert_eq!(other.problems_before(), None);
        let swapped = ReportHistory::open(dir.path(), day("2026-10-10"), &two("two", "one"));
        assert_eq!(swapped.previous_run(), Some(day("2026-10-09")));
    }

    #[test]
    fn a_reader_trusts_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let history = ReportHistory::open(dir.path(), day("2026-10-10"), &builds());
        std::fs::create_dir_all(history.path().parent().unwrap()).unwrap();
        for text in [
            "not json",
            r#"{"version":2,"run":"2026-10-09","builds":["site"],"problems":[]}"#,
            r#"{"version":1,"run":"someday","builds":["site"],"problems":[]}"#,
            r#"{"version":1,"run":"2026-10-09","builds":["site"],"problems":[{"check":"x"}]}"#,
        ] {
            std::fs::write(history.path(), text).unwrap();
            let read = ReportHistory::open(dir.path(), day("2026-10-10"), &builds());
            assert_eq!(read.previous_run(), None, "{text}");
        }
    }
}
