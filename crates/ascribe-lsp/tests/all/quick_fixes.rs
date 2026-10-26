//! Scripted quick fixes and source refactorings over a copy of `examples/quill`.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

use crate::support;

use std::path::{Path, PathBuf};

use crate::support::{Client, Fixture, MODEL, Setup, uri};
use ascribe_core::{LineIndex, Span, TextEdit, apply_edits};
use ascribe_lsp::Encoding;
use lsp_types::{FileChangeType, PositionEncodingKind};
use serde_json::{Value, json};

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
        // A drive path, `C:/dir`, follows a third slash.
        let path = match path.strip_prefix('/') {
            Some(drive) if drive.as_bytes().get(1) == Some(&b':') => drive.to_string(),
            _ => path,
        };
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
fn a_diagnostic_carries_its_advice_and_each_fix_its_safety() {
    // What `ascribe check --format json` says that the protocol has no field
    // for, so the extension can report problems to agents in that shape.
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = format!("{}\nInstall {{{{product}}}}.\n", read(&f, "docs/keys.md"));
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();

    let diag = diagnostic(&client, &path, "phrase-double-braces");
    let help = ascribe_check::Registry::global()
        .find("phrase-double-braces")
        .and_then(|entry| entry.fix.clone())
        .expect("the registry's advice");
    assert_eq!(diag["data"]["help"], help);
    assert_eq!(diag["data"]["next"], "fix");
    let fixes = diag["data"]["fixes"].as_array().expect("fixes");
    assert!(!fixes.is_empty());
    for fix in fixes {
        assert!(
            matches!(fix["applicability"].as_str(), Some("safe" | "unsafe")),
            "{fix}"
        );
    }
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
fn quick_fix_uses_the_current_unsaved_buffer() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let on_disk = read(&f, "docs/keys.md");
    let unsaved = format!("{on_disk}\n@variant {{deployment=cloud}}\nCloud text.\n@end\n");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, &unsaved);
    client.settle();
    let diag = diagnostic(&client, &path, "container-colon-missing");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Add a trailing colon")
        .expect("colon action");
    let edits = action["edit"]["changes"][uri(&path).as_str()]
        .as_array()
        .expect("edits");
    let fixed = apply_edit(&unsaved, edits);
    client.replace(&path, 2, &fixed);
    client.settle();
    assert!(fixed.contains("@variant {deployment=cloud}:\nCloud text."));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "container-colon-missing")
    );
    assert_eq!(read(&f, "docs/keys.md"), on_disk);
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
fn phrase_declaration_adds_a_line_after_a_table_header_at_eof() {
    let f = quill();
    let model = read(&f, "ascribe.toml");
    let phrases = model
        .find("\n[phrases]\n")
        .map(|offset| offset + 1)
        .expect("phrases table");
    let after_phrases = &model[phrases..];
    let next_table = after_phrases
        .find("\n[images.attributes]")
        .expect("next table");
    let without_phrases = format!("{}{}", &model[..phrases], &after_phrases[next_table + 1..]);
    write(
        &f,
        "ascribe.toml",
        &format!("{}\n[phrases]", without_phrases.trim_end_matches('\n')),
    );

    let path = f.path("docs/keys.md");
    let text = format!("{}\nA {{phase24-only}} phrase.\n", read(&f, "docs/keys.md"));
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = client
        .diagnostics(&path)
        .into_iter()
        .find(|diag| {
            support::slug(diag) == "phrase-undeclared" && diag.message.contains("phase24-only")
        })
        .map(|diag| serde_json::to_value(diag).expect("serialize diagnostic"))
        .expect("phase24-only phrase diagnostic");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Declare phrase in ascribe.toml")
        .expect("phrase declaration action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);

    let model = read(&f, "ascribe.toml");
    assert!(
        model.ends_with("[phrases]\n\"phase24-only\" = \"\"\n"),
        "{model:?}"
    );
    assert!(!client.diagnostics(&path).iter().any(|d| {
        support::slug(d) == "phrase-undeclared" && d.message.contains("phase24-only")
    }));
    assert_no_errors(&client);
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
    let setup = Setup {
        encodings: Some(vec![PositionEncodingKind::UTF8]),
        ..Setup::default()
    };
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
    let action = result.response_result.expect("code actions");
    let action = action
        .as_array()
        .unwrap()
        .iter()
        .find(|action| action["title"] == "Add a stable @id for this heading")
        .expect("stable id action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut action_client, &changed);
    assert!(read(&f, "docs/keys.md").contains("@id: protect-secrets"));
    assert_no_errors(&action_client);
}

#[test]
fn id_rename_in_a_fragment_updates_links_through_the_including_page() {
    // SPEC §4.2: a link names a fragment's heading through a page that
    // includes it (#90).
    let f = quill();
    let fragment = f.path("docs/_fragments/limits.md");
    let fragment_text = "## Rate limits\n@id: custom\n\nEach key makes 100 requests a minute.\n";
    write(&f, "docs/_fragments/limits.md", fragment_text);
    write(
        &f,
        "docs/limits.md",
        "---\ntype: page\ntitle: Limits\ndescription: Limits.\n---\n\n@include: _fragments/limits.md\n\nSee [above](#custom).\n",
    );
    let source_text = format!(
        "{}\n[Limits](limits.md#custom)\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/install-agent.md", &source_text);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&fragment).as_str() },
                "position": position(fragment_text, fragment_text.find("custom").expect("id") + 1),
                "newName": "limits"
            }),
        )
        .response_result
        .expect("id rename");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/_fragments/limits.md").contains("@id: limits"));
    assert!(read(&f, "docs/limits.md").contains("[above](#limits)"));
    assert!(read(&f, "docs/install-agent.md").contains("[Limits](limits.md#limits)"));
    assert_no_errors(&client);
}

