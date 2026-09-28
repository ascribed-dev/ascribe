//! One test group per rule of SPEC §8.3, plus the safety rules: what the
//! formatter leaves alone.

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use support::fmt;

#[track_caller]
fn check(input: &str, expected: &str) {
    assert_eq!(fmt(input), expected, "input: {input:?}");
    // Canonical form is a fixed point.
    assert_eq!(fmt(expected), expected, "not idempotent: {expected:?}");
}

#[track_caller]
fn unchanged(input: &str) {
    check(input, input);
}

// ---- One space between a directive name and `{` ------------------------------

#[test]
fn one_space_between_name_and_brace() {
    check("@note{type=tip}: Text.\n", "@note {type=tip}: Text.\n");
    check("@note   {type=tip}: Text.\n", "@note {type=tip}: Text.\n");
    check("@note\t{type=tip}: Text.\n", "@note {type=tip}: Text.\n");
    check(
        "@include{heading=false}: a.md\n",
        "@include {heading=false}: a.md\n",
    );
    unchanged("@note {type=tip}: Text.\n");
}

// ---- No space inside braces; none around `=` or `|`; `, ` between pairs -----

#[test]
fn no_space_inside_braces() {
    check("@note { type=tip }: Text.\n", "@note {type=tip}: Text.\n");
    check("@note {\ttype=tip\t}: Text.\n", "@note {type=tip}: Text.\n");
}

#[test]
fn no_space_around_equals() {
    check("@note {type = tip}: Text.\n", "@note {type=tip}: Text.\n");
    check("@note {type= tip}: Text.\n", "@note {type=tip}: Text.\n");
    check("@note {type =tip}: Text.\n", "@note {type=tip}: Text.\n");
}

#[test]
fn no_space_around_pipes() {
    check(
        "@quill-audience {role = admin | writer |developer}\nText.\n",
        "@quill-audience {role=admin|writer|developer}\nText.\n",
    );
}

#[test]
fn pairs_are_separated_by_comma_and_space() {
    check(
        "@variant {pm=npm,deployment=cloud}:\nA\n@end\n",
        "@variant {pm=npm, deployment=cloud}:\nA\n@end\n",
    );
    check(
        "@variant {pm=npm  ,  deployment=cloud}:\nA\n@end\n",
        "@variant {pm=npm, deployment=cloud}:\nA\n@end\n",
    );
    unchanged("@variant {pm=npm, deployment=cloud}:\nA\n@end\n");
}

// ---- Attributes in the order the schema declares them ------------------------

#[test]
fn attributes_follow_the_schema_order() {
    // A widget: `lab`, then `height`.
    check(
        "@quill-labspace {height=300, lab=intro}\n",
        "@quill-labspace {lab=intro, height=300}\n",
    );
    unchanged("@quill-labspace {lab=intro, height=300}\n");
}

#[test]
fn variant_attributes_follow_the_models_dimensions() {
    // The model declares `pm`, then `deployment`.
    check(
        "@variant {deployment=cloud, pm=npm}:\nA\n@end\n",
        "@variant {pm=npm, deployment=cloud}:\nA\n@end\n",
    );
}

#[test]
fn image_attributes_follow_the_models_order() {
    // `width`, `height`, `loading`.
    check(
        "![Alt](a.png){loading=eager, height=20, width=10}\n",
        "![Alt](a.png){width=10, height=20, loading=eager}\n",
    );
}

#[test]
fn a_block_with_an_unknown_key_keeps_its_order() {
    // `bogus` isn't declared, so nothing can be put in order; the spacing is
    // still fixed.
    check(
        "@quill-labspace {height=300 , bogus=1, lab=x}\n",
        "@quill-labspace {height=300, bogus=1, lab=x}\n",
    );
    check(
        "![Alt](a.png){height=20,  bogus=1, width=10}\n",
        "![Alt](a.png){height=20, bogus=1, width=10}\n",
    );
}

// ---- Values quoted only when necessary --------------------------------------

#[test]
fn quotes_are_dropped_when_not_needed() {
    check("@note {type=\"tip\"}: Text.\n", "@note {type=tip}: Text.\n");
    check(
        "@quill-labspace {lab=\"intro-1\", height=\"300\"}\n",
        "@quill-labspace {lab=intro-1, height=300}\n",
    );
    // An escaped backslash decodes to a single one, which a token may hold.
    check(
        "@quill-labspace {lab=\"a\\\\b\"}\n",
        "@quill-labspace {lab=a\\b}\n",
    );
}

