//! The agent prompts about problems, as snapshots: each kind, and what each
//! part of the agent prompt format says when it applies.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::fs;
use std::path::{Path, PathBuf};

use ascribe_check::prompt::{self, Builds, Context, LIMIT};
use ascribe_check::{Project, Reported, Scope, diagnose};
use ascribe_core::{FileId, RelPath};

const MODEL: &str = r#"
spec = "0.1"

[project]
content-root = "docs"
output-dir = ".out"

[types.page]
default = true

[types.page.frontmatter]
title = "string"

[dimensions.pm]
values = ["npm", "pnpm", "yarn"]
"#;

fn project(files: &[(&str, &str)]) -> Project {
    let model = ascribe_model::load_str(MODEL, FileId::new(0)).expect("the model loads");
    let sources = Project::from_sources(files.iter().map(|(path, text)| {
        (
            RelPath::parse(path).expect("a relative path"),
            (*text).to_owned(),
        )
    }));
    Project::from_parts(
        PathBuf::from("/nonexistent-ascribe-project"),
        RelPath::parse("docs").expect("a relative path"),
        model,
        MODEL.to_owned(),
        sources,
    )
}

fn page(body: &str) -> String {
    format!("---\ntitle: T\n---\n\n{body}\n")
}

/// A context with nothing besides the problems: the project at the
/// repository's root, every build, no `AGENTS.md`.
fn plain() -> Context {
    Context {
        folder: None,
        agents: None,
        builds: Builds::All,
        unsaved: Vec::new(),
    }
}

/// What `ascribe check` reports for `path` (relative to the project root).
fn reported(project: &Project, path: &str) -> Vec<Reported> {
    let diagnosed = diagnose(project, &[]).expect("every build");
    Scope::new([RelPath::parse(path).unwrap()]).report(project, diagnosed.diagnostics)
}

fn all(project: &Project) -> Vec<Reported> {
    Reported::all(diagnose(project, &[]).expect("every build").diagnostics)
}

fn id(project: &Project, content: &str) -> FileId {
    project
        .source_at(&RelPath::parse(content).unwrap())
        .expect("a source")
        .id
}

fn shown_on(project: &Project) -> impl Fn(&RelPath) -> Vec<RelPath> + '_ {
    let index = project.held_index();
    move |fragment| index.including_pages(fragment)
}

fn one_problem(project: &Project, context: &Context, path: &str) -> String {
    let reported = reported(project, path);
    let first = reported.first().expect("a problem");
    prompt::problem(project, context, first, &shown_on(project))
}

#[test]
fn a_broken_link_names_the_file_and_line_quotes_the_message_and_says_how_to_finish() {
    let project = project(&[(
        "guides/install.md",
        &page("Read [the overview](overview.md) first."),
    )]);
    let context = Context {
        agents: Some("AGENTS.md".to_owned()),
        ..plain()
    };
    let text = one_problem(&project, &context, "docs/guides/install.md");
    assert!(text.contains("Where: docs/guides/install.md:5"));
    assert!(text.ends_with(
        "Follow the project's rules in `AGENTS.md`.\nWhen you're done, run `ascribe check docs/guides/install.md` and fix what it reports.\n"
    ));
    insta::assert_snapshot!(text);
}

#[test]
fn a_value_outside_a_dimension_lists_the_allowed_values() {
    let project = project(&[(
        "install.md",
        &page("@variant {pm=bun}:\nRun `bun add quill`.\n@end"),
    )]);
    insta::assert_snapshot!(one_problem(&project, &plain(), "docs/install.md"));
}

#[test]
fn a_fix_says_whether_it_is_safe() {
    let project = project(&[
        ("install.md", &page("See [the guide](/guide/).")),
        ("guide.md", &page("A guide.")),
    ]);
    let text = one_problem(&project, &plain(), "docs/install.md");
    assert!(
        text.contains("Ascribe has a safe automatic fix: `"),
        "{text}"
    );
    insta::assert_snapshot!(text);
}

#[test]
fn a_problem_in_a_fragment_names_the_pages_that_show_it() {
    let project = project(&[
        ("_fragments/prereqs.md", "See [the setup](setup.md).\n"),
        ("install.md", &page("@include: _fragments/prereqs.md")),
        ("upgrade.md", &page("@include: _fragments/prereqs.md")),
    ]);
    insta::assert_snapshot!(one_problem(
        &project,
        &plain(),
        "docs/_fragments/prereqs.md"
    ));
}

#[test]
fn a_fragment_shown_on_many_pages_names_ten() {
    let mut files: Vec<(String, String)> = (1..=17)
        .map(|n| {
            (
                format!("page-{n:02}.md"),
                page("@include: _fragments/prereqs.md"),
            )
        })
        .collect();
    files.push((
        "_fragments/prereqs.md".to_owned(),
        "See [the setup](setup.md).\n".to_owned(),
    ));
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let project = project(&borrowed);
    let text = one_problem(&project, &plain(), "docs/_fragments/prereqs.md");
    assert!(text.contains("page-10.md, and 7 more\n"), "{text}");
}

#[test]
fn a_project_in_a_subfolder_writes_commands_from_the_repository_root() {
    let project = project(&[("install.md", &page("Read [the overview](overview.md)."))]);
    let context = Context {
        folder: Some("site/docs".to_owned()),
        agents: Some("site/docs/AGENTS.md".to_owned()),
        builds: Builds::Named(vec!["self-hosted".to_owned()]),
        unsaved: Vec::new(),
    };
    insta::assert_snapshot!(one_problem(&project, &context, "docs/install.md"));
}

