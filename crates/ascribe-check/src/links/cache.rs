//! The answers `ascribe report links` reuses: an address that answered a
//! few days ago, with nothing to report, isn't asked about again
//! (decision 121).
//!
//! The store is `.ascribe/cache/links/`, one file an address, named by a
//! hash of its key: the address exactly as the checker is given it, and a
//! hash of the project's `lychee.toml` (or of nothing, without one). An
//! entry says the address answered on a day. A link that moved, failed, or
//! timed out is never an entry, so every finding in a report was asked
//! about in that run. Deleting the folder costs time and nothing else.
//!
//! - **Days, not seconds.** The caller gives the day; the library doesn't
//!   read the clock.
//! - **Each address's own limit** falls between half of `[checks.links]
//!   reuse-days` and all of it, fixed by its hash, so addresses first
//!   checked together don't all come due on one day.
//! - **Written whole or not at all,** through the [`Store`] the caller
//!   hands it. A write that fails is skipped, and the next run asks again.
//! - **A reader trusts nothing:** a file that doesn't parse, has another
//!   version, or holds another address or another `lychee.toml`'s hash is
//!   treated as absent.
//! - **Cleanup needs no command:** a run deletes each entry too old to
//!   reuse, and what a write that stopped left over a day ago.
//!
//! These files aren't the project's sources, so they're read through the
//! store, not `ascribe_resolve::FileSystem`; `lychee.toml` is a project
//! file, and is read through it. Only the report's code opens the store
//! (`tests/all/link_cache.rs` holds that).

use std::sync::Arc;

use ascribe_core::{Date, RelPath};
use serde::{Deserialize, Serialize};

use crate::Project;
use crate::store::Store;

/// The store's folder, relative to the project's folder.
pub const FOLDER: &str = ".ascribe/cache/links";

/// The checker's own configuration, whose hash is part of each key.
const CONFIG: &str = "lychee.toml";

/// The version of an entry's format.
const VERSION: u32 = 1;

/// How many hexadecimal digits of the key's hash name its file.
const NAME_DIGITS: usize = 16;

/// The answers a project's link checks reuse, as on one day.
#[derive(Clone, Debug)]
pub struct Cache {
    /// Where the entries are kept.
    store: Arc<dyn Store>,
    /// The hash of `lychee.toml`, or of nothing.
    config: String,
    /// The day the run is on.
    today: Date,
    /// `[checks.links] reuse-days`.
    reuse_days: u32,
    /// Whether to ask about every address and replace the entries
    /// (`--recheck`).
    recheck: bool,
}

/// An entry: an address that answered, with nothing to report, on a day.
#[derive(Serialize, Deserialize)]
struct Entry {
    version: u32,
    url: String,
    /// The hash of `lychee.toml` when it was asked.
    config: String,
    day: String,
}

impl Cache {
    /// The store of `project`'s answers, kept in `store` under
    /// `.ascribe/cache/`, as on `today`. With `recheck`, nothing is reused,
    /// and what's asked replaces the entries.
    pub fn open(project: &Project, store: Arc<dyn Store>, today: Date, recheck: bool) -> Cache {
        let config = RelPath::parse(CONFIG)
            .ok()
            .and_then(|path| project.file_system().read_file(&path).ok())
            .unwrap_or_default();
        Cache {
            store,
            config: ascribe_model::file_hash(&config),
            today,
            reuse_days: project.model().checks.links.reuse_days,
            recheck,
        }
    }

    /// The day `url` last answered, when that answer can be reused today.
    pub(crate) fn reusable(&self, url: &str) -> Option<Date> {
        if self.recheck || self.reuse_days == 0 {
            return None;
        }
        let key = self.key(url);
        let day = self.read(&self.path(&key), Some(url))?;
        self.fresh(&key, day).then_some(day)
    }

    /// Records that `url` answered today with nothing to report. A write
    /// that fails is skipped.
    pub(crate) fn remember(&self, url: &str) {
        if self.reuse_days == 0 {
            return;
        }
        let entry = Entry {
            version: VERSION,
            url: url.to_owned(),
            config: self.config.clone(),
            day: self.today.to_string(),
        };
        let Ok(mut text) = serde_json::to_string(&entry) else {
            return;
        };
        text.push('\n');
        let _ = self.store.write(&self.path(&self.key(url)), &text);
    }

