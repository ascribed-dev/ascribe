//! The OpenAPI reader on arbitrary bytes, read as both YAML and JSON
//! (`[apis.<name>] file`, SPEC §7.9): `ascribe_resolve::openapi::read_spec`
//! never panics, and translating a spec it read into JSON Schema for request
//! and response bodies, which walks and rewrites the document and follows its
//! `$ref`s, terminates and never panics on a `$ref` cycle.

#![no_main]

use ascribe_resolve::openapi::read_spec;
use ascribe_resolve::openapi::schemas::{Direction, dialect, translated};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|text: &str| {
    for json in [false, true] {
        if let Ok(spec) = read_spec(text, json) {
            // Exercise the operation accessors.
            let _ = spec.ids().count();
            for operation in &spec.operations {
                let _ = operation.method_and_path();
            }
            // The recursive walks and `$ref`-following of the schema
            // translation, in both directions.
            let dialect = dialect(&spec.document);
            let _ = translated(&spec.document, dialect, Direction::Request);
            let _ = translated(&spec.document, dialect, Direction::Response);
        }
    }
});
