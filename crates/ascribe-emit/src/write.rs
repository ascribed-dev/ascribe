//! Writing a project's builds: what `ascribe build` does once the checks
//! have passed.

use std::collections::BTreeMap;
use std::path::Path;

use ascribe_model::{Build, ContentModel};
use ascribe_resolve::{AstroRouter, Project, ResolvedBuild};

use ascribe_core::RelPath;

use crate::{
    Contents, EmitContext, EmitError, EmittedFile, Emitter, FileKind, JsonEmitter, OutputDir,
    OutputOptions, PlainEmitter, SiteEmitter, VERSIONS_PATH, emit, versions_json,
};

/// One of the outputs a build can write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// Markdown plus web components, for an Astro site.
    Site,
    /// Fully resolved CommonMark with no HTML.
    Plain,
    /// The resolved tree as JSON.
    Json,
}

/// What [`write_outputs`] writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteOptions {
    /// The outputs, in the order they're written; one named twice is written
    /// once.
    pub outputs: Vec<Output>,
    /// Whether the site output marks each block with the source it came
    /// from, for review.
    pub anchors: bool,
}

/// One output of one build, written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Written {
    /// The build's name.
    pub build: String,
    /// The output's name: `site`, `plain`, or `json`.
    pub output: &'static str,
    /// How many pages it holds.
    pub pages: usize,
    /// How many assets it holds.
    pub assets: usize,
    /// How many files of the previous output it no longer produces, removed.
    pub removed: usize,
}

/// What [`write_outputs`] reports as it goes, in order, so a caller can
/// show progress, and what's already written when a later output fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WriteEvent {
    /// An output's warning about the build as a whole, given once however
    /// many builds it applies to; it comes before that output is written.
    Warning(String),
    /// An output was written.
    Written(Written),
}

/// Resolves each of `builds` and writes the outputs `options` names into the
/// project's output directory, replacing what each wrote before.
///
/// `index` is the project's source index and `root` its root, the folder of
/// `ascribe.toml`. Routes come from the `astro` profile's router, the only
/// profile of spec 0.1, for every output, so a link in the plain output is
/// the URL the site publishes. Each event is passed to `on_event` as it
/// happens, and every output written is also in the result.
///
/// This doesn't check the project: `ascribe build` runs
/// `ascribe_check::diagnose` first, and writes nothing when it finds errors.
///
/// # Errors
///
/// The output directory can't be locked or written, or an output can't be
/// produced. The outputs written before the failure stay written.
pub fn write_outputs(
    index: &Project,
    root: &Path,
    builds: &[&Build],
    options: &WriteOptions,
    on_event: &mut dyn FnMut(&WriteEvent),
) -> Result<Vec<Written>, EmitError> {
    let output = OutputDir::lock_in(root, Path::new(&index.model().project.output_dir))?;
    let mut warned: Vec<String> = Vec::new();
    let mut written = Vec::new();
    for build in builds {
        let emitted = emit_build(index, root, build, options, &mut |warning| {
            if !warned.iter().any(|w| w == warning) {
                on_event(&WriteEvent::Warning(warning.to_owned()));
                warned.push(warning.to_owned());
            }
        })?;
        for (name, emission) in emitted {
            let count = |kind: FileKind| emission.files.iter().filter(|f| f.kind == kind).count();
            let output_options = OutputOptions {
                anchors: options.anchors && name == SITE,
            };
            let replaced =
                output.replace_with(&build.name, name, &emission.files, output_options)?;
            let done = Written {
                build: build.name.clone(),
                output: name,
                pages: count(FileKind::Page),
                assets: count(FileKind::Asset),
                removed: replaced.removed,
            };
            on_event(&WriteEvent::Written(done.clone()));
            written.push(done);
        }
    }
    // What the lines share is above the builds, written whichever builds
    // were asked for, and removed with the project's last `[releases]`.
    match shared_files(index)? {
        Some(files) => {
            output.replace_shared(&files)?;
        }
        None => {
            output.remove_shared()?;
        }
    }
    // A line that has left the window has no output, and what a build of
    // it wrote before goes.
    let emitters = [
        SiteEmitter::new(index.model()).name(),
        PlainEmitter::new(index.model()).name(),
        JsonEmitter.name(),
    ];
    for name in archived_lines(index.model()) {
        for emitter in emitters {
            output.remove_output(&name, emitter)?;
        }
    }
    Ok(written)
}

