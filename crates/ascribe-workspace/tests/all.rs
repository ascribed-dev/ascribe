//! Every integration test of `ascribe-workspace` in one program. Each file
//! is named with `#[path]`, since a plain `mod` here would look beside this
//! file, in `tests/`, where Cargo builds every file as a program of its own.
//! To run one file's tests: `cargo test -p ascribe-workspace --test all load`.

#[path = "all/load.rs"]
mod load;
#[path = "all/locate.rs"]
mod locate;
#[path = "all/workspace.rs"]
mod workspace;
