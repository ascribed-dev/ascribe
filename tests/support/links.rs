//! Making links in tests, the same way on every platform.
//!
//! Include it with a `#[path]` module from a test file:
//!
//! ```ignore
//! #[path = "../../../tests/support/links.rs"]
//! mod links;
//! ```
//!
//! A target is written as a symbolic link's is: absolute, or relative to the
//! link's own folder. On Unix both kinds are symbolic links. On Windows a
//! linked folder is a directory junction, which needs no privilege, and a
//! linked file is a symbolic link, which needs one: where it can't be made,
//! [`file`] prints why and returns `false`, and the test leaves out what
//! needs that link.

#![allow(dead_code)]

use std::io::{self, Write};
use std::path::Path;

/// Links `link` to the folder `target`.
///
/// # Panics
///
/// If the link can't be made.
pub fn dir(target: impl AsRef<Path>, link: impl AsRef<Path>) {
    let (target, link) = (target.as_ref(), link.as_ref());
    if let Err(error) = make_dir(target, link) {
        panic!(
            "a link from {} to {}: {error}",
            link.display(),
            target.display()
        );
    }
}

/// Links `link` to the file `target`, or prints why it can't and returns
/// `false`: on Windows, when the account may not make symbolic links.
///
/// # Panics
///
/// If the link can't be made for any other reason.
#[must_use]
pub fn file(target: impl AsRef<Path>, link: impl AsRef<Path>) -> bool {
    let (target, link) = (target.as_ref(), link.as_ref());
    match make_file(target, link) {
        Ok(()) => true,
        Err(error) if is_privilege(&error) => {
            // Straight to stderr, past the test harness's capture, so the
            // skip shows in the log of a passing run.
            let _ = writeln!(
                io::stderr(),
                "skipped: can't make a symbolic link at {} ({error})",
                link.display()
            );
            false
        }
        Err(error) => panic!(
            "a link from {} to {}: {error}",
            link.display(),
            target.display()
        ),
    }
}

#[cfg(unix)]
fn make_dir(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(unix)]
fn make_file(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(unix)]
fn is_privilege(_: &io::Error) -> bool {
    false
}

/// A junction, made by `mklink /J`: the standard library has no call for
/// one. Its target must be absolute.
#[cfg(windows)]
fn make_dir(target: &Path, link: &Path) -> io::Result<()> {
    let target = absolute(target, link);
    // `cmd` would read a `/` as the start of a switch.
    let link: std::path::PathBuf = link.components().collect();
    let out = std::process::Command::new("cmd")
        .arg("/C")
        .arg("mklink")
        .arg("/J")
        .arg(&link)
        .arg(&target)
        .output()?;
    if out.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "mklink /J failed: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )))
    }
}

#[cfg(windows)]
fn make_file(target: &Path, link: &Path) -> io::Result<()> {
    // Windows reads a relative target with either separator, but write it
    // its own way.
    let target: std::path::PathBuf = target.components().collect();
    std::os::windows::fs::symlink_file(target, link)
}

/// `ERROR_PRIVILEGE_NOT_HELD`.
#[cfg(windows)]
fn is_privilege(error: &io::Error) -> bool {
    error.raw_os_error() == Some(1314)
}

/// `target` made absolute from the link's folder, with `.` and `..` worked
/// out, since a junction's target is stored as written.
#[cfg(windows)]
fn absolute(target: &Path, link: &Path) -> std::path::PathBuf {
    use std::path::{Component, PathBuf};

    let joined = link.parent().unwrap_or(Path::new("")).join(target);
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}
