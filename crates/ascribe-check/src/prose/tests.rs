#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Mutex;

use super::*;

/// How a stand-in for Vale answers a file's text.
type Answer = Box<dyn Fn(&str) -> Vec<Alert> + Send + Sync>;

/// A stand-in for Vale: it answers with the alerts `answer` makes of each
/// file's text, and keeps the requests it's given.
struct Fake {
    answer: Answer,
    seen: Mutex<Vec<Request>>,
}

impl Fake {
    fn new(answer: impl Fn(&str) -> Vec<Alert> + Send + Sync + 'static) -> Fake {
        Fake {
            answer: Box::new(answer),
            seen: Mutex::new(Vec::new()),
        }
    }

    /// Alerts on every match of `word`, as a rule that replaces it with
    /// `with`.
    fn finding(word: &'static str, with: &'static str) -> Fake {
        Fake::new(move |text| matches(text, word, with))
    }
}

impl Linter for Fake {
    fn lint(&self, request: &Request) -> Result<BTreeMap<String, Vec<Alert>>, ValeError> {
        self.seen.lock().unwrap().push(request.clone());
        Ok(request
            .files
            .iter()
            .map(|(path, text)| (path.clone(), (self.answer)(text)))
            .collect())
    }
}

/// Alerts on every match of `word` in `text`, as Vale reports them: by line,
/// and by character from 1.
fn matches(text: &str, word: &str, with: &str) -> Vec<Alert> {
    let mut out = Vec::new();
    for (n, line) in text.lines().enumerate() {
        for (i, _) in line.match_indices(word) {
            let first = line[..i].chars().count() + 1;
            out.push(Alert {
                check: "Test.Words".to_owned(),
                message: format!("Use '{with}' instead of '{word}'."),
                severity: "warning".to_owned(),
                line: n + 1,
                span: (first, first + word.chars().count() - 1),
                link: "https://example.com/words".to_owned(),
                action: Action {
                    name: "replace".to_owned(),
                    ..Action::default()
                },
                suggestions: vec![with.to_owned()],
            });
        }
    }
    out
}

fn model(extra: &str) -> ContentModel {
    let text = format!(
        "spec = \"0.1\"\n[phrases]\nproduct = \"Acme teh Cloud\"\n[checks.vale]\npreset = \"quiet\"\n{extra}"
    );
    ascribe_model::load_str(&text, FileId::new(0)).unwrap()
}

fn lint_text(model: &ContentModel, text: &str, linter: &dyn Linter) -> Vec<Diagnostic> {
    let dir = tempfile::tempdir().unwrap();
    let page = Page {
        id: FileId::new(1),
        path: "docs/page.md",
        text,
    };
    lint_pages(dir.path(), model, &[page], linter, FILE_TIMEOUT)
}

/// The source text each diagnostic is at.
fn found<'a>(text: &'a str, diagnostics: &[Diagnostic]) -> Vec<&'a str> {
    diagnostics
        .iter()
        .map(|d| &text[d.location.span.range()])
        .collect()
}

#[test]
fn an_alert_lands_on_its_source_text() {
    let text = "---\ntitle: teh\n---\n\n# Heading with teh\n\n- one teh\n  and teh\n\n| a | b |\n|---|---|\n| teh | x |\n";
    let fake = Fake::finding("teh", "the");
    let found_here = lint_text(&model(""), text, &fake);
    let at: Vec<usize> = found_here.iter().map(|d| d.location.span.start()).collect();
    assert_eq!(found(text, &found_here), ["teh"; 4], "{at:?}");
    let d = &found_here[0];
    assert_eq!(d.message, "Test.Words: Use 'the' instead of 'teh'.");
    assert_eq!(d.rule.as_deref(), Some("Test.Words"));
    assert_eq!(d.severity, Severity::Warning);
    assert_eq!(d.next(), Some(Next::Fix));
    assert_eq!(d.fixes[0].edits[0].new_text, "the");
    assert_eq!(d.fixes[0].applicability, Applicability::Unsafe);
}