#[test]
fn heading_rename_updates_ids_changed_by_slug_renumbering() {
    let f = quill();
    let target = f.path("docs/keys.md");
    let text = format!(
        "{}\n## Repeated heading\n\nFirst.\n\n## Repeated heading\n\nSecond.\n",
        read(&f, "docs/keys.md")
    );
    let source = format!(
        "{}\n[First](keys.md#repeated-heading)\n[Second](keys.md#repeated-heading-1)\n@include: keys.md#repeated-heading-1\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/keys.md", &text);
    write(&f, "docs/install-agent.md", &source);
    let mut client = Client::start(&f.root());
    client.settle();
    let heading_at = text.find("Repeated heading").expect("first heading");
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&target).as_str() },
                "position": position(&text, heading_at),
                "newName": "Unique heading"
            }),
        )
        .response_result
        .expect("rename heading");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);

    assert!(read(&f, "docs/keys.md").contains("## Unique heading"));
    let source = read(&f, "docs/install-agent.md");
    assert!(source.contains("[First](keys.md#unique-heading)"));
    assert!(source.contains("[Second](keys.md#repeated-heading)"));
    assert!(source.contains("@include: keys.md#repeated-heading"));
    assert_no_errors(&client);
}

#[test]
fn stable_id_action_keeps_setext_headings_intact() {
    let f = quill();
    let target = f.path("docs/keys.md");
    let text = format!(
        "{}\nSetext heading\n--------------\n\nSetext content.\n",
        read(&f, "docs/keys.md")
    );
    let source = format!(
        "{}\n[Setext](keys.md#setext-heading)\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/keys.md", &text);
    write(&f, "docs/install-agent.md", &source);
    let mut client = Client::start(&f.root());
    client.settle();
    let heading_at = text.find("Setext heading").expect("heading");
    let result = client.request(
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri(&target).as_str() },
            "range": {
                "start": position(&text, heading_at),
                "end": position(&text, heading_at + "Setext heading".len())
            },
            "context": { "diagnostics": [] }
        }),
    );
    let response = result.response_result.expect("code actions");
    let action = response
        .as_array()
        .expect("actions")
        .iter()
        .find(|action| action["title"] == "Add a stable @id for this heading")
        .expect("stable id action");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);

    assert!(
        read(&f, "docs/keys.md").contains("Setext heading\n--------------\n@id: setext-heading\n")
    );
    assert_no_errors(&client);
    assert!(changed.contains(&target));
}

#[test]
fn formatting_returns_only_ascribe_fmt_edits() {
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
    write(&f, "docs/keys.md", &formatted);
    notify_changed(&mut client, std::slice::from_ref(&path));
    assert_no_errors(&client);
}

#[test]
fn file_move_updates_incoming_and_outgoing_relative_references() {
    let f = quill();
    let old = f.path("docs/keys.md");
    let new = f.path("docs/guides/my guides/keys.md");
    let old_asset = f.path("docs/playground.png");
    let new_asset = f.path("docs/my images/playground.png");
    let old_hash_asset = f.path("docs/hash#asset.png");
    let new_hash_asset = f.path("docs/my images/hash#asset.png");
    std::fs::create_dir_all(new.parent().expect("parent")).expect("mkdir");
    std::fs::create_dir_all(new_asset.parent().expect("parent")).expect("mkdir");
    std::fs::write(&old_hash_asset, "asset").expect("create asset");
    let model = read(&f, "ascribe.toml").replace(
        "[images.attributes]",
        "asset = \"playground.png\"\n\n[images.attributes]",
    );
    write(&f, "ascribe.toml", &model);
    let linking = format!(
        "{}\n[Keys][keys-ref]\n\n[keys-ref]: keys.md#rotate-keys\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/install-agent.md", &linking);
    let quickstart = format!(
        "{}\n![Phrase asset]({{asset}})\n![Hash asset](hash%23asset.png)\n",
        read(&f, "docs/quickstart.md")
    );
    write(&f, "docs/quickstart.md", &quickstart);
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
                    },
                    {
                        "oldUri": uri(&old_hash_asset).as_str(),
                        "newUri": uri(&new_hash_asset).as_str()
                    }
                ]
            }),
        )
        .response_result
        .expect("will rename");
    let changed = apply_workspace_edit(&f, &result);
    std::fs::rename(&old, &new).expect("move");
    std::fs::rename(&old_asset, &new_asset).expect("move asset");
    std::fs::rename(&old_hash_asset, &new_hash_asset).expect("move hash asset");
    let mut events: Vec<_> = changed
        .iter()
        .filter(|path| *path != &old && *path != &old_asset)
        .map(|path| (path.as_path(), FileChangeType::CHANGED))
        .collect();
    events.push((&old, FileChangeType::DELETED));
    events.push((&new, FileChangeType::CREATED));
    events.push((&old_asset, FileChangeType::DELETED));
    events.push((&new_asset, FileChangeType::CREATED));
    events.push((&old_hash_asset, FileChangeType::DELETED));
    events.push((&new_hash_asset, FileChangeType::CREATED));
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
    assert!(
        read(&f, "docs/quickstart.md").contains("my%20images/{asset}"),
        "{}",
        read(&f, "docs/quickstart.md")
    );
    assert!(read(&f, "docs/quickstart.md").contains("my%20images/hash%23asset.png"));
    assert!(read(&f, "ascribe.toml").contains("asset = \"playground.png\""));
    assert!(changed.contains(&f.path("docs/install-agent.md")));
    assert!(changed.contains(&old));
    assert!(changed.contains(&f.path("docs/quickstart.md")));
    assert_no_errors(&client);
}

