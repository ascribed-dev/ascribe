//! `ascribe_model::load_str_in` on arbitrary TOML text: it never panics. The
//! project directory is one that doesn't exist, so the rules that look at
//! the file system run and find nothing, and no input reads the disk.

#![no_main]

use std::path::Path;

use ascribe_core::FileId;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    let _ =
        ascribe_model::load_str_in(text, FileId::new(0), Path::new("/nonexistent/ascribe-fuzz"));
});
