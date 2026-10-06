//! The site output, construct by construct (SPEC §9.4, element contract).

#![allow(clippy::expect_used, clippy::panic)]

mod support;

use support::{FULL_MODEL, memory_project, memory_project_with_files, page, site};
use tessera_emit::render_site_html;

/// The body of `index.md` under the `site` build of the full model, after the
/// frontmatter.
fn body(source: &str) -> String {
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    strip_frontmatter(&site(&project, "site", "index.md"))
}

fn strip_frontmatter(text: &str) -> String {
    text.strip_prefix("---\ntitle: Test\n---\n\n")
        .unwrap_or_else(|| panic!("unexpected frontmatter in {text:?}"))
        .to_owned()
}

#[test]
fn a_page_is_its_frontmatter_and_its_body() {
    let project = memory_project(FULL_MODEL, &[("index.md", &page("Hello.\n"))]);
    assert_eq!(
        site(&project, "site", "index.md"),
        "---\ntitle: Test\n---\n\nHello.\n"
    );
}

#[test]
fn every_heading_ends_in_a_marker_with_its_page_id() {
    assert_eq!(
        body("# One\n\nTwo\n---\n\n## One\n\n### Custom\n@id: mine\n\n## \n"),
        "# One <ascribe-attributes id=\"one\"></ascribe-attributes>\n\n\
         ## Two <ascribe-attributes id=\"two\"></ascribe-attributes>\n\n\
         ## One <ascribe-attributes id=\"one-1\"></ascribe-attributes>\n\n\
         ### Custom <ascribe-attributes id=\"mine\"></ascribe-attributes>\n\n\
         ##\n"
    );
}

#[test]
fn a_heading_keeps_its_inline_content_and_a_trailing_hash() {
    assert_eq!(
        body("## Run `ascribe check` in *CI* [now](https://example.com)\n"),
        "## Run `ascribe check` in *CI* [now](https://example.com) \
         <ascribe-attributes id=\"run-ascribe-check-in-ci-now\"></ascribe-attributes>\n"
    );
    // A trailing `#` in the text isn't a closing sequence.
    assert_eq!(
        body("## C\\#\n"),
        "## C# <ascribe-attributes id=\"c\"></ascribe-attributes>\n"
    );
    // A heading with no page id has no marker, and its trailing `#` is
    // escaped so it isn't a closing sequence either.
    assert_eq!(body("## ???\n"), "## ???\n");
    assert_eq!(body("## \\#\n"), "## \\#\n");
}

#[test]
fn notes_carry_their_type_label_and_heading() {
    assert_eq!(
        body(".Watch *out* & \"care\"\n@note {type=warning}\nIt bites.\n"),
        "<ascribe-note type=\"warning\" label=\"Warning\" heading=\"Watch out &amp; &quot;care&quot;\">\n\nIt bites.\n\n</ascribe-note>\n"
    );
    // The relabeled built-in type uses its own label; the default type is
    // still written.
    assert_eq!(
        body("@note {type=tip}: Use `x`.\n"),
        "<ascribe-note type=\"tip\" label=\"Pro tip\">\n\nUse `x`.\n\n</ascribe-note>\n"
    );
    assert_eq!(
        body("@note: Plain.\n"),
        "<ascribe-note type=\"note\" label=\"Note\">\n\nPlain.\n\n</ascribe-note>\n"
    );
    assert_eq!(
        body("@note {type=security}:\nOne.\n\nTwo.\n@end\n"),
        "<ascribe-note type=\"security\" label=\"Security\">\n\nOne.\n\nTwo.\n\n</ascribe-note>\n"
    );
}

#[test]
fn steps_wrap_the_list() {
    assert_eq!(
        body("@steps\n3. One\n4. Two\n"),
        "<ascribe-steps>\n\n3. One\n4. Two\n\n</ascribe-steps>\n"
    );
}

#[test]
fn details_render_their_title_as_html() {
    assert_eq!(
        body(
            ".Show the `quill.yaml` *reference* & [keys](index.md)\n@details:\nHidden.\n\n- a\n@end\n"
        ),
        "<details>\n<summary>Show the <code>quill.yaml</code> <em>reference</em> &amp; <a href=\"/docs\">keys</a></summary>\n\nHidden.\n\n- a\n\n</details>\n"
    );
    assert_eq!(
        body(".More\n@details\nHidden.\n"),
        "<details>\n<summary>More</summary>\n\nHidden.\n\n</details>\n"
    );
}