#[test]
fn moving_a_renamed_phrase_destination_updates_its_single_declaration() {
    let f = quill();
    let old = f.path("docs/hash#asset.png");
    let new = f.path("docs/my images/renamed#asset.png");
    std::fs::create_dir_all(new.parent().expect("parent")).expect("mkdir");
    std::fs::write(&old, "asset").expect("create asset");
    let model = read(&f, "ascribe.toml").replace(
        "[images.attributes]",
        "hash-asset = \"hash%23asset.png\"\n\n[images.attributes]",
    );
    write(&f, "ascribe.toml", &model);
    let quickstart = format!(
        "{}\n![Hash asset]({{hash-asset}})\n",
        read(&f, "docs/quickstart.md")
    );
    write(&f, "docs/quickstart.md", &quickstart);
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "workspace/willRenameFiles",
            json!({
                "files": [{
                    "oldUri": uri(&old).as_str(),
                    "newUri": uri(&new).as_str()
                }]
            }),
        )
        .response_result
        .expect("will rename");
    let changed = apply_workspace_edit(&f, &result);
    std::fs::rename(&old, &new).expect("move asset");
    let mut events: Vec<_> = changed
        .iter()
        .map(|path| (path.as_path(), FileChangeType::CHANGED))
        .collect();
    events.push((&old, FileChangeType::DELETED));
    events.push((&new, FileChangeType::CREATED));
    client.watched(&events);
    client.settle();

    assert!(read(&f, "docs/quickstart.md").contains("![Hash asset]({hash-asset})"));
    assert!(
        read(&f, "ascribe.toml").contains("hash-asset = \"my%20images/renamed%23asset.png\""),
        "{}",
        read(&f, "ascribe.toml")
    );
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
fn file_move_by_case_alone_changes_the_links_to_it() {
    // On a case-insensitive file system, as macOS's and Windows' are by
    // default, `Keys.md` is there already: it's `keys.md` itself.
    let f = quill();
    let old = f.path("docs/keys.md");
    let new = f.path("docs/Keys.md");
    let mut client = Client::start(&f.root());
    client.settle();
    let result = client
        .request(
            "workspace/willRenameFiles",
            json!({
                "files": [{
                    "oldUri": uri(&old).as_str(),
                    "newUri": uri(&new).as_str()
                }]
            }),
        )
        .response_result
        .expect("will rename");
    let changed = apply_workspace_edit(&f, &result);
    assert!(!changed.is_empty(), "no edits: {result:#?}");
    let linking = read(&f, "docs/install-agent.md");
    assert!(linking.contains("Keys.md"), "{linking}");
    assert!(!linking.contains("(keys.md"), "{linking}");
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

#[test]
fn renaming_a_phrase_backed_fragment_updates_its_declaration() {
    let f = quill();
    let model = read(&f, "ascribe.toml").replace(
        "api = \"https://api.quill.dev/v3/\"",
        "api = \"https://api.quill.dev/v3/\"\nrotate-link = \"keys.md#rotate-keys\"",
    );
    write(&f, "ascribe.toml", &model);
    let path = f.path("docs/keys.md");
    let source_path = f.path("docs/install-agent.md");
    let source = format!(
        "{}\n[Phrase link]({{rotate-link}})\n",
        read(&f, "docs/install-agent.md")
    );
    write(&f, "docs/install-agent.md", &source);
    let mut client = Client::start(&f.root());
    client.settle();
    let original = read(&f, "docs/keys.md");
    let offset = original.find("rotate-keys").expect("id");
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&path).as_str() },
                "position": position(&original, offset),
                "newName": "turn-keys"
            }),
        )
        .response_result
        .expect("rename");
    let changed = apply_workspace_edit(&f, &result);
    notify_changed(&mut client, &changed);

    assert!(read(&f, "docs/keys.md").contains("@id: turn-keys"));
    assert!(read(&f, "ascribe.toml").contains("rotate-link = \"keys.md#turn-keys\""));
    assert!(read(&f, "docs/install-agent.md").contains("[Phrase link]({rotate-link})"));
    assert!(changed.contains(&source_path) || changed.contains(&f.path("ascribe.toml")));
    assert_no_errors(&client);
}

