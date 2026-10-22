//! The typed content model and the queries other crates ask of it.

use std::cmp::Ordering;

use ascribe_core::availability::{AvailabilitySpec, Detail, Entry, Name, Version};
use ascribe_core::{
    AttributeSchema, Date, DiagnosticSlug, DirectiveSchema, EntryKind, Issue, Span, builtin_schemas,
};

use crate::pattern::Pattern;
use crate::settings;
use crate::types::FrontmatterSchema;

/// The spec versions this crate implements (`spec`).
pub const SUPPORTED_SPECS: &[&str] = &["0.1"];

/// A loaded, validated `ascribe.toml`.
///
/// Every default is applied: the implicit `page`
/// type and `site` build exist when their tables are absent, and the built-in
/// lifecycle states and note types are always present. Collections keep
/// declaration order.
#[derive(Clone, Debug)]
pub struct ContentModel {
    /// The `spec` version.
    pub spec: String,
    /// `[project]`.
    pub project: Project,
    /// Page content types (`[types.<name>]`), or the implicit `page` type.
    pub types: Vec<ContentType>,
    /// `[fragments]`.
    pub fragments: Fragments,
    /// `[dimensions.<name>]`, in declaration order. That order is the
    /// canonical order of `@variant` attributes and decides which dimension a
    /// tab group syncs on.
    pub dimensions: Vec<Dimension>,
    /// `[versions] scheme`.
    pub version_scheme: VersionScheme,
    /// `[releases.<target>]`, in declaration order: the lines of each
    /// versioned target a project lists.
    pub releases: Vec<Releases>,
    /// `[releases] planned-label`: the word a reader sees for a version in a
    /// planned line. `planned` by default.
    pub planned_label: String,
    /// Lifecycle states: the built-ins (possibly adjusted), then declared ones.
    pub lifecycle: Vec<LifecycleState>,
    /// `[features.<key>]`.
    pub features: Vec<Feature>,
    /// Note types: the built-ins (possibly relabeled), then declared ones.
    pub notes: Vec<NoteType>,
    /// `[phrases]`, in declaration order.
    pub phrases: Vec<Phrase>,
    /// `[glossary]`.
    pub glossary: Glossary,
    /// `[images.attributes]`, in declaration order.
    pub image_attributes: Vec<AttributeSchema>,
    /// `[widgets.<name>]`.
    pub widgets: Vec<Widget>,
    /// `[consumer]`.
    pub consumer: Consumer,
    /// `[builds.<name>]`, or the implicit `site` build.
    pub builds: Vec<Build>,
    /// `[sources.<name>]`, in declaration order.
    pub sources: Vec<Source>,
    /// The build the editor checks by default (`[editor] build`).
    pub editor_build: String,
    /// `[checks]`: the level the project sets for each check it names.
    pub checks: Checks,
    /// `[[intended]]`: the acknowledged problems about an entry or an image
    /// (SPEC §4.9), in declaration order.
    pub intended: Vec<Intended>,
    /// `[permalinks]`: the retired permalinks and the aliases.
    pub permalinks: ModelPermalinks,
    /// `[crawlers]`: what the project says to crawlers.
    pub crawlers: Crawlers,
    /// `[listing]`: how the site's, a folder's, or a type's pages are listed.
    pub listing: ListingRules,
    /// Warnings found while loading (`model-name-case`,
    /// `model-build-filter-excluded`). A model with errors doesn't load, so
    /// these are the only issues a loaded model has.
    pub warnings: Vec<Issue>,
}

/// `[checks]`: how loudly each check the project names speaks. Only checks
/// the diagnostics registry marks configurable can be named.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checks {
    /// Each check named, in declaration order.
    pub settings: Vec<CheckSetting>,
    /// `[checks.links]`: how `ascribe report links` checks external links.
    pub links: LinkSettings,
    /// `[checks.vale]`: checking the prose with Vale, when it's there.
    pub vale: Option<ValeSettings>,
}

/// `[checks.links]`: how `ascribe report links` checks the project's
/// external links.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkSettings {
    /// The command that runs the link checker (`command`, `lychee` by
    /// default).
    pub command: String,
    /// The hosts whose links aren't checked (`ignore`): each a host name,
    /// such as `example.com`, or `*.` and a host name, which covers every
    /// host under it, such as `*.example.com`.
    pub ignore: Vec<String>,
    /// How many days an address's answer is reused (`reuse-days`, 14 by
    /// default): each address's own limit falls between half of it and all
    /// of it. `0` asks about every address on every run.
    pub reuse_days: u32,
    /// How many days a link that answered before and fails now is held as
    /// unconfirmed before it's reported (`confirm-days`, 3 by default). `0`
    /// reports every failure at once.
    pub confirm_days: u32,
}

impl LinkSettings {
    /// Whether `ignore` leaves out links to `host`, ignoring case.
    pub fn ignores(&self, host: &str) -> bool {
        let host = host.trim_end_matches('.').to_ascii_lowercase();
        self.ignore.iter().any(|pattern| {
            let pattern = pattern.to_ascii_lowercase();
            match pattern.strip_prefix("*.") {
                Some(parent) => host
                    .strip_suffix(parent)
                    .is_some_and(|rest| rest.ends_with('.')),
                None => host == pattern,
            }
        })
    }
}

/// `[checks.links]` as the registry's defaults have it.
impl Default for LinkSettings {
    fn default() -> LinkSettings {
        const LINKS: &str = "checks.links";
        LinkSettings {
            command: settings::default_text(LINKS, "command").to_owned(),
            ignore: Vec::new(),
            reuse_days: settings::default_number(LINKS, "reuse-days"),
            confirm_days: settings::default_number(LINKS, "confirm-days"),
        }
    }
}

/// `[checks.vale]`: how the prose is checked with Vale.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValeSettings {
    /// The configuration Vale runs with.
    pub source: ValeSource,
    /// The command that runs Vale (`command`, `vale` by default).
    pub command: String,
    /// Whether `ascribe check` runs Vale without `--vale` (`in-check`).
    pub in_check: bool,
    /// The loudest an alert is reported at (`max-level`): an alert above it
    /// is lowered to it. [`CheckLevel::Error`] by default, which lowers
    /// nothing; never [`CheckLevel::Off`].
    pub max_level: CheckLevel,
    /// The preset's rules turned off (`off`), as Vale names them.
    pub off: Vec<String>,
    /// The span of the `vale` key in `ascribe.toml`, where a problem with
    /// running Vale is reported.
    pub span: Span,
}

/// Where Vale's configuration comes from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValeSource {
    /// One of Ascribe's presets, by name (`preset`).
    Preset(String),
    /// The project's own `.vale.ini`, relative to the project root
    /// (`config`).
    Config(String),
}

impl Checks {
    /// The level the project sets for a check, if it sets one.
    pub fn level(&self, slug: DiagnosticSlug) -> Option<CheckLevel> {
        self.settings
            .iter()
            .find(|s| s.slug == slug)
            .and_then(|s| s.level)
    }

