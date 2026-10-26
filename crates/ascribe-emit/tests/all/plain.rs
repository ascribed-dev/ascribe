//! The plain-markdown output, construct by construct (SPEC §9.4).

#![allow(clippy::expect_used, clippy::panic)]

use crate::support;

use support::{FULL_MODEL, memory_project, page, plain};

/// The body of `index.md` under the `site` build of the full model, after the
/// title line.
fn body(source: &str) -> String {
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    let out = plain(&project, "site", "index.md");
    out.strip_prefix("# Test\n\n")
        .unwrap_or_else(|| panic!("no title line in {out:?}"))
        .to_owned()
}

#[test]
fn a_page_starts_with_its_title() {
    let project = memory_project(FULL_MODEL, &[("index.md", &page("Hello.\n"))]);
    assert_eq!(plain(&project, "site", "index.md"), "# Test\n\nHello.\n");
}

#[test]
fn notes_are_blockquotes_with_their_label_and_title() {
    assert_eq!(
        body(".Watch out\n@note {type=warning}\nIt bites.\n"),
        "> **Warning: Watch out**\n>\n> It bites.\n"
    );
    // The relabeled built-in type uses its own label.
    assert_eq!(
        body("@note {type=tip}: Use `x`.\n"),
        "> **Pro tip**\n>\n> Use `x`.\n"
    );
    assert_eq!(
        body("@note {type=security}:\nOne.\n\nTwo.\n@end\n"),
        "> **Security**\n>\n> One.\n>\n> Two.\n"
    );
    assert_eq!(body("@note: Plain.\n"), "> **Note**\n>\n> Plain.\n");
}

#[test]
fn a_note_title_with_emphasis_stays_inside_the_bold() {
    assert_eq!(
        body(".Use *npm* and `pnpm`\n@note: x\n"),
        "> **Note: Use npm and `pnpm`**\n>\n> x\n"
    );
}

#[test]
fn steps_are_the_ordered_list() {
    assert_eq!(body("@steps\n1. One\n2. Two\n"), "1. One\n2. Two\n");
}

#[test]
fn details_are_the_bold_title_then_the_content() {
    assert_eq!(
        body(".More\n@details:\nHidden.\n\n- a\n@end\n"),
        "**More**\n\nHidden.\n\n- a\n"
    );
    assert_eq!(body(".More\n@details\nHidden.\n"), "**More**\n\nHidden.\n");
}

#[test]
fn a_switcher_says_a_reader_sees_one_arm_then_heads_each_with_its_values() {
    let source = "@variant {pm=npm}:\nnpm i\n@variant {pm=yarn}:\nyarn add\n@end\n";
    assert_eq!(
        body(source),
        "A reader sees one of the next 2 sections at a time, chosen by package manager.\n\n\
         ## Package manager: npm\n\nnpm i\n\n## Package manager: Yarn\n\nyarn add\n"
    );
    // A labeled group has no dimension to choose by: its arms' titles head
    // them.
    let labeled = ".Docker\n@variant:\nA\n\n.Other\n@variant:\nB\n@end\n";
    assert_eq!(
        body(labeled),
        "A reader sees one of the next 2 sections at a time.\n\n## Docker\n\nA\n\n## Other\n\nB\n"
    );
    // A value set, and several dimensions on one arm.
    let several = "@variant {deployment=cloud pm=npm|pnpm}:\nA\n@variant {deployment=self-managed pm=yarn}:\nB\n@end\n";
    assert_eq!(
        body(several),
        "A reader sees one of the next 2 sections at a time, chosen by deployment.\n\n\
         ## Deployment: Quill Cloud, Package manager: npm / pnpm\n\nA\n\n\
         ## Deployment: Self-managed, Package manager: Yarn\n\nB\n"
    );
}

