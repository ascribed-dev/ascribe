//! The `astro` profile's router (SPEC §9.5, content-model.md §16): which URL
//! each page has on a site whose content collection is the site output.
//!
//! A page's route is the `[consumer] base-path`, then its **entry id**, then
//! the trailing slash the profile's `trailing-slash` says. The entry id is
//! what Astro's `glob` loader computes for the file: its path relative to the
//! collection's base, without the extension, with each segment slugged by
//! `github-slugger` (the pure `slug()`, without numbering), joined with `/`,
//! and a final `/index` removed (`getContentEntryIdAndSlug`, Astro 7.3, the
//! version phase 20 targets). So `Guides/My Setup.md` is `guides/my-setup`
//! and `guides/index.md` is `guides`. The regex needs the `/`, so the root
//! `index.md` keeps the id `index`, and so does `index/index.md`. Ascribe's
//! route for the id `index` is the base path (content-model.md §16: a final
//! `index` segment is dropped), so those two pages collide, as they do in
//! Astro. Under `trailing-slash = "never"` the base path has no trailing
//! slash (Astro's `BASE_URL`, and the build's URL for the root page), unless
//! it is `/`.
//!
//! Two source files can have the same entry id (`My File.md` and
//! `my-file.md`); [`AstroRouter::collisions`] finds them, and the site output
//! refuses to build with them.
//!
//! This lives here, not in `tessera-emit`, so the source index and the
//! checks can ask which page a route names ([`AstroRouter::page_for_route`])
//! without a dependency on the emitters.

use std::collections::BTreeMap;

use tessera_core::{RelPath, Router};
use tessera_model::{Consumer, TrailingSlash};

use crate::slug::github_slug;

/// Routes pages the way Astro's content collection does, under the model's
/// base path and trailing-slash policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AstroRouter {
    base: String,
    trailing_slash: bool,
}

impl AstroRouter {
    /// A router under `[consumer]`'s `base-path` and `trailing-slash`.
    pub fn from_consumer(consumer: &Consumer) -> AstroRouter {
        AstroRouter::with_base(
            &consumer.base_path,
            consumer.trailing_slash == TrailingSlash::Always,
        )
    }

    /// A router under a base path, such as `/docs/` (a missing leading or
    /// trailing slash is added).
    pub fn with_base(base: &str, trailing_slash: bool) -> AstroRouter {
        let mut base = base.trim().to_owned();
        if !base.starts_with('/') {
            base.insert(0, '/');
        }
        if !base.ends_with('/') {
            base.push('/');
        }
        AstroRouter {
            base,
            trailing_slash,
        }
    }

    /// The base path: it starts and ends with `/`.
    pub fn base(&self) -> &str {
        &self.base
    }

    /// Whether routes end in `/`.
    pub fn trailing_slash(&self) -> bool {
        self.trailing_slash
    }

    /// The entry id Astro's `glob` loader gives a page: its path without the
    /// extension, each segment slugged, and a final `/index` removed. The root
    /// `index.md` (and `index/index.md`) is `index`.
    pub fn entry_id(page: &RelPath) -> String {
        // The extension is removed before slugging: `a.b.md` is `a.b`, then
        // `ab`.
        let count = page.segments().count();
        let slugged: Vec<String> = page
            .segments()
            .enumerate()
            .map(|(n, segment)| match segment.rfind('.') {
                Some(dot) if n + 1 == count => github_slug(&segment[..dot]),
                _ => github_slug(segment),
            })
            .collect();
        let id = slugged.join("/");
        match id.strip_suffix("/index") {
            Some(rest) => rest.to_owned(),
            None => id,
        }
    }

    /// The page in `pages` that has this route, if any. `route` is a URL
    /// path: with the base path or without it, with or without a trailing
    /// slash, and any `#fragment` or `?query` is ignored. Percent-encoding is
    /// decoded. When several pages have the route ([`AstroRouter::collisions`]),
    /// the first in `pages` order wins.
    pub fn page_for_route<'a>(
        &self,
        route: &str,
        pages: impl IntoIterator<Item = &'a RelPath>,
    ) -> Option<&'a RelPath> {
        let wanted = self.route_id(route)?;
        pages
            .into_iter()
            .find(|page| AstroRouter::entry_id(page) == wanted)
    }

    /// The entry id a route path names: the base path (when it starts with
    /// it) and the slashes around the rest removed.
    fn route_id(&self, route: &str) -> Option<String> {
        let path = route.split(['#', '?']).next().unwrap_or("");
        let path = tessera_core::percent_decode(path);
        let base = self.base.trim_end_matches('/');
        let rest = match path.strip_prefix(base) {
            Some(rest) if rest.is_empty() || rest.starts_with('/') => rest,
            _ => path.as_str(),
        };
        let id = rest.trim_matches('/');
        // The base path is the route of the entry `index`.
        Some(if id.is_empty() { "index" } else { id }.to_owned())
    }

    /// The groups of pages that share a route, each with its route, in path
    /// order. Empty when every page has its own.
    pub fn collisions<'a>(
        &self,
        pages: impl IntoIterator<Item = &'a RelPath>,
    ) -> Vec<(String, Vec<&'a RelPath>)> {
        let mut by_route: BTreeMap<String, Vec<&RelPath>> = BTreeMap::new();
        for page in pages {
            by_route.entry(self.route(page)).or_default().push(page);
        }
        by_route.into_iter().filter(|(_, p)| p.len() > 1).collect()
    }
}

