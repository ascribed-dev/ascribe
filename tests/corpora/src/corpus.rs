//! The three corpora, and fetching them.
//!
//! Nothing here is vendored. [`fetch`] makes a shallow, sparse clone of a
//! repository at the commit pinned in [`Corpus::commit`], into a cache
//! directory (`ASCRIBE_CORPORA_DIR`, else `target/corpora` in the workspace),
//! and reuses it while the checkout is at that commit.
//!
//! Tests that need a corpus call [`corpus_or_skip`]: with no network (or no
//! `git`) they print why they were skipped and pass, so `cargo test
//! --workspace` is green offline. `ASCRIBE_CORPORA` controls it:
//!
//! - unset: fetch if needed, skip if that fails;
//! - `require`: a corpus that can't be fetched fails the test (the CI job);
//! - `skip`: never touch the network, use only a checkout already in the cache.

use std::fmt;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A repository whose documentation is a corpus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Corpus {
    /// `withastro/docs`: MDX, with Starlight asides, `<Tabs>`, and `<Steps>`.
    Astro,
    /// `elastic/docs-content`: Markdown with `docs-builder` directives
    /// (`:::{note}`, tab sets), `{{substitutions}}`, and `applies_to`.
    Elastic,
    /// `docker/docs`: Hugo Markdown with shortcodes and GitHub alerts.
    Docker,
}

/// The license of a corpus's repository, as recorded in the repository.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct License {
    /// The SPDX identifier.
    pub spdx: &'static str,
    /// Where the repository states it.
    pub source: &'static str,
    /// Whether text of the corpus, or anything derived from it, may be
    /// committed to this repository as a fixture. See `tests/corpora/LICENSES.md`.
    pub may_commit_content: bool,
}

impl Corpus {
    /// Every corpus, in a fixed order.
    pub const ALL: [Corpus; 3] = [Corpus::Astro, Corpus::Elastic, Corpus::Docker];

    /// The short name, used for directories and in reports.
    pub fn name(self) -> &'static str {
        match self {
            Corpus::Astro => "astro",
            Corpus::Elastic => "elastic",
            Corpus::Docker => "docker",
        }
    }

    /// The corpus with this short name.
    pub fn from_name(name: &str) -> Option<Corpus> {
        Corpus::ALL.into_iter().find(|c| c.name() == name)
    }

    /// The repository's clone URL.
    pub fn repository(self) -> &'static str {
        match self {
            Corpus::Astro => "https://github.com/withastro/docs.git",
            Corpus::Elastic => "https://github.com/elastic/docs-content.git",
            Corpus::Docker => "https://github.com/docker/docs.git",
        }
    }

    /// The pinned commit (2026-09-29, the head of each default branch then).
    /// Bumping it changes the recognition baselines; see `tests/corpora/README.md`.
    pub fn commit(self) -> &'static str {
        match self {
            Corpus::Astro => "e750d92a8e8309e5a968e3450403c8ccb884701d",
            Corpus::Elastic => "651711d37722c1fcad9623d46e4a34eeeedfa008",
            Corpus::Docker => "3633800c79c473180d51ff64f7dffb05ccfb92c9",
        }
    }

    /// The sparse-checkout patterns (`.gitignore` syntax, as `git
    /// sparse-checkout` takes them): the documentation's text, not its images.
    pub fn sparse_patterns(self) -> &'static [&'static str] {
        match self {
            Corpus::Astro => &["/LICENSE", "/src/content/docs/en/**/*.mdx"],
            Corpus::Elastic => &[
                "/LICENSE",
                "/docset.yml",
                "/*.md",
                "/deploy-manage/**/*.md",
                "/explore-analyze/**/*.md",
                "/solutions/**/*.md",
                "/reference/**/*.md",
                "/troubleshoot/**/*.md",
                "/manage-data/**/*.md",
                "/get-started/**/*.md",
                "/cloud-account/**/*.md",
                "/release-notes/**/*.md",
                "/contribute-docs/**/*.md",
            ],
            Corpus::Docker => &["/LICENSE", "/content/**/*.md"],
        }
    }

    /// The directory of the documentation in the checkout.
    pub fn content_dir(self) -> &'static str {
        match self {
            Corpus::Astro => "src/content/docs/en",
            Corpus::Elastic => ".",
            Corpus::Docker => "content",
        }
    }

    /// The file extensions of the corpus's pages.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Corpus::Astro => &["mdx"],
            Corpus::Elastic | Corpus::Docker => &["md"],
        }
    }

    /// Repository-root-relative paths that aren't documentation pages even
    /// though they match the extension (contributor files, agent instructions).
    pub fn is_not_a_page(self, relative: &str) -> bool {
        match self {
            Corpus::Elastic => {
                !relative.contains('/')
                    && matches!(
                        relative,
                        "README.md" | "AGENTS.md" | "AI.md" | "CLAUDE.md" | "GEMINI.md"
                    )
            }
            _ => false,
        }
    }

    /// The repository's license. `LICENSES.md` has the reasoning.
    pub fn license(self) -> License {
        match self {
            Corpus::Astro => License {
                spdx: "MIT",
                source: "LICENSE at the repository root (no separate content license)",
                may_commit_content: true,
            },
            Corpus::Docker => License {
                spdx: "Apache-2.0",
                source: "LICENSE at the repository root; README: \"released under the Apache 2.0 license\"",
                may_commit_content: true,
            },
            Corpus::Elastic => License {
                spdx: "CC-BY-NC-ND-4.0",
                source: "LICENSE at the repository root; README: \"licensed under a Creative Commons Attribution-NonCommercial-NoDerivs 4.0 International License\"",
                may_commit_content: false,
            },
        }
    }
}