    /// Whether the project turns a check off.
    pub fn is_off(&self, slug: DiagnosticSlug) -> bool {
        self.level(slug) == Some(CheckLevel::Off)
    }

    /// The `limit` the project sets for a check, if it sets one: a number
    /// of characters for `page-size`, of bytes for `image-large`.
    pub fn limit(&self, slug: DiagnosticSlug) -> Option<u64> {
        self.settings
            .iter()
            .find(|s| s.slug == slug)
            .and_then(|s| s.limit)
    }
}

/// One acknowledgement in `[[intended]]` (SPEC §4.9): a review check's
/// problem about a content model entry or an image, which is intended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Intended {
    /// The check it acknowledges.
    pub check: DiagnosticSlug,
    /// What it's about: a phrase, feature, glossary term, or image.
    pub kind: EntryKind,
    /// Its key, id, or path.
    pub name: String,
    /// Why the problem is intended.
    pub reason: String,
    /// The table's `[[intended]]` header, in `ascribe.toml`, or the table
    /// itself when it's written inline.
    pub span: Span,
    /// The `check` value.
    pub check_span: Span,
}

/// `[crawlers]`: what the project says to the crawlers that read its site.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Crawlers {
    /// `lines`: the project's own `robots.txt` lines, in order. A path in an
    /// `Allow` or `Disallow` line is from the project's base path.
    pub lines: Vec<CrawlerLine>,
    /// `training`, `search`, and `on-request`, as the project set them, in
    /// [`Purpose::ALL`]'s order.
    pub purposes: Vec<PurposeChoice>,
}

impl Crawlers {
    /// What the project says of `purpose`, if it says anything.
    pub fn choice(&self, purpose: Purpose) -> Option<&PurposeChoice> {
        self.purposes.iter().find(|c| c.purpose == purpose)
    }

    /// Whether the project disallows `purpose`.
    pub fn disallows(&self, purpose: Purpose) -> bool {
        self.choice(purpose)
            .is_some_and(|c| c.permission == Permission::Disallow)
    }
}

/// What a kind of crawler is for: a key of `[crawlers]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Purpose {
    /// Gathering text to train a model.
    Training,
    /// Building the index an AI search product answers from.
    Search,
    /// Fetching a page because a person asked an assistant about it.
    OnRequest,
}

impl Purpose {
    /// Every purpose, in the order `robots.txt` has their groups.
    pub const ALL: [Purpose; 3] = [Purpose::Training, Purpose::Search, Purpose::OnRequest];

    /// The key, as it's written.
    pub fn name(self) -> &'static str {
        match self {
            Purpose::Training => "training",
            Purpose::Search => "search",
            Purpose::OnRequest => "on-request",
        }
    }

    /// The purpose written `name`.
    pub fn from_name(name: &str) -> Option<Purpose> {
        Purpose::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// What a project says a kind of crawler may do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Permission {
    /// `allow`: it may fetch what every crawler may. Writes nothing.
    Allow,
    /// `disallow`: it may fetch nothing. Writes a group naming every token
    /// of the purpose, with `Disallow: /`.
    Disallow,
}

impl Permission {
    /// Both values.
    pub const ALL: [Permission; 2] = [Permission::Allow, Permission::Disallow];

    /// The value, as it's written.
    pub fn name(self) -> &'static str {
        match self {
            Permission::Allow => "allow",
            Permission::Disallow => "disallow",
        }
    }

    /// The value written `name`.
    pub fn from_name(name: &str) -> Option<Permission> {
        Permission::ALL.into_iter().find(|p| p.name() == name)
    }
}

/// One of `[crawlers] training`, `search`, and `on-request`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PurposeChoice {
    /// The key.
    pub purpose: Purpose,
    /// Its value.
    pub permission: Permission,
    /// `key = value`, in `ascribe.toml`.
    pub span: Span,
}

/// One of `[crawlers] lines`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrawlerLine {
    /// The line, as written: one line, not empty.
    pub text: String,
    /// The string, in `ascribe.toml`.
    pub span: Span,
}

/// A page's reserved frontmatter key `listing` (SPEC §2.1): whether a page
/// is offered to readers that aren't people, from open to closed. A page
/// that doesn't set it is [`Listing::Listed`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Listing {
    /// Listed by search engines, and offered to agents.
    #[default]
    Listed,
    /// Listed, and asks not to be quoted.
    Unquoted,
    /// Not listed, and not offered: out of the sitemap and `llms.txt`.
    Unlisted,
}

impl Listing {
    /// The frontmatter key.
    pub const KEY: &'static str = "listing";

    /// Every value, from open to closed.
    pub const ALL: [Listing; 3] = [Listing::Listed, Listing::Unquoted, Listing::Unlisted];

    /// The value as it's written.
    pub fn name(self) -> &'static str {
        match self {
            Listing::Listed => "listed",
            Listing::Unquoted => "unquoted",
            Listing::Unlisted => "unlisted",
        }
    }

    /// The value written `name`.
    pub fn from_name(name: &str) -> Option<Listing> {
        Listing::ALL.into_iter().find(|l| l.name() == name)
    }

    /// The rule a crawler is given for a page with this listing, in a robots
    /// meta tag or an `X-Robots-Tag` header: `noindex` for `unlisted`,
    /// `nosnippet` for `unquoted`, and none for `listed`.
    pub fn robots(self) -> Option<&'static str> {
        match self {
            Listing::Listed => None,
            Listing::Unquoted => Some("nosnippet"),
            Listing::Unlisted => Some("noindex"),
        }
    }
}

/// `[listing]`: how pages are listed, set for many at once: the whole site,
/// an older line of a versioned build, a folder, or a content type. A page's listing is the strictest of the
/// rules it's under and its own, and a page can't be looser than its rules.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListingRules {
    /// `default`: the site's, for every page.
    pub default: Option<ListingRule>,
    /// `older-lines`: for every page of a live line of a versioned build
    /// other than its current line.
    pub older_lines: Option<ListingRule>,
    /// `[listing.folders]`, in declaration order.
    pub folders: Vec<ListingRule>,
    /// `[listing.types]`, in declaration order.
    pub types: Vec<ListingRule>,
}

/// One rule of `[listing]`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ListingRule {
    /// What it applies to.
    pub level: ListingLevel,
    /// Its value.
    pub listing: Listing,
    /// The rule as written, `key = value`.
    pub text: String,
    /// Its line in `ascribe.toml`, from 1.
    pub line: usize,
    /// The rule, `key = value`, in `ascribe.toml`.
    pub span: Span,
}

/// What a rule of `[listing]` applies to.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ListingLevel {
    /// Every page: `default`.
    Site,
    /// Every page of an older line of a versioned build: `older-lines`.
    OlderLines,
    /// The pages under a folder, written from the content root and ending in
    /// `/`: a key of `[listing.folders]`.
    Folder(String),
    /// The pages of a content type: a key of `[listing.types]`.
    Type(String),
}