#[test]
fn an_arms_heading_is_one_level_below_the_section_its_in() {
    let source = "## Install\n\n@variant {pm=npm}:\nA\n@variant {pm=yarn}:\nB\n@end\n\n## Next\n";
    assert_eq!(
        body(source),
        "## Install\n\nA reader sees one of the next 2 sections at a time, chosen by package manager.\n\n\
         ### Package manager: npm\n\nA\n\n### Package manager: Yarn\n\nB\n\n## Next\n"
    );
    // A group in an arm goes a level further, and a level-6 section's arms
    // stay at 6.
    let nested = "@variant {pm=npm}:\n- Item\n\n  .One\n  @variant:\n  A\n\n  .Two\n  @variant:\n  B\n  @end\n@variant {pm=yarn}:\nC\n@end\n";
    let text = body(nested);
    assert!(
        text.contains(
            "  ### One (npm)\n\n  A\n\n  ### Two (npm)\n\n  B\n\n## Package manager: Yarn\n"
        ),
        "{text}"
    );
    let deep = "###### Deep\n\n@variant {pm=npm}:\nA\n@variant {pm=yarn}:\nB\n@end\n";
    assert!(
        body(deep).contains("\n###### Package manager: npm\n"),
        "{}",
        body(deep)
    );
}

#[test]
fn a_heading_inside_an_arm_ends_with_the_arms_label() {
    // Each section's headings stay distinct, so a reader of the Markdown
    // (an agent, a search index) can tell which arm it's in.
    let source =
        "@variant {pm=npm}:\n## Install\nnpm i\n@variant {pm=yarn}:\n## Install\nyarn add\n@end\n";
    assert_eq!(
        body(source),
        "A reader sees one of the next 2 sections at a time, chosen by package manager.\n\n\
         ## Package manager: npm\n\n## Install (npm)\n\nnpm i\n\n\
         ## Package manager: Yarn\n\n## Install (Yarn)\n\nyarn add\n"
    );
    // A group reduced to one arm is that arm's content, headings unchanged.
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    assert_eq!(
        plain(&project, "self-managed-3.5", "index.md"),
        "# Test\n\n## Install\n\nnpm i\n"
    );
}

#[test]
fn a_selection_that_leaves_one_arm_leaves_just_its_content() {
    let source = "@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\nServer.\n@end\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    assert_eq!(plain(&project, "cloud", "index.md"), "# Test\n\nCloud.\n");
    // Several surviving arms stay a switcher, counted as the build keeps it.
    let two = "@variant {pm=npm}:\nA\n@variant {pm=pnpm}:\nB\n@variant {pm=yarn}:\nC\n@end\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(two))]);
    assert_eq!(
        plain(&project, "self-managed-3.5", "index.md"),
        "# Test\n\nA reader sees one of the next 2 sections at a time, chosen by package manager.\n\n\
         ## Package manager: npm\n\nA\n\n## Package manager: pnpm\n\nB\n"
    );
}

#[test]
fn availability_is_a_line_with_display_labels() {
    let source =
        "## Streaming\n@id: streaming\n@available: cloud, self-managed preview 3.4\n\nText.\n";
    assert_eq!(
        body(source),
        "## Streaming\n\nAvailable: Quill Cloud (GA); Self-managed (preview, 3.4+)\n\nText.\n"
    );
    // A feature key shows the spec it stands for.
    let feature = "## Sync\n@available: streaming-sync\n";
    assert_eq!(
        body(feature),
        "## Sync\n\nAvailable: Quill Cloud (GA); Self-managed (preview, 3.4+)\n"
    );
    // A history, a bare version, and a state on a versionless target. A
    // history names each state and the version it begins at, as the element
    // contract does.
    let history = "@available: sso\n\nPara.\n";
    assert_eq!(
        body(history),
        "Available: Quill Cloud (Beta); Self-managed (preview 3.3, GA 3.5, deprecated 4.0)\n\nPara.\n"
    );
    assert_eq!(
        body("@available: self-managed 3.2\n\nPara.\n"),
        "Available: Self-managed (GA, 3.2+)\n\nPara.\n"
    );
}

#[test]
fn availability_of_a_block_is_a_line_before_the_block() {
    assert_eq!(
        body("Intro.\n\n@available: cloud\nCloud only.\n"),
        "Intro.\n\nAvailable: Quill Cloud (GA)\n\nCloud only.\n"
    );
}

#[test]
fn page_level_availability_is_a_line_under_the_title() {
    let source = "---\ntitle: Test\navailable: cloud, self-managed preview 3.3\n---\n\nText.\n";
    let project = memory_project(FULL_MODEL, &[("index.md", source)]);
    assert_eq!(
        plain(&project, "site", "index.md"),
        "# Test\n\nAvailable: Quill Cloud (GA); Self-managed (preview, 3.3+)\n\nText.\n"
    );
}

