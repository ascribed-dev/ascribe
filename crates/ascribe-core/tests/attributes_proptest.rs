//! Property tests: the attribute parser never panics, and every span it
//! returns is in range and on character boundaries, whatever the input.

use ascribe_core::{AttributeValue, FileId, Span, parse_attribute_block};
use proptest::prelude::*;

/// Text weighted toward the characters the grammar cares about.
fn attribute_like() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just("{".to_owned()),
            Just("}".to_owned()),
            Just("=".to_owned()),
            Just(",".to_owned()),
            Just("|".to_owned()),
            Just("\"".to_owned()),
            Just("\\".to_owned()),
            Just(" ".to_owned()),
            Just("\t".to_owned()),
            Just("key".to_owned()),
            Just("é→😀".to_owned()),
            "[a-z0-9-]{1,4}",
            any::<char>().prop_map(String::from),
        ],
        0..24,
    )
    .prop_map(|parts| format!("{{{}", parts.concat()))
}

fn check_span(text: &str, offset: usize, span: Span) {
    assert!(span.start() >= offset && span.end() <= offset + text.len());
    assert!(text.is_char_boundary(span.start() - offset));
    assert!(text.is_char_boundary(span.end() - offset));
}

fn check(text: &str) {
    let offset = 7;
    let Some(p) = parse_attribute_block(text, offset, FileId::new(0)) else {
        assert!(!text.starts_with('{'));
        return;
    };
    assert!(p.len <= text.len() && text.is_char_boundary(p.len));
    check_span(text, offset, p.block.span);
    assert_eq!(p.block.span, Span::new(offset, offset + p.len));
    for a in &p.block.attributes {
        check_span(text, offset, a.key_span);
        check_span(text, offset, a.span);
        match &a.value {
            Some(AttributeValue::Set { members, span }) => {
                assert!(members.len() >= 2);
                check_span(text, offset, *span);
                members
                    .iter()
                    .for_each(|m| check_span(text, offset, m.span));
            }
            Some(v) => check_span(text, offset, v.span()),
            None => {}
        }
    }
    for issue in &p.issues {
        check_span(text, offset, issue.location.span);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn never_panics_on_arbitrary_text(text in "\\PC{0,40}") {
        check(&text);
        check(&format!("{{{text}"));
    }

    #[test]
    fn never_panics_on_attribute_like_text(text in attribute_like()) {
        check(&text);
    }
}
