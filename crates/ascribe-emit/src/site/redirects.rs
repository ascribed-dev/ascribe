//! `_ascribe/redirects.json`: where the addresses a build used to publish
//! lead now, and which were retired, from `ascribe.addresses` alone
//! (output-layout contract §1.8). The base revision isn't read: a source's
//! old address is made from its path by the build's routing today, so the
//! redirect is written as long as the entry is there, whatever the base.
//!
//! - **`redirects`:** for each `[moved]` entry that leads to a page the
//!   build publishes, the key's address and its Markdown version's, each to
//!   the page's, 301, flat and exact: an entry whose value is another
//!   entry's key leads where that one does. A key that's an address from the
//!   host's root is its own `from`. A pair whose `from` and `to` agree once
//!   case and a final `/` are ignored is left out: the host finds the page
//!   anyway. So is one whose `from` the build publishes, since a page there
//!   is live. An id's entry writes nothing: a fragment never reaches the
//!   host.
//! - **`gone`:** each `[retired]` key's address, and its Markdown
//!   version's, with the reason, and where the page saying so is served:
//!   under [`GONE_PREFIX`], at the key's page's address, for both.
//!
//! With `[consumer] host = "netlify"`, the emitter also writes Netlify's
//! files in `_ascribe/host/`: `_redirects` from the pairs, and a rule for
//! each retired address serving its page there with a 404; and `_headers`
//! from the build's header rules.

use std::collections::HashSet;

use ascribe_core::{RelPath, Router};
use ascribe_model::addresses::{Key, split_place};
use ascribe_model::{AddressFile, ContentModel};
use ascribe_resolve::{AstroRouter, ResolvedBuild};
use serde::Serialize;

/// The version of the file's format.
const VERSION: u32 = 1;

/// The status every pair is written with: a permanent redirect.
const MOVED_PERMANENTLY: u16 = 301;

/// Where a retired address's page is served, from the host's root: this,
/// then the address.
pub const GONE_PREFIX: &str = "/_ascribe/gone";

/// Where the addresses a build used to publish lead now, and which were
/// retired: `_ascribe/redirects.json`.
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Redirects {
    /// The format's version.
    pub version: u32,
    /// The build's name.
    pub build: String,
    /// Each pair, sorted by `from`.
    pub redirects: Vec<Redirect>,
    /// Each retired address, sorted by `path`.
    pub gone: Vec<Gone>,
}

/// An address that leads somewhere else now.
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Redirect {
    /// The old address, from the host's root, in the published case and
    /// slash form.
    pub from: String,
    /// Where it leads now, from the host's root.
    pub to: String,
    /// The HTTP status: 301.
    pub status: u16,
    /// The `ascribe.addresses` key the pair comes from.
    pub entry: String,
}

/// An address retired on purpose.
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Gone {
    /// The address, from the host's root.
    pub path: String,
    /// Why it was retired.
    pub reason: String,
    /// The `ascribe.addresses` key it comes from.
    pub entry: String,
    /// Where the page saying so is, from the host's root: [`GONE_PREFIX`]
    /// and the key's address, its page's for a Markdown version.
    pub page: String,
}