/// Where a page's listing comes from.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum ListingFrom {
    /// Nothing sets it: the page is `listed`.
    #[default]
    Default,
    /// The page's own `listing`.
    Page,
    /// A rule of `[listing]`.
    Rule(ListingRule),
}

impl ListingFrom {
    /// The level, as the page records write it: `default`, `page`, `site`,
    /// `folder`, or `type`.
    pub fn level_name(&self) -> &'static str {
        match self {
            ListingFrom::Default => "default",
            ListingFrom::Page => "page",
            ListingFrom::Rule(rule) => match rule.level {
                ListingLevel::Site => "site",
                ListingLevel::OlderLines => "older-lines",
                ListingLevel::Folder(_) => "folder",
                ListingLevel::Type(_) => "type",
            },
        }
    }
}

impl ListingRules {
    /// Whether the project has no rules.
    pub fn is_empty(&self) -> bool {
        self.default.is_none()
            && self.older_lines.is_none()
            && self.folders.is_empty()
            && self.types.is_empty()
    }
}

impl ContentModel {
    /// The strictest rule of `[listing]` a page at `path` (from the content
    /// root, with `/`) of the type named `type_name` is under, or `None` when
    /// it's under none, in every build: [`ContentModel::listing_rule_in`]
    /// outside an older line.
    pub fn listing_rule(&self, path: &str, type_name: Option<&str>) -> Option<&ListingRule> {
        self.listing_rule_in(path, type_name, false)
    }

    /// The strictest rule of `[listing]` a page at `path` (from the content
    /// root, with `/`) of the type named `type_name` is under, or `None` when
    /// it's under none; `older_line` says whether the page is in an older
    /// line of a versioned build, which `older-lines` is for. Of rules
    /// equally strict, the narrowest is given: the deepest folder, then the
    /// type, then `older-lines`, then the site's default. A folder and a type
    /// have no order between them; the folder is named first.
    pub fn listing_rule_in(
        &self,
        path: &str,
        type_name: Option<&str>,
        older_line: bool,
    ) -> Option<&ListingRule> {
        let mut folders: Vec<&ListingRule> = self
            .listing
            .folders
            .iter()
            .filter(|r| matches!(&r.level, ListingLevel::Folder(f) if path.starts_with(f.as_str())))
            .collect();
        folders.sort_by_key(|r| match &r.level {
            ListingLevel::Folder(f) => std::cmp::Reverse(f.len()),
            _ => std::cmp::Reverse(0),
        });
        let types =
            self.listing.types.iter().filter(
                |r| matches!(&r.level, ListingLevel::Type(t) if Some(t.as_str()) == type_name),
            );
        let mut best: Option<&ListingRule> = None;
        for rule in folders
            .into_iter()
            .chain(types)
            .chain(self.listing.older_lines.iter().filter(|_| older_line))
            .chain(self.listing.default.iter())
        {
            if best.is_none_or(|b| rule.listing > b.listing) {
                best = Some(rule);
            }
        }
        best
    }

    /// A page's listing, given its type's name and its own `listing` (`None` when it sets none
    /// or a value that isn't one), and where it comes from: the strictest of
    /// the page's own and the rules it's under. A page as strict as its
    /// rule, or stricter, has its own; a looser one, which `ascribe check`
    /// reports, has its rule's.
    pub fn listing_of(
        &self,
        path: &str,
        type_name: Option<&str>,
        own: Option<Listing>,
    ) -> (Listing, ListingFrom) {
        self.listing_in(path, type_name, own, false)
    }

    /// [`ContentModel::listing_of`] for a page that may be in an older line
    /// of a versioned build, where `older-lines` applies too.
    pub fn listing_in(
        &self,
        path: &str,
        type_name: Option<&str>,
        own: Option<Listing>,
        older_line: bool,
    ) -> (Listing, ListingFrom) {
        let rule = self.listing_rule_in(path, type_name, older_line);
        match (own, rule) {
            (Some(own), Some(rule)) if own < rule.listing => {
                (rule.listing, ListingFrom::Rule(rule.clone()))
            }
            (Some(own), _) => (own, ListingFrom::Page),
            (None, Some(rule)) => (rule.listing, ListingFrom::Rule(rule.clone())),
            (None, None) => (Listing::Listed, ListingFrom::Default),
        }
    }
}

/// `[permalinks]`: the names `ascribe.toml` gives somewhere to lead that
/// no heading or page declares (SPEC §7.6). Each name is in one of them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModelPermalinks {
    /// `[permalinks.retired]`, in declaration order.
    pub retired: Vec<RetiredPermalink>,
    /// `[permalinks.aliases]`, in declaration order.
    pub aliases: Vec<PermalinkAlias>,
}

/// A permalink retired on purpose, in `[permalinks.retired]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RetiredPermalink {
    /// The name.
    pub name: String,
    /// Why it was retired, as the author wrote it.
    pub reason: String,
    /// Where its address leads now, written as a link from the content
    /// root is: a page's content path, and `#` and a heading's id.
    pub to: Option<String>,
    /// The name's key, in `ascribe.toml`.
    pub span: Span,
    /// The `to` value.
    pub to_span: Option<Span>,
}

/// A second name for a permalink, in `[permalinks.aliases]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermalinkAlias {
    /// The alias.
    pub name: String,
    /// The name it leads to: a permalink, or another alias.
    pub target: String,
    /// The alias's key, in `ascribe.toml`.
    pub span: Span,
    /// The target's value.
    pub target_span: Span,
}

impl ModelPermalinks {
    /// Whether `ascribe.toml` retires or aliases nothing.
    pub fn is_empty(&self) -> bool {
        self.retired.is_empty() && self.aliases.is_empty()
    }

    /// The retired permalink `name`.
    pub fn retired(&self, name: &str) -> Option<&RetiredPermalink> {
        self.retired.iter().find(|r| r.name == name)
    }

    /// The alias `name`.
    pub fn alias(&self, name: &str) -> Option<&PermalinkAlias> {
        self.aliases.iter().find(|a| a.name == name)
    }

    /// The name the alias `name` ends at, following aliases of aliases: a
    /// name that isn't an alias. `None` when `name` isn't an alias, or its
    /// aliases make a loop.
    pub fn alias_end<'a>(&'a self, name: &'a str) -> Option<&'a str> {
        let mut at = self.alias(name)?;
        let mut seen = vec![name];
        loop {
            let next = at.target.as_str();
            if seen.contains(&next) {
                return None;
            }
            match self.alias(next) {
                Some(further) => {
                    seen.push(next);
                    at = further;
                }
                None => return Some(next),
            }
        }
    }
}

/// One check named in `[checks]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckSetting {
    /// The check.
    pub slug: DiagnosticSlug,
    /// The level it's set to; `None` for a table without `level`, which
    /// keeps the check's own.
    pub level: Option<CheckLevel>,
    /// `limit`, for a check that takes one: characters for `page-size`,
    /// bytes for `image-large`.
    pub limit: Option<u64>,
}

