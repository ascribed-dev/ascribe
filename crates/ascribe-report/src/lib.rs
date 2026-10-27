//! The programs Ascribe runs and reads the answers of, and what it
//! remembers of them between runs: what `ascribe report` reports beyond
//! `ascribe check`, and the prose check.
//!
//! - **The programs:** [`tool`] runs one, with a fixed argument list and a
//!   time limit; Vale checks the prose ([`prose`]), lychee the external
//!   links ([`links`]), and `afdocs` a published site ([`site`]). Each is
//!   behind a trait, so a test gives a stand-in.
//! - **What's remembered:** the link answers reused and what's known of
//!   each link (both in [`links`]), and the report's own history
//!   ([`report_history`]), read and written through the
//!   [`Store`](store::Store) the caller hands it.
//!
//! The checks themselves, and a finding's identity across runs, are
//! `ascribe_check`'s; this crate words what the programs say as its
//! [`Diagnostic`](ascribe_check::Diagnostic)s.

pub mod links;
pub mod prose;
pub mod report_history;
pub mod site;
pub mod store;
pub mod tool;
