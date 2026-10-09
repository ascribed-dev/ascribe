//! Writing a command a reader can paste into a shell: the `next_command`
//! that `check` and `refs` suggest when they cut a list.
//!
//! The quoting lives with the agent prompts in `ascribe_check::prompt`,
//! which write commands too and can't depend on this crate; this is its
//! name here, so the two can't drift apart.

/// A word as a POSIX shell reads it: as it is when it's plain, else in
/// single quotes.
pub use ascribe_check::prompt::shell_word as quote;
