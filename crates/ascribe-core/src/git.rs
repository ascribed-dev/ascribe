//! What every `git` Ascribe starts runs with, so that reading a repository
//! runs nothing the repository supplies (decision 76).
//!
//! A repository's own configuration can name commands that `git` runs
//! while it reads: a file system monitor, a pager, an external diff, a text
//! conversion filter, hooks. The configuration in `.git/config` is the
//! user's, written on their machine. A committed folder shaped like a bare
//! repository (`HEAD`, `objects/`, `refs/`, and a `config`) is the
//! repository's author's: `git` started in it, or below it, takes it for
//! the repository and reads its `config`. These options stop that.
//!
//! Each crate that starts `git` adds [`OPTIONS`], each as `-c <option>`,
//! and clears [`CLEARED`], before its own options and arguments.
//! `packages/review/src/shared/git.ts` holds the same two lists for the
//! TypeScript packages, and `packages/review/test/git.test.ts` compares
//! them with these.

/// The options, each given as `-c <option>`. Options on the command line
/// are "protected configuration", which `safe.bareRepository` is read only
/// from.
///
/// Checked against `git`'s documentation, and these commands, which are
/// the ones Ascribe runs: `rev-parse`, `ls-tree`, `cat-file --batch`,
/// `merge-base`, `diff --name-status` and `status --porcelain` (the
/// compiler), and `symbolic-ref`, `config --get`, `remote -v`, `rev-list`,
/// and `diff` with patches (the TypeScript packages). Of what a
/// configuration can make them run:
///
/// - A pager starts only when standard output is a terminal, and Ascribe
///   always reads it through a pipe.
/// - An external diff and a text conversion filter apply only to a diff
///   with patches, which only `packages/review` asks for; it passes
///   `--no-ext-diff`.
/// - None of these commands runs a hook.
/// - `diff` and `status` ask the file system monitor, which
///   `core.fsmonitor` names; `core.fsmonitor=false` stops that.
///
/// Every one of those is named in a configuration file, and the one the
/// repository's author can commit is a planted bare repository's, which
/// `safe.bareRepository=explicit` keeps `git` from reading at all.
pub const OPTIONS: &[&str] = &[
    // A bare repository is used only when it's named with `--git-dir` or
    // `GIT_DIR`, never found by looking up from the working folder: a
    // committed folder shaped like one is never taken for the repository,
    // and its `config` is never read. `ascribe-sources` names its cache
    // with `GIT_DIR`, so it's unaffected.
    "safe.bareRepository=explicit",
    // No file system monitor, which `diff` and `status` start to ask what
    // changed: it's a command the configuration names.
    "core.fsmonitor=false",
];

/// Environment variables cleared for every `git` Ascribe starts: each would
/// point it at another repository, index, or working tree than the one it
/// finds from the folder it runs in. `ascribe-sources` sets `GIT_DIR` to
/// its cache's repository after clearing it.
pub const CLEARED: &[&str] = &[
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_NAMESPACE",
    "GIT_COMMON_DIR",
];
