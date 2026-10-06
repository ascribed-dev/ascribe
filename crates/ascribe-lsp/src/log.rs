//! The server's log. Standard error is the log by protocol: the client shows
//! it (VS Code, in the extension's output channel), and standard output
//! carries the protocol itself. Every line the server logs goes through
//! [`line`], the one place in the library crates that prints.

use std::fmt::Arguments;

/// Writes one line to the log, prefixed `ascribe-lsp: `.
#[allow(clippy::print_stderr)] // Standard error is the server's log by protocol.
pub(crate) fn line(text: Arguments<'_>) {
    eprintln!("ascribe-lsp: {text}");
}
