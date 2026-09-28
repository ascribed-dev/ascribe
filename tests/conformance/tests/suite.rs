//! Checks on the conformance suite as a whole, and on `examples/quill`.
//!
//! The suite's cases run in the implementation phases; these tests run now.
//! They keep the suite complete (every SPEC §8.2 row has a case, and every
//! provisional case names an open question) and keep the Quill example project
//! identical to the SPEC's Appendix B page and to its conformance case.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tessera_conformance::{DiagnosticsRegistry, Suite, discover};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cases() -> Vec<tessera_conformance::Case> {
    let suite = Suite::bundled();
    discover(&suite.cases_dir(), &suite.shared_model_path())
        .unwrap()
        .into_iter()
        .map(|d| {
            let id = d.id;
            d.case
                .unwrap_or_else(|e| panic!("case {id} doesn't load: {e}"))
        })
        .collect()
}

#[test]
fn every_section_8_2_row_has_a_case() {
    let suite = Suite::bundled();
    let registry = DiagnosticsRegistry::load(&suite.diagnostics_path()).unwrap();
    let mut expected = BTreeSet::new();
    for case in cases() {
        let file = case.expect.diagnostics.iter().flatten();
        let pages = case
            .expect
            .builds
            .values()
            .flat_map(|b| b.diagnostics.iter().flatten());
        expected.extend(file.chain(pages).map(|d| d.slug.clone()));
    }
    let missing: Vec<_> = registry
        .entries
        .iter()
        .filter(|e| e.row.is_some() && !expected.contains(&e.slug))
        .map(|e| e.slug.as_str())
        .collect();
    assert!(
        missing.is_empty(),
        "SPEC §8.2 rows with no case that expects them: {missing:?}"
    );
}

#[test]
fn every_case_says_what_it_checks() {
    for case in cases() {
        assert!(
            case.expect.description.is_some(),
            "case {} has no description",
            case.id
        );
        assert!(
            !case.expect.spec.is_empty(),
            "case {} lists no SPEC sections",
            case.id
        );
    }
}

/// The ids of the questions in `project-docs/questions.md` whose status is open.
fn open_questions() -> BTreeSet<String> {
    let text = std::fs::read_to_string(repo().join("project-docs/questions.md")).unwrap();
    let mut open = BTreeSet::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("### ")
            && let Some((id, _)) = rest.split_once(':')
            && id.starts_with('Q')
        {
            current = Some(id.to_owned());
        } else if let Some(id) = &current
            && line.starts_with("- **Status:** open")
        {
            open.insert(id.clone());
        }
    }
    open
}

#[test]
fn provisional_cases_name_open_questions() {
    let open = open_questions();
    for case in cases() {
        for q in &case.expect.questions {
            assert!(
                open.contains(q),
                "case {} depends on {q}, which is not an open question in questions.md; \
                 when a question is resolved, update the case and remove its `provisional` tag",
                case.id
            );
        }
    }
}

fn read_tree(root: &Path) -> Vec<(String, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, out);
            } else {
                let rel = path.strip_prefix(root).unwrap();
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((rel, std::fs::read(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out
}

/// The Appendix B page in SPEC.md: the `markdown` block after its heading.
fn appendix_b_page() -> String {
    let spec = std::fs::read_to_string(repo().join("SPEC.md")).unwrap();
    let appendix = spec
        .split("## Appendix B. Complete example")
        .nth(1)
        .unwrap();
    let block = appendix.split("````markdown\n").nth(1).unwrap();
    block.split("````").next().unwrap().to_owned()
}

#[test]
fn quill_example_page_is_the_spec_appendix_b_page() {
    let page = appendix_b_page();
    let example =
        std::fs::read_to_string(repo().join("examples/quill/docs/install-agent.md")).unwrap();
    assert_eq!(
        example, page,
        "examples/quill/docs/install-agent.md drifted from SPEC Appendix B"
    );
    let sample =
        std::fs::read_to_string(repo().join("tests/conformance/cases/samples/appendix-b/input.md"))
            .unwrap();
    assert_eq!(
        sample, page,
        "the appendix-b sample case drifted from SPEC Appendix B"
    );
}

#[test]
fn quill_project_case_matches_the_example() {
    let example = read_tree(&repo().join("examples/quill/docs"));
    let case = read_tree(&repo().join("tests/conformance/cases/projects/quill/files"));
    assert_eq!(
        example.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        case.iter().map(|(p, _)| p).collect::<Vec<_>>(),
        "the case and the example have different files"
    );
    for ((path, a), (_, b)) in example.iter().zip(&case) {
        assert!(
            a == b,
            "{path} differs between examples/quill and the project case"
        );
    }
    let model = std::fs::read_to_string(repo().join("examples/quill/tessera.toml")).unwrap();
    let case_model =
        std::fs::read_to_string(repo().join("tests/conformance/cases/projects/quill/tessera.toml"))
            .unwrap();
    assert_eq!(
        model.replace("content-root = \"docs\"", "content-root = \"files\""),
        case_model,
        "the case's tessera.toml differs from the example's beyond the content root"
    );
}

/// Every local path a Quill page includes, links to, or embeds.
fn references(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("@include")
            && let Some((_, path)) = rest.split_once(':')
        {
            out.push(path.trim().to_owned());
        }
        let mut rest = line;
        while let Some(i) = rest.find("](") {
            let after = &rest[i + 2..];
            if let Some(end) = after.find(')') {
                out.push(after[..end].to_owned());
                rest = &after[end..];
            } else {
                break;
            }
        }
    }
    out.into_iter()
        .filter(|p| !p.is_empty() && !p.contains("://") && !p.starts_with('#') && !p.contains('{'))
        .collect()
}

#[test]
fn quill_example_contains_every_file_its_pages_reference() {
    let docs = repo().join("examples/quill/docs");
    let mut checked = 0;
    for (path, bytes) in read_tree(&docs) {
        if !path.ends_with(".md") {
            continue;
        }
        let text = String::from_utf8(bytes).unwrap();
        let dir = Path::new(&path).parent().unwrap().to_owned();
        for reference in references(&text) {
            let file = reference.split('#').next().unwrap();
            let target = docs.join(&dir).join(file);
            assert!(
                target.is_file(),
                "{path} references {reference}, which doesn't exist under examples/quill/docs"
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 6,
        "expected to check the Appendix B references, saw {checked}"
    );
}

#[test]
fn quill_example_link_ids_exist() {
    // The page links to `quickstart.md#try-in-browser` and `keys.md#rotate-keys`.
    let docs = repo().join("examples/quill/docs");
    let quickstart = std::fs::read_to_string(docs.join("quickstart.md")).unwrap();
    let keys = std::fs::read_to_string(docs.join("keys.md")).unwrap();
    assert!(quickstart.contains("@id: try-in-browser"));
    assert!(keys.contains("@id: rotate-keys"));
}
