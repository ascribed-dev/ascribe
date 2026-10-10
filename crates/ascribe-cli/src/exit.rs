//! Exit codes, shared by every subcommand, and the one place a library's
//! error becomes one.

use std::io::Write;
use std::process::ExitCode;

use ascribe_core::Coded;

/// Everything passed: no errors (and, with `--deny-warnings`, no warnings).
pub const OK: u8 = 0;
/// The documentation set has errors (or warnings under `--deny-warnings`).
pub const PROBLEMS: u8 = 1;
/// The command couldn't do its work: a usage error, no `ascribe.toml`, a
/// content model that doesn't load, or a file that can't be read.
pub const FAILURE: u8 = 2;

/// An exit code as the process's.
pub fn code(code: u8) -> ExitCode {
    ExitCode::from(code)
}

/// The exit code for a command that stopped on `error`. Every error a
/// library returns means the command couldn't do its work: [`FAILURE`],
/// whatever its [code](Coded::code). Problems in the documentation are
/// diagnostics, not errors, and give [`PROBLEMS`].
pub fn of(_error: &dyn Coded) -> u8 {
    FAILURE
}

/// Reports `error` on `err` as `error: <its message>` and returns its exit
/// code.
pub fn fail(err: &mut dyn Write, error: &dyn Coded) -> u8 {
    let _ = writeln!(err, "error: {error}");
    of(error)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::io;
    use std::path::PathBuf;

    use ascribe_check::prose::EjectError;
    use ascribe_check::{LoadError, LocateError, ScopeError, UnknownBuild};
    use ascribe_core::Coded;
    use ascribe_diff::DiffError;
    use ascribe_emit::{EmitError, StoreError};
    use ascribe_fmt::FormatFilesError;
    use ascribe_lsp::ServeError;
    use ascribe_query::QueryError;
    use ascribe_sources::SourcesError;

    use crate::agents::markers::Damage;
    use crate::agents::sync::SyncError;
    use crate::commands::check::CheckError;
    use crate::commands::fmt::FmtError;
    use crate::commands::sources::PagesUnavailable;
    use crate::context::Failure;

    /// Every error code, once, with what it means. A tool that wraps a
    /// command tells failures apart by these, so don't rename one.
    const CODES: &[(&str, &str)] = &[
        (
            "current_dir_unreadable",
            "the current directory can't be read",
        ),
        ("model_not_found", "no ascribe.toml here or above"),
        ("model_not_a_file", "--config names no file"),
        (
            "project_unreadable",
            "a project file or folder can't be read",
        ),
        ("model_invalid", "the content model has errors"),
        ("unknown_build", "a build name isn't one of the model's"),
        ("path_missing", "a path to check doesn't exist"),
        (
            "path_not_in_a_project",
            "no ascribe.toml at or above a path",
        ),
        ("paths_in_two_projects", "a path is in another project"),
        ("path_outside_project", "a path isn't in the project"),
        ("path_not_a_source", "--path names no possible source file"),
        ("stdin_unreadable", "standard input can't be read as text"),
        ("vale_not_set_up", "--vale, and ascribe.toml has no [checks.vale]"),
        ("vale_no_preset", "[checks.vale] names no preset to eject"),
        ("vale_config_exists", ".vale.ini or .vale is there already"),
        ("vale_unwritable", "a Vale file or ascribe.toml can't be written"),
        ("vale_model_unchanged", "[checks.vale] can't be changed in ascribe.toml"),
        ("format_io", "fmt can't read or write a path"),
        ("not_utf8", "a file to format isn't UTF-8"),
        ("format_bad_edits", "the formatter's edits don't apply"),
        ("asset_unreadable", "an asset can't be read"),
        ("render_failed", "a page can't be rendered"),
        ("build_invalid", "the build can't be written in this form"),
        ("output_locked", "another build is writing the output"),
        ("not_a_manifest", "a file is where a manifest goes"),
        (
            "unknown_manifest_version",
            "a manifest's version is unknown",
        ),
        ("bad_manifest_entry", "a manifest lists a path outside it"),
        (
            "output_conflicts",
            "output files would collide or overwrite",
        ),
        ("output_io", "writing the output failed"),
        ("git_not_found", "git isn't on the path"),
        ("not_a_repository", "the project isn't in a git repository"),
        ("unknown_revision", "a revision names no commit"),
        ("no_default_branch", "no default branch to compare with"),
        ("shallow_history", "the clone is too shallow"),
        ("no_common_history", "the base and HEAD share no history"),
        (
            "outside_repository",
            "the content root is outside the repository",
        ),
        ("bad_path", "a path couldn't be made"),
        ("base_model_invalid", "the base's content model has errors"),
        (
            "base_model_not_utf8",
            "the base's content model isn't UTF-8",
        ),
        ("git_failed", "a git command failed"),
        ("unknown_source", "a name isn't a declared source"),
        ("source_not_remote", "a source is in this repository"),
        ("lock_unreadable", "ascribe.lock can't be read"),
        ("to_needs_one_source", "--to with more than one source"),
        ("no_cache", "there's no cache folder"),
        ("cache_unwritable", "the cache can't be made"),
        ("copy_unwritable", "a copy or the lock can't be written"),
        // Its value can't be made outside lsp-server; ascribe-lsp's
        // `protocol_error_code` test checks it.
        ("lsp_protocol", "the client broke the protocol"),
        (
            "lsp_initialize_params",
            "initialize's parameters aren't understood",
        ),
        ("unknown_diagnostic", "no diagnostic has that code or name"),
        ("not_a_source", "a path isn't a page or fragment"),
        ("not_a_page", "a fragment, where only a page will do"),
        ("build_required", "several builds, and none was named"),
        ("bad_target", "refs can't read the target"),
        (
            "markers_damaged",
            "a file's generated block has damaged markers",
        ),
        (
            "instructions_in_content",
            "an instruction file would be a page",
        ),
        (
            "copilot_needs_repository",
            "Copilot's files outside a repository",
        ),
        (
            "instructions_unreadable",
            "an instruction file can't be read",
        ),
        (
            "instructions_unwritable",
            "an instruction file can't be written",
        ),
        (
            "settings_unreadable",
            "an agent's settings file can't be merged into",
        ),
    ];

    fn path() -> PathBuf {
        PathBuf::from("a")
    }

    fn io_error() -> io::Error {
        io::Error::other("x")
    }

    fn text() -> String {
        "x".to_owned()
    }

    /// One error of every variant of every error a command can stop on.
    /// The matches don't compile when a variant is added, so it's listed
    /// here too.
    #[allow(clippy::too_many_lines)] // One line per variant.
    fn samples() -> Vec<Box<dyn Coded>> {
        fn locate(e: &LocateError) {
            match e {
                LocateError::CurrentDir(_)
                | LocateError::NotFound { .. }
                | LocateError::NotAFile { .. } => {}
            }
        }
        fn load(e: &LoadError) {
            match e {
                LoadError::Read { .. } | LoadError::Model { .. } => {}
            }
        }
        fn format(e: &FormatFilesError) {
            match e {
                FormatFilesError::Io { .. }
                | FormatFilesError::NotUtf8 { .. }
                | FormatFilesError::BadEdits { .. } => {}
            }
        }
        fn emit(e: &EmitError) {
            match e {
                EmitError::Read { .. }
                | EmitError::Render { .. }
                | EmitError::Invalid { .. }
                | EmitError::Store(_) => {}
            }
        }
        fn store(e: &StoreError) {
            match e {
                StoreError::Locked { .. }
                | StoreError::NotManifest { .. }
                | StoreError::UnknownVersion { .. }
                | StoreError::BadEntry { .. }
                | StoreError::Conflicts(_)
                | StoreError::Io { .. } => {}
            }
        }
        fn diff(e: &DiffError) {
            match e {
                DiffError::UnknownBuild(_)
                | DiffError::GitNotFound
                | DiffError::NotARepository { .. }
                | DiffError::UnknownRevision(_)
                | DiffError::NoDefaultBranch
                | DiffError::ShallowHistory(_)
                | DiffError::NoCommonHistory(_)
                | DiffError::OutsideRepository(_)
                | DiffError::Path(_)
                | DiffError::BaseModel { .. }
                | DiffError::BaseModelText { .. }
                | DiffError::Git { .. } => {}
            }
        }
        fn sources(e: &SourcesError) {
            match e {
                SourcesError::GitNotFound
                | SourcesError::Git { .. }
                | SourcesError::UnknownSource(_)
                | SourcesError::NotRemote(_)
                | SourcesError::Lock(_)
                | SourcesError::BadRevision(_)
                | SourcesError::ToNeedsOneSource
                | SourcesError::NoCache
                | SourcesError::Cache { .. }
                | SourcesError::Write { .. } => {}
            }
        }
        fn scope(e: &ScopeError) {
            match e {
                ScopeError::Locate(_)
                | ScopeError::Missing { .. }
                | ScopeError::NoProject { .. }
                | ScopeError::OtherProject { .. }
                | ScopeError::Outside { .. }
                | ScopeError::NotASource { .. } => {}
            }
        }
        fn check(e: &CheckError) {
            match e {
                CheckError::Scope(_) | CheckError::Stdin(_) | CheckError::NoVale => {}
            }
        }
        fn eject(e: &EjectError) {
            match e {
                EjectError::NoPreset { .. }
                | EjectError::Exists { .. }
                | EjectError::Write { .. }
                | EjectError::Model(_) => {}
            }
        }
        fn serve(e: &ServeError) {
            match e {
                ServeError::Protocol(_) | ServeError::Params(_) => {}
            }
        }
        fn query(e: &QueryError) {
            match e {
                QueryError::UnknownDiagnostic { .. }
                | QueryError::NotASource { .. }
                | QueryError::NotAPage { .. }
                | QueryError::BuildRequired { .. }
                | QueryError::UnknownBuild(_)
                | QueryError::BadTarget { .. }
                | QueryError::Render(_) => {}
            }
        }
        fn agents(e: &SyncError) {
            match e {
                SyncError::Damaged { .. }
                | SyncError::InContent { .. }
                | SyncError::NoRepository
                | SyncError::Git(_)
                | SyncError::Read { .. }
                | SyncError::Settings { .. }
                | SyncError::Write { .. } => {}
            }
        }
        let _ = (
            locate, load, scope, check, eject, format, emit, store, diff, sources, serve, query, agents,
        );

        let store_errors = || {
            vec![
                StoreError::Locked { dir: text() },
                StoreError::NotManifest { path: text() },
                StoreError::UnknownVersion {
                    path: text(),
                    version: 9,
                },
                StoreError::BadEntry {
                    path: text(),
                    entry: text(),
                },
                StoreError::Conflicts(vec![text()]),
                StoreError::Io {
                    action: "write",
                    path: text(),
                    source: io_error(),
                },
            ]
        };
        let unknown_build = || UnknownBuild {
            name: text(),
            known: vec![text()],
        };
        let model_invalid = || LoadError::Model {
            text: text(),
            diagnostics: Vec::new(),
        };
        let not_a_repository = || DiffError::NotARepository {
            dir: text(),
            message: text(),
        };
        let mut all: Vec<Box<dyn Coded>> = vec![
            Box::new(LocateError::CurrentDir(io_error())),
            Box::new(LocateError::NotFound { dir: path() }),
            Box::new(LocateError::NotAFile { path: path() }),
            Box::new(LoadError::Read {
                path: text(),
                message: text(),
            }),
            Box::new(model_invalid()),
            Box::new(unknown_build()),
            Box::new(FormatFilesError::Io {
                path: path(),
                source: io_error(),
            }),
            Box::new(FormatFilesError::NotUtf8 { path: path() }),
            Box::new(FormatFilesError::BadEdits {
                path: path(),
                message: text(),
            }),
            Box::new(EmitError::Read {
                path: text(),
                message: text(),
            }),
            Box::new(EmitError::Render {
                page: text(),
                message: text(),
            }),
            Box::new(EmitError::Invalid { message: text() }),
            Box::new(DiffError::UnknownBuild(unknown_build())),
            Box::new(DiffError::GitNotFound),
            Box::new(not_a_repository()),
            Box::new(DiffError::UnknownRevision(text())),
            Box::new(DiffError::NoDefaultBranch),
            Box::new(DiffError::ShallowHistory(text())),
            Box::new(DiffError::NoCommonHistory(text())),
            Box::new(DiffError::OutsideRepository(text())),
            Box::new(DiffError::Path(text())),
            Box::new(DiffError::BaseModel {
                commit: text(),
                text: text(),
                issues: Vec::new(),
            }),
            Box::new(DiffError::BaseModelText { commit: text() }),
            Box::new(DiffError::Git {
                command: text(),
                message: text(),
            }),
            Box::new(SourcesError::GitNotFound),
            Box::new(SourcesError::Git {
                name: text(),
                message: text(),
            }),
            Box::new(SourcesError::UnknownSource(text())),
            Box::new(SourcesError::NotRemote(text())),
            Box::new(SourcesError::Lock(text())),
            Box::new(SourcesError::BadRevision(text())),
            Box::new(SourcesError::ToNeedsOneSource),
            Box::new(SourcesError::NoCache),
            Box::new(SourcesError::Cache {
                path: text(),
                message: text(),
            }),
            Box::new(SourcesError::Write {
                path: text(),
                message: text(),
            }),
            Box::new(ServeError::Params(
                serde_json::from_str::<u8>("x").unwrap_err(),
            )),
            Box::new(ScopeError::Locate(LocateError::NotFound { dir: path() })),
            Box::new(ScopeError::Missing { path: path() }),
            Box::new(ScopeError::NoProject { path: path() }),
            Box::new(ScopeError::OtherProject {
                path: path(),
                its: path(),
                config: path(),
            }),
            Box::new(ScopeError::Outside {
                path: path(),
                config: path(),
            }),
            Box::new(ScopeError::NotASource {
                path: path(),
                content_root: text(),
            }),
            // The CLI's own errors wrap these.
            Box::new(CheckError::Scope(ScopeError::Missing { path: path() })),
            Box::new(CheckError::Stdin(io_error())),
            Box::new(CheckError::NoVale),
            Box::new(EjectError::NoPreset { config: None }),
            Box::new(EjectError::Exists { path: text() }),
            Box::new(EjectError::Write {
                path: text(),
                reason: text(),
            }),
            Box::new(EjectError::Model(text())),
            Box::new(Failure::Config(LocateError::NotFound { dir: path() })),
            Box::new(Failure::Load(model_invalid())),
            Box::new(FmtError::Locate(LocateError::NotAFile { path: path() })),
            Box::new(FmtError::Model {
                config: path(),
                error: model_invalid(),
            }),
            Box::new(FmtError::Format(FormatFilesError::NotUtf8 { path: path() })),
            Box::new(PagesUnavailable::Load(Failure::Load(model_invalid()))),
            Box::new(PagesUnavailable::Drift(not_a_repository())),
            Box::new(QueryError::UnknownDiagnostic {
                given: text(),
                closest: Vec::new(),
            }),
            Box::new(QueryError::NotASource { path: text() }),
            Box::new(QueryError::NotAPage { path: text() }),
            Box::new(QueryError::BuildRequired {
                builds: vec![text()],
            }),
            Box::new(QueryError::UnknownBuild(unknown_build())),
            Box::new(QueryError::BadTarget {
                given: text(),
                reason: text(),
            }),
            Box::new(QueryError::Render(EmitError::Invalid { message: text() })),
            Box::new(SyncError::Damaged {
                path: text(),
                name: text(),
                damage: Damage::NoEnd,
            }),
            Box::new(SyncError::InContent { path: text() }),
            Box::new(SyncError::NoRepository),
            Box::new(SyncError::Git(DiffError::GitNotFound)),
            Box::new(SyncError::Read {
                path: text(),
                source: io_error(),
            }),
            Box::new(SyncError::Settings {
                path: text(),
                why: text(),
            }),
            Box::new(SyncError::Write {
                path: text(),
                source: io_error(),
            }),
        ];
        for e in store_errors() {
            all.push(Box::new(EmitError::Store(e)));
        }
        for e in store_errors() {
            all.push(Box::new(e));
        }
        all
    }

    #[test]
    fn each_code_is_listed_once() {
        let mut seen = BTreeSet::new();
        for (code, meaning) in CODES {
            assert!(seen.insert(*code), "{code} is listed twice");
            assert!(!meaning.is_empty());
            assert!(
                !code.is_empty()
                    && code
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "{code} isn't a short lowercase identifier"
            );
        }
        let mut used = BTreeSet::from(["lsp_protocol"]);
        for e in samples() {
            let code = e.code();
            assert!(seen.contains(code), "{code} ({e}) isn't in CODES");
            used.insert(code);
        }
        let unused: Vec<_> = seen.difference(&used).collect();
        assert!(unused.is_empty(), "no error has these codes: {unused:?}");
    }

    #[test]
    fn every_error_is_a_failure() {
        for e in samples() {
            assert_eq!(super::of(&*e), super::FAILURE, "{}", e.code());
        }
    }
}