    /// Forgets `url`'s answer: it was asked about, and has something to
    /// report.
    pub(crate) fn forget(&self, url: &str) {
        self.store.remove(&self.path(&self.key(url)));
    }

    /// Deletes each entry too old to reuse, each file that isn't an entry,
    /// and what a write that stopped left over a day ago.
    pub(crate) fn clean(&self) {
        for folder in self.store.list(FOLDER).folders {
            let folder = format!("{FOLDER}/{folder}");
            self.store.tidy(&folder, self.today);
            for name in self.store.list(&folder).files {
                let Some(key) = name.strip_suffix(".json") else {
                    continue;
                };
                let path = format!("{folder}/{name}");
                match self.read_any(&path) {
                    Read::Entry(day) if !self.too_old(key, day) => {}
                    Read::OtherVersion => {}
                    Read::Entry(_) | Read::Unreadable => self.store.remove(&path),
                }
            }
            // A folder left empty goes too; one that isn't empty stays.
            self.store.remove_folder(&folder);
        }
    }

    /// The hash of `url`'s key, as lowercase hexadecimal digits.
    fn key(&self, url: &str) -> String {
        let hash = ascribe_model::file_hash(format!("{url}\n{}", self.config).as_bytes());
        let digits = hash.strip_prefix("sha256:").unwrap_or(&hash);
        digits.get(..NAME_DIGITS).unwrap_or(digits).to_owned()
    }

    /// The file of the entry with `key`: in a folder named by its first two
    /// digits, so no folder holds too many.
    fn path(&self, key: &str) -> String {
        let folder = key.get(..2).unwrap_or(key);
        format!("{FOLDER}/{folder}/{key}.json")
    }

    /// The days an answer with `key` is reused: from half of `reuse-days`,
    /// rounded up, through all of it, fixed by the key.
    fn limit(&self, key: &str) -> i64 {
        let days = i64::from(self.reuse_days);
        let least = (days + 1) / 2;
        let spread = u64::try_from(days - least + 1).unwrap_or(1).max(1);
        let hash = u64::from_str_radix(key, 16).unwrap_or_default();
        least + i64::try_from(hash % spread).unwrap_or_default()
    }

    /// Whether an answer with `key` from `day` can be reused today. One from
    /// a day after today isn't, and is asked about again.
    fn fresh(&self, key: &str, day: Date) -> bool {
        let age = self.today.days_since(day);
        (0..self.limit(key)).contains(&age)
    }

    /// Whether an entry with `key` from `day` is past its limit, so cleanup
    /// deletes it.
    fn too_old(&self, key: &str, day: Date) -> bool {
        self.today.days_since(day) >= self.limit(key)
    }

    /// The day of the entry at `path`, when it's an entry of this version,
    /// for `url` when one is given, asked with this `lychee.toml`.
    fn read(&self, path: &str, url: Option<&str>) -> Option<Date> {
        let text = self.store.read(path)?;
        let entry: Entry = serde_json::from_str(&text).ok()?;
        let ours = entry.version == VERSION
            && entry.config == self.config
            && url.is_none_or(|url| entry.url == url);
        ours.then(|| Date::parse(&entry.day)).flatten()
    }

    /// What the file at `path` is, for cleanup.
    fn read_any(&self, path: &str) -> Read {
        let Some(text) = self.store.read(path) else {
            return Read::Unreadable;
        };
        let Ok(entry) = serde_json::from_str::<Entry>(&text) else {
            return Read::Unreadable;
        };
        if entry.version != VERSION {
            return Read::OtherVersion;
        }
        match Date::parse(&entry.day) {
            Some(day) => Read::Entry(day),
            None => Read::Unreadable,
        }
    }
}

/// A file in the store, as cleanup sees it.
enum Read {
    /// An entry of this version, from a day.
    Entry(Date),
    /// An entry another version of Ascribe wrote, which it may still use.
    OtherVersion,
    /// Not an entry.
    Unreadable,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::store::MemoryStore;

