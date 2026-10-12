//! The committed set, with no network: each template is a page `ascribe
//! check` has nothing to say about, in a project that declares its type, and
//! one `ascribe fmt` leaves as it is; its body has no level-1 heading, since
//! its title is its frontmatter's, and no block quote, since what the template
//! said about itself is gone (project-docs/page-templates/README.md, decisions
//! 21 and 22). That the copies are what the pinned release gives is
//! `scripts/templates/convert.test.ts`, which needs their repository.

#![allow(clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};

use ascribe_check::{Project, check_all_builds};
use ascribe_core::{FileId, RelPath};
use ascribe_syntax::{Block, BlockKind, parse};
use ascribe_templates::FILES;

/// A template of `set.toml`.
#[derive(serde::Deserialize)]
struct Template {
    name: String,
    template: String,
    guide: String,
}

#[derive(serde::Deserialize)]
struct Set {
    template: Vec<Template>,
}

fn text(name: &str) -> &'static str {
    FILES
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("FILES has no {name}"))
        .text
}

fn set() -> Set {
    toml::from_str(text("set.toml")).expect("set.toml parses")
}

/// A project whose one type is the template's, as `ascribe templates eject`
/// will declare it, with the template copied to a page of that type.
fn project(name: &str, page: &str) -> Project {
    let model = format!(
        "spec = \"0.1\"\n\n[project]\ncontent-root = \"docs\"\n\n[types.{name}]\nfiles = [\"{name}/**\"]\n\n[types.{name}.frontmatter]\ntitle = \"string\"\ndescription = \"string\"\n"
    );
    let path = RelPath::parse(&format!("{name}/page.md")).expect("a relative path");
    Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        ascribe_model::load_str(&model, FileId::new(0)).expect("the model loads"),
        model,
        Project::from_sources([(path, page.to_owned())]),
    )
}

#[test]
fn files_lists_every_file_in_the_set_and_set_toml_names_each_template_s_files() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("set");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .expect("set/ is there")
        .map(|e| {
            e.expect("an entry")
                .file_name()
                .into_string()
                .expect("UTF-8")
        })
        .collect();
    on_disk.sort();
    let mut listed: Vec<String> = FILES.iter().map(|f| f.name.to_owned()).collect();
    listed.sort();
    assert_eq!(listed, on_disk, "FILES and set/ differ");

    let mut named = vec!["set.toml".to_owned()];
    for template in set().template {
        named.push(template.template);
        named.push(template.guide);
    }
    named.sort();
    assert_eq!(named, on_disk, "set.toml and set/ differ");
}

#[test]
fn every_template_checks_clean_in_a_project_that_declares_its_type() {
    for template in set().template {
        let page = text(&template.template);
        let project = project(&template.name, page);
        let found: Vec<_> = check_all_builds(&project)
            .into_iter()
            .map(|d| format!("{}: {}", d.slug, d.message))
            .collect();
        assert!(
            found.is_empty(),
            "{} has diagnostics: {found:#?}",
            template.template
        );
    }
}

#[test]
fn every_template_is_in_canonical_form() {
    for template in set().template {
        let page = text(&template.template);
        let project = project(&template.name, page);
        let model = project.model();
        let edits = ascribe_fmt::format(page, &ascribe_fmt::options_from_model(model), model);
        assert!(
            edits.is_empty(),
            "ascribe fmt would change {}: {edits:#?}",
            template.template
        );
    }
}

#[test]
fn no_template_has_a_level_one_heading_or_a_block_quote() {
    fn walk(blocks: &[Block], name: &str) {
        for block in blocks {
            match &block.kind {
                BlockKind::Heading(h) => {
                    assert!(h.level > 1, "{name} has a level-1 heading");
                }
                BlockKind::BlockQuote(_) => panic!("{name} has a block quote"),
                BlockKind::List(list) => {
                    for item in &list.items {
                        walk(&item.children, name);
                    }
                }
                _ => {}
            }
        }
    }
    for template in set().template {
        let doc = parse(text(&template.template), &Default::default());
        assert!(
            doc.frontmatter.is_some(),
            "{} has no frontmatter",
            template.template
        );
        walk(&doc.blocks, &template.template);
    }
}