#[test]
fn nothing_is_reported_on_directives_attributes_frontmatter_or_code() {
    let text = "---\ndescription: teh\n---\n\n@note {type=teh}: The teh note.\n\n@include: teh.md\n\n```teh\nteh\n```\n\nRun `teh` now. ![teh](teh.png){width=teh} [x](teh)\n";
    let fake = Fake::finding("teh", "the");
    let at = lint_text(&model(""), text, &fake);
    // The note's text, the code span (Vale skips code), and the image's
    // alt text are prose; the rest isn't given.
    let lines: Vec<usize> = at
        .iter()
        .map(|d| text[..d.location.span.start()].lines().count())
        .collect();
    assert_eq!(found(text, &at).len(), at.len());
    assert!(lines.iter().all(|&l| l == 5 || l == 13), "{lines:?}");
    let given = &fake.seen.lock().unwrap()[0].files[0].1;
    assert!(!given.contains("type=teh"), "{given}");
    assert!(!given.contains("description"), "{given}");
    assert!(!given.contains("teh.md"), "{given}");
    assert!(!given.contains("```"), "{given}");
    assert!(!given.contains("width"), "{given}");
    assert!(!given.contains("(teh)"), "{given}");
}

#[test]
fn an_alert_in_a_phrase_lands_on_the_key() {
    let text = "Try {product} today.\n";
    let fake = Fake::finding("teh", "the");
    let at = lint_text(&model(""), text, &fake);
    assert_eq!(found(text, &at), ["{product}"]);
    assert_eq!(
        at[0].message,
        "Test.Words: Use 'the' instead of 'teh'. (in the text of phrase `product`)"
    );
    assert!(at[0].fixes.is_empty(), "the fix belongs in ascribe.toml");
    assert_eq!(at[0].next(), Some(Next::Write));
}

#[test]
fn max_level_lowers_alerts() {
    let fake = Fake::finding("teh", "the");
    let at = lint_text(&model("max-level = \"advice\"\n"), "So teh.\n", &fake);
    assert_eq!(at[0].severity, Severity::Advice);
}

#[test]
fn non_ascii_text_maps_by_character() {
    let text = "Héllo wörld — teh café.\n";
    let fake = Fake::finding("teh", "the");
    let at = lint_text(&model(""), text, &fake);
    assert_eq!(found(text, &at), ["teh"]);
}

#[test]
fn vale_that_can_t_run_is_one_advice() {
    let model = model("command = \"ascribe-test-no-such-vale\"\n");
    let at = lint_text(&model, "Some text.\n", &Program);
    assert_eq!(at.len(), 1);
    assert_eq!(at[0].slug, diagnostics::PROSE_NOT_CHECKED);
    assert_eq!(at[0].severity, Severity::Advice);
    assert_eq!(at[0].location.file, FileId::new(0));
    assert!(
        at[0]
            .message
            .contains("`ascribe-test-no-such-vale` couldn't be run"),
        "{}",
        at[0].message
    );
}

#[test]
fn the_advice_can_be_turned_off() {
    let model =
        model("command = \"ascribe-test-no-such-vale\"\n[checks]\nprose-not-checked = \"off\"\n");
    assert!(lint_text(&model, "Some text.\n", &Program).is_empty());
}

#[test]
fn a_page_with_no_prose_isn_t_given() {
    let fake = Fake::finding("teh", "the");
    let at = lint_text(&model(""), "```\nteh\n```\n", &fake);
    assert!(at.is_empty());
    assert!(fake.seen.lock().unwrap().is_empty());
}

#[test]
fn the_range_of_an_alert_counts_characters() {
    let text = "ab\nçd teh\n";
    assert_eq!(range_of(text, 2, (4, 6)), Some((7, 10)));
    assert_eq!(&text[7..10], "teh");
    assert_eq!(range_of(text, 1, (1, 2)), Some((0, 2)));
    assert_eq!(range_of(text, 9, (1, 2)), None);
}

#[test]
fn an_older_vale_s_action_gives_the_replacement() {
    // Vale before 3.21 writes no `Suggestions`; its action says what to do.
    let alert = |name: &str, params: &[&str]| Alert {
        action: Action {
            name: name.to_owned(),
            params: params.iter().map(|p| (*p).to_owned()).collect(),
        },
        ..Alert::default()
    };
    assert_eq!(
        replacements(&alert("edit", &["truncate", " "]), "the the"),
        ["the"]
    );
    assert_eq!(
        replacements(&alert("replace", &["receive"]), "recieve"),
        ["receive"]
    );
    assert_eq!(
        replacements(&alert("edit", &["trim_right", "!"]), "go!!"),
        ["go"]
    );
    assert_eq!(replacements(&alert("remove", &[]), "very"), [""]);
    assert!(replacements(&alert("edit", &["regex", "a", "b"]), "a").is_empty());
    // A newer Vale's own suggestions win.
    let mut newer = alert("edit", &["truncate", " "]);
    newer.suggestions = vec!["The".to_owned()];
    assert_eq!(replacements(&newer, "the the"), ["The"]);
}
