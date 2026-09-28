//! The project graph and the resolution passes: includes, availability, build modes, phrases, heading ids, links, and glossary.
//!
//! Phase 11 builds the **source index** and expands includes; phases 09, 12, and 13 add slugging, build resolution, and incremental updates.
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
//!   ([`Resolution`]), resolved from the file it's written in
//!   (`project-docs/contracts/assets.md`);
//! - the edges between files, forward and back: [`Project::includers`],
//!   [`Project::including_pages`], [`Project::links_to`],
//!   [`Project::links_to_id`], and [`Project::asset_users`];
//! - the problems that follow from the source ([`Project::problems`]), by
//!   registry slug, for the checks to report.
//!
//! # Includes
//!
//! [`Project::expand`] replaces each `@include` by its target, recursively, into
//! an [`ExpandedPage`] whose every block keeps the file and span it was written
//! in, so relative references resolve from the right file (SPEC §4.2).
//! [`Project::assets`] lists the assets a page uses, with where each reference
//! is written and the includes it came through.
//!
//! Nothing here depends on a build: availability, variants, and phrase
//! substitution in content are phase 12's.

mod expand;
pub mod fs;
mod index;
mod layout;
mod project;
pub mod slug;

pub use expand::{
    ExpandedArm, ExpandedBlock, ExpandedItem, ExpandedKind, ExpandedPage, IncludeSite, PageProblem,
};
pub use fs::{DiskFs, FileSystem, MemoryFs, Probe};
pub use index::{
    AvailabilityMarker, ExplicitId, FileIndex, FileKind, Heading, Include, Local, PhrasePlace,
    PhraseUse, RefKind, Reference, Target, index_file,
};
pub use layout::Layout;
pub use project::{
    AssetSite, IncludeEdge, LinkSite, Missing, PageAsset, Project, Resolution, Unreadable,
};
