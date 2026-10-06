//! The project graph and the resolution passes: includes, availability, build modes, phrases, heading ids, links, and glossary.
//!
//! # The source index
//!
//! [`Project::load`] indexes every `.md` file under the content root, with
//! nothing from any build:
//!
//! - one [`FileIndex`] per file, a pure function of the file's text, its path,
//!   and the content model ([`index_file`]): headings with their **source ids**
//!   (SPEC §5.5), titles, includes, links, images, phrase candidates, and
//!   availability markers;
//! - what every link and image **names** once the project's files are known
//!   ([`Resolution`]), resolved from the file it's written in;
//! - the edges between files, forward and back: [`Project::includers`],
//!   [`Project::including_pages`], [`Project::links_to`],
//!   [`Project::links_to_id`], and [`Project::asset_users`];
//! - the problems that follow from the source ([`Project::problems`]), by
//!   registry slug, for the checks to report.
//!
//! # Includes and snippets
//!
//! [`Project::expand`] replaces each `@include` by its target, recursively, into
//! an [`ExpandedPage`] whose every block keeps the file and span it was written
//! in, so relative references resolve from the right file (SPEC §4.2).
//! [`Project::assets`] lists the assets a page uses, with where each reference
//! is written and the includes it came through.
//!
//! Each `@snippet` becomes a code block in the expansion: the code its
//! address names, read through a source when the project is loaded (SPEC
//! §4.8). The [`snippet`] module holds the rules, shared with the file-level
//! checks.
//!
//! Nothing above depends on a build. What does is [`build`]:
//!
//! # Build resolution
//!
//! [`Project::resolve_page`], [`Project::resolve_build`], and
//! [`BuildResolver`] turn expanded pages into [`ResolvedPage`]s for a build:
//! availability and build modes applied, phrases substituted, every heading
//! given its page id, links resolved to routes, assets carried, and glossary
//! terms linked (SPEC §9.2 steps 2 to 7). Problems that only a build finds
//! (`variant-no-arm-survives`, `available-exceeds-scope`, `link-id-removed`,
//! `link-page-dropped`) are recorded on the page for the page-level checks
//! to report. See the [`build`] module for the passes.
//!
//! # Incremental updates
//!
//! An [`IncrementalProject`] keeps a project current as files change:
//! [`IncrementalProject::apply`] takes [`Change`]s and returns an [`Affected`]
//! (the files to re-check, the pages to re-resolve), and every state is a
//! [`Snapshot`] with a [`Version`]. The result always equals loading the same
//! files from scratch. See the [`incremental`] module for what each change
//! invalidates, what's cached, how versions tell a consumer whether a result
//! is current, and the rules for file ids.
//!
//! ```
//! use std::sync::Arc;
//! use tessera_core::{FileId, RelPath};
//! use tessera_resolve::{Layout, MemoryFs, Project};
//!
//! let model = tessera_model::load_str("spec = \"0.1\"\n", FileId::new(0)).expect("a valid model");
//! let layout = Layout::from_model(&model);
//! let fs = MemoryFs::new(&layout)
//!     .with_source("index.md", "---\ntitle: Home\n---\n\n@include: _f.md#setup\n")
//!     .with_source("_f.md", "## Setup\n\nInstall it.\n");
//! let project = Project::load(Arc::new(model), layout, &fs);
//!
//! let fragment = RelPath::parse("_f.md").expect("a path");
//! assert_eq!(project.heading(&fragment, "setup").map(|h| h.text.as_str()), Some("Setup"));
//! let page = project.expand(&RelPath::parse("index.md").expect("a path")).expect("a page");
//! assert!(page.problems.is_empty());
//! assert_eq!(page.blocks.len(), 2); // the heading and the paragraph
//! ```

mod astro;
pub mod build;
mod expand;
pub mod fs;
pub mod incremental;
mod index;
mod layout;
mod project;
pub mod references;
pub mod slug;
pub mod snippet;

pub use astro::AstroRouter;
pub use build::{
    Annotation, Availability, BuildResolver, DefaultRouter, DropReason, DroppedPage,
    FormattedField, GlossaryUse, HeadingIds, LinkTarget, ResolvedArm, ResolvedBlock, ResolvedBuild,
    ResolvedItem, ResolvedKind, ResolvedLink, ResolvedPage, ResolvedRow, Scope, Substitution,
    glossary_targets,
};
pub use expand::{
    ExpandedArm, ExpandedBlock, ExpandedItem, ExpandedKind, ExpandedPage, IncludeSite, PageProblem,
};
pub use fs::{DiskFs, FileSystem, MemoryFs, Probe, Sources, in_nested_project, is_source_path};
pub use incremental::{
    Affected, ApplyError, Change, FileIds, IncrementalProject, ModelImpact, ResolvedCache,
    Snapshot, Stats, Version,
};
pub use index::{
    AvailabilityMarker, ExplicitId, FileIndex, FileKind, Heading, Include, Local, PhrasePlace,
    PhraseUse, RefKind, Reference, Target, heading_text, index_file, index_parsed, parse_source,
};
pub use layout::Layout;
pub use project::{
    AssetSite, IncludeEdge, LinkSite, Missing, PageAsset, Project, Resolution, Unreadable,
};
pub use references::{
    IncludeTarget, SourceSet, destination_phrases, destination_span, include_issue, include_target,
    reference_issue, reference_target, resolve_reference,
};
pub use snippet::{
    Address, AddressError, CodeFile, CodeFiles, Snippet, SnippetUse, parse_address,
    resolve_snippet, snippet_issues,
};
