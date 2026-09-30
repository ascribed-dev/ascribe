//! Scripted quick fixes and source refactorings over a copy of `examples/quill`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use std::path::{Path, PathBuf};

use lsp_types::{FileChangeType, PositionEncodingKind};
use serde_json::{Value, json};
use support::{Client, Fixture, MODEL, Setup, uri};
use tessera_core::{LineIndex, Span, TextEdit, apply_edits};
use tessera_lsp::Encoding;

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read_dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

fn quill() -> Fixture {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/quill");
    let f = Fixture::new(MODEL, &[]);
    copy_dir(&source, &f.root());
    f
}

fn read(f: &Fixture, rel: &str) -> String {
    std::fs::read_to_string(f.path(rel)).expect("read")
}

fn write(f: &Fixture, rel: &str, text: &str) {
    std::fs::write(f.path(rel), text).expect("write");
}

fn position(text: &str, offset: usize) -> Value {
    let before = &text[..offset];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    let character: usize = text[start..offset].chars().map(char::len_utf16).sum();
    json!({ "line": line, "character": character })
}

fn diagnostic(client: &Client, path: &Path, slug: &str) -> Value {
    let diagnostics = client.diagnostics(path);
    serde_json::to_value(
        diagnostics
            .iter()
            .find(|diagnostic| support::slug(diagnostic) == slug)
            .unwrap_or_else(|| {
                panic!(
                    "no {slug} diagnostic on {}: {:?}",
                    path.display(),
                    diagnostics.iter().map(support::slug).collect::<Vec<_>>()
                )
            }),
    )
    .expect("json")
}

fn actions(client: &mut Client, path: &Path, diagnostic: Value) -> Vec<Value> {
    let range = diagnostic["range"].clone();
    client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(path).as_str() },
                "range": range,
                "context": { "diagnostics": [diagnostic] }
            }),
        )
        .response_result
        .expect("actions")
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn apply_edit(text: &str, edits: &[Value]) -> String {
    apply_edit_with_encoding(text, edits, Encoding::Utf16)
}

fn apply_edit_with_encoding(text: &str, edits: &[Value], encoding: Encoding) -> String {
    let index = LineIndex::new(text);
    let edits = edits
        .iter()
        .map(|edit| {
            let range = &edit["range"];
            let start = encoding
                .offset(
                    &index,
                    serde_json::from_value(range["start"].clone()).expect("start"),
                )
                .expect("start offset");
            let end = encoding
                .offset(
                    &index,
                    serde_json::from_value(range["end"].clone()).expect("end"),
                )
                .expect("end offset");
            TextEdit::replace(
                Span::new(start, end),
                edit["newText"].as_str().expect("text"),
            )
        })
        .collect::<Vec<_>>();
    apply_edits(text, &edits).expect("valid edits")
}

fn apply_workspace_edit(f: &Fixture, edit: &Value) -> Vec<PathBuf> {
    apply_workspace_edit_with_encoding(f, edit, Encoding::Utf16)
}

fn apply_workspace_edit_with_encoding(
    f: &Fixture,
    edit: &Value,
    encoding: Encoding,
) -> Vec<PathBuf> {
    let changes = edit["changes"].as_object().expect("changes");
    let mut changed = Vec::new();
    for (uri_text, edits) in changes {
        let path = uri_text
            .strip_prefix("file://")
            .expect("file URI")
            .replace("%20", " ");
        let path = PathBuf::from(path);
        let original = std::fs::read_to_string(&path).expect("edit source exists");
        std::fs::write(
            &path,
            apply_edit_with_encoding(&original, edits.as_array().expect("edits"), encoding),
        )
        .expect("apply edits");
        assert!(path.starts_with(f.root()));
        changed.push(path);
    }
    changed
}

fn notify_changed(client: &mut Client, paths: &[PathBuf]) {
    let events: Vec<_> = paths
        .iter()
        .map(|path| (path.as_path(), FileChangeType::CHANGED))
        .collect();
    client.watched(&events);
    client.settle();
}

fn assert_no_errors(client: &Client) {
    for publication in client.latest.values() {
        for diagnostic in &publication.diagnostics {
            assert_ne!(
                diagnostic.severity,
                Some(lsp_types::DiagnosticSeverity::ERROR),
                "{}",
                diagnostic.message
            );
        }
    }
}