#[test]
fn a_group_is_tabs_that_sync_on_their_dimension() {
    let source = "@variant {pm=npm}:\nnpm i\n@variant {pm=yarn}:\nyarn add\n@end\n";
    assert_eq!(
        body(source),
        "<ascribe-tabs sync=\"pm\">\n\n\
         <ascribe-tab value=\"npm\" label=\"npm\">\n\nnpm i\n\n</ascribe-tab>\n\n\
         <ascribe-tab value=\"yarn\" label=\"Yarn\">\n\nyarn add\n\n</ascribe-tab>\n\n\
         </ascribe-tabs>\n"
    );
}

#[test]
fn a_labeled_group_has_no_sync_and_no_values() {
    let source = ".Docker\n@variant:\nA\n\n.Use *k8s* now\n@variant:\nB\n@end\n";
    assert_eq!(
        body(source),
        "<ascribe-tabs>\n\n\
         <ascribe-tab label=\"Docker\">\n\nA\n\n</ascribe-tab>\n\n\
         <ascribe-tab label=\"Use k8s now\">\n\nB\n\n</ascribe-tab>\n\n\
         </ascribe-tabs>\n"
    );
}

#[test]
fn a_value_set_arm_lists_its_values_and_joins_their_labels() {
    let source = "@variant {pm=npm|yarn}:\nA\n@variant {pm=pnpm}:\nB\n@end\n";
    let out = body(source);
    assert!(
        out.contains("<ascribe-tab value=\"npm yarn\" label=\"npm / Yarn\">"),
        "{out}"
    );
    assert!(
        out.contains("<ascribe-tab value=\"pnpm\" label=\"pnpm\">"),
        "{out}"
    );
}

#[test]
fn sync_is_the_first_dimension_the_arms_share_and_differ_on() {
    // Both arms name both dimensions; only `pm` differs. Attributes are
    // labeled in the model's order (`deployment` first), whatever the
    // source order.
    let source =
        "@variant {pm=npm, deployment=cloud}:\nA\n@variant {pm=yarn, deployment=cloud}:\nB\n@end\n";
    let out = body(source);
    assert!(out.starts_with("<ascribe-tabs sync=\"pm\">"), "{out}");
    assert!(
        out.contains("<ascribe-tab value=\"npm\" label=\"Quill Cloud, npm\">"),
        "{out}"
    );
    // Every shared dimension has the same values: the first is the sync.
    let same =
        "@variant {pm=npm, deployment=cloud}:\nA\n@variant {pm=npm, deployment=cloud}:\nB\n@end\n";
    assert!(body(same).starts_with("<ascribe-tabs sync=\"deployment\">"));
}

#[test]
fn a_selection_that_leaves_one_arm_leaves_its_content_and_several_stay_tabs() {
    let one = "@variant {deployment=cloud}:\nCloud.\n@variant {deployment=self-managed}:\nServer.\n@end\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(one))]);
    assert_eq!(
        strip_frontmatter(&site(&project, "cloud", "index.md")),
        "Cloud.\n"
    );
    let two = "@variant {pm=npm}:\nA\n@variant {pm=pnpm}:\nB\n@variant {pm=yarn}:\nC\n@end\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(two))]);
    let out = site(&project, "self-managed-3.5", "index.md");
    assert!(out.contains("<ascribe-tabs sync=\"pm\">"), "{out}");
    assert!(!out.contains("value=\"yarn\""), "{out}");
}

#[test]
fn availability_is_a_block_of_targets_where_the_directive_was() {
    let out = body("## Sect\n@available: cloud, self-managed preview 3.4\n\nBody.\n");
    assert_eq!(
        out,
        "## Sect <ascribe-attributes id=\"sect\"></ascribe-attributes>\n\n\
         <ascribe-availability scope=\"section\">\n\
         <ascribe-availability-target target=\"cloud\" dimension=\"deployment\" states=\"ga\">Quill Cloud (GA)</ascribe-availability-target>; \
         <ascribe-availability-target target=\"self-managed\" dimension=\"deployment\" states=\"preview\" versions=\"3.4\">Self-managed (preview, 3.4+)</ascribe-availability-target>\n\
         </ascribe-availability>\n\nBody.\n"
    );
    // A block-scoped spec, a history, a bare version, and a dimension name.
    let block = body("Intro.\n\n@available: sso\nPara.\n");
    assert!(
        block.contains("<ascribe-availability scope=\"block\">"),
        "{block}"
    );
    assert!(block.contains(
        "<ascribe-availability-target target=\"self-managed\" dimension=\"deployment\" states=\"preview ga deprecated\" versions=\"3.3 3.5 4.0\">Self-managed (preview 3.3, GA 3.5, deprecated 4.0)</ascribe-availability-target>"
    ), "{block}");
    assert!(
        block.contains("states=\"beta\">Quill Cloud (Beta)<"),
        "{block}"
    );
    let bare = body("Intro.\n\n@available: self-managed 3.3, deployment\nPara.\n");
    assert!(
        bare.contains("states=\"ga\" versions=\"3.3\">Self-managed (GA, 3.3+)<"),
        "{bare}"
    );
    assert!(
        bare.contains(
            "target=\"deployment\" dimension=\"deployment\" states=\"ga\">Deployment (GA)<"
        ),
        "{bare}"
    );
}

