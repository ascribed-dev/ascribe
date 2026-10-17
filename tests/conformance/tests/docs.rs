//! The diagnostics reference, `docs/content/checks/diagnostics.md` and its
//! pages in `docs/content/checks/diagnostics/`, includes fragments
//! generated from the diagnostics registry, in
//! `docs/content/_generated/diagnostics-*.md`. This test renders them and
//! fails when one is out of date; run it with `ASCRIBE_BLESS=1` to rewrite
//! them.
//!
//! It also checks the docs' phrases whose values come from another file, such
//! as the version, against that file, and that the `ascribe.toml` reference
//! and its contract share their text only through fragments.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;

use ascribe_conformance::registry::{Next, placeholders};
use ascribe_conformance::{DiagnosticsRegistry, Entry, Level, Severity, Suite};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs";

/// The evidence an agent prompt can carry: what an entry's `evidence` may
/// name, from the checks' own lists: what an issue carries, and what a
/// prompt gathers.
fn evidence_names() -> Vec<&'static str> {
    ascribe_check::EVIDENCE
        .iter()
        .map(|(name, _, _)| *name)
        .chain(ascribe_check::GATHERED_EVIDENCE.iter().copied())
        .collect()
}

/// The tables of `[checks]` that aren't a check's: the tools' settings. No
/// diagnostic can have their names.
const CHECK_TOOLS: &[&str] = &["vale", "links"];