#[test]
fn id_rename_rejects_invalid_and_conflicting_ids() {
    let f = quill();
    let path = f.path("docs/keys.md");
    let text = read(&f, "docs/keys.md");
    let offset = text.find("rotate-keys").expect("id");
    let mut client = Client::start(&f.root());
    client.settle();
    for new_name in ["not an id", "create-key"] {
        let result = client.request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&path).as_str() },
                "position": position(&text, offset),
                "newName": new_name
            }),
        );
        assert!(result.response_result.expect("rename result").is_null());
    }
}

/// Every diagnostic whose kind of next step is `fix` has a quick fix that
/// removes it: one sample of each, fixed by its first action. A diagnostic
/// classified `fix` without a sample here fails the test.
#[test]
fn every_fix_diagnostic_has_a_quick_fix() {
    let widget = "\n[widgets.quill-labspace]\nforms = [\"line\"]\nprimary = \"none\"\nbinding = \"self\"\n\n[widgets.quill-labspace.attributes]\nlab = \"string\"\n";
    // The slug, what's added to a page, and what's added to the model. What's
    // added to a page goes at its end, or, when it doesn't start with a line
    // break, at the start of its frontmatter.
    let samples: &[(&str, &str, &str)] = &[
        (
            "attribute-unquoted-reserved",
            "\n@quill-labspace {lab=Using other images}\n",
            widget,
        ),
        (
            "container-colon-unexpected",
            "\n@steps:\n1. Keep this step.\n",
            "",
        ),
        (
            "container-colon-missing",
            "\n@variant {deployment=cloud}\nThis stays in the cloud build.\n@end\n",
            "",
        ),
        ("binding-blank-line", "\n@note\n\nA bound block.\n", ""),
        (
            "heading-phrase-without-id",
            "\n## Install {product}\n\nText.\n",
            "",
        ),
        (
            "heading-duplicate-without-id",
            "\n## Twice\n\nText.\n\n## Twice\n\nMore.\n",
            "",
        ),
        ("directive-extra-text", "\n@steps foo\n1. One.\n", ""),
        ("phrase-double-braces", "\nInstall {{product}}.\n", ""),
        ("heading-level-skipped", "\n#### Too deep\n\nText.\n", ""),
        (
            "listing-disallowed",
            "listing: unlisted\n",
            "\n[crawlers]\nlines = [\"Disallow: /keys/\"]\n",
        ),
        (
            "listing-loosened",
            "listing: listed\n",
            "\n[listing]\ndefault = \"unquoted\"\n",
        ),
        ("memo-empty", "\n@memo:\n@end\nText.\n", ""),
    ];
    let fixable: Vec<&str> = ascribe_check::Registry::global()
        .entries()
        .filter(|e| e.next == Some(ascribe_check::Next::Fix))
        // Whether an acknowledgement covers nothing takes every build, so
        // the editor never reports one; `ascribe check` carries its fix.
        .filter(|e| e.slug.as_str() != "intended-unused")
        // Its fix writes `ascribe.permalinks`, not the page: `across.rs`
        // applies it and sees the warning go.
        .filter(|e| e.slug.as_str() != "permalinks-list-behind")
        // It's reported in `ascribe.toml`, not a page:
        // `a_repeated_field_is_deleted_from_ascribe_toml` applies its fix.
        .filter(|e| e.slug.as_str() != "frontmatter-field-repeated")
        // It needs a spec, a file of its own:
        // `a_link_by_method_and_path_is_fixed_to_the_id` applies its fix.
        .filter(|e| e.slug.as_str() != "link-operation-by-path")
        // Theirs edit `ascribe.nav`, from the file or from a page:
        // `nav_file.rs` applies them and sees the problem go.
        .filter(|e| {
            !matches!(
                e.slug.as_str(),
                "nav-duplicate" | "nav-page-unplaced" | "nav-section-unplaced"
            )
        })
        // Theirs edit `ascribe.addresses`, which only `ascribe check` reads:
        // `ascribe-check`'s `addresses.rs` applies them, and the extension
        // offers them in the file.
        .filter(|e| {
            !matches!(
                e.slug.as_str(),
                "addresses-entry-live" | "addresses-entry-unpublished" | "addresses-chain"
            )
        })
        .map(|e| e.slug.as_str())
        .collect();
    let sampled: Vec<&str> = samples.iter().map(|(slug, _, _)| *slug).collect();
    assert_eq!(
        sampled, fixable,
        "every diagnostic of kind `fix`, in registry order, needs a sample"
    );
    for (slug, page, model) in samples {
        let f = quill();
        let path = f.path("docs/keys.md");
        write(
            &f,
            "ascribe.toml",
            &format!("{}{model}", read(&f, "ascribe.toml")),
        );
        let text = read(&f, "docs/keys.md");
        let text = match text.strip_prefix("---\n") {
            Some(rest) if !page.starts_with('\n') => format!("---\n{page}{rest}"),
            _ => format!("{text}{page}"),
        };
        write(&f, "docs/keys.md", &text);
        let mut client = Client::start(&f.root());
        client.settle();
        let diag = diagnostic(&client, &path, slug);
        let action = actions(&mut client, &path, diag)
            .into_iter()
            .find(|action| action.get("edit").is_some())
            .unwrap_or_else(|| panic!("{slug}: no quick fix"));
        let changed = apply_workspace_edit(&f, &action["edit"]);
        notify_changed(&mut client, &changed);
        assert!(
            !client
                .diagnostics(&path)
                .iter()
                .any(|d| support::slug(d) == *slug),
            "{slug}: `{}` doesn't remove it",
            action["title"]
        );
    }
}