#[test]
fn a_filter_build_annotates_what_remains() {
    let source = "## Sect\n@available: cloud, self-managed preview 3.4\n\nBody.\n\n## Other\n@available: self-managed\n\nGone.\n";
    let project = memory_project(FULL_MODEL, &[("index.md", &page(source))]);
    let out = site(&project, "cloud", "index.md");
    assert!(
        out.contains("<ascribe-availability scope=\"section\">"),
        "{out}"
    );
    assert!(out.contains("Body."), "{out}");
    assert!(!out.contains("Gone."), "{out}");
}

#[test]
fn page_level_availability_is_a_list_of_targets_in_the_frontmatter() {
    let source = "---\ntitle: Test\navailable: cloud, self-managed (preview 3.3, ga 3.5)\nvariant:\n  pm: [npm, yarn]\n---\n\nBody.\n";
    let project = memory_project(FULL_MODEL, &[("index.md", source)]);
    let out = site(&project, "site", "index.md");
    assert_eq!(
        out,
        "---\ntitle: Test\navailable:\n- target: cloud\n  dimension: deployment\n  states:\n  - ga\n  text: Quill Cloud (GA)\n\
         - target: self-managed\n  dimension: deployment\n  states:\n  - preview\n  - ga\n  versions:\n  - '3.3'\n  - '3.5'\n  text: Self-managed (preview 3.3, GA 3.5)\n\
         variant:\n  pm:\n  - npm\n  - yarn\n---\n\nBody.\n"
    );
    // No element is written for it: the layout renders that.
    assert!(!out.contains("<ascribe-availability"));
}

#[test]
fn frontmatter_passes_through_with_phrases_substituted() {
    let source = "---\ntitle: '{product} guide'\ntags: [a, b]\nauthor:\n  name: Ann\nversion-note: x\n---\n\nBody.\n";
    let project = memory_project(FULL_MODEL, &[("index.md", source)]);
    // `version-note` isn't declared: the check fails, but the emitter writes
    // what the resolved page holds.
    let out = site(&project, "site", "index.md");
    assert!(
        out.starts_with("---\ntitle: Quill guide\ntags:\n- a\n- b\nauthor:\n  name: Ann\n"),
        "{out}"
    );
}

#[test]
fn project_widgets_are_elements_named_after_them() {
    // A line widget bound to itself is empty, with its declared attributes.
    assert_eq!(
        body("@quill-labspace {lab=first}\n"),
        "<quill-labspace lab=\"first\"></quill-labspace>\n"
    );
    // Bound to its heading: empty, after the heading, with a primary.
    assert_eq!(
        body("## Api\n@quill-api-ref {version=2}: op-1\n"),
        "## Api <ascribe-attributes id=\"api\"></ascribe-attributes>\n\n<quill-api-ref primary=\"op-1\" version=\"2\"></quill-api-ref>\n"
    );
    // Bound to a block: wraps it.
    assert_eq!(
        body("@quill-aside\nBound.\n"),
        "<quill-aside>\n\nBound.\n\n</quill-aside>\n"
    );
    // A text primary is its content; a container wraps its blocks.
    assert_eq!(
        body("@quill-aside: Text.\n"),
        "<quill-aside>\n\nText.\n\n</quill-aside>\n"
    );
    assert_eq!(
        body(".Titled\n@quill-aside:\nOne.\n@end\n"),
        "<quill-aside heading=\"Titled\">\n\nOne.\n\n</quill-aside>\n"
    );
    // A set value's members are joined with spaces.
    let out = body("## H\n@quill-audience {audience=admin|writer}\n");
    assert!(
        out.contains("<quill-audience audience=\"admin writer\"></quill-audience>"),
        "{out}"
    );
}

#[test]
fn widget_groups_are_wrapped_in_an_ascribe_group() {
    let source =
        ".Before\n@quill-compare:\nX\n\n.After\n@quill-compare {highlight=true}:\nY\n@end\n";
    assert_eq!(
        body(source),
        "<ascribe-group widget=\"quill-compare\">\n\n\
         <quill-compare heading=\"Before\" highlight=\"false\">\n\nX\n\n</quill-compare>\n\n\
         <quill-compare heading=\"After\" highlight=\"true\">\n\nY\n\n</quill-compare>\n\n\
         </ascribe-group>\n"
    );
}