/// No check ships without its next step: every diagnostic has a
/// kind of next step, a `fix` paragraph, an entry in the reference, a
/// section of the directive or `ascribe.toml` reference to send its reader on
/// to, and the list of what its prompt carries. A `review` diagnostic can be
/// acknowledged and set in `[checks]`, so it's configurable. That every `fix`
/// diagnostic has a fix in the code is tested with the quick fixes, in
/// `crates/ascribe-lsp/tests/all/quick_fixes.rs`.
#[test]
fn every_diagnostic_has_its_next_step() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let pages = Pages::load();
    let reference = render(&registry, &pages)
        .into_iter()
        .map(|(_, text)| text)
        .collect::<String>();
    let mut wrong = Vec::new();
    for e in &registry.entries {
        if CHECK_TOOLS.contains(&e.slug.as_str()) {
            wrong.push(format!("{}: `[checks.{}]` is for a tool", e.code, e.slug));
        }
        match &e.reference {
            None => wrong.push(format!("{}: no `reference`", e.code)),
            Some(section) if !pages.sections.contains_key(section) => wrong.push(format!(
                "{}: `reference` is `{section}`, which isn't a heading of {}",
                e.code,
                REFERENCES
                    .map(|(name, _)| format!("{name}.md"))
                    .join(" or ")
            )),
            Some(_) => {}
        }
        if e.fix.as_deref().is_none_or(str::is_empty) {
            wrong.push(format!("{}: no `fix`", e.code));
        }
        if !reference.contains(&format!("### {} `{}`", e.code, e.slug)) {
            wrong.push(format!("{}: no entry in the reference", e.code));
        }
        let review = e.next == Some(Next::Review);
        match (&e.place, review) {
            (None, true) => wrong.push(format!(
                "{}: a `review` diagnostic says where it's acknowledged, with `place`",
                e.code
            )),
            (Some(_), false) => wrong.push(format!(
                "{}: only a `review` diagnostic has a `place`",
                e.code
            )),
            (Some(place), true) if place_words(place).is_none() => wrong.push(format!(
                "{}: `place` is `page`, `block`, or `entry`, not `{place}`",
                e.code
            )),
            _ => {}
        }
        match e.next {
            None => wrong.push(format!("{}: no `next`", e.code)),
            Some(Next::Review) if !e.configurable => {
                wrong.push(format!(
                    "{}: a `review` diagnostic must be configurable",
                    e.code
                ));
            }
            Some(_) => {}
        }
        match &e.evidence {
            None => wrong.push(format!("{}: no `evidence` (it may be empty)", e.code)),
            Some(list) => {
                for name in list {
                    if !evidence_names().contains(&name.as_str()) {
                        wrong.push(format!("{}: unknown evidence `{name}`", e.code));
                    }
                }
                let values = std::iter::once(&e.message)
                    .chain(e.messages.values())
                    .any(|m| placeholders(m).is_ok_and(|p| p.contains(&"values")));
                if list.iter().any(|n| n == "allowed-values") && !values {
                    wrong.push(format!(
                        "{}: `allowed-values` needs a `{{values}}` placeholder",
                        e.code
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn the_diagnostics_reference_is_current() {
    let registry = DiagnosticsRegistry::load(&Suite::bundled().diagnostics_path()).unwrap();
    let fragments = render(&registry, &Pages::load());
    check_fragments("diagnostics-", &fragments);
    for (name, _) in &fragments {
        let page = match DIAGNOSTICS_PAGES
            .iter()
            .find(|(_, fragment)| fragment == name)
        {
            Some((page, _)) => format!("checks/diagnostics/{page}.md"),
            None => "checks/diagnostics.md".to_owned(),
        };
        let depth = "../".repeat(page.matches('/').count());
        assert!(
            read(&format!("docs/content/{page}"))
                .contains(&format!("@include: {depth}_generated/{name}\n")),
            "docs/content/{page} doesn't include _generated/{name}"
        );
    }
}

/// The pages of the diagnostics reference, in `docs/content/checks/diagnostics/`,
/// each with the fragment it lists the diagnostics of. The reference's own page has
/// the index.
const DIAGNOSTICS_PAGES: [(&str, &str); 4] = [
    ("directives", "diagnostics-directives.md"),
    ("pages", "diagnostics-pages.md"),
    ("content-model", "diagnostics-content-model.md"),
    ("content-checks", "diagnostics-content-checks.md"),
];

/// The first construct of the source-file diagnostics listed on the `pages`
/// page; those before it are on the `directives` page.
const FIRST_ON_PAGES: &str = "Links";

/// Each phrase in `docs/ascribe.toml` whose value is also in another file
/// has the value that file has. The `version` phrase is `[releases.ascribe]`'s
/// current line, so the line is what's compared.
#[test]
fn the_docs_phrases_match_their_sources() {
    let text = read("docs/ascribe.toml");
    let docs: toml::Table = text.parse().unwrap();
    assert!(
        docs["phrases"]["version"].get("line").is_some(),
        "the phrase `version` in docs/ascribe.toml is a line, `{{ line = \"ascribe\" }}`"
    );
    let model = ascribe_model::load_str(&text, ascribe_core::FileId::new(0)).unwrap();
    let phrase = |key: &str| {
        model
            .phrases
            .iter()
            .find(|p| p.key == key)
            .map(|p| p.value.clone())
            .unwrap()
    };
    let mut wrong = Vec::new();
    let mut compare = |key: &str, file: &str, value: &str| {
        if phrase(key) != value {
            wrong.push(format!(
                "the phrase `{key}` is \"{}\" in docs/ascribe.toml, but \"{value}\" in {file}",
                phrase(key)
            ));
        }
    };

    // The version: the workspace's, and every package's.
    let cargo: toml::Table = read("Cargo.toml").parse().unwrap();
    let version = cargo["workspace"]["package"]["version"].as_str().unwrap();
    compare("version", "Cargo.toml", version);
    let mut packages: Vec<String> = std::fs::read_dir(repo().join("packages"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| {
            repo()
                .join("packages")
                .join(name)
                .join("package.json")
                .is_file()
        })
        .collect();
    packages.sort();
    for name in packages {
        let file = format!("packages/{name}/package.json");
        let manifest = package(&file);
        // A private package that has no version, as `@ascribed/archive`,
        // which only the binary carries, isn't released.
        if manifest["private"].as_bool() == Some(true) && manifest.get("version").is_none() {
            continue;
        }
        compare("version", &file, manifest["version"].as_str().unwrap());
    }

    // The least Node.js and VS Code versions, as their ranges' lower bounds.
    let cli = package("packages/cli/package.json");
    let node = cli["engines"]["node"].as_str().unwrap();
    compare("node", "packages/cli/package.json", &lower_bound(node));
    let vscode = package("packages/vscode/package.json");
    let editor = vscode["engines"]["vscode"].as_str().unwrap();
    compare(
        "vscode",
        "packages/vscode/package.json",
        &lower_bound(editor),
    );

    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A page, from the repository's root, and its parts: the pages in the folder
/// of its name, as `contracts/content-model.md` has `contracts/content-model/`,
/// or, for a folder's `index.md`, the folder's other pages.
fn with_parts(page: &str) -> Vec<String> {
    let folder = page
        .strip_suffix("/index.md")
        .unwrap_or_else(|| page.trim_end_matches(".md"));
    let mut more: Vec<String> = std::fs::read_dir(repo().join(folder))
        .map(|dir| {
            dir.map(|e| format!("{folder}/{}", e.unwrap().file_name().to_string_lossy()))
                .filter(|f| f.ends_with(".md") && f != page)
                .collect()
        })
        .unwrap_or_default();
    more.sort();
    std::iter::once(page.to_owned()).chain(more).collect()
}

/// The `ascribe.toml` reference and its contract, each a page and its parts,
/// take what they say alike
/// from the fragments in `docs/content/_fragments/content-model/`, so no
/// paragraph, list, table, or example is written in both. Headings, rules,
/// and the includes themselves are the pages' shape, and alike on purpose.
#[test]
fn the_content_model_pages_share_text_only_through_fragments() {
    // The blocks between blank lines, keeping a code block whole.
    let blocks = |path: &str| -> Vec<String> {
        let mut blocks = vec![String::new()];
        let mut fenced = false;
        for line in read(path).lines() {
            fenced ^= line.trim_start().starts_with("```");
            if line.trim().is_empty() && !fenced {
                blocks.push(String::new());
            } else if let Some(block) = blocks.last_mut() {
                block.push_str(line);
                block.push('\n');
            }
        }
        blocks.retain(|block| {
            !(block.is_empty()
                || block == "---\n"
                || block.starts_with('#')
                || block.starts_with("@include: "))
        });
        blocks
    };
    let pages = with_parts;
    let contract: Vec<String> = pages("docs/content/contracts/content-model.md")
        .iter()
        .flat_map(|page| blocks(page))
        .collect();
    let twice: Vec<String> = pages("docs/content/content-model/index.md")
        .iter()
        .flat_map(|page| blocks(page))
        .filter(|block| contract.contains(block))
        .collect();
    assert!(
        twice.is_empty(),
        "move what both pages say into a fragment in docs/content/_fragments/content-model/, \
         and include it from each:\n\n{}",
        twice.join("\n\n")
    );
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap()
}

fn package(path: &str) -> serde_json::Value {
    serde_json::from_str(&read(path)).unwrap()
}

/// A version range's lower bound as the docs write it: `>=24` is `24`, and
/// `^1.138.0` is `1.138`.
fn lower_bound(range: &str) -> String {
    let version = range.trim_start_matches(['^', '~', '>', '=']).trim();
    version.strip_suffix(".0").unwrap_or(version).to_owned()
}

/// Compares each fragment with its file in `docs/content/_generated/`, or,
/// with `ASCRIBE_BLESS`, writes it. A file there whose name starts with
/// `prefix` and that isn't one of `fragments` is left over from something
/// that no longer exists: it fails the test, and blessing removes it.
fn check_fragments(prefix: &str, fragments: &[(String, String)]) {
    let dir = repo().join("docs/content/_generated");
    let bless = std::env::var_os("ASCRIBE_BLESS").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    let mut stale = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.starts_with(prefix) && !fragments.iter().any(|(n, _)| *n == name) {
            if bless {
                std::fs::remove_file(dir.join(&name)).unwrap();
            } else {
                stale.push(name);
            }
        }
    }
    for (name, text) in fragments {
        let path = dir.join(name);
        if bless {
            std::fs::write(&path, text).unwrap();
            continue;
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        if current != *text {
            stale.push(name.clone());
        }
    }
    assert!(
        stale.is_empty(),
        "out of date in docs/content/_generated/: {}. Run `{BLESS}`",
        stale.join(", ")
    );
}

/// The `ascribe.toml` reference's table of the crawlers Ascribe knows, by
/// purpose, `docs/content/_generated/crawlers.md`, from the data file the
/// binary carries.
#[test]
fn the_crawlers_table_is_current() {
    let mut text = String::from(
        "@memo: Generated from crates/ascribe-model/data/crawlers.toml by \
         tests/conformance/tests/docs.rs; edit the data, then run \
         `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.\n\
         | Purpose | Token | Operator | What the operator says of `robots.txt` | Read |\n\
         |---|---|---|---|---|\n",
    );
    for purpose in ascribe_model::Purpose::ALL {
        for c in ascribe_model::known_crawlers()
            .iter()
            .filter(|c| c.purpose == Some(purpose))
        {
            let robots = match c.robots.as_str() {
                "follows" => "Follows it",
                _ => "May not follow it",
            };
            let note = c.note.as_deref().map_or(String::new(), |n| format!(" {n}"));
            writeln!(
                text,
                "| `{}` | `{}` | {} | {robots}.{note} | [{}]({}) |",
                purpose.name(),
                c.token,
                c.operator,
                c.read,
                c.source
            )
            .unwrap();
        }
    }
    check_fragments("crawlers", &[("crawlers.md".to_owned(), text)]);
}

/// The groups loader rules are listed in, as they're titled in the
/// diagnostics reference.
const RULE_GROUPS: &[(&str, &str)] = &[
    ("file", "The file"),
    ("project", "`[project]`"),
    ("sources", "`[sources]`"),
    ("content-types", "Content types, fields, and attributes"),
    (
        "names",
        "Dimensions, names, lifecycle states, notes, and features",
    ),
    ("releases", "`[releases]`"),
    ("phrases", "Phrases and the glossary"),
    ("widgets", "Widgets"),
    ("consumer", "The consumer, builds, and the editor"),
    ("permalinks", "`[permalinks]`"),
    ("crawlers", "`[crawlers]`"),
    ("listing", "`[listing]`"),
    ("checks", "`[checks]`"),
];

/// The areas content checks are listed in, as they're titled in the
/// diagnostics reference.
const AREAS: &[(&str, &str)] = &[
    ("pages", "Pages"),
    ("project", "Across the project"),
    ("outputs", "Outputs"),
    ("prose", "Prose, through Vale"),
    ("links", "External links"),
    ("site", "The published site"),
    ("examples", "Tested examples"),
];

/// What each fragment starts with: a memo saying what generates it, and
/// how. It touches the fragment's first block, or, where that's a heading,
/// which a memo can't be about, it's at the top of the first section.
const HEADER: &str = "@memo: Generated from tests/conformance/diagnostics.toml by \
     tests/conformance/tests/docs.rs; edit the registry, then run \
     `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.\n";

/// The fragments, by file name: an index of every diagnostic, the
/// source-file diagnostics in two (directives, and links, pages, and
/// sources), the content model's, and the content checks. The index links to each entry on the page
/// that includes its fragment (SPEC §4.2).
fn render(registry: &DiagnosticsRegistry, pages: &Pages) -> Vec<(String, String)> {
    let active: Vec<&Entry> = registry.entries.iter().collect();

    // Source-file diagnostics, grouped by the construct their SPEC row names,
    // in the order each construct first appears.
    let mut constructs: Vec<(&str, Vec<(&Entry, &str)>)> = Vec::new();
    for entry in active.iter().filter(|e| !is_model(e) && e.area.is_none()) {
        let row = entry.row.as_deref().unwrap_or_default();
        let (construct, condition) = row.split_once(" | ").unwrap_or((row, ""));
        match constructs.iter_mut().find(|(c, _)| *c == construct) {
            Some((_, entries)) => entries.push((entry, condition)),
            None => constructs.push((construct, vec![(entry, condition)])),
        }
    }
    let split = constructs
        .iter()
        .position(|(construct, _)| *construct == FIRST_ON_PAGES)
        .unwrap_or_else(|| panic!("no construct {FIRST_ON_PAGES}: change FIRST_ON_PAGES"));
    let mut directives = String::new();
    let mut source_pages = String::new();
    let mut page_of: BTreeMap<&str, &str> = BTreeMap::new();
    for (i, (construct, entries)) in constructs.into_iter().enumerate() {
        let (out, page) = if i < split {
            (&mut directives, "directives")
        } else {
            (&mut source_pages, "pages")
        };
        let _ = write!(out, "\n## {construct}\n");
        for (entry, condition) in entries {
            page_of.insert(&entry.code, page);
            write_entry(out, entry, Some(condition), pages);
        }
    }

    let mut content_model = String::new();
    for (group, title) in RULE_GROUPS {
        let _ = write!(content_model, "\n## {title}\n");
        for entry in active
            .iter()
            .filter(|e| is_model(e) && model_group(e) == *group)
        {
            page_of.insert(&entry.code, "content-model");
            write_entry(&mut content_model, entry, None, pages);
        }
    }

    let mut content_checks = String::new();
    for (area, title) in AREAS {
        let _ = write!(content_checks, "\n## {title}\n");
        for entry in active.iter().filter(|e| e.area.as_deref() == Some(*area)) {
            page_of.insert(&entry.code, "content-checks");
            write_entry(&mut content_checks, entry, entry.when.as_deref(), pages);
        }
    }
    let unlisted: Vec<&str> = active
        .iter()
        .filter_map(|e| e.area.as_deref())
        .filter(|a| !AREAS.iter().any(|(area, _)| area == a))
        .collect();
    assert!(unlisted.is_empty(), "areas not in AREAS: {unlisted:?}");

    let all = index(active.iter().copied(), &page_of);
    vec![
        ("diagnostics-index.md".to_owned(), all),
        (
            "diagnostics-directives.md".to_owned(),
            with_memo(&directives),
        ),
        ("diagnostics-pages.md".to_owned(), with_memo(&source_pages)),
        (
            "diagnostics-content-model.md".to_owned(),
            with_memo(&content_model),
        ),
        (
            "diagnostics-content-checks.md".to_owned(),
            with_memo(&content_checks),
        ),
    ]
}

/// A fragment that starts with a heading, with [`HEADER`] at the top of its
/// first section: a memo can't be about a heading.
fn with_memo(body: &str) -> String {
    let body = body.trim_start_matches('\n');
    match body.split_once('\n') {
        Some((heading, rest)) => format!("{heading}\n{HEADER}{rest}"),
        None => format!("{HEADER}{body}"),
    }
}

/// The index: a table of the diagnostics, in code order, each linked to its
/// entry on its page of the reference, whose anchor is its code.
fn index<'a>(entries: impl Iterator<Item = &'a Entry>, page_of: &BTreeMap<&str, &str>) -> String {
    let mut out =
        format!("{HEADER}| Code | Name | Severity | Level | Next step |\n|---|---|---|---|---|\n");
    for entry in entries {
        let _ = writeln!(
            out,
            "| [{code}](../checks/diagnostics/{page}.md#{anchor}) | `{slug}` | {severity} | {level} | {next} |",
            code = entry.code,
            page = page_of[entry.code.as_str()],
            anchor = entry.code,
            slug = entry.slug,
            severity = severity(entry.severity),
            level = level(entry.level),
            next = next(entry),
        );
    }
    out
}

fn is_model(entry: &Entry) -> bool {
    entry.slug.starts_with("model-")
}

/// The group a content-model diagnostic is listed in. The one-role rule for
/// names is a SPEC §8.2 row rather than a loader rule, so it has no `group`;
/// it goes with the other rules about names.
fn model_group(entry: &Entry) -> &str {
    entry.group.as_deref().unwrap_or("names")
}

fn write_entry(out: &mut String, entry: &Entry, condition: Option<&str>, pages: &Pages) {
    let section = match pages.spec.get(&entry.spec) {
        Some(anchor) => format!(
            "[SPEC §{}]({{repo}}/blob/main/SPEC.md#{anchor})",
            entry.spec
        ),
        None => format!("SPEC §{}", entry.spec),
    };
    let configurable = if entry.configurable {
        " · configurable in `[checks]`"
    } else {
        ""
    };
    let acknowledged = entry
        .place
        .as_deref()
        .and_then(place_words)
        .map(|words| format!(" · acknowledged {words}"))
        .unwrap_or_default();
    let _ = write!(
        out,
        "\n### {} `{}`\n@id {{permalink=true}}: {}\n\n{} · {} level · next step: {}{configurable}{acknowledged} · {section}\n\n",
        entry.code,
        entry.slug,
        entry.code,
        severity(entry.severity),
        level(entry.level).to_lowercase(),
        next(entry),
    );
    if let Some(condition) = condition {
        let _ = write!(
            out,
            "**When:** {}.\n\n",
            escape_braces(&sentence(condition))
        );
    }
    let _ = write!(
        out,
        "**Message:** {}\n\n",
        escape_braces(&message(&entry.message))
    );
    let _ = writeln!(
        out,
        "**Fix:** {}",
        escape_braces(&rebase_links(entry.fix.as_deref().unwrap_or_default()))
    );
    let section = entry
        .reference
        .as_ref()
        .and_then(|r| Some((r, pages.sections.get(r)?)));
    if let Some((target, (heading, page))) = section {
        let _ = write!(
            out,
            "\n**Read more:** [{}](../{target}) in the {page}.\n",
            escape_braces(heading)
        );
    }
}

/// The reference pages a diagnostic's entry can send its reader on to, and
/// what the entry calls each: a page, from the content root, and its parts,
/// as `content-model/checks.md`.
const REFERENCES: [(&str, &str); 4] = [
    ("writing/directives.md", "directive reference"),
    ("writing/memo-files.md", "memo files reference"),
    ("content-model/index.md", "`ascribe.toml` reference"),
    ("navigation/index.md", "navigation reference"),
];

/// The files of the reference pages, relative to `docs/content/`, each with
/// what an entry calls it.
fn reference_files() -> Vec<(String, &'static str)> {
    let mut files = Vec::new();
    for (first, page) in REFERENCES {
        for file in with_parts(&format!("docs/content/{first}")) {
            let file = file.trim_start_matches("docs/content/").to_owned();
            files.push((file, page));
        }
    }
    files
}

/// What an entry links to: SPEC.md's numbered sections, and the reference
/// pages' headings.
struct Pages {
    /// SPEC.md's section anchors, by number.
    spec: BTreeMap<String, String>,
    /// Each heading of the reference pages, by `<file>#<id>`: its text,
    /// without a section number, and the page's name.
    sections: BTreeMap<String, (String, &'static str)>,
}

impl Pages {
    fn load() -> Pages {
        let mut sections = BTreeMap::new();
        for (file, page) in reference_files() {
            let text = expanded(&format!("docs/content/{file}"));
            for (id, heading) in headings(&text) {
                sections.insert(format!("{file}#{id}"), (heading, page));
            }
        }
        Pages {
            spec: spec_anchors(&read("SPEC.md")),
            sections,
        }
    }
}

/// A page's text with each `@include` of a whole file, outside fenced code,
/// replaced by that file, so the headings a fragment brings in are the
/// page's.
fn expanded(path: &str) -> String {
    let dir = std::path::Path::new(path).parent().unwrap();
    let mut out = String::new();
    let mut fenced = false;
    for line in read(path).lines() {
        let trimmed = line.trim_start();
        fenced ^= trimmed.starts_with("```") || trimmed.starts_with("~~~");
        match line.strip_prefix("@include: ") {
            Some(target) if !fenced && !target.contains('#') => {
                let target = dir.join(target.trim());
                out.push_str(&expanded(&target.to_string_lossy().replace('\\', "/")));
            }
            _ => {
                out.push_str(line);
                out.push('\n');
            }
        }
    }
    out
}

/// A page's headings below its title, outside fenced code, as their ids and
/// their text without a section number. A heading's id is the `@id` on the
/// line below it, or else its slug.
fn headings(page: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = page.lines().collect();
    let mut out = Vec::new();
    let mut fence: Option<String> = None;
    for (i, line) in lines.iter().enumerate() {
        let marker: String = line
            .chars()
            .take_while(|c| matches!(c, '`' | '~'))
            .collect();
        if marker.len() >= 3 {
            match &fence {
                None => fence = Some(marker),
                Some(open) if line.trim_end() == marker && marker.starts_with(open.as_str()) => {
                    fence = None;
                }
                Some(_) => {}
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }
        let hashes = line.chars().take_while(|c| *c == '#').count();
        let Some(text) = line.get(hashes..).and_then(|t| t.strip_prefix(' ')) else {
            continue;
        };
        if hashes < 2 {
            continue;
        }
        let id = lines
            .get(i + 1)
            .and_then(|next| next.strip_prefix("@id"))
            .and_then(|rest| rest.rsplit_once(": "))
            .map_or_else(|| slug(text), |(_, id)| id.trim().to_owned());
        let unnumbered = match text.split_once(' ') {
            Some((number, rest)) if number.chars().all(|c| c.is_ascii_digit() || c == '.') => rest,
            _ => text,
        };
        out.push((id, unnumbered.to_owned()));
    }
    out
}

/// Where a `review` diagnostic's problem is acknowledged, for its entry in
/// the reference.
fn place_words(place: &str) -> Option<&'static str> {
    match place {
        "page" => Some("in the page's `intended` frontmatter"),
        "block" => Some("with `@intended` above the block, or under the heading for a heading"),
        "entry" => Some("in `[[intended]]`"),
        _ => None,
    }
}

fn severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "Error",
        Severity::Warning => "Warning",
        Severity::Advice => "Advice",
    }
}

/// An entry's kind of next step, as the reference names it.
fn next(entry: &Entry) -> String {
    entry.next.map(|n| n.to_string()).unwrap_or_default()
}

fn level(level: Level) -> &'static str {
    match level {
        Level::File => "File",
        Level::Page => "Page",
    }
}

/// A condition from the spec's table, as a sentence: its first letter
/// capitalized, and the table's "(page level)" notes left out, since the
/// entry's level says so.
fn sentence(condition: &str) -> String {
    let text = condition
        .replace(" (page level, per build)", "")
        .replace(" (page level)", "")
        .replace(
            ", including ids from included content (page level)",
            ", including ids from included content",
        );
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => text,
    }
}

/// A message template as it reads in the reference: a template's literal
/// braces (`{{`, `}}`) are single braces.
fn message(template: &str) -> String {
    template.replace("{{", "{").replace("}}", "}")
}

/// Text whose relative links, written from the content root as the
/// registry's are, resolve from `_generated/` instead, where the fragment is.
/// A destination inside a code span is an example, not a link, and stays as
/// written.
fn rebase_links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut fence = 0;
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if c == '`' {
            let run = rest.len() - rest.trim_start_matches('`').len();
            fence = code_span(fence, run);
            out.push_str(&rest[..run]);
            rest = &rest[run..];
            continue;
        }
        if fence == 0
            && let Some(target) = rest.strip_prefix("](")
        {
            out.push_str("](");
            let dest = target.split(')').next().unwrap_or_default();
            if !(dest.starts_with('#') || dest.starts_with('{') || dest.contains("://")) {
                out.push_str("../");
            }
            rest = target;
            continue;
        }
        out.push(c);
        rest = &rest[c.len_utf8()..];
    }
    out
}

#[test]
fn rebase_links_leaves_a_link_in_a_code_span_as_written() {
    assert_eq!(
        rebase_links(
            "Write `![Alt](image.png){width=600}`, as [the guide](writing/images.md) says."
        ),
        "Write `![Alt](image.png){width=600}`, as [the guide](../writing/images.md) says."
    );
    assert_eq!(
        rebase_links("``a `](x.md)` b`` then [c](y.md)"),
        "``a `](x.md)` b`` then [c](../y.md)"
    );
}

/// Whether text is inside a code span after a run of `run` backticks: the
/// length of the run that opened the span, or 0 outside one. A span closes
/// only on a run as long as the one that opened it.
fn code_span(fence: usize, run: usize) -> usize {
    match fence {
        0 => run,
        open if open == run => 0,
        open => open,
    }
}

/// Text with a backslash before each `{` and `<` outside code spans, so a
/// placeholder such as `{value}` or `<version>` reads as itself, and is never
/// taken for one of the docs' phrases or for HTML.
fn escape_braces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut fence = 0;
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '`' {
            let mut run = 1;
            while chars.peek() == Some(&'`') {
                chars.next();
                run += 1;
            }
            fence = code_span(fence, run);
            out.extend(std::iter::repeat_n('`', run));
            continue;
        }
        if matches!(c, '{' | '<') && fence == 0 {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// SPEC.md's numbered section headings, by number: `3.3` to its anchor.
fn spec_anchors(spec: &str) -> BTreeMap<String, String> {
    let mut anchors = BTreeMap::new();
    for line in spec.lines() {
        let Some(text) = line
            .strip_prefix("## ")
            .or_else(|| line.strip_prefix("### "))
        else {
            continue;
        };
        let Some((number, _)) = text.split_once(' ') else {
            continue;
        };
        let number = number.trim_end_matches('.');
        if !number.is_empty() && number.chars().all(|c| c.is_ascii_digit() || c == '.') {
            anchors.insert(number.to_owned(), slug(text));
        }
    }
    anchors
}

/// A heading's anchor, as GitHub computes it: lowercase, with every character
/// that isn't a letter, digit, space, hyphen, or underscore removed, and
/// spaces turned into hyphens.
fn slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}