#[test]
fn container_colon_action_removes_the_diagnostic() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\n@variant {{deployment=cloud}}\nThis stays in the cloud build.\n@end\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();

    let diag = diagnostic(&client, &path, "container-colon-missing");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Add a trailing colon")
        .expect("colon action");
    let edits = action["edit"]["changes"][uri(&path).as_str()]
        .as_array()
        .expect("edits")
        .clone();
    let fixed = apply_edit(&text, &edits);
    assert!(fixed.contains("@variant {deployment=cloud}:\nThis stays in the cloud build."));
    write(&f, "docs/keys.md", &fixed);
    notify_changed(&mut client, std::slice::from_ref(&path));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "container-colon-missing")
    );
    assert_no_errors(&client);
}

#[test]
fn stray_container_colon_action_removes_the_diagnostic() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\n@steps:\n1. Keep this step.\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "container-colon-unexpected");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Remove the stray colon")
        .expect("colon action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("@steps\n1. Keep this step."));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "container-colon-unexpected")
    );
    assert_no_errors(&client);
}

#[test]
fn typo_action_suggests_a_warning_note() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\n@warning:\nWarning text.\n@end\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "directive-unknown");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Replace with `@note {type=warning}:`")
        .expect("typo action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    let fixed = read(&f, "docs/keys.md");
    assert!(fixed.contains("@note {type=warning}:\nWarning text.\n@end"));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "directive-unknown")
    );
    assert_no_errors(&client);
}

#[test]
fn quote_attribute_action_removes_the_diagnostic() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let model = format!(
        "{}\n[widgets.quill-labspace]\nforms = [\"line\"]\nprimary = \"none\"\nbinding = \"self\"\n\n[widgets.quill-labspace.attributes]\nlab = \"string\"\n",
        read(&f, "ascribe.toml")
    );
    write(&f, "ascribe.toml", &model);
    let text = format!(
        "{}\n@quill-labspace {{lab=Using other images}}\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "attribute-unquoted-reserved");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Quote the attribute value")
        .expect("quote action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("lab=\"Using other images\""));
    assert_no_errors(&client);
}

#[test]
fn blank_line_action_removes_the_diagnostic() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!("{}\n@note\n\nA bound block.\n", read(&f, "docs/keys.md"));
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "binding-blank-line");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Remove the blank line before the bound block")
        .expect("blank-line action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("@note\nA bound block."));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "binding-blank-line")
    );
    assert_no_errors(&client);
}

#[test]
fn undeclared_phrase_can_be_escaped_or_declared() {
    for title in [
        "Escape this phrase as literal text",
        "Declare phrase in ascribe.toml",
    ] {
        let f = quill();
        let path = f.path("docs/keys.md");
        let text = format!("{}\nA {{phase24-only}} phrase.\n", read(&f, "docs/keys.md"));
        write(&f, "docs/keys.md", &text);
        let mut client = Client::start(&f.root());
        client.settle();
        let diag = diagnostic(&client, &path, "phrase-undeclared");
        let action = actions(&mut client, &path, diag)
            .into_iter()
            .find(|action| action["title"] == title)
            .expect("phrase action");
        let changed = apply_workspace_edit(&f, &action["edit"]);
        notify_changed(&mut client, &changed);
        if title.starts_with("Escape") {
            assert!(read(&f, "docs/keys.md").contains(r"\{phase24-only}"));
        } else {
            assert!(read(&f, "ascribe.toml").contains("\"phase24-only\" = \"\""));
        }
        assert!(
            !client
                .diagnostics(&path)
                .iter()
                .any(|d| support::slug(d) == "phrase-undeclared")
        );
        assert_no_errors(&client);
    }
}

#[test]
fn route_action_rewrites_the_destination_to_a_file_path() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\nSee 😀 [Install](/install-agent/).\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "link-route");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Link to the page's file instead of its route")
        .expect("route action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("[Install](/install-agent.md)"));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "link-route")
    );
    assert_no_errors(&client);
}