    fn day(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    fn cache(store: &Arc<MemoryStore>, today: &str, reuse_days: u32) -> Cache {
        Cache {
            store: store.clone(),
            config: ascribe_model::file_hash(b""),
            today: day(today),
            reuse_days,
            recheck: false,
        }
    }

    #[test]
    fn an_answer_is_reused_until_its_own_limit() {
        let store = Arc::new(MemoryStore::new());
        let url = "https://example.com/a";
        cache(&store, "2026-10-01", 14).remember(url);
        let key = cache(&store, "2026-10-01", 14).key(url);
        let limit = cache(&store, "2026-10-01", 14).limit(&key);
        assert!((7..=14).contains(&limit), "{limit}");
        let last = 1 + limit - 1;
        assert_eq!(
            cache(&store, &format!("2026-10-{last:02}"), 14).reusable(url),
            Some(day("2026-10-01"))
        );
        assert_eq!(
            cache(&store, &format!("2026-10-{:02}", last + 1), 14).reusable(url),
            None
        );
        // Not before it was written, nor with reuse off, nor rechecking.
        assert_eq!(cache(&store, "2026-09-30", 14).reusable(url), None);
        assert_eq!(cache(&store, "2026-10-01", 0).reusable(url), None);
        let rechecking = Cache {
            recheck: true,
            ..cache(&store, "2026-10-01", 14)
        };
        assert_eq!(rechecking.reusable(url), None);
    }

    #[test]
    fn limits_spread_over_the_second_half() {
        let store = Arc::new(MemoryStore::new());
        let c = cache(&store, "2026-10-01", 14);
        let limits: std::collections::BTreeSet<i64> = (0..200)
            .map(|i| c.limit(&c.key(&format!("https://example.com/{i}"))))
            .collect();
        assert_eq!(limits.first(), Some(&7));
        assert_eq!(limits.last(), Some(&14));
        assert_eq!(cache(&store, "2026-10-01", 1).limit(&c.key("x")), 1);
    }

    #[test]
    fn a_reader_trusts_nothing() {
        let store = Arc::new(MemoryStore::new());
        let c = cache(&store, "2026-10-01", 14);
        let url = "https://example.com/a";
        let path = c.path(&c.key(url));
        let entry = |version: u32, url: &str, config: &str| {
            format!(
                r#"{{"version":{version},"url":"{url}","config":"{config}","day":"2026-10-01"}}"#
            )
        };
        for text in [
            "not json".to_owned(),
            entry(2, url, &c.config),
            entry(1, "https://example.com/b", &c.config),
            entry(1, url, "sha256:other"),
        ] {
            store.write(&path, &text).unwrap();
            assert_eq!(c.reusable(url), None);
        }
        store.write(&path, &entry(1, url, &c.config)).unwrap();
        assert_eq!(c.reusable(url), Some(day("2026-10-01")));
    }

    #[test]
    fn a_changed_lychee_toml_starts_every_address_fresh() {
        let store = Arc::new(MemoryStore::new());
        let url = "https://example.com/a";
        cache(&store, "2026-10-01", 14).remember(url);
        let changed = Cache {
            config: ascribe_model::file_hash(b"timeout = 5\n"),
            ..cache(&store, "2026-10-01", 14)
        };
        assert_eq!(changed.reusable(url), None);
    }

    #[test]
    fn cleanup_deletes_what_can_t_be_reused() {
        let store = Arc::new(MemoryStore::new());
        let old = "https://example.com/old";
        let new = "https://example.com/new";
        cache(&store, "2026-09-01", 14).remember(old);
        cache(&store, "2026-10-01", 14).remember(new);
        let c = cache(&store, "2026-10-02", 14);
        let new_path = c.path(&c.key(new));
        let folder = new_path.rsplit_once('/').unwrap().0;
        let not_an_entry = format!("{folder}/0000000000000000.json");
        store.write(&not_an_entry, "{").unwrap();
        c.clean();
        assert_eq!(store.read(&c.path(&c.key(old))), None);
        assert_eq!(store.read(&not_an_entry), None);
        assert!(store.read(&new_path).is_some());
        cache(&store, "2099-01-01", 14).clean();
        assert_eq!(store.paths(), Vec::<String>::new());
    }

    #[test]
    fn names_are_lowercase_hex() {
        let store = Arc::new(MemoryStore::new());
        let c = cache(&store, "2026-10-01", 14);
        let key = c.key("https://Example.com/A");
        assert_eq!(key.len(), NAME_DIGITS);
        assert!(key.chars().all(|ch| matches!(ch, '0'..='9' | 'a'..='f')));
        assert_eq!(
            c.path(&key),
            format!(".ascribe/cache/links/{}/{key}.json", &key[..2])
        );
    }
}
