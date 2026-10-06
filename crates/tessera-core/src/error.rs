//! [`Coded`]: how a caller tells one failure from another.

/// An error with a stable code: a short, lowercase identifier (`git_not_found`,
/// `unknown_build`) that names the failure whatever its message says. The
/// message is for people and may be reworded; the code is for programs, such
/// as a tool that wraps a command, and stays the same.
///
/// Two errors share a code only when they're the same failure (`git` isn't on
/// the path, whichever command needed it). An error that wraps another has
/// the inner error's code. The CLI's tests list every code.
pub trait Coded: std::error::Error {
    /// The failure's code.
    fn code(&self) -> &'static str;
}