/// The level a project sets a check to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CheckLevel {
    /// Not reported.
    Off,
    /// Reported as advice, which never fails `ascribe check`.
    Advice,
    /// Reported as a warning.
    Warning,
    /// Reported as an error.
    Error,
}

impl CheckLevel {
    /// The names `[checks]` accepts, from quietest to loudest.
    pub const NAMES: [&'static str; 4] = ["off", "advice", "warning", "error"];

    /// The level with this name.
    pub fn from_name(name: &str) -> Option<CheckLevel> {
        match name {
            "off" => Some(CheckLevel::Off),
            "advice" => Some(CheckLevel::Advice),
            "warning" => Some(CheckLevel::Warning),
            "error" => Some(CheckLevel::Error),
            _ => None,
        }
    }
}

/// `[project]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Project {
    /// The content root, relative to the project root, as written.
    pub content_root: String,
    /// The output directory, relative to the project root, as written.
    pub output_dir: String,
}

/// A source (SPEC §7.3): a named set of files outside the content that
/// pages may take code from, by address (`<source>:<path>`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    /// The source's name, as addresses write it.
    pub name: String,
    /// Its folder, relative to the project root: the `path` written, or, for
    /// a source in another repository, the folder of its copies,
    /// `sources/<name>` ([`Source::copies_folder`]).
    pub path: String,
    /// The files it makes readable, matched against a file's path relative
    /// to the folder. Empty means every file.
    pub include: Vec<Pattern>,
    /// Files it doesn't make readable, though `include` matches them.
    pub ignore: Vec<Pattern>,
    /// The repository, for a source in another repository (SPEC §7.4).
    pub git: Option<Remote>,
    /// The `[sources.<name>]` key in `ascribe.toml`.
    pub span: Span,
}

/// Another repository a source is in (SPEC §7.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Remote {
    /// Its URL, as `git` takes it.
    pub url: String,
    /// The branch an update follows; `None` for the repository's default.
    pub branch: Option<String>,
}

/// What every source leaves out, before its own `ignore` (SPEC §7.3): the
/// registry's default for `ignore`, every `node_modules` and `.git` folder.
/// A source's `ignore` holds only its own patterns.
pub fn always_ignored() -> &'static [Pattern] {
    static ALWAYS: std::sync::OnceLock<Vec<Pattern>> = std::sync::OnceLock::new();
    ALWAYS.get_or_init(|| {
        settings::default_list("sources.<name>", "ignore")
            .iter()
            .filter_map(|p| Pattern::new(p).ok())
            .collect()
    })
}

/// The folder, in the project's folder, that holds the copies of every
/// source in another repository, each in a folder of its own.
pub const COPIES_FOLDER: &str = "sources";

impl Source {
    /// Whether the file at `path`, relative to the source's folder, is
    /// readable: `include` matches it (or is empty), `ignore` doesn't, and
    /// it isn't the source's tested examples' config or one of their
    /// observations ([`crate::examples::is_examples_file`]), which no
    /// snippet can address.
    pub fn reads(&self, path: &str) -> bool {
        !crate::examples::is_examples_file(path)
            && (self.include.is_empty() || self.include.iter().any(|p| p.matches(path)))
            && !self.ignores(path)
    }

    /// Whether `ignore`, or what every source leaves out
    /// ([`always_ignored`]), leaves out `path`.
    fn ignores(&self, path: &str) -> bool {
        always_ignored()
            .iter()
            .chain(&self.ignore)
            .any(|p| p.matches(path))
    }

    /// Whether a change on disk to the file or folder at `path`, relative
    /// to the source's folder, can matter to the source: it's a file the
    /// source reads ([`Source::reads`]), a folder that may hold one that
    /// `ignore` doesn't leave out, or the source's tested examples' config,
    /// their observations' folder, or a file in it.
    pub fn may_use(&self, path: &str) -> bool {
        use crate::examples::{OBSERVATIONS_FOLDER, is_examples_file};
        is_examples_file(path)
            || path == OBSERVATIONS_FOLDER
            || (!self.ignores(path)
                && (self.include.is_empty() || self.include.iter().any(|p| p.may_match_in(path))))
    }

    /// The folder of a source's copies, relative to the project root, when
    /// the source is in another repository: `sources/<name>`.
    pub fn copies_folder(name: &str) -> String {
        format!("{COPIES_FOLDER}/{name}")
    }
}

/// A page content type.
#[derive(Clone, Debug, PartialEq)]
pub struct ContentType {
    /// The type's name.
    pub name: String,
    /// Whether it's the implicit `page` type of a project that declares no
    /// types, which every page has without naming it.
    pub implicit: bool,
    /// The frontmatter schema.
    pub frontmatter: FrontmatterSchema,
    /// What a page of the type is for, in a few words, `purpose`.
    pub purpose: Option<String>,
    /// The page a page of the type starts from, `template`.
    pub template: Option<TypeFile>,
    /// What to read before writing a page of the type, `guide`, in the order
    /// to read it.
    pub guide: Vec<TypeFile>,
}

/// A file a content type names, its `template` or one of its `guide`, from
/// the folder `ascribe.toml` is in. Loading checks only what's written: that
/// the file is there, outside the content root, and that a template reads as
/// a page, is checked where the project's files are read
/// (`ascribe_check::type_files`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeFile {
    /// The path, as written.
    pub path: String,
    /// The path's string in `ascribe.toml`.
    pub span: Span,
}

/// `[fragments]`.
#[derive(Clone, Debug)]
pub struct Fragments {
    /// Additional fragment patterns.
    pub patterns: Vec<Pattern>,
    /// The fragment schema.
    pub frontmatter: FrontmatterSchema,
}

/// A dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimension {
    /// The dimension's name.
    pub name: String,
    /// Its display label (the name when not declared).
    pub label: String,
    /// Its values, in display order.
    pub values: Vec<DimensionValue>,
}

/// One value of a dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DimensionValue {
    /// The value.
    pub value: String,
    /// Its display label (the value when not declared).
    pub label: String,
    /// Whether the value is versionless (SPEC §4.4).
    pub versionless: bool,
}

/// The version scheme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VersionScheme {
    /// Dotted numbers of any length, compared numerically with missing
    /// components as 0.
    Numeric,
}

/// `[releases.<target>]`: the release lines of one versioned target.
///
/// A line is a version as the list writes it, and stands for every version
/// that starts with it: `3.4` holds `3.4`, `3.4.2`, and `3.4.2.1`. Every line
/// of one target has the same number of parts, in every list, and a line is
/// in one list. Planned lines are newer than every live line, and archived
/// lines older.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Releases {
    /// The target: a versioned dimension value.
    pub target: String,
    /// `live`: the lines a reader can be on, newest first. The first is the
    /// current line.
    pub live: Vec<Version>,
    /// `planned`: lines that haven't shipped, newest first. A version in one
    /// is planned.
    pub planned: Vec<Version>,
    /// `ends`: the day support for a live line ends, in declaration order.
    pub ends: Vec<LineEnd>,
    /// `archived`: lines that have left the window, newest first.
    pub archived: Vec<ArchivedLine>,
}