#[test]
fn a_filter_build_keeps_the_annotation_of_what_remains() {
    let source = "## Both\n@available: cloud, self-managed preview 3.4\n\nIn.\n\n## Server only\n@available: self-managed\n\nOut.\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    let out = plain(&project, "cloud", "index.md");
    assert!(
        out.contains("Available: Quill Cloud (GA); Self-managed (preview, 3.4+)"),
        "{out}"
    );
    assert!(!out.contains("Server only"), "{out}");
    assert!(!out.contains("Out."), "{out}");
}

#[test]
fn widgets_are_their_fallback_and_their_content_unless_dropped() {
    assert_eq!(
        body("@quill-labspace {lab=x}\n"),
        "Try this in the Quill lab at labs.quill.dev.\n"
    );
    // No fallback: nothing for the widget, its content stays.
    assert_eq!(body("@quill-aside:\nAside.\n@end\n"), "Aside.\n");
    assert_eq!(body("@quill-aside\nBound.\n"), "Bound.\n");
    // `plain-content = "drop"`.
    assert_eq!(
        body("@quill-interactive:\nHidden.\n@end\n\nAfter.\n"),
        "This section is interactive. See it at docs.quill.dev.\n\nAfter.\n"
    );
}

#[test]
fn phrases_are_substituted_in_prose_code_and_headings() {
    assert_eq!(
        body(
            "## About {product}\n\n{product} `{product}` ok.\n\n```yaml phrases=true\nv: {version}\n```\n"
        ),
        "## About Quill\n\nQuill `{product}` ok.\n\n```yaml\nv: 3.4.1\n```\n"
    );
}

#[test]
fn info_string_pairs_the_checker_read_are_left_out() {
    // SPEC §3.3.1: `phrases`, `request`, and `response` have done their
    // work, and `tested` is its own line; other words stay, in order.
    assert_eq!(
        body(
            "```json {1} title=\"New\" request=\"../api.yaml#create\" tested=\"Node 24\" phrases=true\n{}\n```\n"
        ),
        "```json {1} title=\"New\"\n{}\n```\n\nTested with Node 24.\n"
    );
    // A pair's word inside a quoted value is part of the value.
    assert_eq!(
        body("```yaml title=\"phrases=true\"\nv: {version}\n```\n"),
        "```yaml title=\"phrases=true\"\nv: {version}\n```\n"
    );
}

#[test]
fn an_info_strings_backslashes_survive_a_reader_of_the_output() {
    // The source's `\\\"` is `\"` in the info string, an escaped quote in the
    // title; the output writes `\\"`, which a reader reads as `\"` again.
    assert_eq!(
        body("```text title=\"say \\\\\\\"hi\\\\\\\"\"\nx\n```\n"),
        "```text title=\"say \\\\\"hi\\\\\"\"\nx\n```\n"
    );
}

#[test]
fn lists_keep_their_shape_and_adjacent_lists_stay_apart() {
    assert_eq!(body("- a\n- b\n  - c\n"), "- a\n- b\n  - c\n");
    assert_eq!(body("- a\n\n- b\n"), "- a\n\n- b\n");
    assert_eq!(body("3. a\n4. b\n"), "3. a\n4. b\n");
    // A removed block between two lists leaves them adjacent: another marker.
    let project = memory_project(
        FULL_MODEL,
        &[(
            "index.md",
            &page("- a\n\n@available: self-managed\nGone.\n\n- b\n"),
        )],
    );
    let out = plain(&project, "cloud", "index.md");
    assert_eq!(out, "# Test\n\n- a\n\n* b\n");
}

#[test]
fn code_blocks_are_fenced_beyond_their_own_backticks() {
    assert_eq!(
        body("````\n```\nx\n```\n````\n"),
        "````\n```\nx\n```\n````\n"
    );
    assert_eq!(body("    indented\n"), "```\nindented\n```\n");
    assert_eq!(body("```\n```\n"), "```\n```\n");
}

#[test]
fn block_quotes_tables_and_breaks() {
    assert_eq!(body("> a\n>\n> b\n"), "> a\n>\n> b\n");
    assert_eq!(body("a\n\n---\n\nb\n"), "a\n\n---\n\nb\n");
    assert_eq!(
        body("| a | b |\n| --- | --- |\n| 1 | `x\\|y` |\n"),
        "| a | b |\n| --- | --- |\n| 1 | `x\\|y` |\n"
    );
    // Each column keeps its alignment.
    assert_eq!(
        body("| a | b | c | d |\n| :-- | :-: | --: | --- |\n| 1 | 2 | 3 | 4 |\n"),
        "| a | b | c | d |\n| :--- | :---: | ---: | --- |\n| 1 | 2 | 3 | 4 |\n"
    );
}