#[test]
fn quotes_stay_when_needed() {
    unchanged("@quill-labspace {lab=\"two words\"}\n");
    unchanged("@quill-labspace {lab=\"a,b\"}\n");
    unchanged("@quill-labspace {lab=\"a=b\"}\n");
    unchanged("@quill-labspace {lab=\"a|b\"}\n");
    unchanged("@quill-labspace {lab=\"a{b}\"}\n");
    unchanged("@quill-labspace {lab=\"say \\\"hi\\\"\"}\n");
    // An empty string can't be a token.
    unchanged("@quill-labspace {lab=\"\"}\n");
    // The rest of the block is still normalized.
    check(
        "@quill-labspace {lab = \"two words\" ,height=\"3\"}\n",
        "@quill-labspace {lab=\"two words\", height=3}\n",
    );
}

#[test]
fn a_token_is_never_quoted() {
    unchanged("@note {type=tip}: Text.\n");
    unchanged("@include {heading=false}: a.md\n");
}

// ---- No empty attribute block -----------------------------------------------

#[test]
fn no_empty_attribute_block() {
    check("@note {}: Text.\n", "@note: Text.\n");
    check("@note {}\nText.\n", "@note\nText.\n");
    check("@note {}:\nText.\n@end\n", "@note:\nText.\n@end\n");
    check("@note{ }  :  Text.\n", "@note: Text.\n");
    check("@steps {}\n1. One\n", "@steps\n1. One\n");
    check("![Alt](a.png){}\n", "![Alt](a.png)\n");
}

#[test]
fn an_image_keeps_an_empty_block_before_a_brace() {
    // Without it, `{cloud}` would become the image's attribute block.
    unchanged("![Alt](a.png){}{cloud}\n");
}

// ---- `:` directly after the name or attribute block, one space before a primary

#[test]
fn colon_follows_the_name_directly() {
    check("@note : Text.\n", "@note: Text.\n");
    check("@note\t: Text.\n", "@note: Text.\n");
    check("@include  : a.md\n", "@include: a.md\n");
    check("## Setup\n@id : setup\n", "## Setup\n@id: setup\n");
}

#[test]
fn colon_follows_the_attribute_block_directly() {
    check("@note {type=tip} : Text.\n", "@note {type=tip}: Text.\n");
    check("@note {type=tip}   :Text.\n", "@note {type=tip}: Text.\n");
}

#[test]
fn one_space_between_colon_and_primary() {
    check("@note:Text.\n", "@note: Text.\n");
    check("@note:    Text.\n", "@note: Text.\n");
    check("@note:\tText.\n", "@note: Text.\n");
    check("@include:a.md\n", "@include: a.md\n");
    check(
        "@available:   cloud, self-managed preview 3.4\nText.\n",
        "@available: cloud, self-managed preview 3.4\nText.\n",
    );
    check(
        "## Op\n@quill-api-ref {version=v3}:   createStream\n",
        "## Op\n@quill-api-ref {version=v3}: createStream\n",
    );
    unchanged("@note: Text.\n");
}

#[test]
fn a_text_primary_keeps_its_continuation_lines_and_trailing_space() {
    check(
        "@note:   Back up your database\n  before you upgrade.  \n",
        "@note: Back up your database\n  before you upgrade.  \n",
    );
}

// ---- Nothing after a container's colon, not even whitespace -------------------

#[test]
fn nothing_after_a_containers_colon() {
    check("@note:   \nText.\n@end\n", "@note:\nText.\n@end\n");
    check(
        "@note {type=tip}:\t \nText.\n@end\n",
        "@note {type=tip}:\nText.\n@end\n",
    );
    check(
        ".Title\n@details:  \nText.\n@end\n",
        ".Title\n@details:\nText.\n@end\n",
    );
    check(
        "@variant {pm=npm}:   \nA\n@end\n",
        "@variant {pm=npm}:\nA\n@end\n",
    );
    check(
        "@note:  \r\nText.\r\n@end\r\n",
        "@note:\r\nText.\r\n@end\r\n",
    );
    unchanged("@note:\nText.\n@end\n");
}