/// An entry of `[releases.<target>.ends]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineEnd {
    /// The live line, spanning its key.
    pub line: Version,
    /// The last day it's supported.
    pub date: Date,
}

/// An entry of `[releases.<target>.archived]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchivedLine {
    /// The line, spanning its key.
    pub line: Version,
    /// `url`: where the line can still be read, when anywhere.
    pub url: Option<String>,
    /// `ended`: the day its support ended.
    pub ended: Option<Date>,
}

/// Which list a declared line is in, with the current line apart from the
/// other live ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineStatus {
    /// The newest live line.
    Current,
    /// A live line older than the current one.
    Live,
    /// A line in `planned`.
    Planned,
    /// A line in `archived`.
    Archived,
}

impl LineStatus {
    /// The word for it: `current`, `live`, `planned`, or `archived`.
    pub fn as_str(self) -> &'static str {
        match self {
            LineStatus::Current => "current",
            LineStatus::Live => "live",
            LineStatus::Planned => "planned",
            LineStatus::Archived => "archived",
        }
    }
}

/// Where a version falls among a target's declared lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placement<'a> {
    /// In a declared line.
    In(&'a Version, LineStatus),
    /// Newer than every declared line: `newest` is the newest one.
    Ahead {
        /// The newest declared line.
        newest: &'a Version,
    },
    /// In a line between two declared ones that isn't declared itself.
    Between {
        /// The line the version starts, as text: `3.3`.
        line: String,
        /// The declared line just older than it.
        older: &'a Version,
        /// The declared line just newer than it.
        newer: &'a Version,
    },
    /// Older than every declared line, or shorter than the lines and so in
    /// whichever lines follow it: nothing to say.
    Before,
}

impl Releases {
    /// The current line: the newest live one.
    pub fn current(&self) -> Option<&Version> {
        self.live.first()
    }

    /// The oldest live line.
    pub fn oldest_live(&self) -> Option<&Version> {
        self.live.last()
    }

    /// The live line spelled `text`, as the list writes it.
    pub fn live_line(&self, text: &str) -> Option<&Version> {
        self.live.iter().find(|l| l.text == text)
    }

    /// Every declared line with its status, newest first.
    pub fn lines(&self) -> Vec<(&Version, LineStatus)> {
        let mut out: Vec<(&Version, LineStatus)> = self
            .planned
            .iter()
            .map(|l| (l, LineStatus::Planned))
            .collect();
        out.extend(self.live.iter().enumerate().map(|(i, l)| {
            let status = if i == 0 {
                LineStatus::Current
            } else {
                LineStatus::Live
            };
            (l, status)
        }));
        out.extend(
            self.archived
                .iter()
                .map(|a| (&a.line, LineStatus::Archived)),
        );
        out
    }

    /// Whether `version` is in a planned line.
    pub fn is_planned(&self, version: &Version) -> bool {
        self.planned.iter().any(|l| in_line(l, version))
    }

    /// Where `version` falls among the declared lines, or `None` when the
    /// target declares none.
    pub fn place(&self, version: &Version) -> Option<Placement<'_>> {
        let lines = self.lines();
        let (newest, _) = *lines.first()?;
        let (oldest, _) = *lines.last()?;
        let parts = newest.components.len();
        if version.components.len() < parts {
            return Some(if version.compare(newest) == Ordering::Greater {
                Placement::Ahead { newest }
            } else {
                Placement::Before
            });
        }
        let start = &version.components[..parts];
        if let Some((line, status)) = lines.iter().find(|(l, _)| l.components == start) {
            return Some(Placement::In(line, *status));
        }
        let compare = |line: &Version| compare_components(start, &line.components);
        if compare(newest) == Ordering::Greater {
            return Some(Placement::Ahead { newest });
        }
        if compare(oldest) == Ordering::Less {
            return Some(Placement::Before);
        }
        // Newest first: the first line older than it, and the one before.
        let at = lines
            .iter()
            .position(|(l, _)| compare(l) == Ordering::Greater)?;
        Some(Placement::Between {
            line: start
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join("."),
            older: lines[at].0,
            newer: lines[at.checked_sub(1)?].0,
        })
    }
}

/// Compares two component lists numerically, missing parts as `0`.
fn compare_components(a: &[u64], b: &[u64]) -> Ordering {
    for i in 0..a.len().max(b.len()) {
        match a
            .get(i)
            .copied()
            .unwrap_or(0)
            .cmp(&b.get(i).copied().unwrap_or(0))
        {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

/// Whether `version` is in `line`: whether it starts with the line's parts.
pub fn in_line(line: &Version, version: &Version) -> bool {
    version.components.starts_with(&line.components)
}

/// A line named as a mark names it, `self-managed 3.4`: the target, and the
/// line, with spans in `text`. `None` when it isn't a name and a version
/// with space between.
pub fn named_line(text: &str) -> Option<(String, Version)> {
    let text = text.trim();
    let (target, line) = text.split_once(char::is_whitespace)?;
    let line = line.trim_start();
    let at = text.len() - line.len();
    let version = ascribe_core::availability::parse_version(line, at).ok()?;
    crate::is_name_word(target).then(|| (target.to_owned(), version))
}

impl Releases {
    /// The oldest planned line newer than `line`: the one a feature that
    /// names `line` moves to when `line` is released.
    pub fn following(&self, line: &Version) -> Option<&Version> {
        self.planned
            .iter()
            .rev()
            .find(|l| l.compare(line) == Ordering::Greater)
    }

    /// The status of the line spelled as `line` is, when it's declared.
    pub fn status_of(&self, line: &Version) -> Option<LineStatus> {
        self.lines()
            .into_iter()
            .find(|(l, _)| l.components == line.components)
            .map(|(_, status)| status)
    }
}

/// A lifecycle state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleState {
    /// The state's name.
    pub name: String,
    /// Whether content in this state counts as available.
    pub available: bool,
    /// The display label.
    pub label: String,
    /// Whether the state is built in.
    pub builtin: bool,
}

/// A feature.
#[derive(Clone, Debug)]
pub struct Feature {
    /// The feature key.
    pub key: String,
    /// The display name.
    pub name: String,
    /// The spec text, as written.
    pub available_text: String,
    /// The parsed spec. Spans are offsets into `ascribe.toml`.
    pub available: AvailabilitySpec,
    /// The `[features.<key>]` key in `ascribe.toml`.
    pub span: Span,
}

/// A note type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteType {
    /// The type (the value of `@note`'s `type` attribute).
    pub name: String,
    /// The display label.
    pub label: String,
    /// Whether the type is built in.
    pub builtin: bool,
}

/// A phrase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phrase {
    /// The phrase key.
    pub key: String,
    /// The literal replacement text. For a phrase whose value is a target's
    /// line, the target's current line.
    pub value: String,
    /// `{ line = "<target>" }`: the target whose line the phrase is, as
    /// `[releases]` lists it.
    pub line: Option<String>,
    /// The key in `ascribe.toml`.
    pub span: Span,
}

