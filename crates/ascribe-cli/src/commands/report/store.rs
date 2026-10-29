//! The files `ascribe report` keeps between runs, on the disk under the
//! project's folder: the [`Store`] the CLI hands the library, which doesn't
//! write them itself.
//!
//! - **Written whole or not at all,** to a `tmp-` file beside it and renamed
//!   into place. A folder on the way that's a link is refused, not written
//!   through: a committed one could lead anywhere.
//! - **A `tmp-` file over a day old** is what a run that stopped before its
//!   rename left, and [`Store::tidy`] deletes it.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ascribe_core::Date;
use ascribe_report::store::{Listing, Store};

/// The files under a project's folder.
#[derive(Debug)]
pub struct DiskStore {
    root: PathBuf,
}

impl DiskStore {
    /// The store under `root`, the project's folder.
    pub fn new(root: &Path) -> DiskStore {
        DiskStore {
            root: root.to_owned(),
        }
    }

    fn at(&self, path: &str) -> PathBuf {
        path.split('/').fold(self.root.clone(), |p, s| p.join(s))
    }
}

impl Store for DiskStore {
    fn read(&self, path: &str) -> Option<String> {
        // Outside FileSystem: a file Ascribe writes under `.ascribe/`, not
        // the project's sources.
        std::fs::read_to_string(self.at(path)).ok()
    }

    fn write(&self, path: &str, text: &str) -> std::io::Result<()> {
        let path = self.at(path);
        let folder = path.parent().unwrap_or(&path);
        if let Ok(rel) = folder.strip_prefix(&self.root)
            && ascribe_resolve::link_on_the_way(&self.root, rel).is_some()
        {
            return Err(std::io::Error::other("a folder on the way is a link"));
        }
        std::fs::create_dir_all(folder)?;
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        let temporary = folder.join(format!(
            "tmp-{}-{}",
            std::process::id(),
            name.unwrap_or_default()
        ));
        std::fs::write(&temporary, text)?;
        std::fs::rename(&temporary, &path).inspect_err(|_| {
            let _ = std::fs::remove_file(&temporary);
        })
    }

    fn remove(&self, path: &str) {
        let _ = std::fs::remove_file(self.at(path));
    }

    fn list(&self, folder: &str) -> Listing {
        let mut listing = Listing::default();
        // Outside FileSystem: a folder Ascribe writes under `.ascribe/`, not
        // the project's sources.
        let Ok(entries) = std::fs::read_dir(self.at(folder)) else {
            return listing;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => listing.folders.push(name),
                Ok(_) if !name.starts_with("tmp-") => listing.files.push(name),
                _ => {}
            }
        }
        listing
    }

    fn remove_folder(&self, folder: &str) {
        let _ = std::fs::remove_dir(self.at(folder));
    }

    fn tidy(&self, folder: &str, today: Date) {
        // Outside FileSystem: as in `list`.
        let Ok(files) = std::fs::read_dir(self.at(folder)) else {
            return;
        };
        for file in files.flatten() {
            if file.file_name().to_string_lossy().starts_with("tmp-")
                && over_a_day_old(&file, today)
            {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
}

/// Whether `file` was last written before yesterday, by `today`, not the
/// clock.
fn over_a_day_old(file: &std::fs::DirEntry, today: Date) -> bool {
    // Outside FileSystem: a file Ascribe wrote under `.ascribe/`, for when it
    // was written.
    let Ok(modified) = file.metadata().and_then(|m| m.modified()) else {
        return false;
    };
    let Ok(since) = modified.duration_since(UNIX_EPOCH) else {
        return false;
    };
    today.days_since(Date::from_unix_seconds(since.as_secs())) >= 2
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn day(s: &str) -> Date {
        Date::parse(s).unwrap()
    }

    #[test]
    fn a_write_makes_its_folders_and_leaves_nothing_beside_it() {
        let dir = tempfile::tempdir().unwrap();
        let store = DiskStore::new(dir.path());
        store.write(".ascribe/history/links.json", "{}\n").unwrap();
        assert_eq!(
            store.read(".ascribe/history/links.json").as_deref(),
            Some("{}\n")
        );
        assert_eq!(
            store.list(".ascribe/history"),
            Listing {
                files: vec!["links.json".to_owned()],
                folders: vec![],
            }
        );
        assert_eq!(store.list(".ascribe").folders, vec!["history".to_owned()]);
    }

    #[test]
    fn a_tmp_file_isn_t_listed_and_goes_once_it_s_over_a_day_old() {
        let dir = tempfile::tempdir().unwrap();
        let store = DiskStore::new(dir.path());
        store.write("cache/ab/abcd.json", "{}").unwrap();
        let fresh = dir.path().join("cache/ab/tmp-1-fresh.json");
        std::fs::write(&fresh, "{").unwrap();
        assert_eq!(store.list("cache/ab").files, ["abcd.json"]);
        // Written today by the clock, so not a day old by any day before
        // the clock's.
        store.tidy("cache/ab", day("2026-10-01"));
        assert!(fresh.exists());
        store.tidy("cache/ab", day("2099-01-01"));
        assert!(!fresh.exists());
        // A folder that isn't empty stays; an empty one goes.
        store.remove_folder("cache/ab");
        assert!(dir.path().join("cache/ab").exists());
        store.remove("cache/ab/abcd.json");
        store.remove_folder("cache/ab");
        assert!(!dir.path().join("cache/ab").exists());
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_on_the_way_that_s_a_link_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), dir.path().join(".ascribe")).unwrap();
        let store = DiskStore::new(dir.path());
        assert!(store.write(".ascribe/history/report.json", "{}").is_err());
        assert!(!elsewhere.path().join("history").exists());
    }
}
