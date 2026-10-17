//! `_ascribe/pages.json`: each build's page records, in the site output
//! (output-layout contract §1.3), and `llms.txt` and `_ascribe/headers.json`
//! (§1.4) read against them.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_emit::site::{HEADERS_PATH, PAGES_PATH};
use ascribe_emit::{PlainEmitter, SiteEmitter};
use ascribe_resolve::Project;
use serde_json::{Value, json};

use crate::support::{emit_build, memory_project, site};

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page]\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    description = { type = \"string?\", role = \"description\" }\n\
    [consumer]\nsite = \"https://docs.example.com\"\nbase-path = \"/docs/\"\nagents = true\n\
    [crawlers]\nlines = [\"Disallow: /drafts/\", \"User-agent: Bot\", \"Allow: *.md\", \"Sitemap: https://docs.example.com/x.xml\"]\n";

const FILES: &[(&str, &str)] = &[
    (
        "index.md",
        "---\ntype: page\ntitle: Home\ndescription: Every page of [the docs](index.md).\n---\n",
    ),
    (
        "guides/install.md",
        "---\ntype: page\ntitle: Install the agent\ndescription: Install and connect it.\n---\n",
    ),
    (
        "guides/drafts.md",
        "---\ntype: page\ntitle: Drafts\nlisting: unlisted\n---\n",
    ),
    (
        "faq.md",
        "---\ntype: page\ntitle: FAQ\nlisting: unquoted\n---\n",
    ),
];

fn records(project: &Project) -> Value {
    let emitter = SiteEmitter::new(project.model());
    let text = emit_build(Path::new("/nowhere"), project, "site", &emitter)
        .remove(PAGES_PATH)
        .expect("the site output has the records");
    serde_json::from_str(&text).expect("the records are JSON")
}

fn headers(project: &Project) -> Value {
    let emitter = SiteEmitter::new(project.model());
    let text = emit_build(Path::new("/nowhere"), project, "site", &emitter)
        .remove(HEADERS_PATH)
        .expect("the site output has the headers");
    serde_json::from_str(&text).expect("the headers are JSON")
}

#[test]
fn each_page_has_a_record_keyed_by_its_route() {
    let project = memory_project(MODEL, FILES);
    assert_eq!(
        records(&project),
        json!({
            "version": 1,
            "site": "https://docs.example.com",
            "pages": {
                "/docs/": {
                    "source": "index.md",
                    "title": "Home",
                    "description": "Every page of the docs.",
                    "canonical": "https://docs.example.com/docs/",
                    "listing": "listed",
                    "listing_from": { "level": "default" },
                    "markdown": "/docs/index.md",
                    "breadcrumbs": [{ "route": "/docs/", "title": "Home" }],
                },
                "/docs/faq/": {
                    "source": "faq.md",
                    "title": "FAQ",
                    "canonical": "https://docs.example.com/docs/faq/",
                    "listing": "unquoted",
                    "listing_from": { "level": "page" },
                    "markdown": "/docs/faq.md",
                    "breadcrumbs": [
                        { "route": "/docs/", "title": "Home" },
                        { "route": "/docs/faq/", "title": "FAQ" },
                    ],
                },
                "/docs/guides/drafts/": {
                    "source": "guides/drafts.md",
                    "title": "Drafts",
                    "canonical": "https://docs.example.com/docs/guides/drafts/",
                    "listing": "unlisted",
                    "listing_from": { "level": "page" },
                    "markdown": "/docs/guides/drafts.md",
                    "breadcrumbs": [
                        { "route": "/docs/", "title": "Home" },
                        { "route": "/docs/guides/drafts/", "title": "Drafts" },
                    ],
                },
                "/docs/guides/install/": {
                    "source": "guides/install.md",
                    "title": "Install the agent",
                    "description": "Install and connect it.",
                    "canonical": "https://docs.example.com/docs/guides/install/",
                    "listing": "listed",
                    "listing_from": { "level": "default" },
                    "markdown": "/docs/guides/install.md",
                    "breadcrumbs": [
                        { "route": "/docs/", "title": "Home" },
                        { "route": "/docs/guides/install/", "title": "Install the agent" },
                    ],
                },
            },
            "robots": {
                "lines": [
                    "Disallow: /docs/drafts/",
                    "User-agent: Bot",
                    "Allow: *.md",
                    "Sitemap: https://docs.example.com/x.xml",
                ],
                "purposes": {},
                "groups": [],
            },
        })
    );
}

#[test]
fn without_a_site_or_agents_a_record_has_no_addresses_but_its_route() {
    let model = "spec = \"0.1\"\n";
    let project = memory_project(model, &[("a.md", "---\ntype: page\ntitle: A\n---\n")]);
    assert_eq!(
        records(&project),
        json!({
            "version": 1,
            "pages": {
                "/a/": {
                    "source": "a.md",
                    "title": "A",
                    "listing": "listed",
                    "listing_from": { "level": "default" },
                    "breadcrumbs": [{ "route": "/a/", "title": "A" }],
                },
            },
            "robots": { "lines": [], "purposes": {}, "groups": [] },
        })
    );
}

#[test]
fn a_build_with_no_pages_has_an_empty_table() {
    let project = memory_project("spec = \"0.1\"\n", &[]);
    assert_eq!(
        records(&project),
        json!({ "version": 1, "pages": {}, "robots": { "lines": [], "purposes": {}, "groups": [] } })
    );
}

#[test]
fn a_disallowed_purpose_is_a_group_of_every_token_ascribe_knows_of_it() {
    let model = "spec = \"0.1\"\n\
        [crawlers]\ntraining = \"disallow\"\nsearch = \"allow\"\non-request = \"disallow\"\n\
        lines = [\"Disallow: /drafts/\"]\n";
    let project = memory_project(model, &[]);
    let tokens = |purpose| -> Vec<&str> { ascribe_model::crawler_tokens(purpose).collect() };
    assert_eq!(
        records(&project)["robots"],
        json!({
            "lines": ["Disallow: /drafts/"],
            "purposes": { "training": "disallow", "search": "allow", "on-request": "disallow" },
            "groups": [
                {
                    "purpose": "training",
                    "user_agents": tokens(ascribe_model::Purpose::Training),
                    "lines": ["Disallow: /"],
                },
                {
                    "purpose": "on-request",
                    "user_agents": tokens(ascribe_model::Purpose::OnRequest),
                    "lines": ["Disallow: /"],
                },
            ],
        })
    );
}

/// Each page's breadcrumbs, as `route title` lines, by route.
fn steps(records: &Value) -> Vec<(String, Vec<String>)> {
    let pages = records["pages"].as_object().expect("pages");
    pages
        .iter()
        .map(|(route, page)| {
            let steps = page["breadcrumbs"].as_array().expect("breadcrumbs");
            let steps = steps
                .iter()
                .map(|step| {
                    format!(
                        "{} {}",
                        step["route"].as_str().unwrap_or(""),
                        step["title"].as_str().unwrap_or("")
                    )
                })
                .collect();
            (route.clone(), steps)
        })
        .collect()
}

#[test]
fn breadcrumbs_are_the_pages_at_a_routes_folders() {
    // A folder with an index page is a step, and one without isn't; a page
    // with no title isn't either. The base path's page is the first step.
    let model = "spec = \"0.1\"\n[consumer]\nbase-path = \"/docs/\"\n";
    let files: &[(&str, &str)] = &[
        ("index.md", "---\ntype: page\ntitle: Docs\n---\n"),
        ("guides/index.md", "---\ntype: page\ntitle: Guides\n---\n"),
        (
            "guides/Setup/Install Agent.md",
            "---\ntype: page\ntitle: Install\n---\n",
        ),
        ("reference/cli.md", "---\ntype: page\ntitle: CLI\n---\n"),
        ("untitled/index.md", "Untitled.\n"),
        ("untitled/page.md", "---\ntype: page\ntitle: Page\n---\n"),
    ];
    let project = memory_project(model, files);
    assert_eq!(
        steps(&records(&project)),
        vec![
            ("/docs/".to_owned(), vec!["/docs/ Docs".to_owned()]),
            (
                "/docs/guides/".to_owned(),
                vec!["/docs/ Docs".to_owned(), "/docs/guides/ Guides".to_owned()],
            ),
            (
                "/docs/guides/setup/install-agent/".to_owned(),
                vec![
                    "/docs/ Docs".to_owned(),
                    "/docs/guides/ Guides".to_owned(),
                    "/docs/guides/setup/install-agent/ Install".to_owned(),
                ],
            ),
            (
                "/docs/reference/cli/".to_owned(),
                vec![
                    "/docs/ Docs".to_owned(),
                    "/docs/reference/cli/ CLI".to_owned()
                ],
            ),
            ("/docs/untitled/".to_owned(), vec!["/docs/ Docs".to_owned()]),
            (
                "/docs/untitled/page/".to_owned(),
                vec![
                    "/docs/ Docs".to_owned(),
                    "/docs/untitled/page/ Page".to_owned()
                ],
            ),
        ]
    );
}

#[test]
fn breadcrumbs_at_the_root_and_without_a_trailing_slash() {
    // At the host's root, the root page is the first step; without a
    // trailing slash, each route has none.
    let model = "spec = \"0.1\"\n[consumer]\ntrailing-slash = \"never\"\n";
    let files: &[(&str, &str)] = &[
        ("index.md", "---\ntype: page\ntitle: Home\n---\n"),
        ("a/index.md", "---\ntype: page\ntitle: A\n---\n"),
        ("a/b.md", "---\ntype: page\ntitle: B\n---\n"),
    ];
    assert_eq!(
        steps(&records(&memory_project(model, files))),
        vec![
            ("/".to_owned(), vec!["/ Home".to_owned()]),
            (
                "/a".to_owned(),
                vec!["/ Home".to_owned(), "/a A".to_owned()]
            ),
            (
                "/a/b".to_owned(),
                vec!["/ Home".to_owned(), "/a A".to_owned(), "/a/b B".to_owned()],
            ),
        ]
    );
}

#[test]
fn listing_is_in_the_record_and_not_the_frontmatter() {
    let project = memory_project(MODEL, FILES);
    let page = site(&project, "site", "guides/drafts.md");
    assert!(!page.contains("listing"), "{page}");
}

/// The line of `llms.txt` that links to `url`, if any.
fn line_for<'a>(llms: &'a str, url: &str) -> Option<&'a str> {
    let link = format!("]({url})");
    llms.lines().find(|line| line.contains(&link))
}

