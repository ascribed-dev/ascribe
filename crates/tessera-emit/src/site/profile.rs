//! The `astro` consumer profile (SPEC §9.5, §9.6): how the site output fits
//! Astro's content collections.
//!
//! - **Routing:** `[consumer]`'s `base-path` and `trailing-slash`, and the
//!   entry ids Astro's `glob` loader gives files ([`AstroRouter`]).
//! - **Slugger:** `github`.
//! - **HTML passthrough:** always on (the profile supports only `html = true`).
//! - **Assets:** images stay beside their page, referenced by a relative
//!   path, so Astro's image processing applies; other files a page links to
//!   are published under `_tessera/files/` (asset contract §3.2).
//! - **Heading ids and image attributes:** the site-render contract's
//!   `<tessera-attributes>` marker, which the Astro markdown plugin applies
//!   (phase 21). They aren't settings.

use tessera_core::{AssetPlacement, AssetUse, ConsumerProfile, Router, Slugger};
use tessera_model::Consumer;
use tessera_resolve::AstroRouter;
use tessera_resolve::slug::GithubSlugger;

/// The Astro version the profile is written against: the `glob` loader's entry
/// ids, `astro/zod`, and how images in collection entries are processed.
pub const ASTRO_VERSION: &str = "7.3.5";

/// The `astro` profile.
#[derive(Clone, Debug)]
pub struct AstroProfile {
    router: AstroRouter,
    slugger: GithubSlugger,
}

impl AstroProfile {
    /// The profile for a content model's `[consumer]` table.
    pub fn from_consumer(consumer: &Consumer) -> AstroProfile {
        AstroProfile {
            router: AstroRouter::from_consumer(consumer),
            slugger: GithubSlugger,
        }
    }

    /// The router, as its own type: it can also say which page has a route.
    pub fn astro_router(&self) -> &AstroRouter {
        &self.router
    }
}

impl ConsumerProfile for AstroProfile {
    fn name(&self) -> &str {
        "astro"
    }

    fn router(&self) -> &dyn Router {
        &self.router
    }

    fn slugger(&self) -> &dyn Slugger {
        &self.slugger
    }

    fn html_passthrough(&self) -> bool {
        true
    }

    fn asset_placement(&self, usage: AssetUse) -> AssetPlacement {
        match usage {
            AssetUse::Image => AssetPlacement::Mirror,
            AssetUse::Link => AssetPlacement::Published,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_are_mirrored_and_other_files_published() {
        let profile = AstroProfile::from_consumer(&Consumer {
            profile: "astro".to_owned(),
            site: None,
            base_path: "/docs/".to_owned(),
            trailing_slash: tessera_model::TrailingSlash::Always,
            slugger: "github".to_owned(),
            html: true,
        });
        assert_eq!(profile.name(), "astro");
        assert!(profile.html_passthrough());
        assert_eq!(profile.slugger().name(), "github");
        assert_eq!(
            profile.asset_placement(AssetUse::Image),
            AssetPlacement::Mirror
        );
        assert_eq!(
            profile.asset_placement(AssetUse::Link),
            AssetPlacement::Published
        );
    }
}
