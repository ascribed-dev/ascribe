//! A project kept checked without an editor: the server's state and its
//! incremental index, driven by a caller that says which files changed, and
//! asks for a file's problems. `ascribe agents hook-server` keeps one per
//! project, so the check after an agent's edit redoes only what the edit
//! affects, as the server does after a keystroke.
//!
//! The problems are what the server would publish for the file: its
//! file-level diagnostics and the page-level ones of the editor's build,
//! located in it.

use std::path::{Path, PathBuf};

use ascribe_core::path::normalize;
use crossbeam_channel::Receiver;
use lsp_server::Message;
use lsp_types::{DiagnosticSeverity, FileChangeType, FileEvent, NumberOrString};

use crate::compute::compute;
use crate::core::Core;
use crate::uri::path_to_uri;

/// A problem the server found in a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// Its line, from 1.
    pub line: u32,
    /// Its diagnostic's code, `ASC036`.
    pub code: String,
    /// Its message.
    pub message: String,
    /// Whether it's an error; otherwise a warning.
    pub error: bool,
}

/// One project, loaded and kept current.
pub struct Watched {
    core: Core,
    // What the server would send a client: drained, and dropped.
    sent: Receiver<Message>,
}

impl Watched {
    /// Loads the project of `config`, an `ascribe.toml`.
    pub fn load(config: &Path) -> Watched {
        let (out, sent) = crossbeam_channel::unbounded();
        let mut core = Core::new(out);
        let config = normalize(config);
        core.folders = config.parent().map(Path::to_owned).into_iter().collect();
        core.config = Some(config);
        core.start();
        Watched { core, sent }
    }

    /// Takes in files created or changed (`present`) and deleted (`gone`) on
    /// disk since the last call, as a file watcher reports them.
    pub fn changed(&mut self, present: &[PathBuf], gone: &[PathBuf]) {
        let event = |path: &PathBuf, typ| Some(FileEvent::new(path_to_uri(path)?, typ));
        let events: Vec<FileEvent> = present
            .iter()
            .filter_map(|p| event(p, FileChangeType::CHANGED))
            .chain(
                gone.iter()
                    .filter_map(|p| event(p, FileChangeType::DELETED)),
            )
            .collect();
        if !events.is_empty() {
            self.core.did_change_watched(events);
        }
    }

    /// The problems in `file` now, in line order: after computing what the
    /// changes so far affect.
    pub fn problems(&mut self, file: &Path) -> Vec<Problem> {
        while let Some(job) = self.core.plan() {
            let outcome = compute(&job, &|| true);
            self.core.finish(&job, outcome);
        }
        self.sent.try_iter().for_each(drop);
        let mut problems: Vec<Problem> = self
            .core
            .published(&normalize(file))
            .iter()
            .map(|d| Problem {
                line: d.range.start.line + 1,
                code: match &d.code {
                    Some(NumberOrString::String(code)) => code.clone(),
                    Some(NumberOrString::Number(n)) => n.to_string(),
                    None => String::new(),
                },
                message: d.message.clone(),
                error: d.severity != Some(DiagnosticSeverity::WARNING),
            })
            .collect();
        problems.sort_by_key(|p| p.line);
        problems
    }

    /// The editor's build, which the problems are of; `None` before the
    /// project has loaded.
    pub fn build(&self) -> Option<String> {
        let loaded = self.core.loaded.as_ref()?;
        Some(loaded.model.editor_default_build().name.clone())
    }

    /// The project root, `ascribe.toml`'s folder; `None` before the project
    /// has loaded.
    pub fn root(&self) -> Option<&Path> {
        self.core.loaded.as_ref().map(|l| l.root.as_path())
    }
}
