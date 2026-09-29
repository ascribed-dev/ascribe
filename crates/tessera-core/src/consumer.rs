//! The consumer profile (SPEC §9.5) and the pieces it's made of: heading
//! slugs, routes, and asset placement.
//!
//! These are traits so that the crates that need them (resolution, phase 12;
//! the emitters, phases 18 and 20) don't depend on the crates that implement
//! them (the slugger, phase 09; the Astro profile, phase 20). Spec 0.1 has one
//! profile, `astro`, selected by `[consumer] profile` in `ascribe.toml`.
//!
//! Two things SPEC §9.5 lists are not here, because the contracts fix them
//! for every profile in spec 0.1: how heading ids and image attributes are
//! written (`project-docs/contracts/site-render.md`), and HTML passthrough
//! beyond a yes or no.

use crate::RelPath;

/// Computes heading slugs (SPEC §5.5).
///
/// Implementations reproduce a consumer's algorithm exactly, including how it
/// numbers duplicates, because the slugs Ascribe validates must be the
/// anchors the consumer publishes. Phase 09 implements `github`, a port of
/// `github-slugger`, which Astro uses.
pub trait Slugger: Send + Sync {
    /// The name the content model uses for this algorithm (`[consumer] slugger`).
    fn name(&self) -> &str;

    /// Starts an empty scope. Duplicates are numbered within one scope: one
    /// per source file for source ids (phase 11), one per expanded page and
    /// build for page ids (phase 12).
    fn new_scope(&self) -> Box<dyn SlugScope + Send + '_>;
}

/// One scope of slugs, remembering the slugs it has given out.
pub trait SlugScope {
    /// The slug for a heading's text (with phrases already substituted),
    /// numbered if an earlier call in this scope produced the same slug:
    /// `intro`, then `intro-1`, and so on, as the algorithm numbers them.
    ///
    /// Only headings without `@id` go through the scope: an explicit id is
    /// not a slug and doesn't take part in numbering (SPEC §5.5).
    fn slug(&mut self, text: &str) -> String;
}

/// Maps pages to the consumer's URLs (SPEC §5.2, §9.5 "Routing").
pub trait Router: Send + Sync {
    /// The route of a page: a root-relative URL path that starts with `/`
    /// and includes the profile's base path, percent-encoded where URLs
    /// need it. `page` is the page's source path relative to the content
    /// root, such as `guides/setup.md`.
    ///
    /// A route depends only on the page's path, so it can be computed
    /// without reading the page.
    fn route(&self, page: &RelPath) -> String;

    /// A link to a page, or to a heading on it by page id: the route,
    /// followed by `#` and the id when there is one. Ids contain no
    /// whitespace, so they're inserted as they are.
    fn link(&self, page: &RelPath, id: Option<&str>) -> String {
        let mut url = self.route(page);
        if let Some(id) = id {
            url.push('#');
            url.push_str(id);
        }
        url
    }
}

/// How a page uses an asset (`project-docs/contracts/assets.md`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetUse {
    /// The source of an image.
    Image,
    /// The target of a link that isn't a page, such as a PDF or a sample file.
    Link,
}

/// Where the site output puts an asset's copy, and how it refers to it.
/// The rules for each are in `project-docs/contracts/assets.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AssetPlacement {
    /// At its mirrored path inside the emitter's output, referenced by a
    /// relative path from the page (`./`, `../`). The consumer's markdown
    /// pipeline resolves the reference, so its image processing applies.
    Mirror,
    /// Under the output's `_ascribe/files/` directory, which the consumer
    /// serves as static files, referenced by root-relative URL.
    Published,
}

/// A consumer profile (SPEC §9.5): how the site output fits one consumer.
///
/// Phase 20 implements the `astro` profile. The plain-markdown and JSON
/// outputs don't depend on any profile, except for routes in links.
pub trait ConsumerProfile: Send + Sync {
    /// The profile's name, as in `[consumer] profile`.
    fn name(&self) -> &str;

    /// How pages map to URLs.
    fn router(&self) -> &dyn Router;

    /// The heading-slug algorithm the consumer uses.
    fn slugger(&self) -> &dyn Slugger;

    /// Whether the consumer renders raw HTML in markdown. The site output's
    /// elements need it; the `astro` profile always returns `true` (Q11 in
    /// content-model.md).
    fn html_passthrough(&self) -> bool;

    /// Where the site output puts a copy of an asset used this way.
    /// For `astro`: images [`Mirror`](AssetPlacement::Mirror), links
    /// [`Published`](AssetPlacement::Published).
    fn asset_placement(&self, usage: AssetUse) -> AssetPlacement;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A toy router: base path plus the path without `.md`, then a slash.
    struct Toy;

    impl Router for Toy {
        fn route(&self, page: &RelPath) -> String {
            let path = page.as_str().trim_end_matches(".md");
            format!("/docs/{path}/")
        }
    }

    #[test]
    fn links_append_the_page_id() {
        let page = RelPath::parse("guides/keys.md").unwrap();
        assert_eq!(Toy.link(&page, None), "/docs/guides/keys/");
        assert_eq!(
            Toy.link(&page, Some("rotate-keys")),
            "/docs/guides/keys/#rotate-keys"
        );
    }

    /// The traits are object-safe and can be shared across threads.
    #[test]
    fn traits_are_usable_as_shared_objects() {
        fn shared<T: Send + Sync + ?Sized>() {}
        shared::<dyn Router>();
        shared::<dyn Slugger>();
        shared::<dyn ConsumerProfile>();
    }
}
