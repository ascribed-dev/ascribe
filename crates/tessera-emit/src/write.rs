//! Writing a project's builds: what `ascribe build` does once the checks
//! have passed.

use std::path::Path;

use tessera_model::Build;
use tessera_resolve::{AstroRouter, Project};

use crate::{
    EmitContext, EmitError, Emitter, FileKind, JsonEmitter, OutputDir, OutputOptions, PlainEmitter,
    SiteEmitter, emit,
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
/// `tessera_check::diagnose` first, and writes nothing when it finds errors.
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
    let model = index.model();
    let output = OutputDir::lock(&root.join(&model.project.output_dir))?;
    let router = AstroRouter::from_consumer(&model.consumer);

    let plain = PlainEmitter;
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

    let mut warned: Vec<String> = Vec::new();
    let mut written = Vec::new();
    for build in builds {
        let resolved = index.resolve_build(build, &router);
        let cx = EmitContext::new(index, root, build);
        for emitter in &emitters {
            for warning in emitter.warnings(&cx) {
                if !warned.contains(&warning) {
                    on_event(&WriteEvent::Warning(warning.clone()));
                    warned.push(warning);
                }
            }
            let emission = emit(*emitter, &cx, &resolved)?;
            let count = |kind: FileKind| emission.files.iter().filter(|f| f.kind == kind).count();
            let output_options = OutputOptions {
                anchors: options.anchors && emitter.name() == site.name(),
            };
            let replaced = output.replace_with(
                &build.name,
                emitter.name(),
                &emission.files,
                output_options,
            )?;
            let done = Written {
                build: build.name.clone(),
                output: emitter.name(),
                pages: count(FileKind::Page),
                assets: count(FileKind::Asset),
                removed: replaced.removed,
            };
            on_event(&WriteEvent::Written(done.clone()));
            written.push(done);
        }
    }
    Ok(written)
}