impl Router for AstroRouter {
    fn route(&self, page: &RelPath) -> String {
        let id = AstroRouter::entry_id(page);
        let mut url = self.base.clone();
        if id == "index" {
            if !self.trailing_slash && url.len() > 1 {
                url.pop();
            }
            return url;
        }
        let encoded: Vec<String> = id.split('/').map(encode_segment).collect();
        url.push_str(&encoded.join("/"));
        if self.trailing_slash {
            url.push('/');
        }
        url
    }
}

/// Percent-encodes a path segment: everything but unreserved characters.
pub(crate) fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(path: &str) -> RelPath {
        RelPath::parse(path).expect("a path")
    }

    fn route(router: &AstroRouter, path: &str) -> String {
        router.route(&page(path))
    }

    #[test]
    fn entry_ids_follow_astros_glob_loader() {
        let id = |p: &str| AstroRouter::entry_id(&page(p));
        // The regex needs a `/`: the root page keeps `index`, as does
        // `index/index.md`.
        assert_eq!(id("index.md"), "index");
        assert_eq!(id("index/index.md"), "index");
        assert_eq!(id("keys.md"), "keys");
        assert_eq!(id("guides/index.md"), "guides");
        assert_eq!(id("Guides/My Setup.md"), "guides/my-setup");
        assert_eq!(id("install_agent.md"), "install_agent");
        // Only the last extension is removed; the dot in what's left is
        // dropped by the slug.
        assert_eq!(id("v1.2/notes.md"), "v12/notes");
        assert_eq!(id("release.notes.md"), "releasenotes");
        assert_eq!(id("a/Index.md"), "a");
    }

    #[test]
    fn routes_use_the_base_path_and_trailing_slash() {
        let with = AstroRouter::with_base("/docs/", true);
        assert_eq!(route(&with, "index.md"), "/docs/");
        assert_eq!(route(&with, "guides/setup.md"), "/docs/guides/setup/");
        assert_eq!(route(&with, "guides/index.md"), "/docs/guides/");
        let without = AstroRouter::with_base("docs", false);
        assert_eq!(route(&without, "guides/setup.md"), "/docs/guides/setup");
        // With no trailing slash, the root page is the base without one,
        // unless the base is `/` (Astro's `BASE_URL`).
        assert_eq!(route(&without, "index.md"), "/docs");
        assert_eq!(route(&AstroRouter::with_base("/", false), "index.md"), "/");
        assert_eq!(
            route(&AstroRouter::with_base("/", true), "keys.md"),
            "/keys/"
        );
    }

    #[test]
    fn segments_are_slugged_then_percent_encoded() {
        let router = AstroRouter::with_base("/", true);
        assert_eq!(route(&router, "My Guides/A#B.md"), "/my-guides/ab/");
        assert_eq!(route(&router, "café.md"), "/caf%C3%A9/");
    }

    #[test]
    fn a_route_names_the_page_that_has_it() {
        let router = AstroRouter::with_base("/docs/", true);
        let pages = [
            page("index.md"),
            page("guides/My Setup.md"),
            page("keys.md"),
        ];
        let find = |route: &str| router.page_for_route(route, &pages).map(RelPath::to_string);
        assert_eq!(
            find("/docs/guides/my-setup/").as_deref(),
            Some("guides/My Setup.md")
        );
        assert_eq!(
            find("/docs/guides/my-setup").as_deref(),
            Some("guides/My Setup.md")
        );
        assert_eq!(
            find("/guides/my-setup/#x").as_deref(),
            Some("guides/My Setup.md")
        );
        assert_eq!(find("/docs/").as_deref(), Some("index.md"));
        assert_eq!(find("/docs/keys/?a=1").as_deref(), Some("keys.md"));
        assert_eq!(find("/docs/nope/"), None);
    }

    #[test]
    fn pages_that_share_a_route_are_found() {
        let router = AstroRouter::with_base("/", true);
        let pages = [page("My File.md"), page("my-file.md"), page("keys.md")];
        let found = router.collisions(&pages);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "/my-file/");
        assert_eq!(found[0].1.len(), 2);
        // Astro gives `index.md` and `index/index.md` the same id.
        let roots = [page("index.md"), page("index/index.md")];
        let found = router.collisions(&roots);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "/");
    }
}