impl fmt::Display for Corpus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A corpus checked out in the cache.
#[derive(Clone, Debug)]
pub struct Fetched {
    /// Which corpus.
    pub corpus: Corpus,
    /// The root of the checkout.
    pub root: PathBuf,
}

impl Fetched {
    /// The directory of the documentation.
    pub fn content(&self) -> PathBuf {
        self.root.join(self.corpus.content_dir())
    }
}

/// Why a corpus couldn't be fetched.
#[derive(Debug)]
pub enum FetchError {
    /// `git` isn't installed, or the network or the host is unreachable: the
    /// case tests skip on.
    Unavailable(String),
    /// Anything else (a commit that no longer exists, a full disk).
    Failed(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetchError::Unavailable(m) | FetchError::Failed(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for FetchError {}

/// The cache directory: `ASCRIBE_CORPORA_DIR`, else `target/corpora` of the
/// workspace.
pub fn cache_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("ASCRIBE_CORPORA_DIR") {
        return PathBuf::from(dir);
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    // `tests/corpora` is two levels below the workspace root.
    let workspace = manifest.ancestors().nth(2).unwrap_or(manifest);
    workspace.join("target").join("corpora")
}

/// How long one `git` command may take before it's given up on.
const GIT_TIMEOUT: Duration = Duration::from_secs(300);

fn git(dir: &Path, args: &[&str]) -> Result<String, FetchError> {
    let mut child = Command::new("git")
        .current_dir(dir)
        // Give up on a stalled connection rather than hanging offline.
        .args([
            "-c",
            "http.lowSpeedLimit=1000",
            "-c",
            "http.lowSpeedTime=30",
        ])
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| FetchError::Unavailable(format!("can't run git: {e}")))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > GIT_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(FetchError::Unavailable(format!(
                    "git {} took longer than {}s",
                    args.first().copied().unwrap_or(""),
                    GIT_TIMEOUT.as_secs()
                )));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(FetchError::Failed(format!("waiting for git: {e}"))),
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| FetchError::Failed(format!("reading git's output: {e}")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if out.status.success() {
        return Ok(stdout);
    }
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_owned();
    let message = format!("git {}: {stderr}", args.first().copied().unwrap_or(""));
    let network = [
        "Could not resolve host",
        "unable to access",
        "Failed to connect",
        "Connection refused",
        "Connection timed out",
        "Network is unreachable",
        "early EOF",
        "RPC failed",
        "Proxy",
        "SSL",
        "TLS",
    ];
    if args.first() == Some(&"fetch") && network.iter().any(|n| stderr.contains(n)) {
        Err(FetchError::Unavailable(message))
    } else {
        Err(FetchError::Failed(message))
    }
}

/// The checkout of `corpus` in the cache, if it's there and at the pinned
/// commit. Never touches the network.
pub fn cached(corpus: Corpus) -> Option<Fetched> {
    let root = cache_dir().join(corpus.name());
    if !root.join(".git").exists() {
        return None;
    }
    let head = git(&root, &["rev-parse", "HEAD"]).ok()?;
    // The marker is written after the checkout finishes, so an interrupted
    // fetch is never taken for a complete one.
    (head == corpus.commit() && root.join(".ascribe-fetched").exists())
        .then_some(Fetched { corpus, root })
}

/// Fetches `corpus` into the cache (a shallow, sparse clone at the pinned
/// commit), unless it's already there.
///
/// # Errors
///
/// [`FetchError::Unavailable`] when `git` or the network isn't, and
/// [`FetchError::Failed`] for anything else.
pub fn fetch(corpus: Corpus) -> Result<Fetched, FetchError> {
    if let Some(found) = cached(corpus) {
        return Ok(found);
    }
    let root = cache_dir().join(corpus.name());
    if root.exists() {
        std::fs::remove_dir_all(&root)
            .map_err(|e| FetchError::Failed(format!("removing {}: {e}", root.display())))?;
    }
    std::fs::create_dir_all(&root)
        .map_err(|e| FetchError::Failed(format!("creating {}: {e}", root.display())))?;
    git(&root, &["init", "-q"])?;
    git(&root, &["remote", "add", "origin", corpus.repository()])?;
    git(&root, &["sparse-checkout", "init", "--no-cone"])?;
    let patterns = root.join(".git").join("info").join("sparse-checkout");
    std::fs::write(&patterns, corpus.sparse_patterns().join("\n") + "\n")
        .map_err(|e| FetchError::Failed(format!("writing {}: {e}", patterns.display())))?;
    // Shallow (one commit) and blobless: the checkout downloads only the files
    // the patterns select.
    git(
        &root,
        &[
            "fetch",
            "-q",
            "--depth",
            "1",
            "--filter=blob:none",
            "origin",
            corpus.commit(),
        ],
    )?;
    git(&root, &["checkout", "-q", "FETCH_HEAD"])?;
    std::fs::write(root.join(".ascribe-fetched"), corpus.commit())
        .map_err(|e| FetchError::Failed(format!("writing the marker: {e}")))?;
    Ok(Fetched { corpus, root })
}

/// What a test does about a corpus it can't have.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Fetch if needed; skip when that's impossible.
    Auto,
    /// Fail when the corpus can't be had.
    Require,
    /// Use only the cache.
    CacheOnly,
}

impl Mode {
    /// The mode `ASCRIBE_CORPORA` selects.
    pub fn from_env() -> Mode {
        match std::env::var("ASCRIBE_CORPORA").as_deref() {
            Ok("require") => Mode::Require,
            Ok("skip") => Mode::CacheOnly,
            _ => Mode::Auto,
        }
    }
}

/// The corpus, or `None` after printing why the test is skipping. Panics in
/// `require` mode, where a missing corpus is a failure.
// The panic is the point of `require` mode: this is a test helper, and a corpus
// the CI job needs but can't get must fail the job.
#[allow(clippy::panic)]
pub fn corpus_or_skip(corpus: Corpus) -> Option<Fetched> {
    let mode = Mode::from_env();
    let result = match mode {
        Mode::CacheOnly => cached(corpus).ok_or_else(|| {
            FetchError::Unavailable("ASCRIBE_CORPORA=skip and the corpus isn't cached".into())
        }),
        Mode::Auto | Mode::Require => fetch(corpus),
    };
    match result {
        Ok(found) => Some(found),
        Err(e) if mode == Mode::Require => {
            panic!("the {corpus} corpus is required (ASCRIBE_CORPORA=require) but: {e}")
        }
        Err(e) => {
            // Written to stderr directly, not with `eprintln!`, so the test
            // harness doesn't swallow it: a skip should be visible.
            let _ = writeln!(
                std::io::stderr(),
                "SKIPPED: the {corpus} corpus isn't available ({e}). The test passes without \
                 checking anything; set ASCRIBE_CORPORA=require to make this a failure."
            );
            None
        }
    }
}

/// Every page of the corpus, as `(path relative to the content directory,
/// text)`, in path order. Files that aren't UTF-8 are left out (there are
/// none in the pinned commits; a test checks that).
///
/// # Errors
///
/// Any error walking or reading the checkout.
pub fn pages(fetched: &Fetched) -> std::io::Result<Vec<(String, String)>> {
    let content = fetched.content();
    let mut out = Vec::new();
    let mut stack = vec![content.clone()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') {
                continue;
            }
            if entry.file_type()?.is_dir() {
                stack.push(path);
                continue;
            }
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !fetched.corpus.extensions().contains(&extension) {
                continue;
            }
            let Ok(relative) = path.strip_prefix(&content) else {
                continue;
            };
            let relative = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if fetched.corpus.is_not_a_page(&relative) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&path) {
                out.push((relative, text));
            }
        }
    }
    out.sort();
    Ok(out)
}