#[test]
fn a_br_separates_what_a_cell_holds() {
    // Code spans either side of a `<br>` stay two code spans (#84).
    assert_eq!(
        body("| Messages |\n|---|\n| `one`<br>`two` |\n"),
        "| Messages |\n| --- |\n| `one`; `two` |\n"
    );
    assert_eq!(
        body("| a |\n|---|\n| x <br/> y<BR /> |\n| <br>z |\n"),
        "| a |\n| --- |\n| x; y |\n| z |\n"
    );
    // In a paragraph it's a line break, which a line break in the source
    // right after it joins, and which the end of the paragraph drops.
    assert_eq!(body("a<br>b\n"), "a\\\nb\n");
    assert_eq!(body("line one<br>\nline two\n"), "line one\\\nline two\n");
    assert_eq!(body("a <br>  \nb\n"), "a\\\nb\n");
    assert_eq!(body("`end`<br>\n\nNext.\n"), "`end`\n\nNext.\n");
    assert_eq!(body("*a<br>*\nb\n"), "*a*\\\nb\n");
    assert_eq!(
        body("1. Step one<br>\n   continues\n"),
        "1. Step one\\\n   continues\n"
    );
}

#[test]
fn code_spans_that_touch_stay_apart() {
    assert_eq!(body("`a`<span></span>`b`\n"), "`a` `b`\n");
    // A literal backtick before a code span isn't a code span.
    assert_eq!(body("\\``b`\n"), "\\``b`\n");
}

#[test]
fn text_is_escaped_so_it_reads_back_the_same() {
    assert_eq!(
        body("Use \\*stars\\* and snake_case and \\[brackets\\].\n"),
        "Use \\*stars\\* and snake_case and \\[brackets\\].\n"
    );
    assert_eq!(body("\\# not a heading\n"), "\\# not a heading\n");
    assert_eq!(body("1986\\. A year\n"), "1986\\. A year\n");
}

#[test]
fn raw_html_keeps_its_text_and_drops_its_tags() {
    // The output has no HTML, and a reader sees the text, not the tags.
    assert_eq!(body("<div>\nhi\n</div>\n"), "hi\n");
    assert_eq!(body("a <kbd>x</kbd> b\n"), "a x b\n");
    // Comments, scripts, and styles have no text to keep.
    assert_eq!(body("<!-- note -->\n\nText.\n"), "Text.\n");
    assert_eq!(
        body("<script>\nlet x = 1 < 2;\n</script>\n\nText.\n"),
        "Text.\n"
    );
    // A `<` that doesn't start a tag is text.
    assert_eq!(body("<div>\n1 < 2\n</div>\n"), "1 \\< 2\n");
}

#[test]
fn links_are_absolute_and_images_keep_their_alt_text() {
    let project = memory_project(
        FULL_MODEL,
        &[
            (
                "index.md",
                &page(
                    "[Other](guides/other.md#deep) and [](guides/other.md) and <https://x.dev/a> and [ext](https://x.dev/b \"T\")\n",
                ),
            ),
            (
                "guides/other.md",
                "---\ntitle: The other page\n---\n\n## Deep\n",
            ),
        ],
    );
    assert_eq!(
        plain(&project, "site", "index.md"),
        "# Test\n\n[Other](https://docs.quill.dev/docs/guides/other#deep) and [The other page](https://docs.quill.dev/docs/guides/other) and <https://x.dev/a> and [ext](https://x.dev/b \"T\")\n"
    );
}

#[test]
fn glossary_terms_are_linked() {
    let project = memory_project(
        FULL_MODEL,
        &[
            ("index.md", &page("Rotate your API key.\n")),
            (
                "reference/glossary.md",
                "---\ntitle: Glossary\n---\n\n## API key\n@term {aliases=\"API keys\"}\n\nA secret token that authenticates the Quill agent to Quill Cloud.\n",
            ),
        ],
    );
    assert_eq!(
        plain(&project, "site", "index.md"),
        "# Test\n\nRotate your [API key](https://docs.quill.dev/docs/reference/glossary#api-key).\n"
    );
}