/// [`MODEL`] with a rule at every level of `[listing]`.
fn with_rules() -> String {
    format!(
        "{MODEL}[types.changelog]\n\
         [types.changelog.frontmatter]\ntitle = \"string\"\n\
         [listing]\ndefault = \"listed\"\n\
         [listing.folders]\n\"internal/\" = \"unlisted\"\n\
         [listing.types]\nchangelog = \"unquoted\"\n"
    )
}

/// [`FILES`], with pages under each rule of [`with_rules`].
fn files_with_rules() -> Vec<(&'static str, &'static str)> {
    let mut files = FILES.to_vec();
    files.extend([
        ("internal/plan.md", "---\ntype: page\ntitle: Plan\n---\n"),
        ("changes/one.md", "---\ntype: changelog\ntitle: One\n---\n"),
        (
            "changes/two.md",
            "---\ntype: changelog\ntitle: Two\nlisting: unlisted\n---\n",
        ),
    ]);
    files
}

#[test]
fn a_record_says_which_rule_its_listing_comes_from() {
    let model = with_rules();
    let project = memory_project(&model, &files_with_rules());
    let pages = records(&project)["pages"].clone();
    let line = |rule: &str| {
        model
            .lines()
            .position(|l| l == rule)
            .expect("the rule is in the model")
            + 1
    };
    assert_eq!(pages["/docs/internal/plan/"]["listing"], "unlisted");
    assert_eq!(
        pages["/docs/internal/plan/"]["listing_from"],
        json!({
            "level": "folder",
            "folder": "internal/",
            "rule": "\"internal/\" = \"unlisted\"",
            "line": line("\"internal/\" = \"unlisted\""),
        })
    );
    assert_eq!(pages["/docs/changes/one/"]["listing"], "unquoted");
    assert_eq!(
        pages["/docs/changes/one/"]["listing_from"],
        json!({
            "level": "type",
            "type": "changelog",
            "rule": "changelog = \"unquoted\"",
            "line": line("changelog = \"unquoted\""),
        })
    );
    assert_eq!(pages["/docs/changes/two/"]["listing"], "unlisted");
    assert_eq!(
        pages["/docs/changes/two/"]["listing_from"],
        json!({ "level": "page" })
    );
    assert_eq!(pages["/docs/guides/install/"]["listing"], "listed");
    assert_eq!(
        pages["/docs/guides/install/"]["listing_from"],
        json!({
            "level": "site",
            "rule": "default = \"listed\"",
            "line": line("default = \"listed\""),
        })
    );
}

