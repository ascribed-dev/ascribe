//! Semantic tokens: the legend, what each construct is colored as, and
//! positions in both encodings.

#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod support;

use lsp_types::{PositionEncodingKind, SemanticToken};
use serde_json::json;
use support::{Client, Fixture, Setup, uri};

const MODEL_WITH_WIDGET: &str = "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[phrases]\nproduct = \"Quill\"\n\n[images.attributes]\nwidth = \"number?\"\n\n[widgets.my-callout]\nforms = [\"line\"]\nprimary = \"text\"\nbinding = \"self\"\n\n[dimensions.pm]\nvalues = [\"npm\", \"pnpm\"]\n";

/// A decoded token: line, column, length, type name, modifier names.
#[derive(Debug, PartialEq, Eq)]
struct Tok {
    line: u32,
    col: u32,
    len: u32,
    ty: String,
    mods: Vec<String>,
    text: String,
}

fn fetch(
    client: &mut Client,
    path: &std::path::Path,
    text: &str,
    range: Option<(u32, u32, u32, u32)>,
) -> Vec<Tok> {
    let legend =
        client.initialize_result["capabilities"]["semanticTokensProvider"]["legend"].clone();
    let types: Vec<String> = serde_json::from_value(legend["tokenTypes"].clone()).unwrap();
    let modifiers: Vec<String> = serde_json::from_value(legend["tokenModifiers"].clone()).unwrap();
    let response = match range {
        None => client.request(
            "textDocument/semanticTokens/full",
            json!({ "textDocument": { "uri": uri(path).as_str() } }),
        ),
        Some((a, b, c, d)) => client.request(
            "textDocument/semanticTokens/range",
            json!({
                "textDocument": { "uri": uri(path).as_str() },
                "range": { "start": { "line": a, "character": b }, "end": { "line": c, "character": d } },
            }),
        ),
    };
    let value = response.response_result.expect("a result");
    let data: Vec<u32> = serde_json::from_value(value["data"].clone()).expect("data");
    assert_eq!(data.len() % 5, 0);
    let lines: Vec<&str> = text.split('\n').collect();
    let (mut line, mut col) = (0u32, 0u32);
    data.chunks(5)
        .map(|c| {
            let t = SemanticToken {
                delta_line: c[0],
                delta_start: c[1],
                length: c[2],
                token_type: c[3],
                token_modifiers_bitset: c[4],
            };
            line += t.delta_line;
            col = if t.delta_line == 0 {
                col + t.delta_start
            } else {
                t.delta_start
            };
            // The text, for readable assertions (UTF-16 columns).
            let units: Vec<u16> = lines[line as usize].encode_utf16().collect();
            let text = String::from_utf16_lossy(&units[col as usize..(col + t.length) as usize]);
            Tok {
                line,
                col,
                len: t.length,
                ty: types[t.token_type as usize].clone(),
                mods: (0..modifiers.len())
                    .filter(|i| t.token_modifiers_bitset & (1 << i) != 0)
                    .map(|i| modifiers[i].clone())
                    .collect(),
                text,
            }
        })
        .collect()
}

fn summary(tokens: &[Tok]) -> Vec<(String, &str)> {
    tokens
        .iter()
        .map(|t| (t.ty.clone(), t.text.as_str()))
        .collect()
}

fn pairs<'a>(items: &[(&'a str, &'a str)]) -> Vec<(String, &'a str)> {
    items.iter().map(|(a, b)| ((*a).to_owned(), *b)).collect()
}

