//! Exit codes, shared by every subcommand, and the one place a library's
//! error becomes one.

use std::io::Write;
use std::process::ExitCode;

use ascribe_core::Coded;

/// Everything passed: no errors (and, with `--exit-code=warning`, no warnings).
pub const OK: u8 = 0;
/// The documentation set has errors (or warnings under `--exit-code=warning`).
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
    use ascribe_check::{BaselineError, DiagnoseError, ScopeError, UnknownBuild};
    use ascribe_core::Coded;
    use ascribe_diff::DiffError;
    use ascribe_edit::{ReleaseError, RenameError, RetireError};
    use ascribe_emit::{EmitError, StoreError};
    use ascribe_fmt::FormatFilesError;
    use ascribe_lsp::ServeError;
    use ascribe_query::QueryError;
    use ascribe_resolve::reading::BadQuery;
    use ascribe_sources::SourcesError;
    use ascribe_workspace::{LoadError, LocateError};

    use crate::agents::markers::Damage;
    use crate::agents::sync::SyncError;
    use crate::commands::baseline::BaselineCommandError;
    use crate::commands::check::CheckError;
    use crate::commands::fmt::FmtError;
    use crate::commands::permalinks::PermalinksError;
    use crate::commands::preview::PreviewError;
    use crate::commands::rename::RenameCommandError;
    use crate::commands::report::ReportError;
    use crate::commands::sources::PagesUnavailable;
    use crate::commands::templates::TemplatesError;
    use crate::commands::versions::VersionsError;
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
        (
            "vale_not_set_up",
            "--vale, and ascribe.toml has no [checks.vale]",
        ),
        (
            "baseline_invalid",
            "a line of ascribe.baseline doesn't parse",
        ),
        ("baseline_unwritable", "ascribe.baseline can't be written"),
        (
            "fix_unwritable",
            "a file check --fix fixed can't be written",
        ),
        (
            "limit_not_concise",
            "check --limit, and the format isn't concise",
        ),
        ("vale_no_preset", "[checks.vale] names no preset to eject"),
        ("vale_config_exists", ".vale.ini or .vale is there already"),
        (
            "vale_unwritable",
            "a Vale file or ascribe.toml can't be written",
        ),
        (
            "vale_model_unchanged",
            "[checks.vale] can't be changed in ascribe.toml",
        ),
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
        ("output_link", "a link is on the way to the output"),
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
        (
            "prompt_needs_one_section",
            "report --format prompt without one section",
        ),
        ("not_a_source", "a path isn't a page or fragment"),
        ("not_a_page", "a fragment, where only a page will do"),
        ("build_required", "several builds, and none was named"),
        ("bad_target", "refs can't read the target"),
        ("empty_query", "search has no words to look for"),
        ("unclosed_quote", "a quote in search's query isn't closed"),
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
        (
            "permalinks_list_invalid",
            "a permalinks list can't be read as one",
        ),
        (
            "permalinks_list_not_found",
            "verify's permalinks list isn't there",
        ),
        (
            "permalinks_file_unreadable",
            "a permalinks list or names file can't be read",
        ),
        (
            "permalinks_list_unwritable",
            "ascribe.permalinks can't be written",
        ),
        (
            "permalink_not_retirable",
            "retire can't retire a name, page, or folder",
        ),
        ("retire_reason_missing", "retire's --reason is empty"),
        (
            "model_edit_refused",
            "ascribe.toml can't be changed in place",
        ),
        ("model_unwritable", "ascribe.toml can't be written"),
        ("bad_line", "a line isn't named as a mark names one"),
        ("no_lines", "a target has no lines in [releases]"),
        ("undeclared_line", "a target's list doesn't declare a line"),
        ("unknown_type", "a content type isn't one of the model's"),
        ("no_source_at_path", "a path has no page or fragment"),
        ("release_no_lines", "release's target has no lines"),
        (
            "release_bad_line",
            "release's line isn't one the target's lines can hold",
        ),
        ("release_line_live", "release's line is live already"),
        ("release_line_archived", "release's line is archived"),
        (
            "release_line_not_next",
            "a planned line comes before release's line",
        ),
        (
            "release_line_unplanned",
            "release's line isn't planned, nor between current and planned",
        ),
        ("release_next_bad", "--next can't be the next line"),
        (
            "release_mark_unrewritable",
            "a mark release would rewrite can't be rewritten in place",
        ),
        (
            "release_model_invalid",
            "ascribe.toml wouldn't load after the release",
        ),
        (
            "release_build_failed",
            "release can't build the outputs to compare",
        ),
        (
            "release_outputs_changed",
            "a release step that should change no output would",
        ),
        (
            "release_without_git",
            "no git to say whether release's or retire's files can be put back",
        ),
        (
            "release_files_unrestorable",
            "files release or retire would write are untracked or changed",
        ),
        (
            "release_unwritable",
            "a file release changes can't be written",
        ),
        (
            "archive_no_build",
            "no build is versioned by the line's target",
        ),
        (
            "archive_line_archived",
            "archive's line is archived, so has no output",
        ),
        (
            "archive_has_errors",
            "the checks found errors in the line's build",
        ),
        (
            "archive_folder_not_empty",
            "archive's folder has files and isn't an archive",
        ),
        ("retire_no_lines", "retire's target has no lines"),
        ("retire_line_not_live", "retire's line isn't live"),
        ("retire_line_current", "retire's line is the current one"),
        (
            "retire_line_not_oldest",
            "an older live line comes before retire's line",
        ),
        (
            "retire_not_versioned",
            "no build is versioned by retire's target",
        ),
        ("retire_no_address", "retire's line has no address"),
        (
            "retire_mark_unrewritable",
            "a mark retire would rewrite can't be rewritten in place",
        ),
        (
            "retire_model_invalid",
            "ascribe.toml wouldn't load after the retirement",
        ),
        (
            "retire_build_failed",
            "retire can't build the outputs to compare",
        ),
        (
            "retire_outputs_changed",
            "retire's rewrite would change a live line's output",
        ),
        (
            "retire_builds_changed",
            "retire's rewrite would change a build that isn't a line",
        ),
        (
            "rename_target_missing",
            "rename names something not in the project",
        ),
        ("rename_refused", "rename can't give it the new name"),
        ("rename_bad_target", "rename can't read the target"),
        (
            "rename_unwritable",
            "a file rename changes can't be written",
        ),
        (
            "templates_folder_refused",
            "templates can't be written in that folder",
        ),
        ("templates_lock_invalid", "templates.lock can't be added to"),
        (
            "unknown_template",
            "a name isn't one of the set's templates",
        ),
        (
            "templates_unwritable",
            "a template or templates.lock can't be written",
        ),
        (
            "project_exists",
            "init found an ascribe.toml at or above the folder",
        ),
        ("init_folder_refused", "init's folder is a file"),
        (
            "init_content_root_refused",
            "init's content root isn't a folder inside the project's",
        ),
        (
            "init_templates_refused",
            "init was asked for the templates in a folder of pages",
        ),
        (
            "init_unwritable",
            "init can't write ascribe.toml or make the content root",
        ),
        (
            "preview_port_unavailable",
            "preview can't listen on its port",
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
                | StoreError::Link { .. }
                | StoreError::NotEmpty { .. }
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
                CheckError::Scope(_)
                | CheckError::Stdin(_)
                | CheckError::NoVale
                | CheckError::Fix(_)
                | CheckError::Code(_)
                | CheckError::Baseline(_)
                | CheckError::LimitNotConcise => {}
            }
        }
        fn report(e: &ReportError) {
            match e {
                ReportError::PromptSection => {}
            }
        }
        fn diagnose(e: &DiagnoseError) {
            match e {
                DiagnoseError::UnknownBuild(_) | DiagnoseError::Baseline(_) => {}
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
                | QueryError::BadQuery(_)
                | QueryError::BadTarget { .. }
                | QueryError::Render(_)
                | QueryError::PermalinksListInvalid { .. }
                | QueryError::NotRetirable { .. }
                | QueryError::BadLine { .. }
                | QueryError::NoLines { .. }
                | QueryError::UndeclaredLine { .. }
                | QueryError::UnknownType { .. }
                | QueryError::NoSource { .. }
                | QueryError::Scope(_) => {}
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
        fn baseline(e: &BaselineCommandError) {
            match e {
                BaselineCommandError::Unwritable(_) => {}
            }
        }
        fn permalinks(e: &PermalinksError) {
            match e {
                PermalinksError::ListNotFound(_)
                | PermalinksError::Unreadable { .. }
                | PermalinksError::Unwritable { .. }
                | PermalinksError::NoReason
                | PermalinksError::Addresses(_) => {}
            }
        }
        fn rename(e: &RenameError) {
            match e {
                RenameError::Missing(_) | RenameError::Refused(_) => {}
            }
        }
        fn templates(e: &ascribe_templates::EjectError) {
            match e {
                ascribe_templates::EjectError::Folder { .. }
                | ascribe_templates::EjectError::Lock { .. }
                | ascribe_templates::EjectError::Model { .. }
                | ascribe_templates::EjectError::Write { .. } => {}
            }
        }
        fn init(e: &ascribe_templates::InitError) {
            match e {
                ascribe_templates::InitError::Project { .. }
                | ascribe_templates::InitError::NotAFolder { .. }
                | ascribe_templates::InitError::ContentRoot { .. }
                | ascribe_templates::InitError::TemplatesWithPages { .. }
                | ascribe_templates::InitError::Templates(_)
                | ascribe_templates::InitError::Write { .. } => {}
            }
        }
        fn templates_command(e: &TemplatesError) {
            match e {
                TemplatesError::NoProject(_)
                | TemplatesError::ToOutside { .. }
                | TemplatesError::Unknown { .. } => {}
            }
        }
        fn release(e: &ReleaseError) {
            match e {
                ReleaseError::NoLines { .. }
                | ReleaseError::BadLine { .. }
                | ReleaseError::AlreadyLive { .. }
                | ReleaseError::Archived { .. }
                | ReleaseError::NotOldestPlanned { .. }
                | ReleaseError::Unplanned { .. }
                | ReleaseError::BadNext { .. }
                | ReleaseError::MarkUnrewritable { .. }
                | ReleaseError::ModelEdit(_)
                | ReleaseError::ModelInvalid(_)
                | ReleaseError::Build(_)
                | ReleaseError::OutputsChanged { .. } => {}
            }
        }
        fn retire(e: &RetireError) {
            match e {
                RetireError::NoLines { .. }
                | RetireError::NotLive { .. }
                | RetireError::Current { .. }
                | RetireError::NotOldest { .. }
                | RetireError::NotVersioned { .. }
                | RetireError::NoAddress { .. }
                | RetireError::MarkUnrewritable { .. }
                | RetireError::ModelEdit(_)
                | RetireError::ModelInvalid(_)
                | RetireError::Build(_)
                | RetireError::OutputsChanged { .. }
                | RetireError::OtherBuildsChange { .. } => {}
            }
        }
        fn versions(e: &VersionsError) {
            match e {
                VersionsError::BadLine(_)
                | VersionsError::BadNext(_)
                | VersionsError::NoGit { .. }
                | VersionsError::NotRestorable { .. }
                | VersionsError::Unwritable { .. }
                | VersionsError::ArchiveNoBuild { .. }
                | VersionsError::ArchiveBuildRequired { .. }
                | VersionsError::ArchiveUnknownBuild { .. }
                | VersionsError::ArchiveLineArchived { .. }
                | VersionsError::ArchiveErrors { .. } => {}
            }
        }
        fn rename_command(e: &RenameCommandError) {
            match e {
                RenameCommandError::BadTarget { .. } | RenameCommandError::Unwritable { .. } => {}
            }
        }
        fn preview(e: &PreviewError) {
            match e {
                PreviewError::Listen { .. } => {}
            }
        }
        let _ = (
            locate,
            load,
            scope,
            check,
            report,
            diagnose,
            eject,
            format,
            emit,
            store,
            diff,
            sources,
            serve,
            query,
            agents,
            baseline,
            permalinks,
            rename,
            rename_command,
            release,
            retire,
            versions,
            templates,
            templates_command,
            init,
            preview,
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
                StoreError::Link { path: text() },
                StoreError::NotEmpty { dir: text() },
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
            issues: Vec::new(),
        };
        let not_a_repository = || DiffError::NotARepository {
            dir: text(),
            message: text(),
        };
        let mut all: Vec<Box<dyn Coded>> = vec![
            Box::new(LocateError::CurrentDir(io_error())),
            Box::new(LocateError::NotFound {
                dir: path(),
                below: Vec::new(),
            }),
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
            Box::new(ScopeError::Locate(LocateError::NotFound {
                dir: path(),
                below: Vec::new(),
            })),
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
            Box::new(CheckError::LimitNotConcise),
            Box::new(CheckError::Fix(ascribe_check::fix::FixError {
                path: path(),
                source: io_error(),
            })),
            Box::new(CheckError::Code(QueryError::UnknownDiagnostic {
                given: text(),
                closest: Vec::new(),
            })),
            Box::new(CheckError::Baseline(BaselineError {
                line: 1,
                message: text(),
            })),
            Box::new(BaselineCommandError::Unwritable(text())),
            Box::new(DiagnoseError::UnknownBuild(unknown_build())),
            Box::new(DiagnoseError::Baseline(BaselineError {
                line: 1,
                message: text(),
            })),
            Box::new(ReportError::PromptSection),
            Box::new(EjectError::NoPreset { config: None }),
            Box::new(EjectError::Exists { path: text() }),
            Box::new(EjectError::Write {
                path: text(),
                reason: text(),
            }),
            Box::new(EjectError::Model(text())),
            Box::new(ascribe_templates::EjectError::Folder {
                folder: text(),
                reason: text(),
            }),
            Box::new(ascribe_templates::EjectError::Lock {
                path: text(),
                reason: text(),
            }),
            Box::new(ascribe_templates::EjectError::Model {
                types: text(),
                reason: text(),
            }),
            Box::new(ascribe_templates::EjectError::Write {
                path: text(),
                reason: text(),
            }),
            Box::new(ascribe_templates::InitError::Project { found: path() }),
            Box::new(ascribe_templates::InitError::NotAFolder { folder: path() }),
            Box::new(ascribe_templates::InitError::ContentRoot {
                given: text(),
                reason: "isn't a relative path",
            }),
            Box::new(ascribe_templates::InitError::TemplatesWithPages {
                pages: 2,
                content_root: text(),
            }),
            Box::new(ascribe_templates::InitError::Templates(
                ascribe_templates::EjectError::Write {
                    path: text(),
                    reason: text(),
                },
            )),
            Box::new(ascribe_templates::InitError::Write {
                path: text(),
                reason: text(),
            }),
            Box::new(PreviewError::Listen {
                address: text(),
                message: text(),
            }),
            Box::new(TemplatesError::NoProject(LocateError::NotFound {
                dir: path(),
                below: Vec::new(),
            })),
            Box::new(TemplatesError::ToOutside {
                to: path(),
                root: path(),
            }),
            Box::new(TemplatesError::Unknown {
                name: text(),
                known: text(),
            }),
            Box::new(Failure::Config(LocateError::NotFound {
                dir: path(),
                below: Vec::new(),
            })),
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
            Box::new(QueryError::BadQuery(BadQuery::Empty)),
            Box::new(QueryError::BadQuery(BadQuery::UnclosedQuote {
                query: text(),
            })),
            Box::new(QueryError::BadTarget {
                given: text(),
                reason: text(),
            }),
            Box::new(QueryError::Render(EmitError::Invalid { message: text() })),
            Box::new(QueryError::PermalinksListInvalid {
                file: text(),
                detail: text(),
            }),
            Box::new(QueryError::NotRetirable {
                name: text(),
                why: text(),
            }),
            Box::new(PermalinksError::ListNotFound(path())),
            Box::new(PermalinksError::Unreadable {
                path: path(),
                message: text(),
            }),
            Box::new(PermalinksError::Unwritable {
                path: path(),
                message: text(),
            }),
            Box::new(PermalinksError::NoReason),
            // `ascribe addresses init` writing `ascribe.toml` without its
            // `[permalinks]`.
            Box::new(crate::commands::addresses::AddressesError::Model(
                "write",
                text(),
            )),
            Box::new(QueryError::BadLine { given: text() }),
            Box::new(QueryError::NoLines {
                target: text(),
                targets: Vec::new(),
            }),
            Box::new(QueryError::UndeclaredLine {
                target: text(),
                line: text(),
                lines: Vec::new(),
            }),
            Box::new(QueryError::UnknownType {
                given: text(),
                closest: None,
                types: Vec::new(),
            }),
            Box::new(QueryError::NoSource {
                path: text(),
                missing: true,
                query: None,
            }),
            Box::new(QueryError::NoSource {
                path: text(),
                missing: false,
                query: None,
            }),
            Box::new(QueryError::Scope(ScopeError::Outside {
                path: path(),
                config: path(),
            })),
            Box::new(ReleaseError::NoLines { target: text() }),
            Box::new(ReleaseError::BadLine {
                target: text(),
                line: text(),
                why: text(),
            }),
            Box::new(ReleaseError::AlreadyLive {
                target: text(),
                line: text(),
            }),
            Box::new(ReleaseError::Archived {
                target: text(),
                line: text(),
            }),
            Box::new(ReleaseError::NotOldestPlanned {
                target: text(),
                line: text(),
                oldest: text(),
            }),
            Box::new(ReleaseError::Unplanned {
                target: text(),
                line: text(),
                why: text(),
            }),
            Box::new(ReleaseError::BadNext {
                next: text(),
                why: text(),
            }),
            Box::new(ReleaseError::MarkUnrewritable {
                path: text(),
                line: 1,
                written: text(),
            }),
            Box::new(ReleaseError::ModelEdit(text())),
            Box::new(ReleaseError::ModelInvalid(text())),
            Box::new(ReleaseError::Build(text())),
            Box::new(ReleaseError::OutputsChanged {
                step: "a step",
                files: vec![text()],
            }),
            Box::new(RetireError::NoLines { target: text() }),
            Box::new(RetireError::NotLive {
                target: text(),
                line: text(),
                why: text(),
            }),
            Box::new(RetireError::Current {
                target: text(),
                line: text(),
                oldest: text(),
            }),
            Box::new(RetireError::NotOldest {
                target: text(),
                line: text(),
                oldest: text(),
            }),
            Box::new(RetireError::NotVersioned { target: text() }),
            Box::new(RetireError::NoAddress {
                target: text(),
                line: text(),
            }),
            Box::new(RetireError::MarkUnrewritable {
                path: text(),
                line: 1,
                written: text(),
            }),
            Box::new(RetireError::ModelEdit(text())),
            Box::new(RetireError::ModelInvalid(text())),
            Box::new(RetireError::Build(text())),
            Box::new(RetireError::OutputsChanged {
                files: vec![text()],
            }),
            Box::new(RetireError::OtherBuildsChange {
                target: text(),
                builds: vec![text()],
            }),
            Box::new(VersionsError::BadLine(text())),
            Box::new(VersionsError::BadNext(text())),
            Box::new(VersionsError::NoGit {
                command: "release",
                why: text(),
            }),
            Box::new(VersionsError::NotRestorable {
                command: "release",
                files: vec![text()],
            }),
            Box::new(VersionsError::Unwritable {
                path: path(),
                message: text(),
            }),
            Box::new(VersionsError::ArchiveNoBuild { target: text() }),
            Box::new(VersionsError::ArchiveBuildRequired {
                target: text(),
                builds: vec![text()],
            }),
            Box::new(VersionsError::ArchiveUnknownBuild {
                build: text(),
                target: text(),
                builds: vec![text()],
            }),
            Box::new(VersionsError::ArchiveLineArchived {
                target: text(),
                line: text(),
            }),
            Box::new(VersionsError::ArchiveErrors {
                count: 1,
                build: text(),
                listed: text(),
            }),
            Box::new(RenameError::Missing(text())),
            Box::new(RenameError::Refused(text())),
            Box::new(RenameCommandError::BadTarget {
                given: text(),
                reason: text(),
            }),
            Box::new(RenameCommandError::Unwritable {
                path: path(),
                message: text(),
            }),
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
