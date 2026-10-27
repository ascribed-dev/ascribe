//! Generates `diagnostics.rs` in `OUT_DIR` from the diagnostics registry,
//! `data/diagnostics.toml`, which `src/diagnostics.rs` includes: one
//! `DiagnosticSlug` constant per entry, named after its slug and in registry
//! order, and the lists `ALL`, `CONFIGURABLE`, `OFF_BY_DEFAULT`, and
//! `ACKNOWLEDGEABLE`. Adding a diagnostic is one edit, to the registry.

// A build script talks to Cargo on standard output.
#![allow(clippy::print_stdout)]

use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

const REGISTRY: &str = "data/diagnostics.toml";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed={REGISTRY}");
    // Outside FileSystem: the build script reads the registry this crate
    // ships, not a project's file.
    let text = fs::read_to_string(REGISTRY)?;
    let doc: toml::Table = text.parse()?;
    let entries = doc
        .get("diagnostic")
        .and_then(toml::Value::as_array)
        .ok_or("the registry has no [[diagnostic]] entries")?;
    let entries = entries
        .iter()
        .map(Entry::read)
        .collect::<Result<Vec<_>, _>>()?;

    let mut seen = HashSet::new();
    for e in &entries {
        if !seen.insert(e.slug.as_str()) {
            return Err(format!("`{}` is in the registry twice", e.slug).into());
        }
        if e.off_by_default && !e.configurable {
            return Err(format!(
                "`{}` is off by default, so it must be configurable to be turned on",
                e.slug
            )
            .into());
        }
    }

    let mut out = String::new();
    for e in &entries {
        writeln!(out, "#[doc = {:?}]", e.summary())?;
        if let Some(when) = &e.when {
            writeln!(out, "#[doc = \"\"]")?;
            writeln!(out, "#[doc = {:?}]", format!(" {when}."))?;
        }
        writeln!(
            out,
            "pub const {}: DiagnosticSlug = DiagnosticSlug({:?});\n",
            e.constant(),
            e.slug
        )?;
    }
    list(
        &mut out,
        "/// Every slug, in registry order.",
        "ALL",
        entries.iter(),
    )?;
    list(
        &mut out,
        "/// The diagnostics a project may set the level of in `[checks]`: the\n\
         /// registry's entries with `configurable = true`, in registry order.",
        "CONFIGURABLE",
        entries.iter().filter(|e| e.configurable),
    )?;
    list(
        &mut out,
        "/// The configurable checks that are off unless a project names them in\n\
         /// `[checks]`: the registry's entries with `off-by-default = true`, in\n\
         /// registry order.",
        "OFF_BY_DEFAULT",
        entries.iter().filter(|e| e.off_by_default),
    )?;

    out.push_str(
        "/// The checks an author can acknowledge (SPEC §4.9): the registry's entries\n\
         /// whose next step is `review`, each with the place its problems are reported\n\
         /// at, in registry order.\n\
         pub const ACKNOWLEDGEABLE: &[(DiagnosticSlug, crate::Place)] = &[\n",
    );
    for e in entries.iter().filter(|e| e.next == "review") {
        let place = match e.place.as_deref() {
            Some("page") => "Page",
            Some("block") => "Block",
            Some("entry") => "Entry",
            _ => {
                return Err(format!(
                    "`{}` is a \"review\" entry, so its `place` must be \"page\", \"block\", or \"entry\"",
                    e.slug
                )
                .into());
            }
        };
        writeln!(out, "    ({}, crate::Place::{place}),", e.constant())?;
    }
    out.push_str("];\n");

    let path = PathBuf::from(env::var("OUT_DIR")?).join("diagnostics.rs");
    fs::write(path, out)?;
    Ok(())
}

/// Writes `pub const <name>: &[DiagnosticSlug]` listing `entries`.
fn list<'a>(
    out: &mut String,
    doc: &str,
    name: &str,
    entries: impl Iterator<Item = &'a Entry>,
) -> Result<(), std::fmt::Error> {
    writeln!(out, "{doc}\npub const {name}: &[DiagnosticSlug] = &[")?;
    for e in entries {
        writeln!(out, "    {},", e.constant())?;
    }
    writeln!(out, "];\n")
}

/// The fields of a registry entry the constants are made from.
struct Entry {
    code: String,
    slug: String,
    severity: String,
    level: String,
    next: String,
    row: Option<String>,
    area: Option<String>,
    when: Option<String>,
    place: Option<String>,
    configurable: bool,
    off_by_default: bool,
}

impl Entry {
    fn read(value: &toml::Value) -> Result<Entry, String> {
        let text = |key: &str| {
            value
                .get(key)
                .and_then(toml::Value::as_str)
                .map(str::to_owned)
        };
        let flag = |key: &str| value.get(key).and_then(toml::Value::as_bool) == Some(true);
        let slug = text("slug").ok_or("a registry entry has no `slug`")?;
        let required = |key: &str| text(key).ok_or_else(|| format!("`{slug}` has no `{key}`"));
        let entry = Entry {
            code: required("code")?,
            severity: required("severity")?,
            level: required("level")?,
            next: required("next")?,
            row: text("row"),
            area: text("area"),
            when: text("when"),
            place: text("place"),
            configurable: flag("configurable"),
            off_by_default: flag("off-by-default"),
            slug,
        };
        let kebab = entry.slug.starts_with(|c: char| c.is_ascii_lowercase())
            && entry.slug.split('-').all(|word| {
                !word.is_empty()
                    && word
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
            });
        if !kebab {
            return Err(format!("`{}` isn't a kebab-case slug", entry.slug));
        }
        Ok(entry)
    }

    /// The constant's name: the slug in upper case, `-` as `_`.
    fn constant(&self) -> String {
        self.slug.to_uppercase().replace('-', "_")
    }

    /// The first line of the constant's doc comment.
    fn summary(&self) -> String {
        let kind = match (&self.row, &self.area) {
            (Some(row), _) => format!("SPEC §8.2, \"{row}\""),
            (None, Some(area)) => format!("a content check (area `{area}`)"),
            (None, None) => "a rule for loading `ascribe.toml`".to_owned(),
        };
        format!(
            " `{}`, {}, {} level: {kind}.",
            self.code, self.severity, self.level
        )
    }
}
