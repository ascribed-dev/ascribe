//! What `ascribe report links` remembers of each address from one run to
//! the next, so a link that answered before isn't reported the first day it
//! fails (decision 125).
//!
//! The rule, whole: a link that fails is reported, unless the history shows
//! it answering before. Then it's **unconfirmed** until it has kept failing
//! for `[checks.links] confirm-days`, with no answer in between.
//!
//! - **Holding takes evidence.** A link is held only because the checker
//!   got an answer from that address before. With no history, as on a
//!   first run, every failure is reported.
//! - **A page that's gone isn't held:** a `404` or a `410` is reported at
//!   once. A permanent redirect is an answer, not a failure.
//! - **The history is one file,** `.ascribe/history/links.json`, sorted,
//!   and written whole through a rename after each run. It holds, for each
//!   address the pages link to, the last day it answered, the day it
//!   started failing, and whether the run reported it, which says what's
//!   new in the next report. An address the pages no longer link to is
//!   forgotten.
//! - **Two runs that finish together** lose one's observations. A run reads
//!   the file again just before it writes, and when another run wrote it in
//!   between, keeps for each address whichever record confirms its failure
//!   later. So a lost observation makes a confirmation later, never earlier.
//! - **A reader trusts nothing:** a file that doesn't parse or has another
//!   version is no history, and a date that doesn't parse is no date.
//!   Deleting the file costs one noisier report, and never a missed one.
//!
//! The file isn't the project's sources, so it's read outside
//! `ascribe_resolve::FileSystem`. Only the report's code opens it
//! (`tests/all/link_cache.rs` holds that).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ascribe_core::Date;
use serde::{Deserialize, Serialize};

use super::Answer;
use super::cache::{over_a_day_old, write_whole};

/// The history's file, relative to the project's folder.
pub const FILE: &str = ".ascribe/history/links.json";

/// The version of the file's format.
const VERSION: u32 = 1;

/// What a project's link checks remember from earlier runs, as on one day.
#[derive(Clone, Debug)]
pub struct History {
    /// The folder `.ascribe/` is in: the project's folder.
    root: PathBuf,
    /// The day the run is on.
    today: Date,
    /// `[checks.links] confirm-days`.
    confirm_days: u32,
    /// Whether to report as a first run would (`--no-history`).
    first_run: bool,
    /// What the file held when it was opened.
    read: Option<Saved>,
}

/// What the history knows of an address.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Record {
    /// The last day it answered.
    pub answered: Option<Date>,
    /// The day it started failing, when it fails.
    pub failing_since: Option<Date>,
    /// Whether the run reported it.
    pub reported: bool,
}

/// What a run saw of an address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Seen {
    /// It answered on a day: today, or the day of an answer reused.
    Answered(Date),
    /// It failed today.
    Failed,
}

/// Why a failing link isn't reported yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Held {
    /// The last day it answered.
    pub answered: Date,
    /// The day it started failing.
    pub failing_since: Date,
    /// The day it's reported from, if it's still failing.
    pub reported_from: Date,
}

/// The file, read.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Saved {
    /// The day of the run that wrote it.
    run: Option<Date>,
    addresses: BTreeMap<String, Record>,
}

/// The file, as written.
#[derive(Serialize, Deserialize)]
struct FileJson {
    version: u32,
    run: String,
    addresses: BTreeMap<String, RecordJson>,
}

#[derive(Serialize, Deserialize)]
struct RecordJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    answered: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    failing_since: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    reported: bool,
}

impl History {
    /// The history of the project under `root` (`root` is the project's
    /// folder, but for a test), as on `today`, holding failures for
    /// `confirm_days`. With `first_run`, nothing is held and nothing is
    /// new, as with no history, and the run's observations are still kept.
    pub fn open(root: &Path, today: Date, confirm_days: u32, first_run: bool) -> History {
        let mut history = History {
            root: root.to_owned(),
            today,
            confirm_days,
            first_run,
            read: None,
        };
        history.read = history.load();
        history
    }