#[test]
fn a_link_by_method_and_path_is_fixed_to_the_id() {
    let f = quill();
    write(
        &f,
        "ascribe.toml",
        &format!(
            "{}\n[apis.public]\nspec = \"api/openapi.yaml\"\nreference = {{ renderer = \"redoc\", at = \"/api/\" }}\n",
            read(&f, "ascribe.toml")
        ),
    );
    std::fs::create_dir_all(f.path("api")).expect("api");
    write(
        &f,
        "api/openapi.yaml",
        "openapi: 3.1.0\ninfo: { title: Keys, version: \"1\" }\npaths:\n  /keys/{id}:\n    get:\n      operationId: getKey\n",
    );
    let path = f.path("docs/keys.md");
    let text = format!(
        "{}\nFetch it with [Get a key](../api/openapi.yaml#get/keys/{{id}}).\n",
        read(&f, "docs/keys.md")
    );
    write(&f, "docs/keys.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "link-operation-by-path");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action.get("edit").is_some())
        .expect("a quick fix");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert!(read(&f, "docs/keys.md").contains("[Get a key](../api/openapi.yaml#getKey)"));
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "link-operation-by-path")
    );
}

/// The page `body` gives, after its HTML comment's **Rewrite as a memo**,
/// once the server has checked it again; `None` when the comment has no
/// such fix.
fn rewritten_as_memo(body: &str) -> Option<String> {
    let f = quill();
    let path = f.path("docs/plans.md");
    let text = format!("---\ntype: page\ntitle: Plans\ndescription: Plans.\n---\n\n{body}");
    write(&f, "docs/plans.md", &text);
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "html-comment");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action["title"] == "Rewrite as a memo")?;
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    // Nothing links to the new page, which is all that's left.
    assert_eq!(client.codes(&path), ["page-orphan"]);
    let fixed = read(&f, "docs/plans.md");
    Some(fixed[fixed.find("---\n\n").expect("frontmatter") + 5..].to_owned())
}

#[test]
fn a_comment_above_a_block_becomes_a_memo_in_its_place() {
    assert_eq!(
        rewritten_as_memo(
            "<!-- Keep this table in the order of the plans' prices. -->\n\n| Plan | Price |\n|---|---|\n| Team | $10 |\n"
        )
        .as_deref(),
        Some(
            "@memo: Keep this table in the order of the plans' prices.\n| Plan | Price |\n|---|---|\n| Team | $10 |\n"
        )
    );
    // More lines make a container, in its own block quote.
    assert_eq!(
        rewritten_as_memo("> <!-- One,\n>      and two. -->\n> Quoted.\n").as_deref(),
        Some("> @memo:\n> One,\n> and two.\n> @end\n> Quoted.\n")
    );
}

#[test]
fn a_comment_that_ends_a_section_becomes_a_memo_at_its_top() {
    assert_eq!(
        rewritten_as_memo(
            "## Plans\n@id: plans\n\nText.\n\n<!-- Prices are monthly. -->\n\n## Next\n"
        )
        .as_deref(),
        Some("## Plans\n@id: plans\n@memo: Prices are monthly.\n\nText.\n\n## Next\n")
    );
}

#[test]
fn a_comment_that_ends_a_list_item_gets_no_fix() {
    assert_eq!(
        rewritten_as_memo("## Plans\n\n- Team\n\n  <!-- Ask sales. -->\n- Business\n"),
        None
    );
    assert_eq!(
        rewritten_as_memo("Text.\n\n<!-- Before any heading. -->\n"),
        None
    );
}

/// The action that applies a page's safe fixes, asked for as VS Code asks
/// on save, or `None` when the server offers none.
fn fix_all(client: &mut Client, path: &Path) -> Option<Value> {
    let actions = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(path).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "context": { "diagnostics": [], "only": ["source.fixAll"] }
            }),
        )
        .response_result
        .expect("actions");
    let mut found = actions.as_array().cloned().unwrap_or_default();
    assert!(found.len() <= 1, "{found:?}");
    found.pop()
}

