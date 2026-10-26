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
//! - **`[former]`'s pairs:** under each former `base-path`, every address
//!   of each page the build publishes, its Markdown version's, and each
//!   `llms.txt` file's, to today's; under each former permalinks' path,
//!   `permalink-path`'s, or `go/` under a former base path while
//!   `permalink-path` is the default, each name the build answers to, to
//!   its address today. With `agents = [true]` and `agents` off now, each
//!   Markdown version leads to its page, and each `llms.txt` file to the
//!   page at the base path. A `[moved]` key's addresses are made under each
//!   base path too. Every pair is to today's address, so a second change
//!   composes with the first: each former value forwards straight there.
//!   A pair an entry writes comes first, and the first pair from an
//!   address is the one written.
//! - **`gone`:** each `[retired]` key's address, and its Markdown
//!   version's, with the reason, and where the page saying so is served:
//!   in [`GONE_FOLDER`] under the build's base path, at the key's page's
//!   address from there, for each, under each base path.
//!
//! With `[consumer] host = "netlify"`, the emitter also writes Netlify's
//! files in `_ascribe/host/`: `_redirects` from the pairs, and a rule for
//! each retired address serving its page there with a 404; and `_headers`
//! from the build's header rules.

use std::collections::HashSet;

use ascribe_core::{RelPath, Router};
use ascribe_model::addresses::{Former, Key, split_place};
use ascribe_model::{AddressFile, Build, ContentModel};
use ascribe_resolve::{AstroRouter, ResolvedBuild};
use serde::Serialize;

/// The version of the file's format.
const VERSION: u32 = 1;

/// The status every pair is written with: a permanent redirect.
const MOVED_PERMANENTLY: u16 = 301;

/// Where a retired address's page is served: this folder under the build's
/// base path, where a site generator serves it, then the address from the
/// base path.
pub const GONE_FOLDER: &str = "_ascribe/gone/";

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
    /// The `ascribe.addresses` key the pair comes from; or, for a pair a
    /// `[former]` setting makes, `former.` and the setting:
    /// `former.base-path`, `former.permalink-path`, or `former.agents`.
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
    /// Where the page saying so is, from the host's root: the base path,
    /// [`GONE_FOLDER`], and the key's address from the base path, its page's
    /// for a Markdown version.
    pub page: String,
}

/// The redirects and retired addresses `file` gives `build`, a build of
/// `project` declared as `declared`, with `model`, the model its pages are
/// routed with ([`ContentModel::for_build`]); `None` for a project without
/// the file, whose lists are empty.
pub fn redirects(
    model: &ContentModel,
    project: &ascribe_resolve::Project,
    declared: &Build,
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
    let former = &file.former;
    // Whether the build published Markdown versions, now or under a
    // setting `[former]` records.
    let had_agents = agents || former.agents.contains(&true);
    // Today's router, then one for each former base path, with an older
    // line's folder after it.
    let folder = declared.line_folder().unwrap_or_default();
    let mut routers = vec![router.clone()];
    for old in &former.base_path {
        let mut consumer = model.consumer.clone();
        consumer.base_path = format!("{}/{folder}", old.trim_end_matches('/'));
        let old = AstroRouter::from_consumer(&consumer);
        if !routers.contains(&old) {
            routers.push(old);
        }
    }
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
    // Where a page's Markdown version leads: its Markdown version now, or,
    // with `agents` off, the page.
    let markdown_to = |page: &RelPath| {
        if agents {
            router.markdown_url(page)
        } else {
            router.route(page)
        }
    };
    // The addresses a key had, under each base path: a page's, and its
    // Markdown version's.
    let addresses_of = |key: &str| -> Vec<(String, bool)> {
        match Key::parse(key) {
            Some(Key::Source(path)) => match RelPath::parse(path) {
                Ok(path) => {
                    let mut out = Vec::new();
                    for router in &routers {
                        out.push((router.route(&path), false));
                        if had_agents {
                            out.push((router.markdown_url(&path), true));
                        }
                    }
                    out
                }
                Err(_) => Vec::new(),
            },
            Some(Key::Address(address)) => vec![(address.to_owned(), false)],
            // A permalink's address is the permalinks list's to publish.
            Some(Key::Id { .. } | Key::Permalink(_)) | None => Vec::new(),
        }
    };
    let mut pairs = Pairs {
        out: &mut out.redirects,
        is_live: &is_live,
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
                markdown_to(&page)
            } else {
                match id {
                    Some(id) => format!("{}#{id}", router.route(&page)),
                    None => router.route(&page),
                }
            };
            pairs.push(from, to, key);
        }
    }
    former_pairs(
        &Now {
            model,
            project,
            declared,
            build,
            former,
        },
        &routers,
        had_agents,
        &mut pairs,
    );
    for (key, entry) in &file.retired {
        let addresses = addresses_of(key);
        let Some((page, _)) = addresses.iter().find(|(_, markdown)| !markdown) else {
            continue;
        };
        let page = gone_page(router.base(), page);
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

/// What a build publishes now, which `[former]`'s pairs lead to.
struct Now<'a> {
    model: &'a ContentModel,
    project: &'a ascribe_resolve::Project,
    declared: &'a Build,
    build: &'a ResolvedBuild,
    former: &'a Former,
}