/// The redirects and retired addresses `file` gives `build`, a build of a
/// project with `model`, the model its pages are routed with
/// ([`ContentModel::for_build`]); `None` for a project without the file,
/// whose lists are empty.
pub fn redirects(
    model: &ContentModel,
    build: &ResolvedBuild,
    file: Option<&AddressFile>,
) -> Redirects {
    let mut out = Redirects {
        version: VERSION,
        build: build.build.clone(),
        ..Redirects::default()
    };
    let Some(file) = file else {
        return out;
    };
    let router = AstroRouter::from_consumer(&model.consumer);
    let agents = model.consumer.agents;
    let live: HashSet<String> = build
        .pages
        .iter()
        .flat_map(|p| {
            let mut addresses = vec![normalized(&p.route)];
            if agents {
                addresses.push(normalized(&router.markdown_url(&p.path)));
            }
            addresses
        })
        .collect();
    let is_live = |address: &str| live.contains(&normalized(address));
    // The addresses a key had: a page's, and its Markdown version's.
    let addresses_of = |key: &str| -> Vec<(String, bool)> {
        match Key::parse(key) {
            Some(Key::Source(path)) => match RelPath::parse(path) {
                Ok(path) => {
                    let mut out = vec![(router.route(&path), false)];
                    if agents {
                        out.push((router.markdown_url(&path), true));
                    }
                    out
                }
                Err(_) => Vec::new(),
            },
            Some(Key::Address(address)) => vec![(address.to_owned(), false)],
            Some(Key::Id { .. }) | None => Vec::new(),
        }
    };
    for (key, entry) in &file.moved {
        // Where the entry leads in the end, so each pair is one step.
        let Some(target) = file.follow(&entry.value) else {
            continue;
        };
        let Some((path, id)) = split_place(&target) else {
            continue;
        };
        let Some(page) = RelPath::parse(path)
            .ok()
            .filter(|p| build.page(p).is_some())
        else {
            continue;
        };
        for (from, markdown) in addresses_of(key) {
            let to = if markdown {
                router.markdown_url(&page)
            } else {
                match id {
                    Some(id) => format!("{}#{id}", router.route(&page)),
                    None => router.route(&page),
                }
            };
            if is_live(&from) || normalized(&from) == normalized(&to) {
                continue;
            }
            if out.redirects.iter().any(|r| r.from == from) {
                continue;
            }
            out.redirects.push(Redirect {
                from,
                to,
                status: MOVED_PERMANENTLY,
                entry: key.clone(),
            });
        }
    }
    for (key, entry) in &file.retired {
        let addresses = addresses_of(key);
        let Some((page, _)) = addresses.iter().find(|(_, markdown)| !markdown) else {
            continue;
        };
        let page = format!("{GONE_PREFIX}{page}");
        for (path, _) in addresses {
            if is_live(&path) || out.gone.iter().any(|g| g.path == path) {
                continue;
            }
            out.gone.push(Gone {
                path,
                reason: entry.value.clone(),
                entry: key.clone(),
                page: page.clone(),
            });
        }
    }
    out.redirects.sort_by(|a, b| a.from.cmp(&b.from));
    out.gone.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// What [`redirects`] gives `build` of the project `index`, resolving it:
/// what `ascribe addresses list` shows, and the build writes.
pub fn of_build(index: &ascribe_resolve::Project, build: &ascribe_model::Build) -> Redirects {
    let index = index.for_build(build);
    let router = AstroRouter::from_consumer(&index.model().consumer);
    let resolved = index.resolve_build(build, &router);
    redirects(index.model(), &resolved, index.addresses_file())
}

/// An address as a host matches it: without its fragment, in lower case,
/// and without a final `/`.
fn normalized(address: &str) -> String {
    let address = address.split('#').next().unwrap_or(address);
    let trimmed = address.trim_end_matches('/');
    if trimmed.is_empty() { "/" } else { trimmed }.to_lowercase()
}

/// The file, as JSON.
pub(super) fn json(redirects: &Redirects) -> String {
    // Serializing strings and numbers can't fail.
    let mut text = serde_json::to_string_pretty(redirects).unwrap_or_default();
    text.push('\n');
    text
}

/// Netlify's `_redirects`: each pair on a line, `from`, `to`, and the
/// status; then each retired address, its page, and 404, which serves the
/// page there with that status.
pub(super) fn netlify(redirects: &Redirects) -> String {
    let pairs = redirects
        .redirects
        .iter()
        .map(|r| format!("{} {} {}\n", r.from, r.to, r.status));
    let gone = redirects
        .gone
        .iter()
        .map(|g| format!("{} {} {NOT_FOUND}\n", g.path, g.page));
    pairs.chain(gone).collect()
}

/// The status a retired address's page is served with.
const NOT_FOUND: u16 = 404;
