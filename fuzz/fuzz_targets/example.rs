//! The JSON example validator on an arbitrary schema and example
//! (`api-example-*`, SPEC §7.9): the two halves of the input are a schema
//! document and an example text. `ascribe_resolve::json_schema::check`
//! compiles the schema and checks the text against it, in both dialects,
//! loading nothing from outside the document, and never panics, hangs, or
//! runs away on a recursive schema or a `$ref` cycle.

#![no_main]

use ascribe_resolve::json_schema::{Dialect, check};
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

fuzz_target!(|data: (&str, &str)| {
    let (schema_text, example) = data;
    // The schema is a JSON document; an example is checked against the value
    // at its root pointer.
    let Ok(document) = serde_json::from_str::<Value>(schema_text) else {
        return;
    };
    let _ = check(&document, Dialect::Draft2020, "", example);
    let _ = check(&document, Dialect::Draft4, "", example);
});
