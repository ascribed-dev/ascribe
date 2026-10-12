//! Every integration test of `ascribe-templates` in one program. Each file in
//! `all/` is named with `#[path]`, since a plain `mod` here would look beside
//! this file. To run one file's tests: `cargo test -p ascribe-templates --test
//! all set::`.

#[path = "all/set.rs"]
mod set;