#[test]
fn elements_nest_in_lists_and_block_quotes_with_their_indentation() {
    assert_eq!(
        body("- item\n\n  @note\n  In list.\n"),
        "- item\n\n  <ascribe-note type=\"note\" label=\"Note\">\n\n  In list.\n\n  </ascribe-note>\n"
    );
    assert_eq!(
        body("> @note: Quoted.\n"),
        "> <ascribe-note type=\"note\" label=\"Note\">\n>\n> Quoted.\n>\n> </ascribe-note>\n"
    );
}

#[test]
fn raw_html_passes_through_unchanged() {
    assert_eq!(
        body("<div class=\"x\">\n  <b>raw</b>\n</div>\n\nA <kbd>Ctrl</kbd> & <!-- c --> b.\n"),
        "<div class=\"x\">\n  <b>raw</b>\n</div>\n\nA <kbd>Ctrl</kbd> & <!-- c --> b.\n"
    );
}

#[test]
fn text_is_escaped_so_it_reads_back_the_same() {
    let out = body("A \\*star\\* and \\<b> and 1\\. two.\n");
    assert_eq!(out, "A \\*star\\* and \\<b> and 1. two.\n");
    assert!(!render_site_html(&out).contains("<b>"));
}

#[test]
fn images_carry_their_attributes_and_the_models_defaults() {
    let project = memory_project_with_files(
        FULL_MODEL,
        &[(
            "index.md",
            &page("![Shot](shot.png \"T\"){theme=dark|light, width=600}\n\n![Plain](plain.png)\n"),
        )],
        &[("docs/shot.png", "x"), ("docs/plain.png", "y")],
    );
    let out = strip_frontmatter(&site(&project, "site", "index.md"));
    // Canonical order is the model's: width, height, loading, theme. The
    // default `loading` applies to every image.
    assert_eq!(
        out,
        "![Shot](./shot.png \"T\")<ascribe-attributes width=\"600\" loading=\"lazy\" theme=\"dark light\"></ascribe-attributes>\n\n\
         ![Plain](./plain.png)<ascribe-attributes loading=\"lazy\"></ascribe-attributes>\n"
    );
    let html = render_site_html(&out);
    assert!(html.contains("<img src=\"./shot.png\" alt=\"Shot\" title=\"T\" width=\"600\" loading=\"lazy\" theme=\"dark light\" />"), "{html}");
}

#[test]
fn glossary_links_carry_the_definition_as_their_title_and_name_the_term() {
    let project = memory_project(
        FULL_MODEL,
        &[
            ("index.md", &page("Keep your API key safe.\n")),
            ("reference/glossary.md", &page("## API key\n")),
        ],
    );
    let out = strip_frontmatter(&site(&project, "site", "index.md"));
    assert_eq!(
        out,
        "Keep your [API key](/docs/reference/glossary#api-key \"A secret token that authenticates the Quill agent to Quill Cloud.\")<ascribe-attributes data-ascribe-term=\"api-key\"></ascribe-attributes> safe.\n"
    );
}

#[test]
fn links_are_the_routes_of_the_consumer_profile() {
    let source =
        "[a](guides/My%20Setup.md#run-it) and [b](index.md) and [c](https://example.com/x)\n";
    let project = memory_project(
        FULL_MODEL,
        &[
            ("index.md", &page(source)),
            ("guides/My Setup.md", &page("## Run it\n")),
        ],
    );
    // base-path `/docs/`, no trailing slash, and the entry id Astro gives
    // `guides/My Setup.md`.
    assert_eq!(
        strip_frontmatter(&site(&project, "site", "index.md")),
        "[a](/docs/guides/my-setup#run-it) and [b](/docs) and [c](https://example.com/x)\n"
    );
}

#[test]
fn two_pages_with_one_route_are_refused() {
    let project = memory_project(
        FULL_MODEL,
        &[("My File.md", &page("A.\n")), ("my-file.md", &page("B.\n"))],
    );
    let emitter = tessera_emit::SiteEmitter::new(project.model());
    let build = project.model().build("site").expect("a build");
    let router = tessera_resolve::AstroRouter::from_consumer(&project.model().consumer);
    let resolved = project.resolve_build(build, &router);
    let cx = tessera_emit::EmitContext::new(&project, std::path::Path::new("/nowhere"), build);
    let error = tessera_emit::emit(&emitter, &cx, &resolved).expect_err("a collision");
    let message = error.to_string();
    assert!(message.contains("/docs/my-file"), "{message}");
    assert!(
        message.contains("My File.md") && message.contains("my-file.md"),
        "{message}"
    );
}
