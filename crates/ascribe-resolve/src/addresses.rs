//! The addresses a build publishes, which `ascribe.addresses` keeps (SPEC
//! §7.10): each page's route, its Markdown version's when the build
//! publishes one, and each explicit `@id`'s `route#id`; each permalink's
//! address under the permalinks' path; and each `llms.txt` file the build
//! publishes for agents. A heading's generated slug isn't one: it changes
//! with its text.
//!
//! Each address has its **source**, the key `ascribe.addresses` records it
//! by: a page's content path, or, for an id, the content path of the file
//! the `@id` is written in, `#`, and the id, so a fragment's id is one
//! source however many pages include it. The listing is what the page
//! records compute, in one pass over the resolved build.
//!
//! Two sources at one address in a build is listed as it is: the listing
//! returns both, and the comparison reports neither.
//!
//! With the addresses, each build's [`Settings`]: the routing settings they
//! were made with, which tell a settings change (`[former]`) from a page
//! that's lost.

use ascribe_core::RelPath;
use ascribe_model::{Build, ContentModel, End, TrailingSlash};

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
    /// A permalink's address, under the permalinks' path: a name the build
    /// answers to, declared, aliased, or retired.
    Permalink,
    /// An `llms.txt` file, the root's or a section's.
    Llms,
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
    /// in, `#`, and the id. A permalink's is its name, and an `llms.txt`
    /// file's its path from the base path: neither is a key the file
    /// records.
    pub source: String,
    /// The page it's on: a permalink's page, and an `llms.txt` file's
    /// section's page, or the first it lists.
    pub page: RelPath,
}

/// Every address one build publishes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildAddresses {
    /// The build's name.
    pub build: String,
    /// The routing settings its addresses were made with.
    pub settings: Settings,
    /// Its addresses: each page's, in path order, then its Markdown
    /// version's, then its anchors', in document order; then each
    /// permalink's, in name order; then each `llms.txt` file's.
    pub addresses: Vec<Address>,
}

/// The routing settings a build's addresses were made with: those
/// `[former]` records (SPEC §7.10), as the content model has them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Settings {
    /// `[consumer] base-path`, starting and ending with `/`.
    pub base_path: String,
    /// The build's own base path, which its pages' and `llms.txt`'s
    /// addresses start with: `base_path`, and after it an older line's
    /// folder.
    pub base: String,
    /// `[consumer] permalink-path` as written; `None` for its default, `go/`
    /// under the base path.
    pub permalink_path: Option<String>,
    /// The path permalinks' addresses start with: `permalink-path`, or its
    /// default ([`ascribe_model::Consumer::permalink_path`]).
    pub permalinks: String,
    /// `[consumer] agents`: whether the build publishes Markdown versions
    /// and `llms.txt`.
    pub agents: bool,
    /// Whether routes, and permalinks' addresses, end in `/`.
    pub trailing_slash: bool,
}

impl Settings {
    /// The settings of `build` of a project with `model`, the model as
    /// `ascribe.toml` declares it, not one made for a line.
    pub fn of(model: &ContentModel, build: &Build) -> Settings {
        let consumer = &model.consumer;
        let base_path = AstroRouter::from_consumer(consumer).base().to_owned();
        let base = AstroRouter::from_consumer(&model.for_build(build).consumer)
            .base()
            .to_owned();
        Settings {
            base_path,
            base,
            permalink_path: consumer.permalink_path.clone(),
            permalinks: consumer.permalink_path(),
            agents: consumer.agents,
            trailing_slash: consumer.trailing_slash == TrailingSlash::Always,
        }
    }

    /// The address of the permalink `name`: the permalinks' path, the name,
    /// and `/` when routes end in one.
    pub fn permalink_address(&self, name: &str) -> String {
        let slash = if self.trailing_slash { "/" } else { "" };
        format!("{}{name}{slash}", self.permalinks)
    }
}

/// `address`, an address under `from`, a path that starts and ends with
/// `/`, as it is under `to`: the rest of it after `to`. The address of the
/// path itself, without its final `/`, is `to`'s, without its own. `None`
/// for an address outside `from`.
pub fn rebase(address: &str, from: &str, to: &str) -> Option<String> {
    if let Some(rest) = address.strip_prefix(from) {
        return Some(format!("{to}{rest}"));
    }
    (format!("{address}/") == from).then(|| {
        let bare = to.trim_end_matches('/');
        if bare.is_empty() { "/" } else { bare }.to_owned()
    })
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
    let settings = Settings::of(index.model(), build);
    let index = index.for_build(build);
    let router = AstroRouter::from_consumer(&index.model().consumer);
    let resolved = index.resolve_build(build, &router);
    of_resolved(&index, build, &resolved, settings)
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

/// The addresses of `build` already resolved, with `index` the project its
/// pages are routed in ([`Project::for_build`]), and `settings` the build's
/// ([`Settings::of`]).
pub fn of_resolved(
    index: &Project,
    build: &Build,
    resolved: &ResolvedBuild,
    settings: Settings,
) -> BuildAddresses {
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
        if settings.agents {
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
    addresses.extend(permalinks(&settings, index, resolved));
    if settings.agents && model.consumer.site.is_some() {
        addresses.extend(llms(index, build, resolved));
    }
    BuildAddresses {
        build: resolved.build.clone(),
        settings,
        addresses,
    }
}

/// The address of each name `resolved` answers to under the permalinks'
/// path, in name order: each permalink it publishes, and each name
/// `ascribe.addresses` retires, or moves to something the build publishes,
/// unless the content declares it too. A page is served at each, which
/// forwards to the permalink's page or says why it was removed.
fn permalinks(settings: &Settings, index: &Project, resolved: &ResolvedBuild) -> Vec<Address> {
    let found = resolved.permalinks();
    let mut names: std::collections::BTreeMap<String, Option<RelPath>> =
        std::collections::BTreeMap::new();
    for permalink in &found {
        names
            .entry(permalink.name.clone())
            .or_insert(Some(permalink.page.clone()));
    }
    for entry in index.recorded_permalinks().names {
        let page = match &entry.end {
            End::Retired(_) => None,
            End::Place(place) => match resolved.retired_destination(place) {
                Some(to) => Some(to.page),
                None => continue,
            },
            End::Permalink(end) => match found.iter().find(|p| p.name == *end) {
                Some(target) => Some(target.page.clone()),
                None => continue,
            },
            End::Cycle => continue,
        };
        names.entry(entry.name).or_insert(page);
    }
    let home = RelPath::parse("index.md").ok();
    names
        .into_iter()
        .filter_map(|(name, page)| {
            Some(Address {
                address: settings.permalink_address(&name),
                kind: AddressKind::Permalink,
                page: page.or_else(|| home.clone())?,
                source: name,
            })
        })
        .collect()
}

/// The address of each `llms.txt` file `build` publishes, the root's first.
fn llms(index: &Project, build: &Build, resolved: &ResolvedBuild) -> Vec<Address> {
    let model = index.model();
    let router = AstroRouter::from_consumer(&model.consumer);
    let navigation = index.navigation();
    let share = navigation.share(index, resolved);
    let records = crate::pages::records(model, &resolved.pages);
    crate::llms::llms_files(model, build, &records, &share)
        .into_iter()
        .filter_map(|file| {
            Some(Address {
                address: router.url_of(&file.path),
                kind: AddressKind::Llms,
                page: file.page.or_else(|| RelPath::parse("index.md").ok())?,
                source: file.path,
            })
        })
        .collect()
}
