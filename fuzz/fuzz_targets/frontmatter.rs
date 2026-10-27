//! The frontmatter reader on arbitrary text, under the conformance suite's
//! shared model: `ascribe_resolve::index_file` parses the document, finds the
//! frontmatter block, and reads its YAML (SPEC §2.1), and
//! `ascribe_syntax::frontmatter_lines` reads it line by line as the editor
//! does. Neither panics, and every line `frontmatter_lines` returns names a
//! range inside the text.

#![no_main]

use std::sync::LazyLock;

use ascribe_core::{FileId, RelPath};
use ascribe_model::ContentModel;
use ascribe_resolve::index_file;
use ascribe_resolve::slug::GithubSlugger;
use ascribe_syntax::frontmatter_lines;
use libfuzzer_sys::fuzz_target;

static MODEL: LazyLock<ContentModel> = LazyLock::new(|| {
    let text = include_str!("../../tests/conformance/_model/ascribe.toml");
    match ascribe_model::load_str(text, FileId::new(0)) {
        Ok(model) => model,
        Err(issues) => panic!("the shared model doesn't load: {issues:?}"),
    }
});

fuzz_target!(|source: &str| {
    // The full reader: parse, find the frontmatter, read its YAML.
    let path = RelPath::parse("page.md").expect("a literal path");
    let _ = index_file(FileId::new(0), &path, source, &MODEL, &GithubSlugger);

    // The line reader the editor uses on the same text; every span it hands
    // back must be inside the text, so a slice of it can't panic.
    for line in frontmatter_lines(source) {
        let end = line.value_start + line.value.len();
        assert!(
            line.start <= source.len() && line.value_start <= source.len() && end <= source.len(),
            "a frontmatter line names a range outside the text: {line:?} in {} bytes",
            source.len()
        );
        assert!(source.get(line.value_start..end).is_some(), "the value isn't a slice: {line:?}");
    }
});