/// A page with an empty memo and two links to routes, each with a safe
/// fix, and a misspelled link, whose fix isn't. (An acknowledgement that
/// covers nothing takes every build to tell, so the editor doesn't report
/// one, and has no fix for it.)
const FIX_ME: &str = "---\ntitle: Fix me\n---\n\n@memo:\n@end\n\nSee 😀 [Install](/install-agent/), [Keys](/keys/), and [Gone](keyz.md).\n";

#[test]
fn fix_all_makes_the_edits_check_fix_makes() {
    let f = quill();
    write(&f, "docs/fix-me.md", FIX_ME);
    let path = f.path("docs/fix-me.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, FIX_ME);
    client.settle();

    let action = fix_all(&mut client, &path).expect("the action");
    assert_eq!(action["title"], "Apply the safe fixes in this page");
    assert_eq!(action["kind"], "source.fixAll.ascribe");
    let codes: Vec<String> = client
        .diagnostics(&path)
        .iter()
        .map(support::slug)
        .collect();
    assert_eq!(
        action["diagnostics"].as_array().unwrap().len(),
        3,
        "{codes:?}"
    );
    let changes = action["edit"]["changes"].as_object().unwrap();
    assert_eq!(changes.len(), 1);
    let edits = changes[uri(&path).as_str()].as_array().unwrap();
    let fixed = apply_edit(FIX_ME, edits);

    // `ascribe check --editor-build --fix`, as the command takes them.
    let project = ascribe_check::Project::load(&f.path("ascribe.toml")).expect("a project");
    let mut diagnostics = ascribe_check::diagnose_editor_build(&project, None).diagnostics;
    diagnostics.sort_by_key(|d| d.severity == ascribe_check::Severity::Advice);
    let files = ascribe_check::fix::safe_fixes(&project, &f.path("ascribe.toml"), &diagnostics);
    let expected = files
        .iter()
        .find(|file| file.path.ends_with("fix-me.md"))
        .expect("check --fix changes the page");
    assert_eq!(fixed, expected.text);
    assert_eq!(
        fixed,
        "---\ntitle: Fix me\n---\n\nSee 😀 [Install](/install-agent.md), [Keys](/keys.md), and [Gone](keyz.md).\n"
    );
    // Asked for again, it has nothing left to do.
    client.replace(&path, 2, &fixed);
    client.settle();
    assert!(fix_all(&mut client, &path).is_none());
}

#[test]
fn fix_all_is_offered_only_with_a_safe_fix_and_only_when_asked() {
    let f = quill();
    let path = f.path("docs/fix-me.md");
    let only_unsafe = "---\ntitle: Fix me\n---\n\nSee [Gone](keyz.md).\n";
    write(&f, "docs/fix-me.md", only_unsafe);
    let mut client = Client::start(&f.root());
    client.open(&path, 1, only_unsafe);
    client.settle();
    assert!(fix_all(&mut client, &path).is_none());

    client.replace(&path, 2, FIX_ME);
    client.settle();
    assert!(fix_all(&mut client, &path).is_some());
    // The lightbulb's quick fixes don't include it.
    let quick = client
        .request(
            "textDocument/codeAction",
            json!({
                "textDocument": { "uri": uri(&path).as_str() },
                "range": { "start": { "line": 0, "character": 0 }, "end": { "line": 0, "character": 0 } },
                "context": { "diagnostics": [], "only": ["quickfix"] }
            }),
        )
        .response_result
        .expect("actions");
    assert!(
        !quick
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["kind"] == "source.fixAll.ascribe")
    );
}

/// `ascribe/projectEdit` of `kind`, asked through the project's
/// `ascribe.toml`.
fn project_edit(client: &mut Client, f: &Fixture, kind: &str) -> Value {
    client
        .request(
            "ascribe/projectEdit",
            json!({
                "textDocument": { "uri": uri(&f.path("ascribe.toml")).as_str() },
                "kind": kind
            }),
        )
        .response_result
        .expect("the project's edit")
}

/// The edit to `path` in a project edit, and the version it's for.
fn document_edit<'a>(answer: &'a Value, path: &Path) -> Option<(&'a Value, &'a [Value])> {
    answer["edit"]["documentChanges"]
        .as_array()
        .expect("documentChanges")
        .iter()
        .find(|change| change["textDocument"]["uri"] == uri(path).as_str())
        .map(|change| {
            (
                &change["textDocument"]["version"],
                change["edits"].as_array().expect("edits").as_slice(),
            )
        })
}

