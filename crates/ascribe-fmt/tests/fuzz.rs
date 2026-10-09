//! Random documents: whatever the parser makes of them, formatting applies
//! cleanly, is idempotent, and keeps the outline.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use ascribe_core::apply_edits;
use ascribe_fmt::format;
use proptest::prelude::*;
use support::{options, outline, shared_model_ref};

/// One line of a random document: directive-looking lines with random
/// spacing, attributes, and indentation, and the markdown around them.
fn line() -> impl Strategy<Value = String> {
    prop_oneof![
        6 => "( {0,5}|\t|>| >  |> > |- |1\\. |-   |  - )@(note|steps|available|include|variant|end|id|details|quill-labspace|quill-aside|quill-audience|quill-compare)([ \t]{0,3}\\{[ \t]{0,2}([a-z]{1,6}[ \t]{0,2}=[ \t]{0,2}(\"[a-z ,|=]{0,6}\"|[a-z0-9.|-]{1,6})[ \t]{0,2},?[ \t]{0,2}){0,3}\\})?[ \t]{0,2}:?[ \t]{0,2}([A-Za-z {}.]{0,12})[ \t]{0,3}",
        2 => "( {0,4})@end[ ]{0,3}",
        2 => "( {0,4})\\.[A-Za-z ]{1,8}",
        3 => "[ ]{0,3}[A-Za-z ]{0,12}",
        2 => Just(String::new()),
        1 => Just(">".to_owned()),
        1 => Just("- item".to_owned()),
        1 => Just("1. item".to_owned()),
        1 => Just("## Heading".to_owned()),
        1 => Just("```".to_owned()),
        1 => Just("    code".to_owned()),
        1 => Just("[ref]: {api}x \"t\"".to_owned()),
        1 => "!\\[a\\]\\(b\\)\\{[ ]{0,2}([a-z]{1,6}[ ]{0,2}=[ ]{0,2}(\"[a-z ]{0,4}\"|[a-z0-9]{1,4}),?[ ]{0,2}){0,3}\\}[ a-z]{0,3}",
        1 => Just("| a | b |".to_owned()),
        1 => Just("|---|---|".to_owned()),
    ]
}

fn document() -> impl Strategy<Value = String> {
    prop::collection::vec(line(), 0..24).prop_map(|lines| lines.join("\n") + "\n")
}

proptest! {
    #![proptest_config(ProptestConfig { failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn formatting_is_idempotent_and_keeps_the_outline(source in document()) {
        let model = shared_model_ref();
        let options = options(model);
        let edits = format(&source, &options, model);
        let once = apply_edits(&source, &edits).expect("edits apply");
        prop_assert!(
            format(&once, &options, model).is_empty(),
            "not idempotent:\n{source:?}\n{once:?}"
        );
        prop_assert_eq!(
            outline(&source, &options),
            outline(&once, &options),
            "outline changed:\n{:?}\n{:?}", source, once
        );
    }
}

// Text soup: arbitrary text, and it never panics.
proptest! {
    #![proptest_config(ProptestConfig { failure_persistence: None, ..ProptestConfig::default() })]

    #[test]
    fn arbitrary_text_never_panics(source in "[@{}:=,|\" \\n\\t>.a-z0-9!\\[\\]()\\\\-]{0,200}") {
        let model = shared_model_ref();
        let options = options(model);
        let edits = format(&source, &options, model);
        let once = apply_edits(&source, &edits).expect("edits apply");
        prop_assert!(format(&once, &options, model).is_empty(), "{source:?} -> {once:?}");
    }
}
