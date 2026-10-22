//! The hover on a content type's name in `ascribe.toml` (#809): where its
//! template and guide come from, the lock `ascribe templates eject` wrote or
//! its own table, and that it has none; and the lock followed on disk.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use std::path::Path;

use lsp_types::{FileChangeType, Position};
use serde_json::json;

use crate::support::{Client, Fixture, MODEL, uri};

const LOCK: &str = "version = 1\n\n[[template]]\nname = \"how-to\"\n\n[template.files]\n\"templates/how-to.md\" = \"sha256:00\"\n\"templates/how-to.guide.md\" = \"sha256:00\"\n\n[template.ascribe-files]\n\"templates/how-to.ascribe.md\" = \"sha256:00\"\n";

const TEMPLATE: &str = "---\ntitle: \"Title\"\n---\n\nTo \\{do something}, follow these steps.\n";

fn model() -> String {
    format!(
        "{MODEL}\n[types.index]\n\n[types.how-to]\npurpose = \"Doing one thing.\"\n\n[types.concept]\ntemplate = \"writing/concept.md\"\n"
    )
}

fn project() -> Fixture {
    Fixture::new(
        &model(),
        &[
            (
                "docs/index.md",
                "---\ntype: index\ntitle: Home\n---\n\nHello.\n",
            ),
            ("templates/how-to.md", TEMPLATE),
            ("templates/how-to.guide.md", "How to write a how-to.\n"),
            ("templates/how-to.ascribe.md", "Our notes.\n"),
            ("templates/templates.lock", LOCK),
            ("writing/concept.md", TEMPLATE),
        ],
    )
}

/// The hover on `name` in its `[types.<name>]` header, in `text`.
fn hover(client: &mut Client, config: &Path, text: &str, name: &str) -> Option<String> {
    let header = format!("[types.{name}]");
    let line = text
        .lines()
        .position(|l| l == header)
        .unwrap_or_else(|| panic!("no {header}"));
    let result = client
        .request(
            "textDocument/hover",
            json!({
                "textDocument": { "uri": uri(config).as_str() },
                "position": Position::new(u32::try_from(line).unwrap(), 9),
            }),
        )
        .response_result
        .expect("a hover");
    result["contents"]["value"].as_str().map(str::to_owned)
}

#[test]
fn the_hover_on_a_type_says_where_its_template_comes_from() {
    let f = project();
    let config = f.path("ascribe.toml");
    let text = model();
    let mut client = Client::start(&f.root());
    client.open(&config, 1, &text);
    client.settle();

    let how_to = hover(&mut client, &config, &text, "how-to").expect("a hover");
    assert!(how_to.contains("Doing one thing."), "{how_to}");
    assert!(
        how_to.contains(
            "Its template is `templates/how-to.md`, and its guide `templates/how-to.guide.md`, `templates/how-to.ascribe.md`, from `templates/templates.lock`"
        ),
        "{how_to}"
    );
    let concept = hover(&mut client, &config, &text, "concept").expect("a hover");
    assert!(
        concept.contains("Its template is `writing/concept.md`, named in this table."),
        "{concept}"
    );
    let index = hover(&mut client, &config, &text, "index").expect("a hover");
    assert!(index.contains("It has no template or guide."), "{index}");

    // Without the lock, the how-to has none either.
    f.remove("templates/templates.lock");
    client.watched(&[(&f.path("templates/templates.lock"), FileChangeType::DELETED)]);
    client.settle();
    let how_to = hover(&mut client, &config, &text, "how-to").expect("a hover");
    assert!(how_to.contains("It has no template or guide."), "{how_to}");
}