#[test]
fn trailing_space_elsewhere_is_left_alone() {
    unchanged("@note {type=tip}  \nText.\n");
    unchanged("@steps  \n1. One\n");
    unchanged("@note:\nText.\n@end  \n");
}

// ---- No blank line between a following-block directive and its block ----------

#[test]
fn no_blank_line_before_the_block() {
    check("@steps\n\n1. One\n2. Two\n", "@steps\n1. One\n2. Two\n");
    check("@steps\n\n\n\n1. One\n", "@steps\n1. One\n");
    check(
        "@note {type=tip}\n\nText here.\n",
        "@note {type=tip}\nText here.\n",
    );
    check("@note\n   \t\nText.\n", "@note\nText.\n");
    check("@steps\r\n\r\n1. One\r\n", "@steps\r\n1. One\r\n");
    unchanged("@steps\n1. One\n");
}

#[test]
fn stacked_directives_touch_their_block() {
    check(
        "@available: cloud\n\n@note\n\nText.\n",
        "@available: cloud\n@note\nText.\n",
    );
}

#[test]
fn a_bound_container_or_table_or_code_block() {
    check(
        "@available: cloud\n\n@note:\nA.\n@end\n",
        "@available: cloud\n@note:\nA.\n@end\n",
    );
    check(
        "@available: cloud\n\n| a | b |\n|---|---|\n| 1 | 2 |\n",
        "@available: cloud\n| a | b |\n|---|---|\n| 1 | 2 |\n",
    );
    check(
        "@available: cloud\n\n```sh\nnpm i\n```\n",
        "@available: cloud\n```sh\nnpm i\n```\n",
    );
    check(
        "@available: cloud\n\n.Title\n@note:\nA.\n@end\n",
        "@available: cloud\n.Title\n@note:\nA.\n@end\n",
    );
}

#[test]
fn blank_lines_in_lists_and_quotes_are_removed_too() {
    check(
        "- item\n\n  @steps\n\n  1. One\n",
        "- item\n\n  @steps\n  1. One\n",
    );
    check("> @steps\n>\n> 1. One\n", "> @steps\n> 1. One\n");
}

#[test]
fn a_heading_bound_directive_may_have_blank_lines() {
    unchanged("## Setup\n\n@id: setup\n\nText.\n");
    unchanged("## Setup\n\n@available: cloud\n\nText.\n");
}

#[test]
fn own_text_is_never_pulled_into_the_block() {
    // A blank line after a one-line note is what ends its text.
    unchanged("@note: Text.\n\nNext paragraph.\n");
}

#[test]
fn a_definition_between_a_directive_and_its_block_is_never_touched() {
    // comrak consumes the definition, so the paragraph isn't in the tree; the
    // rule sees a gap that isn't blank and leaves it.
    unchanged("@steps\n[ref]: /docs\n\n1. One\n");
}

// ---- Directive lines indented to the item's content column ---------------------

#[test]
fn extra_indentation_in_the_document_is_removed() {
    check("  @note: Text.\n", "@note: Text.\n");
    check("   @note:\nText.\n   @end\n", "@note:\nText.\n@end\n");
    check("## Setup\n   @id: setup\n", "## Setup\n@id: setup\n");
}

#[test]
fn extra_indentation_in_a_list_item_is_removed() {
    // Content column 2: `@note` is at 3, 4, and 5.
    check("- item\n   @note: Text.\n", "- item\n  @note: Text.\n");
    check("- item\n    @note: Text.\n", "- item\n  @note: Text.\n");
    check("- item\n     @note: Text.\n", "- item\n  @note: Text.\n");
    // Content column 3.
    check("1. item\n    @note: Text.\n", "1. item\n   @note: Text.\n");
    // Marker followed by three spaces: content column 4.
    check(
        "-   item\n     @note: Text.\n",
        "-   item\n    @note: Text.\n",
    );
    unchanged("- item\n  @note: Text.\n");
}

#[test]
fn a_container_and_its_end_line_in_a_list_item() {
    check(
        "- item\n   @note:\n  Text.\n   @end\n",
        "- item\n  @note:\n  Text.\n  @end\n",
    );
}

