//! `_ascribe/versions.json`: what every line of each versioned build knows
//! about the others (output layout contract §6). It's above the builds'
//! folders, the one file the builds share, and says, for each versioned
//! build, its target, its lines, newest first, and each page's route in
//! each line that publishes it.

use std::collections::BTreeMap;

use ascribe_core::Router;
use ascribe_model::{Build, ContentModel};
use ascribe_resolve::{AstroRouter, Project};
use serde::Serialize;
use serde::ser::{SerializeMap, Serializer};

use crate::EmitError;

/// The file's path in the shared folder, `<output-dir>/_ascribe/`.
pub const VERSIONS_PATH: &str = "versions.json";
/// The file's version.
pub const VERSIONS_VERSION: u32 = 1;

#[derive(Serialize)]
struct Versions<'a> {
    version: u32,
    builds: BTreeMap<&'a str, VersionedBuild<'a>>,
}

#[derive(Serialize)]
struct VersionedBuild<'a> {
    target: &'a str,
    lines: Vec<Line<'a>>,
    pages: BTreeMap<String, Page<'a>>,
}

#[derive(Serialize)]
struct Line<'a> {
    line: &'a str,
    status: &'static str,
    build: &'a str,
    base: String,
}

#[derive(Serialize)]
struct Page<'a> {
    lines: Routes<'a>,
}

/// A page's route in each line that publishes it, newest line first.
struct Routes<'a>(Vec<(&'a str, String)>);

impl Serialize for Routes<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (line, route) in &self.0 {
            map.serialize_entry(line, route)?;
        }
        map.end()
    }
}

/// `versions.json` for a project, or `None` for one with no `[releases]`,
/// which has no shared file. Every line of every versioned build is in it,
/// whichever builds are being written.
///
/// # Errors
///
/// It can't be serialized.
pub fn versions_json(index: &Project) -> Result<Option<String>, EmitError> {
    let model = index.model();
    if model.releases.is_empty() {
        return Ok(None);
    }
    let mut builds: BTreeMap<&str, VersionedBuild<'_>> = BTreeMap::new();
    for build in &model.builds {
        let (Some(target), Some(line)) = (&build.versions, &build.line) else {
            continue;
        };
        let entry = builds
            .entry(build.declared_name())
            .or_insert_with(|| VersionedBuild {
                target,
                lines: Vec::new(),
                pages: BTreeMap::new(),
            });
        let router = AstroRouter::from_consumer(&model.for_build(build).consumer);
        entry.lines.push(Line {
            line: &line.line.text,
            status: if line.current { "current" } else { "live" },
            build: &build.name,
            base: base_of(model, build),
        });
        for page in index.pages() {
            if index.dropped(&page.path, build).is_none() {
                entry
                    .pages
                    .entry(page.path.to_string())
                    .or_insert_with(|| Page {
                        lines: Routes(Vec::new()),
                    })
                    .lines
                    .0
                    .push((&line.line.text, router.route(&page.path)));
            }
        }
    }
    let file = Versions {
        version: VERSIONS_VERSION,
        builds,
    };
    let mut text = serde_json::to_string_pretty(&file).map_err(|e| EmitError::Invalid {
        message: format!("can't write {VERSIONS_PATH}: {e}"),
    })?;
    text.push('\n');
    Ok(Some(text))
}

/// The path a line's pages are under, ending in `/`.
fn base_of(model: &ContentModel, build: &Build) -> String {
    let base = model.for_build(build).consumer.base_path.clone();
    let mut base = if base.starts_with('/') {
        base
    } else {
        format!("/{base}")
    };
    if !base.ends_with('/') {
        base.push('/');
    }
    base
}
