//! Minimal reproductions of the recognition risks the phase names, written by
//! hand (no corpus, so they run offline). Each says what the parser and
//! file-level checks find in one small page, and whether that is intended.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::print_stderr
)]

use tessera_corpora::recognize::recognize;

fn classes(text: &str) -> Vec<String> {
    recognize(&[("page.md".to_owned(), text.to_owned())])
        .findings
        .into_iter()
        .map(|f| f.class)
        // Frontmatter isn't what these pages test.
        .collect()
}

#[test]
fn a_package_name_at_line_start_is_prose() {
    // SPEC §3.2: `@` followed by an unknown word and `/` isn't directive-shaped.
    assert!(classes("@astrojs/react is an integration.\n").is_empty());
    assert!(classes("Install `@astrojs/react`, then run it.\n").is_empty());
}

#[test]
fn a_word_after_at_and_a_space_is_prose() {
    assert!(classes("@timestamp is a field.\n").is_empty());
    assert!(classes("- @timestamp is a field.\n").is_empty());
}

#[test]
fn prose_at_in_the_middle_of_a_line_is_never_a_directive() {
    assert!(classes("Write to support@example.com or @note me.\n").is_empty());
}

#[test]
fn a_directive_shaped_line_with_an_unknown_name_warns() {
    // Intended (SPEC §3.2, §8.2): `@word` alone, or before `:` or `{`, is
    // literal text but SHOULD warn. `\@` silences it.
    assert_eq!(classes("@timestamp\n"), ["diagnostic:directive-unknown"]);
    assert_eq!(
        classes("@timestamp: the event time\n"),
        ["diagnostic:directive-unknown"]
    );
    assert!(classes("\\@timestamp\n").is_empty());
}

#[test]
fn a_known_keyword_in_prose_is_a_directive() {
    // Intended, and a real risk for documentation about other tools that use
    // the same word at line start (Sass's `@include`): SPEC §3.2 makes a known
    // keyword a directive wherever it begins a line outside code.
    let found = classes("@include mixin(x)\n");
    assert!(found.contains(&"directive:include".to_owned()), "{found:?}");
    assert!(
        found.contains(&"diagnostic:directive-extra-text".to_owned()),
        "{found:?}"
    );
    // Escaped, or in code, it isn't.
    assert!(classes("\\@include mixin(x)\n").is_empty());
    assert!(classes("```scss\n@include mixin(x)\n```\n").is_empty());
}

#[test]
fn a_dot_word_line_is_prose_unless_a_directive_follows() {
    // SPEC §3.7: `.NET is a framework` is unaffected.
    assert!(classes(".NET is a framework\n").is_empty());
    assert!(classes("Text\n\n.env holds secrets\n").is_empty());
    assert!(classes(". Try it\n").is_empty());
}

#[test]
fn a_dot_line_above_a_directive_is_a_title() {
    let found = classes(".NET\n@note: hi\n");
    assert!(found.contains(&"title".to_owned()), "{found:?}");
}

#[test]
fn braces_in_prose_are_candidates_and_warn() {
    // Intended (SPEC §5.1): `{key}` text whose key isn't declared is literal,
    // with a warning that says it would change if the key were declared.
    let found = classes("Open {namespace}/{name}.\n");
    assert_eq!(
        found
            .iter()
            .filter(|c| *c == "phrase-candidate:prose:plain")
            .count(),
        2
    );
    assert_eq!(
        found
            .iter()
            .filter(|c| *c == "diagnostic:phrase-undeclared:plain")
            .count(),
        2
    );
}

#[test]
fn braces_that_are_not_keys_are_not_candidates() {
    for text in [
        "A JSON object: { \"a\": 1 } and {} and {1,2}.\n",
        "The regex a{2,3} and {Object} and {a b} and {A}.\n",
        "Use `{key}` in a span.\n",
        "```json\n{key}\n```\n",
        "    {key} in an indented block\n",
        "<div>{key}</div>\n",
    ] {
        assert!(classes(text).is_empty(), "{text:?} -> {:?}", classes(text));
    }
}

#[test]
fn an_escaped_brace_is_recorded_and_silent() {
    assert_eq!(classes("Use \\{key} literally.\n"), ["escaped-phrase"]);
}

#[test]
fn double_braces_hold_a_candidate() {
    // `{{es}}` (a mustache substitution) contains the candidate `{es}`. The
    // spec lets braces sit against punctuation, so this is intended; with
    // `es` declared, `ascribe check` warns (`phrase-double-braces`, Q191).
    let found = classes("Use {{es}} here.\n");
    assert!(
        found.contains(&"phrase-candidate:prose:mustache".to_owned()),
        "{found:?}"
    );
}

#[test]
fn a_directive_fence_of_another_tool_warns_as_prose() {
    // Elastic's `:::{note}` is a paragraph here; its `{note}` is a candidate.
    let found = classes(":::{note}\nText.\n:::\n");
    assert!(
        found.contains(&"phrase-candidate:prose:directive-fence".to_owned()),
        "{found:?}"
    );
}

// The findings of FINDINGS.md that are about the language, each as the
// smallest page that shows it.

#[test]
fn f1_an_availability_line_above_an_include_binds_nothing() {
    let found = classes("Text.\n\n@available: cloud\n@include: _f.md\n");
    assert!(
        found.contains(&"diagnostic:binding-no-block".to_owned()),
        "{found:?}"
    );
}

#[test]
fn f2_a_title_that_starts_with_a_dot_escapes_it() {
    // Resolved Q193: `.\.NET` is the title `.NET`. `..NET` is still no title,
    // so that arm has none.
    let found = classes(".\\.NET\n@variant:\ntext\n@end\n");
    assert!(
        !found.contains(&"diagnostic:variant-arm-kind".to_owned()),
        "{found:?}"
    );
    let found = classes("..NET\n@variant:\ntext\n@end\n");
    assert!(
        found.contains(&"diagnostic:variant-arm-kind".to_owned()),
        "{found:?}"
    );
}

#[test]
fn f5_an_id_can_contain_an_underscore_and_a_period() {
    // Resolved Q196.
    let found = classes("## Setup\n@id: ece_setup.v2\n");
    assert!(
        !found.contains(&"diagnostic:id-invalid".to_owned()),
        "{found:?}"
    );
    let found = classes("## Setup\n@id: ece:setup\n");
    assert!(
        found.contains(&"diagnostic:id-invalid".to_owned()),
        "{found:?}"
    );
}