/// Which glossary occurrences are linked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlossaryMatch {
    /// The first occurrence of each term on each page.
    First,
    /// Every occurrence.
    Every,
    /// None: the term is linked only where an author writes the link. Set
    /// per term, for words that also have ordinary senses.
    Marked,
}

/// `[glossary]` (SPEC §5.4). The terms are headings marked `@term` on its
/// pages; the project's source index reads them.
#[derive(Clone, Debug)]
pub struct Glossary {
    /// The default for terms that don't set `match` (`First` or `Every`).
    pub match_mode: GlossaryMatch,
    /// The default for terms that don't set `case-sensitive`.
    pub case_sensitive: bool,
    /// The glossary's pages, in the order `pages` lists them.
    pub pages: Vec<GlossaryPage>,
}

impl Glossary {
    /// Whether the glossary lists this page, by its path from the content
    /// root.
    pub fn has_page(&self, path: &str) -> bool {
        self.pages.iter().any(|p| p.path == path)
    }
}

/// A page in `[glossary] pages`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlossaryPage {
    /// The page's path from the content root, with no leading `/`.
    pub path: String,
    /// The path's value in `ascribe.toml`.
    pub span: Span,
}

/// A project widget.
#[derive(Clone, Debug)]
pub struct Widget {
    /// The directive schema, exactly as `ascribe-core` defines it.
    pub schema: DirectiveSchema,
    /// The plain-markdown fallback text.
    pub plain_fallback: Option<String>,
    /// Whether the plain-markdown output keeps the widget's wrapped content.
    pub plain_content: PlainContent,
}

/// What the plain-markdown output does with a widget's wrapped content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlainContent {
    /// Keep it after the fallback.
    Keep,
    /// Drop it.
    Drop,
}

/// How trailing slashes appear in routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrailingSlash {
    /// `/guides/setup/`.
    Always,
    /// `/guides/setup`.
    Never,
}

/// `[consumer]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Consumer {
    /// The profile; `"astro"`.
    pub profile: String,
    /// The published site's origin.
    pub site: Option<String>,
    /// The URL path every route starts with.
    pub base_path: String,
    /// Trailing slash policy.
    pub trailing_slash: TrailingSlash,
    /// The slugger name; `"github"`.
    pub slugger: String,
    /// Whether the consumer renders raw HTML.
    pub html: bool,
    /// Whether each build publishes what agents read (`agents = true`): an
    /// `llms.txt`, a Markdown version of every page, and a pointer to the
    /// index on each page. Only with a `site`, since every link they hold is
    /// absolute.
    pub agents: bool,
    /// Where permalinks' addresses are published, as written in
    /// `permalink-path`: a path from the site's root, starting with `/`.
    /// [`Consumer::permalink_path`] gives the path with its default.
    pub permalink_path: Option<String>,
    /// The host whose format the site's response headers are written in
    /// (`headers`), by the site generator's integration from
    /// `_ascribe/headers.json`. `None`: nothing is written for a host.
    pub headers: Option<HeadersFormat>,
}

/// A host's format for response headers, `[consumer] headers`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeadersFormat {
    /// A `_headers` file at the root of what's deployed, as Netlify reads it.
    Netlify,
}

impl HeadersFormat {
    /// The value as `ascribe.toml` writes it.
    pub fn name(self) -> &'static str {
        match self {
            HeadersFormat::Netlify => "netlify",
        }
    }
}

impl Consumer {
    /// The path each permalink's address is published under, from the
    /// site's root and ending in `/`: `permalink-path`, or `go/` under the
    /// base path when it isn't set.
    pub fn permalink_path(&self) -> String {
        let mut path = match &self.permalink_path {
            Some(path) => path.clone(),
            None => format!("{}/go/", self.base_path.trim_end_matches('/')),
        };
        if !path.ends_with('/') {
            path.push('/');
        }
        path
    }
}

/// A build's variant mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VariantMode {
    /// Keep every arm and page.
    Switch,
    /// Keep only the selected values: dimension name to values.
    Select(Vec<(String, Vec<String>)>),
}

/// A build's availability mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityMode {
    /// Keep everything and annotate it.
    Badge,
    /// Remove content not available for the target at the version.
    Filter {
        /// A dimension value.
        target: String,
        /// The version; present exactly when the target is versioned.
        version: Option<Version>,
    },
}

/// A build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    /// The build name.
    pub name: String,
    /// The variant mode.
    pub variants: VariantMode,
    /// The availability mode.
    pub availability: AvailabilityMode,
    /// `versions`: the target the build is versioned by, which gives it one
    /// output for each of the target's live lines. Its availability mode is
    /// `Badge`, and the target has at least one live line.
    pub versions: Option<String>,
    /// The line this build is, for a versioned build: the model holds one
    /// `Build` for each live line, the current line named as the build is
    /// declared and each other `<build>@<line>`. `None` for a build that
    /// isn't versioned.
    pub line: Option<BuildLine>,
}

/// One live line of a versioned build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildLine {
    /// The name the build is declared with, which the current line keeps.
    pub build: String,
    /// The line, as `[releases.<target>] live` writes it.
    pub line: Version,
    /// Whether it's the current line, the newest live one.
    pub current: bool,
    /// The oldest live line of the target. A version at or before it isn't
    /// shown in any line's output, since every live line has the content.
    pub oldest: Version,
}

impl Build {
    /// The folder a line's pages are under, after the base path: `3.4/` for
    /// an older line, and nothing for the current line or a build that
    /// isn't versioned.
    pub fn line_folder(&self) -> Option<String> {
        let line = self.line.as_ref()?;
        (!line.current).then(|| format!("{}/", line.line.text))
    }

    /// The base path of the current line of this build's versioned build,
    /// given `base`, the base path this build's pages are under, starting
    /// and ending with `/`: `base` without the line's folder at its end. For
    /// the current line and a build that isn't versioned, `base` itself.
    pub fn current_base(&self, base: &str) -> String {
        match self.line_folder() {
            Some(folder) => match base.strip_suffix(folder.as_str()) {
                Some(root) if root.ends_with('/') => root.to_owned(),
                _ => base.to_owned(),
            },
            None => base.to_owned(),
        }
    }

    /// The name the build is declared with: a line's build's, or its own.
    pub fn declared_name(&self) -> &str {
        self.line.as_ref().map_or(&self.name, |l| &l.build)
    }
}

/// The reserved frontmatter key a page names its content type with (SPEC
/// §2.1).
pub const TYPE_KEY: &str = "type";

