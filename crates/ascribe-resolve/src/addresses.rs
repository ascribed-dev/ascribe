//! The addresses a build publishes, which `ascribe.addresses` keeps (SPEC
//! §7.10): each page's route, its Markdown version's when the build
//! publishes one, and each explicit `@id`'s `route#id`. A heading's
//! generated slug isn't one: it changes with its text.
//!
//! Each address has its **source**, the key `ascribe.addresses` records it
//! by: a page's content path, or, for an id, the content path of the file
//! the `@id` is written in, `#`, and the id, so a fragment's id is one
//! source however many pages include it. The listing is what the page
//! records compute, in one pass over the resolved build.
//!
//! Two sources at one address in a build is listed as it is: the listing
//! returns both, and the comparison reports neither.

use ascribe_core::RelPath;
use ascribe_model::Build;

use crate::AstroRouter;
use crate::build::ResolvedBuild;
use crate::project::Project;

/// What an address is the address of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AddressKind {
    /// A page's route.
    Page,
    /// A page's Markdown version.
    Markdown,
    /// A heading's explicit `@id` on a page: `route#id`.
    Anchor,
}

/// One address a build publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Address {
    /// The address, from the host's root, in the published case and slash
    /// form; an anchor's is its page's route, `#`, and the id.
    pub address: String,
    /// What it's the address of.
    pub kind: AddressKind,
    /// The key `ascribe.addresses` records it by: the page's content path,
    /// or, for an anchor, the content path of the file the `@id` is written
    /// in, `#`, and the id.
    pub source: String,
    /// The page it's on.
    pub page: RelPath,
}

/// Every address one build publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildAddresses {
    /// The build's name.
    pub build: String,
    /// Its addresses: each page's, in path order, then its Markdown
    /// version's, then its anchors', in document order.
    pub addresses: Vec<Address>,
}

impl BuildAddresses {
    /// Whether the build publishes `address`.
    pub fn has(&self, address: &str) -> bool {
        self.addresses.iter().any(|a| a.address == address)
    }

    /// Whether the build publishes the page at `path`.
    pub fn publishes(&self, path: &RelPath) -> bool {
        self.addresses
            .iter()
            .any(|a| a.kind == AddressKind::Page && &a.page == path)
    }
}

/// The addresses `build` publishes, resolving it. `index` is the project's
/// source index, as loaded; an older line's addresses are under its folder.
pub fn build_addresses(index: &Project, build: &Build) -> BuildAddresses {
    let index = index.for_build(build);
    let router = AstroRouter::from_consumer(&index.model().consumer);
    let resolved = index.resolve_build(build, &router);
    of_resolved(&index, &resolved)
}

/// The addresses of every build of the project, in the content model's
/// order.
pub fn every_build(index: &Project) -> Vec<BuildAddresses> {
    index
        .model()
        .builds
        .iter()
        .map(|build| build_addresses(index, build))
        .collect()
}

/// The addresses of a build already resolved, with `index` the project its
/// pages are routed in ([`Project::for_build`]).
pub fn of_resolved(index: &Project, resolved: &ResolvedBuild) -> BuildAddresses {
    let model = index.model();
    let router = AstroRouter::from_consumer(&model.consumer);
    let mut addresses = Vec::new();
    for page in &resolved.pages {
        let source = page.path.to_string();
        addresses.push(Address {
            address: page.route.clone(),
            kind: AddressKind::Page,
            source: source.clone(),
            page: page.path.clone(),
        });
        if model.consumer.agents {
            addresses.push(Address {
                address: router.markdown_url(&page.path),
                kind: AddressKind::Markdown,
                source,
                page: page.path.clone(),
            });
        }
        for (_, ids) in page.headings() {
            if !ids.explicit {
                continue;
            }
            let written = ids
                .explicit_at
                .and_then(|at| index.path_of(at.file))
                .unwrap_or(&page.path);
            addresses.push(Address {
                address: format!("{}#{}", page.route, ids.page_id),
                kind: AddressKind::Anchor,
                source: format!("{written}#{}", ids.page_id),
                page: page.path.clone(),
            });
        }
    }
    BuildAddresses {
        build: resolved.build.clone(),
        addresses,
    }
}