#[test]
fn every_safe_fix_in_the_project_is_one_edit_over_unsaved_text() {
    let f = quill();
    // On disk, closed.
    write(&f, "docs/fix-me.md", FIX_ME);
    // Open, its fixes in its unsaved text alone.
    let open = f.path("docs/fix-me-too.md");
    write(
        &f,
        "docs/fix-me-too.md",
        "---\ntitle: Fix me too\n---\n\nFine.\n",
    );
    let unsaved = FIX_ME.replace("Fix me", "Fix me too");
    let mut client = Client::start(&f.root());
    client.open(&open, 3, &unsaved);
    // Asked before the edit's diagnostics are published: it waits for them.
    let answer = project_edit(&mut client, &f, "fixAll");

    // check --fix's edits to the page on disk, as the editor's build has it.
    let project = ascribe_check::Project::load(&f.path("ascribe.toml")).expect("a project");
    let mut diagnostics = ascribe_check::diagnose_editor_build(&project, None).diagnostics;
    diagnostics.sort_by_key(|d| d.severity == ascribe_check::Severity::Advice);
    let files = ascribe_check::fix::safe_fixes(&project, &f.path("ascribe.toml"), &diagnostics);
    let fixed = "---\ntitle: Fix me\n---\n\nSee 😀 [Install](/install-agent.md), [Keys](/keys.md), and [Gone](keyz.md).\n";
    assert_eq!(
        files
            .iter()
            .find(|file| file.path.ends_with("fix-me.md"))
            .map(|file| file.text.as_str()),
        Some(fixed)
    );

    let (version, edits) = document_edit(&answer, &f.path("docs/fix-me.md")).expect("closed");
    assert_eq!(*version, Value::Null);
    assert_eq!(apply_edit(FIX_ME, edits), fixed);
    let (version, edits) = document_edit(&answer, &open).expect("open");
    assert_eq!(*version, json!(3));
    assert_eq!(
        apply_edit(&unsaved, edits),
        fixed.replace("Fix me", "Fix me too")
    );
    // Every file check --fix changes on disk, and the unsaved one, with
    // three problems fixed in each.
    let changed = answer["edit"]["documentChanges"].as_array().unwrap().len();
    assert_eq!(answer["files"], json!(changed));
    assert_eq!(changed, files.len() + 1);
    assert_eq!(answer["fixes"], json!(3 * changed), "{answer}");
    assert!(answer["edit"].get("changes").is_none());
}

#[test]
fn every_page_formatted_is_one_edit_ascribe_fmt_makes() {
    let f = quill();
    let keys = f.path("docs/keys.md");
    let original = read(&f, "docs/keys.md");
    let on_disk = format!("{original}\n@note{{ type = tip }}: Be careful.\n");
    write(&f, "docs/keys.md", &on_disk);
    let quickstart = f.path("docs/quickstart.md");
    let unsaved = format!(
        "{}\n@note{{ type = tip }}: Unsaved.\n",
        read(&f, "docs/quickstart.md")
    );
    let mut client = Client::start(&f.root());
    client.open(&quickstart, 7, &unsaved);
    let answer = project_edit(&mut client, &f, "format");

    assert_eq!(answer["files"], json!(2), "{answer}");
    assert_eq!(answer["fixes"], Value::Null);
    let (version, edits) = document_edit(&answer, &keys).expect("closed");
    assert_eq!(*version, Value::Null);
    assert_eq!(
        apply_edit(&on_disk, edits),
        format!("{original}\n@note {{type=tip}}: Be careful.\n")
    );
    let (version, edits) = document_edit(&answer, &quickstart).expect("open");
    assert_eq!(*version, json!(7));
    assert!(apply_edit(&unsaved, edits).ends_with("\n@note {type=tip}: Unsaved.\n"));

    // Formatted, there's nothing left to do.
    client.replace(&quickstart, 8, &apply_edit(&unsaved, edits));
    write(
        &f,
        "docs/keys.md",
        &format!("{original}\n@note {{type=tip}}: Be careful.\n"),
    );
    notify_changed(&mut client, std::slice::from_ref(&keys));
    let answer = project_edit(&mut client, &f, "format");
    assert_eq!(answer["files"], json!(0), "{answer}");
}

#[test]
fn a_project_edit_through_a_file_of_no_project_says_so() {
    let f = quill();
    let mut client = Client::start(&f.root());
    let answer = client
        .request(
            "ascribe/projectEdit",
            json!({ "textDocument": { "uri": "file:///elsewhere/page.md" }, "kind": "format" }),
        )
        .response_result
        .expect("an answer");
    assert_eq!(
        answer,
        json!({ "error": "This file isn't one of the project's." })
    );
}

#[test]
fn a_repeated_field_is_deleted_from_ascribe_toml() {
    let model = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n\
                 [frontmatter]\ndescription = \"string\"\n\n\
                 [types.guide.frontmatter]\ndescription = \"string\"\nsteps = \"number\"\n";
    let page = "---\ntype: guide\ntitle: Setup\ndescription: How.\nsteps: 2\n---\n\nText.\n";
    let f = Fixture::new(model, &[("docs/setup.md", page)]);
    let path = f.path("ascribe.toml");
    let mut client = Client::start(&f.root());
    client.settle();
    let diag = diagnostic(&client, &path, "frontmatter-field-repeated");
    let action = actions(&mut client, &path, diag)
        .into_iter()
        .find(|action| action.get("edit").is_some())
        .expect("a quick fix");
    assert_eq!(action["title"], "Delete the type's `description`");
    let changed = apply_workspace_edit(&f, &action["edit"]);
    notify_changed(&mut client, &changed);
    assert_eq!(
        read(&f, "ascribe.toml"),
        model.replace(
            "[types.guide.frontmatter]\ndescription = \"string\"\n",
            "[types.guide.frontmatter]\n"
        )
    );
    assert!(
        !client
            .diagnostics(&path)
            .iter()
            .any(|d| support::slug(d) == "frontmatter-field-repeated")
    );
}