/// Which content type a page has (SPEC §7.2).
#[derive(Clone, Debug, PartialEq)]
pub enum TypeMatch<'a> {
    /// The type the page names, or the implicit `page` type of a project
    /// that declares none.
    One(&'a ContentType),
    /// The project declares types and the page names none: an error
    /// (`content-type-missing`).
    Missing,
    /// The page names a type the project doesn't declare, or its `type`
    /// isn't a string: an error (`content-type-unknown`).
    Unknown,
}

impl<'a> TypeMatch<'a> {
    /// The type, when the page has one.
    pub fn found(self) -> Option<&'a ContentType> {
        match self {
            TypeMatch::One(t) => Some(t),
            TypeMatch::Missing | TypeMatch::Unknown => None,
        }
    }
}

/// A problem in an availability spec that needs the model to detect. The
/// caller turns it into a diagnostic: the loader into `model-availability-*`,
/// the document checks into `available-*`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AvailabilityProblem {
    /// A target isn't a declared dimension value or dimension name.
    UnknownTarget(Name),
    /// A state isn't a declared lifecycle state.
    UnknownState(Name),
    /// A versionless target was given a version.
    VersionlessVersion {
        /// The target.
        target: Name,
        /// The offending version.
        version: Version,
    },
    /// A dimension name was given a version (SPEC §4.4): its values don't
    /// share one version line.
    DimensionVersion {
        /// The dimension name.
        target: Name,
        /// The offending version.
        version: Version,
        /// A value of the dimension to name instead: its first versioned
        /// value, or its first value if all are versionless.
        example: String,
    },
    /// A history isn't in chronological order.
    HistoryOrder {
        /// The target.
        target: Name,
        /// The step written first, whose version is the greater: "state version".
        later: String,
        /// The step written after it, with the lesser version.
        earlier: String,
        /// The span of the step written after (the out-of-order one).
        span: Span,
    },
}

impl ContentModel {
    /// Whether `key` is a declared phrase.
    pub fn has_phrase(&self, key: &str) -> bool {
        self.phrases.iter().any(|p| p.key == key)
    }

    /// A phrase's replacement text.
    pub fn phrase(&self, key: &str) -> Option<&str> {
        self.phrases
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.as_str())
    }

    /// A phrase's replacement text in `build`'s output. A phrase that's a
    /// target's line is the line the build is for, in a line of a build the
    /// target versions, and the target's current line elsewhere.
    pub fn phrase_in<'a>(&'a self, key: &str, build: &'a Build) -> Option<&'a str> {
        let phrase = self.phrases.iter().find(|p| p.key == key)?;
        if let (Some(target), Some(versions), Some(line)) =
            (&phrase.line, &build.versions, &build.line)
            && target == versions
        {
            return Some(&line.line.text);
        }
        Some(&phrase.value)
    }

    /// A dimension by name.
    pub fn dimension(&self, name: &str) -> Option<&Dimension> {
        self.dimensions.iter().find(|d| d.name == name)
    }

    /// The dimension a value belongs to (a value belongs to only one).
    pub fn dimension_of_value(&self, value: &str) -> Option<&Dimension> {
        self.dimensions
            .iter()
            .find(|d| d.values.iter().any(|v| v.value == value))
    }

    /// A dimension's values with their labels, in display order.
    pub fn dimension_values(&self, name: &str) -> Option<&[DimensionValue]> {
        self.dimension(name).map(|d| d.values.as_slice())
    }

    /// The display label of a dimension value.
    pub fn value_label(&self, value: &str) -> Option<&str> {
        self.dimension_of_value(value).and_then(|d| {
            d.values
                .iter()
                .find(|v| v.value == value)
                .map(|v| v.label.as_str())
        })
    }

    /// The model a build's pages are routed and written with. For an older
    /// line of a versioned build, the base path is under the line's folder,
    /// `/docs/3.4/`, so every address in its output starts there; a
    /// permalink's address doesn't move, and its outputs for agents are its
    /// own, under its folder. Otherwise, the model itself.
    pub fn for_build(&self, build: &Build) -> std::borrow::Cow<'_, ContentModel> {
        let Some(folder) = build.line_folder() else {
            return std::borrow::Cow::Borrowed(self);
        };
        let mut model = self.clone();
        model.consumer.permalink_path = Some(self.consumer.permalink_path());
        let base = &self.consumer.base_path;
        let base = if base.ends_with('/') {
            base.clone()
        } else {
            format!("{base}/")
        };
        model.consumer.base_path = format!("{base}{folder}");
        std::borrow::Cow::Owned(model)
    }

    /// The build that renders `line` of the versioned build declared as
    /// `build`: the model's own for a live line, and for a planned line one
    /// made as an older line's is, named `<build>@<line>`, which no output
    /// of `ascribe build` is. `None` when no build is declared as `build`
    /// and versioned, or the line is neither live nor planned.
    pub fn line_build(&self, build: &str, line: &Version) -> Option<Build> {
        let lines: Vec<&Build> = self
            .builds
            .iter()
            .filter(|b| b.declared_name() == build && b.line.is_some())
            .collect();
        if let Some(found) = lines.iter().find(|b| {
            b.line
                .as_ref()
                .is_some_and(|l| l.line.components == line.components)
        }) {
            return Some((*found).clone());
        }
        let current = lines
            .iter()
            .find(|b| b.line.as_ref().is_some_and(|l| l.current))?;
        let releases = self.releases_of(current.versions.as_deref()?)?;
        if releases.status_of(line) != Some(LineStatus::Planned) {
            return None;
        }
        let mut made = (*current).clone();
        made.name = format!("{build}@{}", line.text);
        if let Some(l) = made.line.as_mut() {
            l.line = line.clone();
            l.current = false;
        }
        Some(made)
    }

    /// `[releases.<target>]` for a target, when the project lists its lines.
    pub fn releases_of(&self, target: &str) -> Option<&Releases> {
        self.releases.iter().find(|r| r.target == target)
    }

    /// Whether `value` is a declared, versionless dimension value.
    pub fn is_versionless(&self, value: &str) -> bool {
        self.dimensions
            .iter()
            .flat_map(|d| &d.values)
            .any(|v| v.value == value && v.versionless)
    }

    /// A lifecycle state by name.
    pub fn lifecycle_state(&self, name: &str) -> Option<&LifecycleState> {
        self.lifecycle.iter().find(|s| s.name == name)
    }

    /// Whether content in the lifecycle state counts as available. `None`
    /// when the state isn't declared.
    pub fn state_is_available(&self, name: &str) -> Option<bool> {
        self.lifecycle_state(name).map(|s| s.available)
    }

    /// A feature by key.
    pub fn feature(&self, key: &str) -> Option<&Feature> {
        self.features.iter().find(|f| f.key == key)
    }

    /// A note type by name.
    pub fn note_type(&self, name: &str) -> Option<&NoteType> {
        self.notes.iter().find(|n| n.name == name)
    }

    /// A widget's directive schema.
    pub fn widget_schema(&self, name: &str) -> Option<&DirectiveSchema> {
        self.widgets
            .iter()
            .map(|w| &w.schema)
            .find(|s| s.name == name)
    }

    /// A widget by name.
    pub fn widget(&self, name: &str) -> Option<&Widget> {
        self.widgets.iter().find(|w| w.schema.name == name)
    }

    /// The schemas of every directive: the built-ins (SPEC §4), then the
    /// project's widgets. This is what the parser takes as input.
    pub fn directive_schemas(&self) -> Vec<DirectiveSchema> {
        let mut all = builtin_schemas();
        all.extend(self.widgets.iter().map(|w| w.schema.clone()));
        all
    }

    /// The directive keyword set: built-in names, then widget names.
    pub fn directive_keywords(&self) -> Vec<String> {
        self.directive_schemas()
            .into_iter()
            .map(|s| s.name)
            .collect()
    }

    /// A source by name.
    pub fn source(&self, name: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.name == name)
    }

    /// A build by name.
    pub fn build(&self, name: &str) -> Option<&Build> {
        self.builds.iter().find(|b| b.name == name)
    }

    /// The build whose page-level diagnostics the editor reports by default.
    pub fn editor_default_build(&self) -> &Build {
        // Loading guarantees the build exists and that there is at least one.
        self.build(&self.editor_build)
            .unwrap_or_else(|| &self.builds[0])
    }

    /// The consumer settings.
    pub fn consumer(&self) -> &Consumer {
        &self.consumer
    }

    /// Whether a file (path relative to the content root, `/`-separated) is a
    /// fragment: any segment begins with `_`, or a fragment pattern matches
    /// (SPEC §2.2).
    pub fn is_fragment(&self, path: &str) -> bool {
        path.split('/').any(|s| s.starts_with('_'))
            || self.fragments.patterns.iter().any(|p| p.matches(path))
    }

    /// Whether the project declares its own content types, rather than
    /// having the implicit `page` type.
    pub fn declares_types(&self) -> bool {
        !self.types.iter().any(|t| t.implicit)
    }

    /// The content type a page has, given its frontmatter (`None` when it
    /// has none, or it isn't valid YAML): the type its `type` names (SPEC
    /// §7.2). Don't call it for fragments.
    pub fn type_of(&self, frontmatter: Option<&serde_yaml_ng::Value>) -> TypeMatch<'_> {
        match frontmatter.and_then(|f| f.get(TYPE_KEY)) {
            None => self.type_named(None),
            Some(serde_yaml_ng::Value::String(name)) => self.type_named(Some(name)),
            Some(_) => TypeMatch::Unknown,
        }
    }

    /// The content type a page that names `declared` with `type` has, or,
    /// for `None`, one that names none.
    pub fn type_named(&self, declared: Option<&str>) -> TypeMatch<'_> {
        match declared {
            None => match self.types.iter().find(|t| t.implicit) {
                Some(t) => TypeMatch::One(t),
                None => TypeMatch::Missing,
            },
            Some(name) => match self.types.iter().find(|t| !t.implicit && t.name == name) {
                Some(t) => TypeMatch::One(t),
                None => TypeMatch::Unknown,
            },
        }
    }

    /// The names an availability spec may use as targets: dimension values
    /// and dimension names.
    pub fn is_target(&self, name: &str) -> bool {
        self.dimension(name).is_some() || self.dimension_of_value(name).is_some()
    }

    /// Checks an availability spec against the model: names, versionless
    /// targets, and history order. A spec that is one bare name declared as a
    /// feature key is a feature reference and has no problems here.
    pub fn check_availability(&self, spec: &AvailabilitySpec) -> Vec<AvailabilityProblem> {
        if spec
            .bare_name()
            .is_some_and(|n| self.feature(&n.text).is_some())
        {
            return Vec::new();
        }
        check_entries(&self.dimensions, &self.lifecycle, &spec.entries)
    }
}

