//! `ascribe_fmt::format` on arbitrary text, under the conformance suite's
//! shared model (its widgets and note types make more lines directives): it
//! never panics, its edits apply, and formatting its own output changes
//! nothing.

#![no_main]

use std::sync::LazyLock;

use ascribe_core::{FileId, apply_edits};
use ascribe_model::ContentModel;
use ascribe_syntax::ParseOptions;
use libfuzzer_sys::fuzz_target;

static MODEL: LazyLock<(ContentModel, ParseOptions)> = LazyLock::new(|| {
    let text = include_str!("../../tests/conformance/_model/ascribe.toml");
    let model = match ascribe_model::load_str(text, FileId::new(0)) {
        Ok(model) => model,
        Err(issues) => panic!("the shared model doesn't load: {issues:?}"),
    };
    let options = ascribe_fmt::options_from_model(&model);
    (model, options)
});

fuzz_target!(|source: &str| {
    // A lone carriage return shifts the spans after it, and the formatter
    // edits the wrong bytes (#1121). libFuzzer stops at the first crash, so
    // until that's fixed these inputs are left out, or the target would find
    // only that.
    if source.replace("\r\n", "").contains('\r') {
        return;
    }
    let (model, options) = &*MODEL;
    let edits = ascribe_fmt::format(source, options, model);
    let once = match apply_edits(source, &edits) {
        Ok(once) => once,
        Err(e) => panic!("the edits don't apply: {e:?}\n{edits:?}"),
    };
    let again = ascribe_fmt::format(&once, options, model);
    assert!(
        again.is_empty(),
        "formatting again changes it:\n{once:?}\n{again:?}"
    );
});