/// The pairs `[former]` makes: from each address under each former base
/// path, `routers` after the first, today's; each permalinks' path; and,
/// with `agents` off while `had_agents`, each Markdown version and
/// `llms.txt` file.
fn former_pairs(now: &Now<'_>, routers: &[AstroRouter], had_agents: bool, pairs: &mut Pairs<'_>) {
    let Now {
        model,
        project,
        declared,
        build,
        former,
    } = *now;
    let router = &routers[0];
    let agents = model.consumer.agents;
    let markdown_to = |page: &RelPath| {
        if agents {
            router.markdown_url(page)
        } else {
            router.route(page)
        }
    };
    // Every address of a page now under each former base path, and its
    // Markdown version's while `agents` was on.
    for page in &build.pages {
        for old in &routers[1..] {
            pairs.push(old.route(&page.path), page.route.clone(), FORMER_BASE_PATH);
            if had_agents {
                pairs.push(
                    old.markdown_url(&page.path),
                    markdown_to(&page.path),
                    FORMER_BASE_PATH,
                );
            }
        }
        if !agents && had_agents {
            pairs.push(
                router.markdown_url(&page.path),
                page.route.clone(),
                FORMER_AGENTS,
            );
        }
    }
    // The `llms.txt` files under each former base path, to today's; with
    // `agents` off, each to the page at the base path.
    if routers.len() > 1 || !agents && had_agents {
        let home = RelPath::parse("index.md")
            .map(|p| router.route(&p))
            .unwrap_or_else(|_| router.base().to_owned());
        let navigation = project.navigation();
        let share = navigation.share(project, build);
        let records = ascribe_resolve::pages::records(model, &build.pages);
        let files = ascribe_resolve::llms::llms_files(model, declared, &records, &share);
        for llms in &files {
            let to = if agents {
                router.url_of(&llms.path)
            } else {
                home.clone()
            };
            for (i, old) in routers.iter().enumerate() {
                if i == 0 && agents {
                    continue;
                }
                let entry = if i == 0 {
                    FORMER_AGENTS
                } else {
                    FORMER_BASE_PATH
                };
                pairs.push(old.url_of(&llms.path), to.clone(), entry);
            }
        }
    }
    // Each name the build answers to under each former permalinks' path:
    // `permalink-path`'s, and, while it's the default, `go/` under each
    // former base path.
    let slash = if router.trailing_slash() { "/" } else { "" };
    let now = model.consumer.permalink_path();
    let mut paths: Vec<(String, &str)> = former
        .permalink_path
        .iter()
        .map(|p| (with_slash(p), FORMER_PERMALINK_PATH))
        .collect();
    if model.consumer.permalink_path.is_none() {
        paths.extend(
            former
                .base_path
                .iter()
                .map(|b| (format!("{}/go/", b.trim_end_matches('/')), FORMER_BASE_PATH)),
        );
    }
    paths.retain(|(path, _)| *path != now);
    if !paths.is_empty() {
        let found = build.permalinks();
        let records = project.permalink_records();
        for name in super::permalinks::named(&records, build, &found).keys() {
            for (path, entry) in &paths {
                pairs.push(
                    format!("{path}{name}{slash}"),
                    format!("{now}{name}{slash}"),
                    entry,
                );
            }
        }
    }
}

/// The `entry` of a pair a former base path makes.
const FORMER_BASE_PATH: &str = "former.base-path";

/// The `entry` of a pair a former permalinks' path makes.
const FORMER_PERMALINK_PATH: &str = "former.permalink-path";

/// The `entry` of a pair `agents` turned off makes.
const FORMER_AGENTS: &str = "former.agents";

/// The pairs being written: each `from` once, the first written, and none
/// from an address the build publishes, or to the same address once case
/// and a final `/` are ignored.
struct Pairs<'a> {
    out: &'a mut Vec<Redirect>,
    is_live: &'a dyn Fn(&str) -> bool,
}

impl Pairs<'_> {
    fn push(&mut self, from: String, to: String, entry: &str) {
        if (self.is_live)(&from)
            || normalized(&from) == normalized(&to)
            || self.out.iter().any(|r| r.from == from)
        {
            return;
        }
        self.out.push(Redirect {
            from,
            to,
            status: MOVED_PERMANENTLY,
            entry: entry.to_owned(),
        });
    }
}

/// `path`, ending in `/`.
fn with_slash(path: &str) -> String {
    if path.ends_with('/') {
        path.to_owned()
    } else {
        format!("{path}/")
    }
}

/// What [`redirects`] gives `build` of the project `index`, resolving it:
/// what `ascribe addresses list` shows, and the build writes.
pub fn of_build(index: &ascribe_resolve::Project, build: &ascribe_model::Build) -> Redirects {
    let index = index.for_build(build);
    let router = AstroRouter::from_consumer(&index.model().consumer);
    let resolved = index.resolve_build(build, &router);
    redirects(
        index.model(),
        &index,
        build,
        &resolved,
        index.addresses_file(),
    )
}

/// Where the page saying `address` was retired is, under `base`, the
/// build's base path, which starts and ends with `/`: `base`, [`GONE_FOLDER`],
/// and the address from `base`, or from the host's root for one outside it;
/// `index` for the base path itself.
fn gone_page(base: &str, address: &str) -> String {
    let rest = address
        .strip_prefix(base)
        .or_else(|| format!("{address}/").strip_prefix(base).map(|_| ""))
        .unwrap_or_else(|| address.trim_start_matches('/'));
    let rest = if rest.is_empty() { "index" } else { rest };
    format!("{base}{GONE_FOLDER}{rest}")
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
