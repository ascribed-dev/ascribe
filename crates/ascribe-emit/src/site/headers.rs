//! `_ascribe/headers.json`: the response headers a build's files need from
//! the host ([`ascribe_resolve::pages::header_rules`]), as a list of rules,
//! each a path and its headers (output-layout contract §1.4). A Markdown
//! version has no head to carry its page's rules, so they're headers, which
//! only the host can send.

use std::collections::BTreeMap;

use ascribe_model::ContentModel;
use ascribe_resolve::ResolvedBuild;
use ascribe_resolve::pages::{header_rules, records};
use serde::Serialize;

/// The version of the file's format.
const VERSION: u32 = 1;

#[derive(Serialize)]
struct List<'a> {
    version: u32,
    headers: Vec<Rule<'a>>,
}

#[derive(Serialize)]
struct Rule<'a> {
    path: &'a str,
    headers: BTreeMap<&'static str, &'a str>,
}

/// The rules, as JSON, sorted by path. A build with no Markdown versions has
/// an empty list, so a reader can tell "none" from an output without the
/// file.
pub(super) fn list(model: &ContentModel, build: &ResolvedBuild) -> String {
    let rules = header_rules(&records(model, &build.pages));
    let list = List {
        version: VERSION,
        headers: rules
            .iter()
            .map(|rule| Rule {
                path: &rule.path,
                headers: rule
                    .headers
                    .iter()
                    .map(|(name, value)| (*name, value.as_str()))
                    .collect(),
            })
            .collect(),
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&list).unwrap_or_default();
    text.push('\n');
    text
}