/// Checks availability entries against the declared dimensions and states.
pub(crate) fn check_entries(
    dimensions: &[Dimension],
    lifecycle: &[LifecycleState],
    entries: &[Entry],
) -> Vec<AvailabilityProblem> {
    let mut problems = Vec::new();
    let state_known = |n: &Name| lifecycle.iter().any(|s| s.name == n.text);
    for entry in entries {
        let target = &entry.target;
        let versionless = target_is_versionless(dimensions, &target.text);
        if versionless.is_none() {
            problems.push(AvailabilityProblem::UnknownTarget(target.clone()));
        }
        // SPEC §4.4: a dimension name takes no version, whether or not its
        // values are versionless.
        let dimension = dimensions.iter().find(|d| d.name == target.text);
        let check_version = |v: &Version, problems: &mut Vec<AvailabilityProblem>| {
            if let Some(d) = dimension {
                let example = d
                    .values
                    .iter()
                    .find(|value| !value.versionless)
                    .or(d.values.first())
                    .map(|value| value.value.clone())
                    .unwrap_or_default();
                problems.push(AvailabilityProblem::DimensionVersion {
                    target: target.clone(),
                    version: v.clone(),
                    example,
                });
            } else if versionless == Some(true) {
                problems.push(AvailabilityProblem::VersionlessVersion {
                    target: target.clone(),
                    version: v.clone(),
                });
            }
        };
        match &entry.detail {
            Detail::None => {}
            Detail::Version(v) => check_version(v, &mut problems),
            Detail::State { state, version } => {
                if !state_known(state) {
                    problems.push(AvailabilityProblem::UnknownState(state.clone()));
                }
                if let Some(v) = version {
                    check_version(v, &mut problems);
                }
            }
            Detail::History(steps) => {
                for step in steps {
                    if !state_known(&step.state) {
                        problems.push(AvailabilityProblem::UnknownState(step.state.clone()));
                    }
                    check_version(&step.version, &mut problems);
                }
                for pair in steps.windows(2) {
                    if pair[0].version.compare(&pair[1].version).is_gt() {
                        let show = |s: &ascribe_core::availability::HistoryStep| {
                            format!("{} {}", s.state.text, s.version.text)
                        };
                        problems.push(AvailabilityProblem::HistoryOrder {
                            target: target.clone(),
                            later: show(&pair[0]),
                            earlier: show(&pair[1]),
                            span: pair[1].span,
                        });
                        break;
                    }
                }
            }
        }
    }
    problems
}

/// `None` when `name` isn't a target. `Some(true)` when it is a versionless
/// dimension value. A dimension name is `Some(false)`; versions on it are
/// rejected separately.
fn target_is_versionless(dimensions: &[Dimension], name: &str) -> Option<bool> {
    if dimensions.iter().any(|d| d.name == name) {
        return Some(false);
    }
    dimensions
        .iter()
        .flat_map(|d| &d.values)
        .find(|v| v.value == name)
        .map(|v| v.versionless)
}
