//! The Ascribe conformance harness.
//!
//! Cases live in `tests/conformance/cases/`, one directory each, with an
//! `expect.yaml` describing the expected outline, diagnostics, and per-build
//! results. `tests/conformance/README.md` documents the format.
//!
//! The harness discovers cases ([`discover`]), applies `SKIPS.toml`
//! ([`Skips`]), and runs every remaining case through the registered
//! [`ConformanceAdapter`]s ([`Suite::run`]). Every case ends as passed, failed,
//! or skipped, and a case whose area tag has neither an adapter nor a skip
//! entry fails, so skips are always deliberate. Every diagnostic slug a case
//! expects is checked against the registry, `diagnostics.toml`
//! ([`DiagnosticsRegistry`]), whether or not the case runs.
//!
//! The harness depends on no Ascribe crate. Implementation phases write
//! adapters in `tests/adapters/` and register them in `tests/conformance.rs`.

pub mod adapter;
pub mod case;
pub mod expect;
pub mod outline;
pub mod registry;
pub mod runner;
pub mod skips;

pub use adapter::{
    AdapterError, AdapterResult, BuildResult, ConformanceAdapter, Diagnostic, PageResult, Registry,
};
pub use case::{Case, CaseError, CaseKind, discover};
pub use expect::{BuildExpect, Expect, ExpectedDiagnostic, OutputKind, OutputSlot, PageExpect};
pub use outline::{Arm, AttrValue, Attributes, Binding, Directive, Form, Group, Node, Outline};
pub use registry::{DiagnosticsRegistry, Entry, Level, RegistryError, Severity};
pub use runner::{CaseReport, Filter, Outcome, Report, RunError, Suite, filter_from_args};
pub use skips::{Check, SkipEntry, SkipTarget, Skips};
