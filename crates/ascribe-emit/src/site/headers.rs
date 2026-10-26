//! `_ascribe/headers.json`: the response headers a build's files need from
//! the host ([`ascribe_resolve::pages::header_rules`]), as a list of rules,
//! each a path and its headers (output-layout contract §1.4). A Markdown
//! version has no head to carry its page's rules, so they're headers, which
//! only the host can send. Each help entry's own addresses, its JSON and its
//! Markdown, get `Access-Control-Allow-Origin: *`, so a product on another
//! origin can fetch them, and the JSON's `Link` names the Markdown as its
//! alternate.

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

/// The rules, sorted by path: each Markdown version's, and each of `help`,
/// the paths the help entries are served at as JSON, with the same path in
/// `.md`, where each is served as Markdown.
fn rules(
    model: &ContentModel,
    build: &ResolvedBuild,
    help: &[String],
) -> Vec<(String, BTreeMap<&'static str, String>)> {
    let mut rules: Vec<(String, BTreeMap<&'static str, String>)> =
        header_rules(&records(model, &build.pages))
            .into_iter()
            .map(|rule| (rule.path, rule.headers.into_iter().collect()))
            .collect();
    for path in help {
        let cors = ("Access-Control-Allow-Origin", "*".to_owned());
        let Some(markdown) = path.strip_suffix(".json").map(|p| format!("{p}.md")) else {
            rules.push((path.clone(), BTreeMap::from([cors])));
            continue;
        };
        let alternate = format!("<{markdown}>; rel=\"alternate\"; type=\"text/markdown\"");
        rules.push((
            path.clone(),
            BTreeMap::from([cors.clone(), ("Link", alternate)]),
        ));
        rules.push((markdown, BTreeMap::from([cors])));
    }
    rules.sort_by(|a, b| a.0.cmp(&b.0));
    rules
}

/// The rules, as JSON, sorted by path: each Markdown version's, and each of
/// `help`, the paths the help entries are served at. A build with neither
/// has an empty list, so a reader can tell "none" from an output without the
/// file.
pub(super) fn list(model: &ContentModel, build: &ResolvedBuild, help: &[String]) -> String {
    let rules = rules(model, build, help);
    let list = List {
        version: VERSION,
        headers: rules
            .iter()
            .map(|(path, headers)| Rule {
                path,
                headers: headers.iter().map(|(n, v)| (*n, v.as_str())).collect(),
            })
            .collect(),
    };
    // Serializing strings and maps of strings can't fail.
    let mut text = serde_json::to_string_pretty(&list).unwrap_or_default();
    text.push('\n');
    text
}

/// The same rules in Netlify's `_headers`: each path, then its headers
/// indented below it.
pub(super) fn netlify(model: &ContentModel, build: &ResolvedBuild, help: &[String]) -> String {
    let mut out = String::new();
    for (path, headers) in rules(model, build, help) {
        out.push_str(&path);
        out.push('\n');
        for (name, value) in headers {
            out.push_str(&format!("  {name}: {value}\n"));
        }
    }
    out
}