#[test]
fn without_a_site_origin_links_stay_root_relative() {
    let model = FULL_MODEL.replace("site = \"https://docs.quill.dev\"\n", "");
    let project = memory_project(
        &model,
        &[
            ("index.md", &page("[Other](other.md)\n")),
            ("other.md", "---\ntitle: Other\n---\n\nx\n"),
        ],
    );
    assert_eq!(
        plain(&project, "site", "index.md"),
        "# Test\n\n[Other](/docs/other)\n"
    );
}

#[test]
fn titles_are_escaped() {
    let source = "---\ntitle: '*Star* #1'\n---\n\nx\n";
    let project = memory_project(FULL_MODEL, &[("index.md", source)]);
    assert_eq!(
        plain(&project, "site", "index.md"),
        "# \\*Star\\* #1\n\nx\n"
    );
}

#[test]
fn a_title_that_spans_lines_is_one_heading() {
    let source = "---\ntitle: >\n  A long\n  title\n---\n\nx\n";
    let project = memory_project(FULL_MODEL, &[("index.md", source)]);
    assert_eq!(plain(&project, "site", "index.md"), "# A long title\n\nx\n");
}

/// The text a CommonMark parser reads from `markdown`: text and code, with
/// breaks as newlines.
fn text_of(markdown: &str) -> String {
    use comrak_ascribe::nodes::NodeValue;
    let arena = comrak_ascribe::Arena::new();
    let root =
        comrak_ascribe::parse_document(&arena, markdown, &comrak_ascribe::Options::default());
    let mut out = String::new();
    for node in root.descendants() {
        match &node.data.borrow().value {
            NodeValue::Text(t) => out.push_str(t),
            NodeValue::Code(c) => {
                out.push('`');
                out.push_str(&c.literal);
                out.push('`');
            }
            NodeValue::SoftBreak | NodeValue::LineBreak => out.push('\n'),
            NodeValue::Paragraph | NodeValue::Heading(_) => out.push('\u{1}'),
            _ => {}
        }
    }
    out
}

#[test]
fn escaped_text_reads_back_as_the_same_text() {
    for source in [
        "Stars \\*a\\* and \\_b\\_ and snake_case and 2 * 3.\n",
        "A [bracket] and \\[escaped\\] and a \\<tag> and & and &amp; and \\&copy;.\n",
        "line one\n\\# hash\n\\> quote\n\\- dash\n1\\. one\n\\+ plus\n\\=\\=\\=\n",
        "Tildes \\~\\~x\\~\\~ and `` `code` `` and ``` a``b ```.\n",
        "Backslash \\\\ and \\`tick\\` and a_b_c and _lead and trail_.\n",
        "Trailing hard break\\\nnext\n",
        "URL-ish https://example.com/a_b?c=d&e=f and www.example.com.\n",
    ] {
        let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
        let out = plain(&project, "site", "index.md");
        let out = out.strip_prefix("# Test\n\n").expect("a title line");
        assert_eq!(
            text_of(out),
            text_of(source),
            "source:\n{source}\noutput:\n{out}"
        );
    }
}

#[test]
fn a_heading_that_ends_in_a_hash_isnt_read_as_closed() {
    // An escaped `#` is already safe; one that isn't gets escaped.
    assert_eq!(body("## \\#\n"), "## \\#\n");
    assert_eq!(body("## C\\#\n"), "## C\\#\n");
}

#[test]
fn a_rows_availability_ends_its_first_cell() {
    assert_eq!(
        body(
            "| Key | Meaning |\n|---|---|\n| `a` | All. |\n| `b` {available=\"self-managed preview 3.4\"} | New. |\n"
        ),
        "| Key | Meaning |\n| --- | --- |\n| `a` | All. |\n| `b` (Available: Self-managed (preview, 3.4+)) | New. |\n"
    );
}

#[test]
fn a_summary_writes_nothing_of_its_own() {
    // `@summary: text` writes nothing, and the paragraph `@summary` marks is
    // written as it would be without it.
    assert_eq!(
        body("## Install\n@id: install\n@summary: Install it.\n\n@summary\nIt installs.\n"),
        body("## Install\n@id: install\n\nIt installs.\n")
    );
}
