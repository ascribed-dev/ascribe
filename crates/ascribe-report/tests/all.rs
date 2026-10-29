//! Every integration test of `ascribe-report` in one program, as
//! `ascribe-check`'s are. Each file in `all/` is named with `#[path]`. To run
//! one file's tests: `cargo test -p ascribe-report --test all vale`.

#[path = "all/link_cache.rs"]
mod link_cache;
#[path = "all/link_retry.rs"]
mod link_retry;
#[path = "all/site.rs"]
mod site;
#[path = "all/vale.rs"]
mod vale;
