//! What each line of a versioned build tells a reader that isn't a person
//! about the line: its page records' canonical addresses, and its outputs
//! for agents.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::Path;

use ascribe_emit::SiteEmitter;
use ascribe_emit::site::PAGES_PATH;
use ascribe_resolve::Project;
use serde_json::Value;

use crate::support::{emit_build, memory_project};

const MODEL: &str = "spec = \"0.1\"\n\
    [types.page]\n\
    [types.page.frontmatter]\ntitle = \"string\"\n\
    description = { type = \"string?\", role = \"description\" }\n\
    [dimensions.deployment]\nvalues = [\"cloud\", \"self-managed\"]\nversionless = [\"cloud\"]\n\
    labels = { \"self-managed\" = \"Self-managed\" }\n\
    [releases.self-managed]\nlive = [\"3.4\", \"3.5\"]\n\
    [releases.self-managed.ends]\n\"3.4\" = 2027-01-31\n\
    [consumer]\nsite = \"https://docs.example.com\"\nbase-path = \"/docs/\"\nagents = true\n\
    [builds.site]\nvariants = \"switch\"\navailability = \"badge\"\nversions = \"self-managed\"\n";

/// Pages in both lines, one only the current line has, and one the current
/// line rewrote under a new name.
const FILES: &[(&str, &str)] = &[
    ("index.md", "---\ntype: page\ntitle: Home\n---\n"),
    (
        "install.md",
        "---\ntype: page\ntitle: Install\ndescription: Install it.\n---\n",
    ),
    (
        "webhooks.md",
        "---\ntype: page\ntitle: Webhooks\navailable: self-managed 3.5\n---\n",
    ),
    (
        "configure-legacy.md",
        "---\ntype: page\ntitle: Configure (legacy)\navailable: self-managed (ga 3.4, removed 3.5)\n---\n",
    ),
    (
        "configure.md",
        "---\ntype: page\ntitle: Configure\navailable: self-managed 3.5\nreplaces: configure-legacy.md\n---\n",
    ),
    (
        "old-only.md",
        "---\ntype: page\ntitle: Old only\navailable: self-managed (ga 3.4, removed 3.5)\n---\n",
    ),
];

fn records(project: &Project, build: &str) -> Value {
    let emitter = SiteEmitter::new(project.model());
    let text = emit_build(Path::new("/nowhere"), project, build, &emitter)
        .remove(PAGES_PATH)
        .expect("the site output has the records");
    serde_json::from_str(&text).expect("the records are JSON")
}

fn canonical<'a>(records: &'a Value, route: &str) -> &'a str {
    records["pages"][route]["canonical"]
        .as_str()
        .unwrap_or_else(|| panic!("{route} has a canonical address: {records:#}"))
}

#[test]
fn an_older_line_s_canonical_address_is_the_current_line_s_page() {
    let project = memory_project(MODEL, FILES);
    let older = records(&project, "site@3.4");
    // The same page in the current line.
    assert_eq!(
        canonical(&older, "/docs/3.4/install/"),
        "https://docs.example.com/docs/install/"
    );
    assert_eq!(
        canonical(&older, "/docs/3.4/"),
        "https://docs.example.com/docs/"
    );
    // The page that replaced it.
    assert_eq!(
        canonical(&older, "/docs/3.4/configure-legacy/"),
        "https://docs.example.com/docs/configure/"
    );
    // Its own, when the current line has neither.
    assert_eq!(
        canonical(&older, "/docs/3.4/old-only/"),
        "https://docs.example.com/docs/3.4/old-only/"
    );
    // The current line's are its own.
    let current = records(&project, "site");
    assert_eq!(
        canonical(&current, "/docs/install/"),
        "https://docs.example.com/docs/install/"
    );
    assert_eq!(
        canonical(&current, "/docs/configure/"),
        "https://docs.example.com/docs/configure/"
    );
}

#[test]
fn without_a_site_no_line_has_a_canonical_address() {
    let model = MODEL
        .replace("site = \"https://docs.example.com\"\n", "")
        .replace("agents = true\n", "");
    let project = memory_project(&model, FILES);
    let older = records(&project, "site@3.4");
    assert_eq!(older["pages"]["/docs/3.4/install/"].get("canonical"), None);
}