#[test]
fn an_unsaved_file_is_saved_first() {
    let project = project(&[("install.md", &page("Read [the overview](overview.md)."))]);
    let context = Context {
        builds: Builds::Editor("site".to_owned()),
        unsaved: vec!["docs/install.md".to_owned()],
        ..plain()
    };
    let text = one_problem(&project, &context, "docs/install.md");
    assert!(text.contains(
        "Where: docs/install.md:5\nThe file has unsaved changes; save it before you start.\n"
    ));
    insta::assert_snapshot!(text);
}

#[test]
fn a_file_prompt_lists_each_problem_in_a_line() {
    let project = project(&[(
        "install.md",
        &page("Read [the overview](overview.md).\n\n@variant {pm=bun}:\nRun it.\n@end"),
    )]);
    let reported = reported(&project, "docs/install.md");
    let text = prompt::file(
        &project,
        &plain(),
        id(&project, "install.md"),
        &reported,
        &shown_on(&project),
    )
    .expect("problems");
    insta::assert_snapshot!(text);
}

#[test]
fn a_file_prompt_lists_twenty_problems_and_names_the_command_for_the_rest() {
    let links: String = (1..=25)
        .map(|n| format!("Read [part {n}](missing-{n}.md).\n\n"))
        .collect();
    let project = project(&[("install.md", &page(&links))]);
    let reported = reported(&project, "docs/install.md");
    assert_eq!(reported.len(), 25);
    let text = prompt::file(
        &project,
        &plain(),
        id(&project, "install.md"),
        &reported,
        &shown_on(&project),
    )
    .expect("problems");
    assert!(text.contains("\nand 5 more: `ascribe check docs/install.md --format concise`\n"));
    assert_eq!(text.matches("\n- ").count(), 20);
}

#[test]
fn a_project_prompt_counts_problems_by_file() {
    let project = project(&[
        ("install.md", &page("Read [a](a.md) and [b](b.md).")),
        ("upgrade.md", &page("Read [c](c.md).")),
    ]);
    let context = Context {
        folder: Some("docs".to_owned()),
        unsaved: vec!["docs/upgrade.md".to_owned()],
        ..plain()
    };
    let text = prompt::project(&project, &context, &[], &all(&project)).expect("problems");
    insta::assert_snapshot!(text);
}

#[test]
fn a_project_prompt_counts_twenty_files() {
    let files: Vec<(String, String)> = (1..=23)
        .map(|n| (format!("page-{n:02}.md"), page("Read [a](a.md).")))
        .collect();
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, t)| (p.as_str(), t.as_str()))
        .collect();
    let project = project(&borrowed);
    let text = prompt::project(&project, &plain(), &[], &all(&project)).expect("problems");
    assert!(text.contains("\nand 3 more files.\n"), "{text}");
    assert_eq!(text.matches("\n- ").count(), 20);
}

#[test]
fn no_problems_no_prompt() {
    let project = project(&[("install.md", &page("All good."))]);
    assert!(all(&project).is_empty());
    assert_eq!(prompt::project(&project, &plain(), &[], &[]), None);
    assert_eq!(
        prompt::file(
            &project,
            &plain(),
            id(&project, "install.md"),
            &[],
            &shown_on(&project)
        ),
        None
    );
}

#[test]
fn a_prompt_past_the_limit_is_cut_at_a_block_and_keeps_how_to_finish() {
    let long = "word ".repeat(1_500);
    let project = project(&[(
        "install.md",
        &page(&format!("{long}[the overview](overview.md) {long}")),
    )]);
    let text = one_problem(&project, &plain(), "docs/install.md");
    assert!(text.chars().count() <= LIMIT, "{} characters", text.len());
    assert!(text.contains("(cut: read the rest in `docs/install.md`)\n"));
    assert!(text.contains("Message: "));
    assert!(text.ends_with("and fix what it reports.\n"));
}

/// A repository in a temporary folder, with the project in `docs/`.
fn repository(agents_at_root: bool, agents_in_project: bool) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("a temporary folder");
    fs::create_dir(dir.path().join(".git")).unwrap();
    let docs = dir.path().join("docs");
    fs::create_dir(&docs).unwrap();
    if agents_at_root {
        fs::write(dir.path().join("AGENTS.md"), "# Rules\n").unwrap();
    }
    if agents_in_project {
        fs::write(docs.join("AGENTS.md"), "# Rules\n").unwrap();
    }
    (dir, docs)
}

fn context_of(root: &Path) -> Context {
    Context::of_project(root, Builds::All)
}

#[test]
fn the_context_finds_the_projects_folder_and_its_agents_md() {
    let (_dir, docs) = repository(false, false);
    let found = context_of(&docs);
    assert_eq!(found.folder.as_deref(), Some("docs"));
    assert_eq!(found.agents, None);

    let (_dir, docs) = repository(true, false);
    assert_eq!(context_of(&docs).agents.as_deref(), Some("AGENTS.md"));

    let (_dir, docs) = repository(true, true);
    assert_eq!(context_of(&docs).agents.as_deref(), Some("docs/AGENTS.md"));

    let (dir, _docs) = repository(true, false);
    let at_root = context_of(dir.path());
    assert_eq!(at_root.folder, None);
    assert_eq!(at_root.agents.as_deref(), Some("AGENTS.md"));
}
