//! Property tests: the info-string parser never panics, every word and value
//! it returns is in range and on character boundaries, and a quoted value
//! comes back from `quote`, whatever the input.

use ascribe_core::info_string::{InfoString, quote};
use proptest::prelude::*;

/// Text weighted toward the characters the grammar cares about.
fn info_like() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just("=".to_owned()),
            Just("\"".to_owned()),
            Just("\\".to_owned()),
            Just(" ".to_owned()),
            Just("\t".to_owned()),
            Just("title".to_owned()),
            Just("é→😀".to_owned()),
            "[a-z0-9-]{1,4}",
            any::<char>().prop_map(String::from),
        ],
        0..24,
    )
    .prop_map(|parts| parts.concat())
}

proptest! {
    #[test]
    fn words_and_values_are_in_range(text in info_like()) {
        let info = InfoString::parse(&text);
        for word in &info.words {
            let end = word.start + word.text.len();
            prop_assert!(end <= text.len());
            prop_assert_eq!(&text[word.start..end], word.text);
            if let Some(pair) = &word.pair {
                let value_end = pair.value_start + pair.value_len;
                prop_assert!(pair.value_start >= word.start && value_end <= end);
                prop_assert!(text.is_char_boundary(pair.value_start));
                prop_assert!(text.is_char_boundary(value_end));
            }
        }
    }

    #[test]
    fn a_quoted_value_round_trips(value in info_like()) {
        let text = format!("text title={} x", quote(&value));
        let info = InfoString::parse(&text);
        prop_assert_eq!(info.get("title"), Some(value.as_str()));
    }
}