#[test]
fn llms_txt_lists_each_offered_record_as_the_record_says() {
    let project = memory_project(MODEL, FILES);
    llms_txt_agrees_with_the_records(&project);
}

#[test]
fn llms_txt_agrees_with_the_records_with_rules_at_every_level() {
    let project = memory_project(&with_rules(), &files_with_rules());
    llms_txt_agrees_with_the_records(&project);
}

#[test]
fn a_page_that_leaves_its_description_to_the_default_has_the_default() {
    let model = MODEL.replace(
        "role = \"description\" }",
        "role = \"description\", default = \"A guide to *Lantern*.\" }",
    );
    assert_ne!(model, MODEL, "the model's description field moved");
    let files = [
        ("index.md", "---\ntype: page\ntitle: Home\n---\n"),
        (
            "empty.md",
            "---\ntype: page\ntitle: Empty\ndescription: \" \"\n---\n",
        ),
        (
            "own.md",
            "---\ntype: page\ntitle: Own\ndescription: Its own.\n---\n",
        ),
    ];
    let project = memory_project(&model, &files);
    let pages = records(&project)["pages"].clone();
    assert_eq!(pages["/docs/"]["description"], "A guide to Lantern.");
    assert_eq!(pages["/docs/empty/"]["description"], "A guide to Lantern.");
    assert_eq!(pages["/docs/own/"]["description"], "Its own.");
    llms_txt_agrees_with_the_records(&project);
}