#[test]
fn every_kind_of_token_is_marked() {
    let page = "---\ntitle: T\n---\n# H\n\n.A {product} title\n@note {type=tip, bogus=1}:\nUse {product} and {nope}.\n@end\n\n@my-callout: Careful.\n\n@available: cloud, self-managed preview 3.3\n\n![alt](a.png){width=600, height=2}\n";
    let f = Fixture::new(MODEL_WITH_WIDGET, &[("docs/t.md", page)]);
    let path = f.path("docs/t.md");
    let mut client = Client::start(&f.root());
    let tokens = fetch(&mut client, &path, page, None);
    assert_eq!(
        summary(&tokens),
        pairs(&[
            ("ascribeTitle", ".A "),
            ("ascribePhrase", "{product}"),
            ("ascribeTitle", " title"),
            ("ascribeDirective", "@note"),
            ("ascribeAttributeKey", "type"),
            ("ascribeAttributeValue", "tip"),
            ("ascribeAttributeKey", "bogus"),
            ("ascribeAttributeValue", "1"),
            ("ascribeColon", ":"),
            ("ascribePhrase", "{product}"),
            ("ascribePhraseUndeclared", "{nope}"),
            ("ascribeEnd", "@end"),
            ("ascribeWidget", "@my-callout"),
            ("ascribeColon", ":"),
            ("ascribeDirective", "@available"),
            ("ascribeColon", ":"),
            ("ascribeAvailability", "cloud, self-managed preview 3.3"),
            ("ascribeAttributeKey", "width"),
            ("ascribeAttributeValue", "600"),
            ("ascribeAttributeKey", "height"),
            ("ascribeAttributeValue", "2"),
        ])
    );
    // Only the keys the schema doesn't declare are `unknown`.
    let unknown: Vec<&str> = tokens
        .iter()
        .filter(|t| t.mods == ["unknown"])
        .map(|t| t.text.as_str())
        .collect();
    assert_eq!(unknown, ["bogus", "height"]);
}

#[test]
fn a_range_request_gives_only_the_tokens_in_it() {
    let page = "---\ntitle: T\n---\n@note {type=tip}:\nUse {product}.\n@end\n\n@my-callout: Hi.\n";
    let f = Fixture::new(MODEL_WITH_WIDGET, &[("docs/t.md", page)]);
    let path = f.path("docs/t.md");
    let mut client = Client::start(&f.root());
    let all = fetch(&mut client, &path, page, None);
    let some = fetch(&mut client, &path, page, Some((7, 0, 8, 0)));
    assert!(all.len() > some.len());
    assert_eq!(
        summary(&some),
        pairs(&[("ascribeWidget", "@my-callout"), ("ascribeColon", ":")])
    );
}

#[test]
fn token_columns_follow_the_negotiated_encoding() {
    let page = "---\ntitle: T\n---\n😀 {product} and {nope}\n";
    let f = Fixture::new(MODEL_WITH_WIDGET, &[("docs/t.md", page)]);
    let path = f.path("docs/t.md");
    let mut utf16 = Client::start(&f.root());
    let tokens = fetch(&mut utf16, &path, page, None);
    assert_eq!(tokens[0].col, 3); // 😀 is two units
    assert_eq!(tokens[0].len, 9);
    let mut utf8 = Client::start_with(
        &f.root(),
        Setup {
            encodings: Some(vec![PositionEncodingKind::UTF8]),
            ..Setup::default()
        },
    );
    let response = utf8.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri(&path).as_str() } }),
    );
    let data: Vec<u32> =
        serde_json::from_value(response.response_result.expect("ok")["data"].clone()).unwrap();
    assert_eq!(data[1], 5); // 😀 is four bytes and a space
    assert_eq!(data[2], 9);
}

#[test]
fn tokens_follow_an_edit_of_an_open_document() {
    let page = "---\ntitle: T\n---\nUse {nope}.\n";
    let f = Fixture::new(MODEL_WITH_WIDGET, &[("docs/t.md", page)]);
    let path = f.path("docs/t.md");
    let mut client = Client::start(&f.root());
    client.open(&path, 1, page);
    let before = fetch(&mut client, &path, page, None);
    assert_eq!(
        summary(&before),
        pairs(&[("ascribePhraseUndeclared", "{nope}")])
    );
    let edited = "---\ntitle: T\n---\nUse {product}.\n";
    client.replace(&path, 2, edited);
    let after = fetch(&mut client, &path, edited, None);
    assert_eq!(summary(&after), pairs(&[("ascribePhrase", "{product}")]));
}

#[test]
fn a_file_outside_the_project_has_no_tokens() {
    let f = Fixture::new(MODEL_WITH_WIDGET, &[("README.md", "# Readme\n")]);
    let mut client = Client::start(&f.root());
    let response = client.request(
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri(&f.path("README.md")).as_str() } }),
    );
    assert_eq!(response.response_result.expect("ok"), json!(null));
}