#[test]
fn an_end_line_with_extra_spaces_still_closes_and_loses_them() {
    // Q19: extra spaces in the same container don't matter, and canonical
    // form removes them.
    check("@note:\nText.\n   @end\n", "@note:\nText.\n@end\n");
    check(
        "@variant {pm=npm}:\nA\n\n @variant {pm=yarn}:\nB\n   @end\n",
        "@variant {pm=npm}:\nA\n\n@variant {pm=yarn}:\nB\n@end\n",
    );
}

#[test]
fn nested_lists_use_the_innermost_items_column() {
    check(
        "- a\n  - b\n     @note: In b.\n",
        "- a\n  - b\n    @note: In b.\n",
    );
    // At column 3 the line is still in the outer item, which has column 2.
    check(
        "- a\n  - b\n   @note: In a.\n",
        "- a\n  - b\n  @note: In a.\n",
    );
}

#[test]
fn block_quotes_keep_the_marker_and_one_space() {
    check("> @note: Text.\n", "> @note: Text.\n");
    check(">   @note: Text.\n", "> @note: Text.\n");
    check(
        ">    @note:\n> Text.\n>   @end\n",
        "> @note:\n> Text.\n> @end\n",
    );
    // No space after the marker is left as written.
    unchanged(">@note: Text.\n");
    check("> > @note:   Text.\n", "> > @note: Text.\n");
    check("> >   @note: Text.\n", "> > @note: Text.\n");
}

#[test]
fn a_list_in_a_block_quote() {
    check(
        "> - item\n>    @note: Text.\n",
        "> - item\n>   @note: Text.\n",
    );
}

#[test]
fn a_directive_on_a_list_markers_line_is_left_alone() {
    unchanged("-   @note: Text.\n");
    unchanged("1.  @note: Text.\n");
}

#[test]
fn a_directive_in_a_container_in_a_list_item() {
    check(
        "- item\n  @note:\n   @include: a.md\n  @end\n",
        "- item\n  @note:\n  @include: a.md\n  @end\n",
    );
}

#[test]
fn tabs_in_indentation_are_left_alone() {
    unchanged("- item\n\t@note: Text.\n");
    unchanged("\t@note: Text.\n");
}

// ---- Everything else is left alone --------------------------------------------

#[test]
fn ordinary_markdown_is_untouched() {
    let source = "\
# A   heading   with   spaces

Hand-wrapped prose that goes on and on and on
and on, then breaks   here,
with  odd   spacing and a trailing hard break.  
Next line.

| a   | b |
|-----|---|
| 1   |  2 |

```yaml
key:   value
@note {type = tip}: inside code
```

    @note {type = tip}: indented code

* one
*   two
    continued

[ref]:   /docs/ref   \"Title\"
> quote   text
";
    unchanged(source);
}

#[test]
fn prose_spacing_around_directives_is_untouched() {
    check(
        "Some  prose.\n  Wrapped   line.\n@note{type=tip}:Note.\nMore  prose.\n",
        "Some  prose.\n  Wrapped   line.\n@note {type=tip}: Note.\nMore  prose.\n",
    );
}

#[test]
fn title_lines_are_left_alone() {
    unchanged(
        ".Try   it now\n@note {type=tip}:   \nText.\n@end\n"
            .replace(":   \n", ":\n")
            .as_str(),
    );
    check(
        ".Try   it now\n@note{type=tip}:\nText.\n@end\n",
        ".Try   it now\n@note {type=tip}:\nText.\n@end\n",
    );
}

#[test]
fn an_escaped_directive_is_prose() {
    unchanged("\\@note{type = tip}: Not a directive.\n");
    unchanged("\\.Title\n@note\nText.\n");
}

#[test]
fn an_unknown_directive_is_prose() {
    unchanged("@astrojs/react{a = b}\n@timestamp : x\n");
}

#[test]
fn a_definition_is_never_changed() {
    unchanged("[ref]:   {api}streaming   \"Title\"\n\nSee [x][ref].\n");
    unchanged(
        "[a]:\n  {api}x\n\n@note:Text.\n"
            .replace("@note:Text.", "@note: Text.")
            .as_str(),
    );
}

// ---- Constructs with errors are left alone -------------------------------------

#[test]
fn a_directive_with_an_unclosed_block_is_left_alone() {
    unchanged("@note {type = tip\nText.\n");
    unchanged("@note{type=tip :   Text.\n");
}