#[test]
fn route_action_uses_utf8_positions_after_multibyte_text() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\nSee 😀 [Install](/install-agent/).\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut setup = Setup::default();
    setup.encodings = Some(vec![PositionEncodingKind::UTF8]);
    let mut client = Client::start_with(&f.root(), setup);
    client.settle();
    let diag = diagnostic(&client, &path, "link-route");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Link to the page's file instead of its route")
        .expect("route action");
    let changed = apply_workspace_edit_with_encoding(&f, &action["edit"], Encoding::Utf8);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("[Install](/install-agent.md)"));
    assert_no_errors(&client);
}

#[test]
fn renaming_a_phrase_updates_parsed_uses_only() {
    let f = quill();
    let config_path = f.path("ascribe.toml");
    let mut model = read(&f, "ascribe.toml");
    model = model.replace(
        "api = \"https://api.quill.dev/v3/\"",
        "api = \"https://api.quill.dev/v3/\"\npath = \"install-agent.md\"",
    );
    write(&f, "ascribe.toml", &model);
    let text = format!(
        "{}\nSee [install]({{path}}).\n`{{path}}`\n\\{{path}}\n\n```yaml phrases=true\nfile: {{path}}\n```\n\n```yaml\nfile: {{path}}\n```\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let position = position(&model, model.find("path =").expect("key"));
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&config_path).as_str() },
                "position": position,
                "newName": "guide"
            }),
        )
        .response_result
        .expect("phrase rename");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);
    let updated_model = read(&f, "ascribe.toml");
    let updated_source = read(&f, "docs/keys.md");
    assert!(
        updated_model.contains("guide = \"install-agent.md\""),
        "{updated_model}"
    );
    assert!(updated_source.contains("[install]({guide})"));
    assert!(updated_source.contains("file: {guide}"));
    assert!(updated_source.contains("`{path}`"));
    assert!(updated_source.contains(r"\{path}"));
    assert!(updated_source.contains("file: {path}"));
    assert_no_errors(&client);
}

#[test]
fn heading_rename_updates_fragment_links_and_offers_a_stable_id() {
    let f = quill();
    let target = f.path("docs/keys.md");
    let target_text = format!(
        "{}\n## Keep secrets\n\nProtect them.\n",
        read(&f, "docs/keys.md")
    );
    let source_text = format!(
        "{}\n[Secrets](keys.md#keep-secrets)\n@include: keys.md#keep-secrets\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/keys.md", &target_text);
    write(&f, "docs/install-agent.md", &source_text);
    let mut client = Client::start(&f.root());
    client.settle();
    let heading_at = target_text.find("Keep secrets").expect("heading");
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&target).as_str() },
                "position": position(&target_text, heading_at),
                "newName": "Protect secrets"
            }),
        )
        .response_result
        .expect("heading rename");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("## Protect secrets"));
    assert!(read(&f, "docs/install-agent.md").contains("#protect-secrets"));
    assert!(read(&f, "docs/install-agent.md").contains("@include: keys.md#protect-secrets"));
    assert_no_errors(&client);

    let mut action_client = Client::start(&f.root());
    action_client.settle();
    let text = read(&f, "docs/keys.md");
    let heading_at = text.find("Protect secrets").expect("heading");
    let range = json!({
        "start": position(&text, heading_at),
        "end": position(&text, heading_at + "Protect secrets".len())
    });
    let result = action_client.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri(&target).as_str() },
            "range": range,
            "context": { "diagnostics": [] }
        }),
    );
    assert!(
        result
            .response_result
            .expect("code actions")
            .as_array()
            .unwrap()
            .iter()
            .any(|action| action["title"] == "Add a stable @id for this heading")
    );
}

#[test]
fn formatting_returns_only_tessera_fmt_edits() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let original = read(&f, "docs/keys.md");
    let text = format!("{original}\n@note{{ type = tip }}: Be careful.\n");
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "textDocument/formatting",
            json!({
                "textDocument": { "uri": uri(&path).as_str() },
                "options": { "tabSize": 2, "insertSpaces": true }
            }),
        )
        .response_result
        .expect("format edits");
    let edits = result.as_array().expect("edit list");
    let formatted = apply_edit(&text, edits);
    let expected = format!("{original}\n@note {{type=tip}}: Be careful.\n");
    assert_eq!(formatted, expected);
}

