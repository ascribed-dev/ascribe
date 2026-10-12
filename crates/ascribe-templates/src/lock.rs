//! Reading `templates.lock`: what a type ejected from the set was copied
//! from. Eject writes it ([`crate::eject`]); this reads what the rules for
//! agents take from it, their index's summary of each type.

use std::collections::BTreeMap;

use ascribe_core::RelPath;
use ascribe_model::ContentModel;
use ascribe_resolve::{FileSystem, Probe};

use crate::eject::{LOCK_FILE, LOCK_VERSION};

/// Their index's summary of each type whose `template` a `templates.lock`
/// lists, by the type's name. The lock is the one beside the template, as
/// eject writes it, and an entry is matched by the template's path, which
/// it records among its files. A lock that isn't there, or isn't one eject
/// wrote, gives nothing: the summary is a help, and the model's own checks
/// are where a problem is reported.
pub fn summaries(model: &ContentModel, fs: &dyn FileSystem) -> BTreeMap<String, String> {
    let mut locks: BTreeMap<RelPath, Option<toml::Table>> = BTreeMap::new();
    let mut out = BTreeMap::new();
    for t in &model.types {
        let Some(template) = t
            .template
            .as_ref()
            .and_then(|f| RelPath::parse(&f.path).ok())
        else {
            continue;
        };
        let Some(lock_path) = template.parent().and_then(|d| d.join(LOCK_FILE).ok()) else {
            continue;
        };
        let lock = locks
            .entry(lock_path)
            .or_insert_with_key(|path| read(path, fs));
        if let Some(summary) = lock.as_ref().and_then(|l| summary(l, &template)) {
            out.insert(t.name.clone(), summary);
        }
    }
    out
}

/// The lock at `path`, when it's there and of the version eject writes.
fn read(path: &RelPath, fs: &dyn FileSystem) -> Option<toml::Table> {
    if fs.probe(path) != Probe::File {
        return None;
    }
    let text = String::from_utf8(fs.read_file(path).ok()?).ok()?;
    let lock: toml::Table = toml::from_str(&text).ok()?;
    (lock.get("version")?.as_integer()? == LOCK_VERSION).then_some(lock)
}

/// The summary of the entry that lists `template` among its files.
fn summary(lock: &toml::Table, template: &RelPath) -> Option<String> {
    lock.get("template")?
        .as_array()?
        .iter()
        .filter_map(toml::Value::as_table)
        .find(|entry| {
            entry
                .get("files")
                .and_then(toml::Value::as_table)
                .is_some_and(|files| files.contains_key(template.as_str()))
        })?
        .get("summary")?
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
