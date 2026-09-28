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