#[test]
fn a_block_with_a_bad_attribute_is_left_alone() {
    // A bare key, an unquoted value with a space, a repeated key.
    unchanged("@note { bare }: Text.\n");
    unchanged("@note {type = tip, type = caution}: Text.\n");
    unchanged("@quill-labspace {lab=Using other images  ,  height=3}\n");
    unchanged("![Alt](a.png){ width = 3, width = 4 }\n");
    unchanged("![Alt](a.png){ bare }\n");
}

#[test]
fn extra_text_on_a_directive_line_is_left_alone() {
    unchanged("@steps   foo\n1. One\n");
    unchanged("@note   hello : text\n");
    unchanged("@include :  my file.md\n");
    unchanged("@end   : x\n");
}

#[test]
fn a_primary_on_a_directive_that_takes_none_is_left_alone() {
    unchanged("@steps :   text\n1. One\n");
}

#[test]
fn a_container_that_is_not_closed_is_left_alone() {
    unchanged("@note  {type=tip}  :  \nText.\n");
}

#[test]
fn a_container_only_directive_without_its_colon_is_left_alone() {
    unchanged("@variant   {pm=npm}\nText.\n@end\n");
}

#[test]
fn a_stray_end_line_is_left_alone() {
    unchanged("Text.\n   @end\n");
}

#[test]
fn an_end_line_in_another_container_is_left_alone() {
    unchanged("- @note:\n- item\n   @end\n");
}

#[test]
fn a_following_block_directive_with_no_block_is_left_alone() {
    unchanged("@note   {type=tip}\n\n");
    unchanged("@available:   cloud\n\n@include: a.md\n");
}

#[test]
fn a_directive_bound_to_a_heading_is_left_alone() {
    unchanged("@note   {type=tip}\n\n## Heading\n");
}

#[test]
fn errors_elsewhere_do_not_stop_other_constructs() {
    check(
        "@note {type = tip\nText.\n\n@note {type = caution}:Text.\n",
        "@note {type = tip\nText.\n\n@note {type=caution}: Text.\n",
    );
}

// ---- Miscellany --------------------------------------------------------------------

#[test]
fn everything_at_once() {
    check(
        "   @quill-labspace{ height = \"300\" ,lab=\"intro\" }\n",
        "@quill-labspace {lab=intro, height=300}\n",
    );
    check(
        "- item\n\n    @available :  cloud\n\n    @note{ }  :  \n    Text.\n    @end\n",
        "- item\n\n  @available: cloud\n  @note:\n    Text.\n  @end\n",
    );
}

#[test]
fn edits_are_minimal_and_ordered() {
    let model = support::shared_model();
    let options = support::options(&model);
    let source = "@note{type=tip}:Text.\n";
    let edits = tessera_fmt::format(source, &options, &model);
    // Two insertions, and nothing else: the block itself is already canonical.
    assert_eq!(edits.len(), 2, "{edits:?}");
    assert!(
        edits
            .windows(2)
            .all(|w| w[0].span.end() <= w[1].span.start())
    );
    assert!(tessera_fmt::format("@note {type=tip}: Text.\n", &options, &model).is_empty());
}

#[test]
fn an_empty_document_and_a_document_without_directives() {
    unchanged("");
    unchanged("\n");
    unchanged("Just text.\n");
    unchanged("---\ntitle: T\n---\n\n# H\n");
}

#[test]
fn frontmatter_is_untouched() {
    unchanged("---\ntitle:   T\nnote: \"@note {a = b}: x\"\n---\n\n@note: Text.\n");
    check(
        "---\ntitle:   T\n---\n@note{type=tip}:x\n",
        "---\ntitle:   T\n---\n@note {type=tip}: x\n",
    );
}

#[test]
fn line_endings_are_preserved() {
    check(
        "@note{type=tip}:\r\nText.\r\n@end\r\n",
        "@note {type=tip}:\r\nText.\r\n@end\r\n",
    );
    check("@steps\r\n\r\n1. One\r\n", "@steps\r\n1. One\r\n");
}

#[test]
fn utf8_is_preserved() {
    check(
        "@note{type=tip}:é → 😀 text\n",
        "@note {type=tip}: é → 😀 text\n",
    );
    check(
        "@quill-labspace {lab = \"é\"}\n",
        "@quill-labspace {lab=é}\n",
    );
}
