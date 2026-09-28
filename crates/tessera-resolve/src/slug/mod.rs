//! Heading slugs (SPEC §5.5, §9.5): the algorithms behind `tessera-core`'s
//! [`Slugger`] trait, and a registry that finds one by the name
//! `[consumer] slugger` uses.
//!
//! Spec 0.1 has one algorithm, `github`, a port of the
//! [`github-slugger`](https://www.npmjs.com/package/github-slugger) npm
//! package, which Astro uses for heading ids.
//!
//! # Upstream version
//!
//! The port matches **`github-slugger` 2.0.0**, the latest release when it was
//! written ([`GITHUB_SLUGGER_VERSION`]). Its Unicode table and the test
//! fixtures under `fixtures/` are generated from that package's real output by
//! `generate.mjs`; when upstream changes, rerun the script against the new
//! version and update the constant.
//!
//! # Scopes
//!
//! Duplicates are numbered within a scope ([`Slugger::new_scope`]): one per
//! source file for source ids, one per expanded page for page ids. A new scope
//! starts empty, so starting over is always an explicit call. Explicit `@id`s
//! never go through a scope (SPEC §5.5).

mod github;
mod table;

use tessera_core::Slugger;

pub use github::{GithubSlugger, github_slug};

/// The `github-slugger` release the `github` slugger matches.
pub const GITHUB_SLUGGER_VERSION: &str = "2.0.0";

/// The slugger used when `[consumer] slugger` isn't set.
pub const DEFAULT_SLUGGER_NAME: &str = GithubSlugger::NAME;

/// The names of the built-in sluggers, [`DEFAULT_SLUGGER_NAME`] first.
pub const SLUGGER_NAMES: &[&str] = &[GithubSlugger::NAME];

/// Returns the slugger with this name, or `None` if there's no such slugger.
/// Names are exact: `github`, not `GitHub`.
pub fn slugger_by_name(name: &str) -> Option<Box<dyn Slugger>> {
    match name {
        GithubSlugger::NAME => Some(Box::new(GithubSlugger)),
        _ => None,
    }
}

/// Returns the default slugger, `github`.
pub fn default_slugger() -> Box<dyn Slugger> {
    Box::new(GithubSlugger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_finds_github_by_name() {
        let slugger = slugger_by_name("github").expect("github is registered");
        assert_eq!(slugger.name(), "github");
        assert!(slugger_by_name("GitHub").is_none());
        assert!(slugger_by_name("").is_none());
        assert!(slugger_by_name("hugo").is_none());
    }

    #[test]
    fn default_is_github_and_listed_first() {
        assert_eq!(default_slugger().name(), DEFAULT_SLUGGER_NAME);
        assert_eq!(SLUGGER_NAMES[0], DEFAULT_SLUGGER_NAME);
        for name in SLUGGER_NAMES {
            assert_eq!(
                slugger_by_name(name)
                    .map(|s| s.name().to_owned())
                    .as_deref(),
                Some(*name)
            );
        }
    }
}