    /// The day of the run the history is from, unless this one reports as
    /// a first run.
    pub fn previous_run(&self) -> Option<Date> {
        self.earlier()?.run
    }

    /// The addresses the previous run reported, unless this one reports as
    /// a first run.
    pub fn reported_before(&self) -> BTreeSet<&str> {
        self.earlier()
            .map(|saved| {
                saved
                    .addresses
                    .iter()
                    .filter(|(_, r)| r.reported)
                    .map(|(url, _)| url.as_str())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What the history held, unless this run reports as a first run.
    fn earlier(&self) -> Option<&Saved> {
        self.read.as_ref().filter(|_| !self.first_run)
    }

    /// The day the run is on.
    pub(crate) fn today(&self) -> Date {
        self.today
    }

    /// Why `url`, which failed with `answer` today, isn't reported yet:
    /// when it answered before and hasn't kept failing for `confirm-days`.
    pub(crate) fn held(&self, url: &str, answer: &Answer) -> Option<Held> {
        if self.confirm_days == 0 || !can_be_held(answer) {
            return None;
        }
        let record = self.earlier()?.addresses.get(url)?;
        let answered = record.answered?;
        let failing_since = streak(record).unwrap_or(self.today);
        let reported_from = failing_since.plus_days(self.confirm_days);
        (self.today < reported_from).then_some(Held {
            answered,
            failing_since,
            reported_from,
        })
    }

    /// What the history knows of `url` after this run saw `seen`, before
    /// whether it's reported is known.
    pub(crate) fn observe(&self, url: &str, seen: Seen) -> Record {
        let before = self
            .read
            .as_ref()
            .and_then(|saved| saved.addresses.get(url))
            .copied()
            .unwrap_or_default();
        match seen {
            Seen::Answered(day) => Record {
                answered: before.answered.max(Some(day)),
                failing_since: streak(&before).filter(|since| *since > day),
                reported: false,
            },
            Seen::Failed => Record {
                answered: before.answered,
                failing_since: Some(streak(&before).unwrap_or(self.today)),
                reported: false,
            },
        }
    }

    /// Writes `addresses` as the history, after the run. When another run
    /// wrote the file since this one read it, what each knows is merged, so
    /// that a failure is confirmed no earlier than either would confirm it.
    /// A write that fails is skipped.
    pub fn save(&self, addresses: BTreeMap<String, Record>) {
        let now = self.load();
        let addresses = if now == self.read {
            addresses
        } else {
            merge(addresses, self.read.as_ref(), now.as_ref())
        };
        let file = FileJson {
            version: VERSION,
            run: self.today.to_string(),
            addresses: addresses
                .into_iter()
                .map(|(url, r)| {
                    let json = RecordJson {
                        answered: r.answered.map(|d| d.to_string()),
                        failing_since: r.failing_since.map(|d| d.to_string()),
                        reported: r.reported,
                    };
                    (url, json)
                })
                .collect(),
        };
        let Ok(mut text) = serde_json::to_string_pretty(&file) else {
            return;
        };
        text.push('\n');
        let path = self.path();
        let _ = write_whole(&self.root, &path, &text);
        self.clean();
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
        let day = |d: Option<String>| d.as_deref().and_then(Date::parse);
        Some(Saved {
            run: Date::parse(&file.run),
            addresses: file
                .addresses
                .into_iter()
                .map(|(url, r)| {
                    let record = Record {
                        answered: day(r.answered),
                        failing_since: day(r.failing_since),
                        reported: r.reported,
                    };
                    (url, record)
                })
                .collect(),
        })
    }

    /// Deletes each `tmp-` file over a day old beside the file: a run that
    /// stopped before its rename left it.
    fn clean(&self) {
        let path = self.path();
        let Some(folder) = path.parent() else {
            return;
        };
        // Outside FileSystem: as in `load`.
        let Ok(files) = std::fs::read_dir(folder) else {
            return;
        };
        for file in files.flatten() {
            if file.file_name().to_string_lossy().starts_with("tmp-")
                && over_a_day_old(&file, self.today)
            {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
}

/// Whether a failure can be held: any but a page that's gone, `404` or
/// `410`. A link that moved answered.
fn can_be_held(answer: &Answer) -> bool {
    match answer {
        Answer::Fine | Answer::Moved { .. } => false,
        Answer::Broken { status } => {
            !matches!(status.split_whitespace().next(), Some("404" | "410"))
        }
        Answer::TimedOut { .. } | Answer::Unreachable { .. } => true,
    }
}

/// The day `record`'s address started failing, when it's failing since its
/// last answer. A streak that starts before the last answer isn't one.
fn streak(record: &Record) -> Option<Date> {
    let since = record.failing_since?;
    match record.answered {
        Some(answered) if since < answered => None,
        _ => Some(since),
    }
}

/// `ours`, with what another run wrote since `read`: each address only the
/// other run knows, and for each both changed, the record that confirms a
/// failure later.
fn merge(
    mut ours: BTreeMap<String, Record>,
    read: Option<&Saved>,
    now: Option<&Saved>,
) -> BTreeMap<String, Record> {
    let Some(now) = now else {
        return ours;
    };
    for (url, theirs) in &now.addresses {
        if read.and_then(|r| r.addresses.get(url)) == Some(theirs) {
            continue;
        }
        let merged = match ours.get(url) {
            None => *theirs,
            Some(mine) => later(mine, theirs),
        };
        ours.insert(url.clone(), merged);
    }
    ours
}

/// Of two records of an address, the one that confirms its failure later,
/// with the later of their answers: one that isn't failing never confirms
/// it, and one that never answered confirms it at once.
fn later(a: &Record, b: &Record) -> Record {
    let confirms = |r: &Record| match (streak(r), r.answered) {
        (None, _) => (2, None),
        (Some(_), None) => (0, None),
        (Some(since), Some(_)) => (1, Some(since)),
    };
    let picked = if confirms(b) > confirms(a) { b } else { a };
    Record {
        answered: a.answered.max(b.answered),
        failing_since: picked.failing_since,
        reported: a.reported || b.reported,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn day(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    const URL: &str = "https://api.example.com/status";

    fn unavailable() -> Answer {
        Answer::Broken {
            status: "503 Service Unavailable".to_owned(),
        }
    }

    /// A history under `root` on `today` that has `record` for [`URL`].
    fn with(root: &Path, today: &str, record: Record) -> History {
        History::open(root, day("2026-10-01"), 3, false)
            .save(BTreeMap::from([(URL.to_owned(), record)]));
        History::open(root, day(today), 3, false)
    }

    fn record(answered: Option<&str>, failing_since: Option<&str>) -> Record {
        Record {
            answered: answered.map(day),
            failing_since: failing_since.map(day),
            reported: false,
        }
    }

    /// A run on `today` that sees `seen` of [`URL`]: the record it keeps,
    /// and whether it's held.
    fn run(root: &Path, today: &str, seen: Seen) -> (Record, Option<Held>) {
        let history = History::open(root, day(today), 3, false);
        let held = match seen {
            Seen::Failed => history.held(URL, &unavailable()),
            Seen::Answered(_) => None,
        };
        let mut kept = history.observe(URL, seen);
        kept.reported = matches!(seen, Seen::Failed) && held.is_none();
        history.save(BTreeMap::from([(URL.to_owned(), kept)]));
        (kept, held)
    }

    // One test for each row of the proposal's table.

    #[test]
    fn with_nothing_about_the_address_a_failure_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let (kept, held) = run(dir.path(), "2026-10-10", Seen::Failed);
        assert_eq!(held, None);
        assert_eq!(kept.failing_since, Some(day("2026-10-10")));
        assert_eq!(kept.answered, None);
    }

    #[test]
    fn a_first_failure_after_an_answer_is_held_for_confirm_days() {
        let dir = tempfile::tempdir().unwrap();
        run(dir.path(), "2026-10-08", Seen::Answered(day("2026-10-08")));
        let (kept, held) = run(dir.path(), "2026-10-10", Seen::Failed);
        assert_eq!(
            held,
            Some(Held {
                answered: day("2026-10-08"),
                failing_since: day("2026-10-10"),
                reported_from: day("2026-10-13"),
            })
        );
        assert_eq!(kept, record(Some("2026-10-08"), Some("2026-10-10")));
        // Still held the day before, with the streak's first day kept.
        let (_, held) = run(dir.path(), "2026-10-12", Seen::Failed);
        assert_eq!(held.map(|h| h.failing_since), Some(day("2026-10-10")));
    }

    #[test]
    fn a_failure_that_kept_on_for_confirm_days_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        let history = with(
            dir.path(),
            "2026-10-13",
            record(Some("2026-10-08"), Some("2026-10-10")),
        );
        assert_eq!(history.held(URL, &unavailable()), None);
    }

    #[test]
    fn an_answer_forgets_the_streak() {
        let dir = tempfile::tempdir().unwrap();
        with(
            dir.path(),
            "2026-10-12",
            record(Some("2026-10-08"), Some("2026-10-10")),
        );
        let (kept, _) = run(dir.path(), "2026-10-12", Seen::Answered(day("2026-10-12")));
        assert_eq!(kept, record(Some("2026-10-12"), None));
        // The next failure starts a streak of its own.
        let (_, held) = run(dir.path(), "2026-10-14", Seen::Failed);
        assert_eq!(held.map(|h| h.reported_from), Some(day("2026-10-17")));
    }

    #[test]
    fn a_page_that_s_gone_or_moved_isn_t_held() {
        let dir = tempfile::tempdir().unwrap();
        let history = with(dir.path(), "2026-10-10", record(Some("2026-10-08"), None));
        for status in ["404 Not Found", "410 Gone"] {
            let gone = Answer::Broken {
                status: status.to_owned(),
            };
            assert_eq!(history.held(URL, &gone), None);
        }
        let moved = Answer::Moved {
            to: "https://new.example.com".to_owned(),
            code: 301,
        };
        assert_eq!(history.held(URL, &moved), None);
        for held in [
            Answer::TimedOut {
                said: "Timeout".to_owned(),
            },
            Answer::Unreachable {
                reason: "Network error".to_owned(),
            },
            Answer::Broken {
                status: "429 Too Many Requests".to_owned(),
            },
        ] {
            assert!(history.held(URL, &held).is_some(), "{held:?}");
        }
    }

    #[test]
    fn confirm_days_of_0_or_a_first_run_holds_nothing() {
        let dir = tempfile::tempdir().unwrap();
        with(dir.path(), "2026-10-10", record(Some("2026-10-08"), None));
        let off = History::open(dir.path(), day("2026-10-10"), 0, false);
        assert_eq!(off.held(URL, &unavailable()), None);
        let first = History::open(dir.path(), day("2026-10-10"), 3, true);
        assert_eq!(first.held(URL, &unavailable()), None);
        assert_eq!(first.previous_run(), None);
        // A first run still keeps what it sees, with the answers before it.
        assert_eq!(
            first.observe(URL, Seen::Failed),
            record(Some("2026-10-08"), Some("2026-10-10"))
        );
    }

    #[test]
    fn a_reader_trusts_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let history = History::open(dir.path(), day("2026-10-10"), 3, false);
        std::fs::create_dir_all(history.path().parent().unwrap()).unwrap();
        for text in [
            "not json",
            r#"{"version":2,"run":"2026-10-09","addresses":{}}"#,
        ] {
            std::fs::write(history.path(), text).unwrap();
            assert_eq!(
                History::open(dir.path(), day("2026-10-10"), 3, false).previous_run(),
                None
            );
        }
        std::fs::write(
            history.path(),
            format!(
                r#"{{"version":1,"run":"2026-10-09","addresses":{{"{URL}":{{"answered":"someday","failing_since":"2026-10-09"}}}}}}"#
            ),
        )
        .unwrap();
        let history = History::open(dir.path(), day("2026-10-10"), 3, false);
        assert_eq!(history.previous_run(), Some(day("2026-10-09")));
        // An answer that isn't a date is no evidence.
        assert_eq!(history.held(URL, &unavailable()), None);
    }

    #[test]
    fn the_file_is_sorted_and_says_what_was_reported() {
        let dir = tempfile::tempdir().unwrap();
        let history = History::open(dir.path(), day("2026-10-10"), 3, false);
        history.save(BTreeMap::from([
            (
                "https://vale.sh/docs/install".to_owned(),
                Record {
                    reported: true,
                    ..record(Some("2026-10-10"), None)
                },
            ),
            (
                "https://api.example.com/status".to_owned(),
                record(Some("2026-10-08"), Some("2026-10-10")),
            ),
        ]));
        assert_eq!(
            std::fs::read_to_string(history.path()).unwrap(),
            r#"{
  "version": 1,
  "run": "2026-10-10",
  "addresses": {
    "https://api.example.com/status": {
      "answered": "2026-10-08",
      "failing_since": "2026-10-10"
    },
    "https://vale.sh/docs/install": {
      "answered": "2026-10-10",
      "reported": true
    }
  }
}
"#
        );
        let next = History::open(dir.path(), day("2026-10-11"), 3, false);
        assert_eq!(
            next.reported_before(),
            BTreeSet::from(["https://vale.sh/docs/install"])
        );
    }

    /// Two runs on 12 October read the same history, a link failing since
    /// 10 October: one gets an answer, and one a failure. Run one after the
    /// other, the answer first, the failure would be confirmed on 15
    /// October, the streak starting again on the 12th.
    #[test]
    fn two_runs_finishing_together_confirm_later_never_earlier() {
        for answer_saves_first in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            with(
                dir.path(),
                "2026-10-12",
                record(Some("2026-10-08"), Some("2026-10-10")),
            );
            let answering = History::open(dir.path(), day("2026-10-12"), 3, false);
            let failing = History::open(dir.path(), day("2026-10-12"), 3, false);
            let answered = answering.observe(URL, Seen::Answered(day("2026-10-12")));
            let failed = failing.observe(URL, Seen::Failed);
            let save = |h: &History, r: Record| h.save(BTreeMap::from([(URL.to_owned(), r)]));
            if answer_saves_first {
                save(&answering, answered);
                save(&failing, failed);
            } else {
                save(&failing, failed);
                save(&answering, answered);
            }
            // The failure's streak from the 10th, which would be confirmed
            // on the 13th, isn't kept: the answer is.
            let after = History::open(dir.path(), day("2026-10-13"), 3, false);
            assert_eq!(
                after.held(URL, &unavailable()).map(|h| h.failing_since),
                Some(day("2026-10-13"))
            );
            let kept = after.read.as_ref().unwrap().addresses[URL];
            assert_eq!(
                kept,
                record(Some("2026-10-12"), None),
                "{answer_saves_first}"
            );
        }
    }

    #[test]
    fn an_address_the_other_run_saw_is_kept_and_one_nobody_links_to_is_forgotten() {
        let dir = tempfile::tempdir().unwrap();
        with(dir.path(), "2026-10-12", record(Some("2026-10-08"), None));
        let one = History::open(dir.path(), day("2026-10-12"), 3, false);
        let other = History::open(dir.path(), day("2026-10-12"), 3, false);
        let new = "https://new.example.com";
        other.save(BTreeMap::from([(
            new.to_owned(),
            record(Some("2026-10-12"), None),
        )]));
        one.save(BTreeMap::new());
        let after = History::open(dir.path(), day("2026-10-13"), 3, false);
        let addresses = &after.read.as_ref().unwrap().addresses;
        assert_eq!(addresses.keys().collect::<Vec<_>>(), [new]);
    }
}