fn llms_txt_agrees_with_the_records(project: &Project) {
    let records = records(project);
    let site = records["site"].as_str().expect("a site");
    let plain = emit_build(
        Path::new("/nowhere"),
        project,
        "site",
        &PlainEmitter::new(project.model()),
    );
    let llms = &plain["llms.txt"];
    let pages = records["pages"].as_object().expect("pages");
    let mut listed = 0;
    for (route, record) in pages {
        let url = format!("{site}{}", record["markdown"].as_str().expect("markdown"));
        let line = line_for(llms, &url);
        if record["listing"] == "unlisted" {
            assert_eq!(
                line, None,
                "{route} is unlisted, and llms.txt has it:\n{llms}"
            );
            continue;
        }
        let line = line.unwrap_or_else(|| panic!("{route} isn't in llms.txt:\n{llms}"));
        let mut want = format!("- [{}]({url})", record["title"].as_str().expect("a title"));
        if let Some(description) = record["description"].as_str() {
            want = format!("{want}: {description}");
        }
        assert_eq!(line, want, "{route}");
        listed += 1;
    }
    let lines = llms.lines().filter(|l| l.starts_with("- [")).count();
    assert_eq!(lines, listed, "llms.txt has a line with no record:\n{llms}");
}

#[test]
fn each_markdown_version_has_its_headers() {
    let project = memory_project(MODEL, FILES);
    assert_eq!(
        headers(&project),
        json!({
            "version": 1,
            "headers": [
                {
                    "path": "/docs/faq.md",
                    "headers": {
                        "Link": "<https://docs.example.com/docs/faq/>; rel=\"canonical\"",
                        "X-Robots-Tag": "nosnippet",
                    },
                },
                {
                    "path": "/docs/guides/drafts.md",
                    "headers": {
                        "Link": "<https://docs.example.com/docs/guides/drafts/>; rel=\"canonical\"",
                        "X-Robots-Tag": "noindex",
                    },
                },
                {
                    "path": "/docs/guides/install.md",
                    "headers": {
                        "Link": "<https://docs.example.com/docs/guides/install/>; rel=\"canonical\"",
                    },
                },
                {
                    "path": "/docs/index.md",
                    "headers": {
                        "Link": "<https://docs.example.com/docs/>; rel=\"canonical\"",
                    },
                },
            ],
        })
    );
}

#[test]
fn without_markdown_versions_the_headers_are_an_empty_list() {
    let project = memory_project(
        "spec = \"0.1\"\n",
        &[(
            "a.md",
            "---\ntype: page\ntitle: A\nlisting: unlisted\n---\n",
        )],
    );
    assert_eq!(headers(&project), json!({ "version": 1, "headers": [] }));
}

#[test]
fn the_headers_agree_with_the_records() {
    let project = memory_project(&with_rules(), &files_with_rules());
    let records = records(&project);
    let headers = headers(&project);
    let rules = headers["headers"].as_array().expect("a list");
    let pages = records["pages"].as_object().expect("pages");
    // Every page with a Markdown version has a rule, and nothing else does.
    let markdown: Vec<&str> = pages
        .values()
        .filter_map(|record| record["markdown"].as_str())
        .collect();
    let paths: Vec<&str> = rules
        .iter()
        .map(|rule| rule["path"].as_str().expect("a path"))
        .collect();
    let mut sorted = markdown.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted, "one rule for each Markdown version, sorted");
    for (route, record) in pages {
        let path = record["markdown"].as_str().expect("markdown");
        let rule = rules
            .iter()
            .find(|rule| rule["path"] == path)
            .unwrap_or_else(|| panic!("{route}'s Markdown version has no rule"));
        let canonical = record["canonical"].as_str().expect("canonical");
        assert_eq!(
            rule["headers"]["Link"],
            format!("<{canonical}>; rel=\"canonical\""),
            "{route}"
        );
        let robots = match record["listing"].as_str() {
            Some("listed") => Value::Null,
            Some("unquoted") => json!("nosnippet"),
            Some("unlisted") => json!("noindex"),
            other => panic!("{route}: listing {other:?}"),
        };
        assert_eq!(rule["headers"]["X-Robots-Tag"], robots, "{route}");
    }
}