#[test]
fn file_move_updates_incoming_and_outgoing_relative_references() {
    let f = quill();
    let old = f.path("docs/keys.md");
    let new = f.path("docs/guides/my guides/keys.md");
    let old_asset = f.path("docs/playground.png");
    let new_asset = f.path("docs/my images/playground.png");
    std::fs::create_dir_all(new.parent().expect("parent")).expect("mkdir");
    std::fs::create_dir_all(new_asset.parent().expect("parent")).expect("mkdir");
    let linking = format!(
        "{}\n[Keys][keys-ref]\n\n[keys-ref]: keys.md#rotate-keys\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/install-agent.md", &linking);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "workspace/willRenameFiles",
            json!({
                "files": [
                    {
                        "oldUri": uri(&old).as_str(),
                        "newUri": uri(&new).as_str()
                    },
                    {
                        "oldUri": uri(&old_asset).as_str(),
                        "newUri": uri(&new_asset).as_str()
                    }
                ]
            }),
        )
        .response_result
        .expect("will rename");
    let changed = apply_workspace_edit(&f, &result);
    std::fs::rename(&old, &new).expect("move");
    std::fs::rename(&old_asset, &new_asset).expect("move asset");
    let mut events: Vec<_> = changed
        .iter()
        .filter(|path| *path != &old && *path != &old_asset)
        .map(|path| (path.as_path(), FileChangeType::CHANGED))
        .collect();
    events.push((&old, FileChangeType::DELETED));
    events.push((&new, FileChangeType::CREATED));
    events.push((&old_asset, FileChangeType::DELETED));
    events.push((&new_asset, FileChangeType::CREATED));
    client.watched(&events);
    client.settle();

    assert!(
        read(&f, "docs/install-agent.md").contains("guides/my%20guides/keys.md#rotate-keys"),
        "{}",
        read(&f, "docs/install-agent.md")
    );
    assert!(
        read(&f, "docs/install-agent.md")
            .contains("[keys-ref]: guides/my%20guides/keys.md#rotate-keys")
    );
    assert!(
        read(&f, "docs/guides/my guides/keys.md").contains("../../install-agent.md#install-agent")
    );
    assert!(read(&f, "docs/quickstart.md").contains("my%20images/playground.png"));
    assert!(changed.contains(&f.path("docs/install-agent.md")));
    assert!(changed.contains(&old));
    assert!(changed.contains(&f.path("docs/quickstart.md")));
    assert_no_errors(&client);
}

#[test]
fn file_move_refuses_conflicting_or_outside_targets() {
    let f = quill();
    let old = f.path("docs/keys.md");
    let conflict = f.path("docs/guides/keys.md");
    let outside = f.path("generated/keys.md");
    std::fs::create_dir_all(conflict.parent().expect("parent")).expect("mkdir");
    std::fs::write(&conflict, "occupied").expect("conflicting file");
    let mut client = Client::start(&f.root());
    client.settle();
    for new in [&conflict, &outside] {
        let result = client.request(
            "workspace/willRenameFiles",
            json!({
                "files": [{
                    "oldUri": uri(&old).as_str(),
                    "newUri": uri(new).as_str()
                }]
            }),
        );
        let value = result.response_result.expect("rename result");
        assert!(
            value["changes"].is_null(),
            "unexpected move edit: {value:#?}"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&conflict).expect("still occupied"),
        "occupied"
    );
}

#[test]
fn renaming_an_explicit_id_updates_links_and_includes() {
    let f = quill();
    let target = f.path("docs/keys.md");
    let linking_text = format!(
        "{}\n@include: keys.md#rotate-keys\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/install-agent.md", &linking_text);
    let mut client = Client::start(&f.root());
    client.settle();
    let original = read(&f, "docs/keys.md");
    let offset = original.find("rotate-keys").expect("id");
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&target).as_str() },
                "position": position(&original, offset),
                "newName": "turn-keys"
            }),
        )
        .response_result
        .expect("rename");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("@id: turn-keys"));
    assert!(read(&f, "docs/install-agent.md").contains("keys.md#turn-keys"));
    assert!(read(&f, "docs/install-agent.md").contains("@include: keys.md#turn-keys"));
    assert_no_errors(&client);
}