/// The name a build of each archived line of a versioned build would have
/// had: `site@3.4`.
fn archived_lines(model: &ContentModel) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for build in &model.builds {
        let Some(releases) = build.versions.as_deref().and_then(|t| model.releases_of(t)) else {
            continue;
        };
        for archived in &releases.archived {
            let name = format!("{}@{}", build.declared_name(), archived.line.text);
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out
}

/// The name of the site output, whose files can carry anchors.
const SITE: &str = "site";

/// Every file each output of `build` would hold, by the output's name, in
/// the order `options` names them; `warn` is given each output's warnings
/// about the build as a whole.
fn emit_build(
    index: &Project,
    root: &Path,
    build: &Build,
    options: &WriteOptions,
    warn: &mut dyn FnMut(&str),
) -> Result<Vec<(&'static str, crate::Emission)>, EmitError> {
    // An older line's addresses are under its folder.
    let index = index.for_build(build);
    let index = index.as_ref();
    let model = index.model();
    let router = AstroRouter::from_consumer(&model.consumer);
    let plain = PlainEmitter::new(model);
    let json = JsonEmitter;
    let site = SiteEmitter::new(model).with_anchors(options.anchors);
    let mut emitters: Vec<&dyn Emitter> = Vec::new();
    for o in &options.outputs {
        let emitter: &dyn Emitter = match o {
            Output::Plain => &plain,
            Output::Json => &json,
            Output::Site => &site,
        };
        if !emitters.iter().any(|x| x.name() == emitter.name()) {
            emitters.push(emitter);
        }
    }
    let resolved = index.resolve_build(build, &router);
    under_older_line(index.model(), build, &resolved)?;
    let cx = EmitContext::new(index, root, build);
    let mut out = Vec::new();
    for emitter in &emitters {
        for warning in emitter.warnings(&cx) {
            warn(&warning);
        }
        out.push((emitter.name(), emit(*emitter, &cx, &resolved)?));
    }
    Ok(out)
}

/// The files above the builds' folders, in `<output-dir>/_ascribe/`: none
/// for a project with no `[releases]`.
fn shared_files(index: &Project) -> Result<Option<Vec<EmittedFile>>, EmitError> {
    let Some(text) = versions_json(index)? else {
        return Ok(None);
    };
    let file = EmittedFile {
        path: RelPath::parse(VERSIONS_PATH).map_err(|e| EmitError::Invalid {
            message: format!("{VERSIONS_PATH}: {e}"),
        })?,
        kind: FileKind::Generated,
        source: None,
        url: None,
        contents: Contents::Text(text),
    };
    Ok(Some(vec![file]))
}

/// What [`write_outputs`] would write for every build of the project, held
/// in memory and written nowhere: each file by the path it would have
/// under the output directory, `<build>/<output>/<path>`, or
/// `_ascribe/<path>` for a file the builds share, with its contents. A
/// copied asset is held as the path it's copied from.
///
/// # Errors
///
/// An output can't be produced.
pub fn outputs_in_memory(
    index: &Project,
    root: &Path,
    options: &WriteOptions,
) -> Result<BTreeMap<String, Contents>, EmitError> {
    let mut files = BTreeMap::new();
    for build in &index.model().builds {
        for (name, emission) in emit_build(index, root, build, options, &mut |_| {})? {
            for file in emission.files {
                files.insert(
                    format!("{}/{name}/{}", build.name, file.path),
                    file.contents,
                );
            }
        }
    }
    for file in shared_files(index)?.into_iter().flatten() {
        files.insert(format!("_ascribe/{}", file.path), file.contents);
    }
    Ok(files)
}

/// Refuses a page of a versioned build's current line whose route would be
/// under an older line's, `3.4/index.md` beside a line `3.4`, since the two
/// can't be published at one address.
fn under_older_line(
    model: &ContentModel,
    build: &Build,
    resolved: &ResolvedBuild,
) -> Result<(), EmitError> {
    if !build.line.as_ref().is_some_and(|l| l.current) {
        return Ok(());
    }
    let older: Vec<(&str, String)> = model
        .builds
        .iter()
        .filter(|b| b.declared_name() == build.declared_name())
        .filter_map(|b| Some((b.name.as_str(), b.line_folder()?)))
        .collect();
    let mut found = Vec::new();
    for page in &resolved.pages {
        let id = format!("{}/", AstroRouter::entry_id(&page.path));
        for (name, folder) in &older {
            if id.starts_with(folder.as_str()) {
                found.push(format!("`{}` is under `{name}`'s pages", page.path));
            }
        }
    }
    if found.is_empty() {
        return Ok(());
    }
    Err(EmitError::Invalid {
        message: format!(
            "build `{}` can't publish a page where an older line's pages are: {}. Rename the folder",
            build.name,
            found.join("; ")
        ),
    })
}