/// A page whose memo says it's generated, with a link to a route, whose fix
/// is safe, and a blank line under a directive, which has a quick fix of its
/// own.
const GENERATED: &str = "---\ntitle: Generated\n---\n\n@memo: Generated from keys.toml by scripts/keys.ts; edit the keys, then run `node scripts/keys.ts` to rewrite it.\nSee [Keys](/keys/).\n\n@note\n\nA bound block.\n";

#[test]
fn a_generated_page_names_its_memo_and_offers_no_edit() {
    let f = quill();
    write(&f, "docs/generated.md", GENERATED);
    let path = f.path("docs/generated.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, GENERATED);
    client.settle();

    let page = uri(&path);
    for slug in ["link-route", "binding-blank-line"] {
        let diag = diagnostic(&client, &path, slug);
        let related = &diag["relatedInformation"][0];
        assert_eq!(related["location"]["uri"], page.as_str(), "{diag}");
        assert_eq!(related["location"]["range"]["start"]["line"], 4, "{diag}");
        assert!(
            related["message"]
                .as_str()
                .unwrap()
                .ends_with("then run `node scripts/keys.ts` to rewrite it."),
            "{diag}"
        );
        assert!(
            diag["data"]["fixes"].as_array().unwrap().is_empty(),
            "{diag}"
        );
        let offered = actions(&mut client, &path, diag);
        assert!(
            offered.iter().all(|action| {
                action["edit"]["changes"].get(page.as_str()).is_none()
                    && action["edit"]["documentChanges"].is_null()
            }),
            "{offered:?}"
        );
    }
    assert!(fix_all(&mut client, &path).is_none());
}

#[test]
fn moving_a_page_records_the_move_in_the_addresses_file_when_there_is_one() {
    let rename = |f: &Fixture| {
        let mut client = Client::start(&f.root());
        client.settle();
        let result = client
            .request(
                "workspace/willRenameFiles",
                json!({
                    "files": [{
                        "oldUri": uri(&f.path("docs/keys.md")).as_str(),
                        "newUri": uri(&f.path("docs/api-keys.md")).as_str()
                    }]
                }),
            )
            .response_result
            .expect("will rename");
        apply_workspace_edit(f, &result)
    };
    // A permalink moved to a heading on the page follows it.
    let f = quill();
    write(
        &f,
        "ascribe.addresses",
        "version = 1\n\n[moved]\n\"go:old-rotate\" = \"keys.md#rotate-keys\"\n\n[retired]\n",
    );
    let changed = rename(&f);
    assert!(changed.contains(&f.path("ascribe.addresses")));
    assert!(
        read(&f, "ascribe.addresses").contains(
            "[moved]\n\"go:old-rotate\" = \"api-keys.md#rotate-keys\"\n\"keys.md\" = \"api-keys.md\"\n"
        ),
        "{}",
        read(&f, "ascribe.addresses")
    );
    // Without the file, a move records nothing.
    let f = quill();
    let changed = rename(&f);
    assert!(!changed.contains(&f.path("ascribe.addresses")));
    assert!(!f.path("ascribe.addresses").exists());
}

#[test]
fn renaming_a_page_s_id_records_it_in_the_addresses_file_when_there_is_one() {
    // A permalink moved to the heading follows its id.
    let f = quill();
    write(
        &f,
        "ascribe.addresses",
        "version = 1\n\n[moved]\n\"go:old-rotate\" = \"keys.md#rotate-keys\"\n\n[retired]\n",
    );
    let page = f.path("docs/keys.md");
    let mut client = Client::start(&f.root());
    client.open(&page, 1, &read(&f, "docs/keys.md"));
    client.settle();
    let result = client
        .request(
            "textDocument/rename",
            json!({
                "textDocument": { "uri": uri(&page).as_str() },
                "position": { "line": 12, "character": 6 },
                "newName": "rotate-a-key",
            }),
        )
        .response_result
        .expect("an edit");
    let changed = apply_workspace_edit(&f, &result);
    assert!(changed.contains(&f.path("ascribe.addresses")), "{result}");
    assert!(
        read(&f, "ascribe.addresses")
            .contains("[moved]\n\"go:old-rotate\" = \"keys.md#rotate-a-key\"\n\"keys.md#rotate-keys\" = \"keys.md#rotate-a-key\"\n"),
        "{}",
        read(&f, "ascribe.addresses")
    );
}
