//! Every span in the tree covers exactly its source text, across every input
//! we have: the CommonMark examples (alone, and with directives around them),
//! the example projects, and every conformance case input.

mod support;

use support::{check_tree, markdown_files, repo_root};
use tessera_syntax::{ParseOptions, parse};

fn options() -> ParseOptions {
    let mut schemas = tessera_core::builtin_schemas();
    // A project widget with a text primary, so continuation lines get read.
    let mut widget = tessera_core::Builtin::Note.schema();
    widget.name = "quill-demo".into();
    widget.origin = tessera_core::Origin::Widget;
    schemas.push(widget);
    ParseOptions::new(schemas)
}

#[track_caller]
fn assert_exact(name: &str, source: &str) {
    let doc = parse(source, &options());
    let problems = check_tree(source, &doc);
    assert!(
        problems.is_empty(),
        "{name}: spans don't match the source:\n  {}\ninput: {:?}",
        problems.join("\n  "),
        source.chars().take(300).collect::<String>()
    );
}

#[test]
fn commonmark_examples() {
    let examples = tessera_commonmark_suite::load_bundled_examples().expect("spec.json loads");
    assert_eq!(examples.len(), 652);
    for ex in &examples {
        assert_exact(&format!("commonmark example {}", ex.example), &ex.markdown);
    }
}

#[test]
fn commonmark_examples_beside_directive_lines() {
    let examples = tessera_commonmark_suite::load_bundled_examples().expect("spec.json loads");
    for ex in &examples {
        let name = format!("commonmark example {} with directives", ex.example);
        let md = &ex.markdown;
        assert_exact(&name, &format!("@note {{type=tip}}: A tip.\n{md}\n@end\n"));
        assert_exact(&name, &format!("@id: x\n\n{md}@steps\n{md}"));
        assert_exact(
            &name,
            &format!("> @note: quoted\n> {}", md.replace('\n', "\n> ")),
        );
    }
}

#[test]
fn directives_in_every_container() {
    for source in [
        "- item\n  @note: in the item\n  continued\n- next\n",
        "1. Install\n  @note: two spaces, so outside the item\n",
        "> @note {type=caution}: in a quote\n> more text\n> @end\n",
        "- a\n\n  @include: x.md\n\n  @end\n",
        "> - @note: deep\n>   lazy\n",
        "@note:\n\n@end\n",
        "@note {type=tip}:   \t\n@end   \n",
        "@note: é → 😀 text\nnext é\n",
        "@note\r\n@id: x\r\n@note: a\r\nb\r\n",
        "@note: a\rb\r@end\r",
        "  @note: three spaces\n   @end\n",
        "\t@note: tab\n",
        "@id: abc  \n",
        "@include: my file.md\n",
        "@available: cloud, self-managed preview 3.4\nA paragraph.\n",
        "@quill-demo {a=b}: text\ncontinues\n\n@end\n",
        "@note {type=tip: broken\n",
        "@note hello: text\n@end foo\n@end: x\n@end {a=b}\n",
        "@warning: nope\n\nprose\n@availible\n",
        "---\ntitle: x\n---\n@note: after frontmatter\n",
        "---\n---\n",
    ] {
        assert_exact("container case", source);
    }
}

#[test]
fn appendix_b_and_examples() {
    let root = repo_root();
    let mut count = 0;
    for dir in [
        root.join("tests/conformance"),
        root.join("examples"),
        root.join("project-docs"),
        root.join("SPEC.md"),
    ] {
        let files = if dir.is_file() {
            vec![dir.clone()]
        } else {
            markdown_files(&dir)
        };
        for path in files {
            let source = std::fs::read_to_string(&path).expect("readable");
            assert_exact(&path.display().to_string(), &source);
            count += 1;
        }
    }
    assert!(count > 3, "found only {count} files");
}
