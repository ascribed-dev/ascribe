//! The command reference, `docs/content/command/index.md` and its pages, each
//! in the folder of the feature its commands are for, includes fragments
//! generated from the `clap` definitions: the synopsis, the options every
//! command accepts, and each command's own, in
//! `docs/content/_generated/cli-*.md`. This test renders them and fails when
//! one is out of date; run it with `ASCRIBE_BLESS=1` to rewrite them. To
//! change what a fragment says, change the help text.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fmt::Write as _;
use std::path::PathBuf;

use clap::{Arg, ArgAction, Command, CommandFactory};

use crate::cli::Cli;

mod builtins;

const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs";

/// What each fragment starts with: a memo, touching its first block, saying
/// what generates it, and how.
const HEADER: &str = "@memo: Generated from the help text in crates/ascribe-cli/src/ by \
     crates/ascribe-cli/src/docs.rs; edit the help text, then run \
     `ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs` to rewrite it.\n";

/// Options newer than their command, as `(command, option, availability)`:
/// the reference marks each with `@available: <availability>`. An option no
/// release has yet is `ascribe planned`; when a release ships it, name the
/// release, as `ascribe 0.2.0`. Remove an entry when the option's command section says
/// the same, or when every live line has the option.
const AVAILABLE: &[(&str, &str, &str)] = &[];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The permalinks the docs' `site` build publishes, with `base-path` set to
/// `base`.
fn docs_permalinks(base: &str) -> Vec<ascribe_resolve::Permalink> {
    let root = repo().join("docs");
    let mut model = ascribe_model::load(root.join("ascribe.toml")).unwrap();
    base.clone_into(&mut model.consumer.base_path);
    let layout = ascribe_resolve::Layout::from_model(&model);
    let fs = ascribe_resolve::DiskFs::new(&root, &layout);
    let project = ascribe_resolve::Project::load(std::sync::Arc::new(model), layout, &fs);
    let build = project.model().build("site").unwrap();
    let router = ascribe_resolve::AstroRouter::from_consumer(&project.model().consumer);
    project.resolve_build(build, &router).permalinks()
}

/// Every address of the docs the binary builds, each with the page and the
/// heading, as written, it must lead to: each command's help, each
/// diagnostic's entry (in `check`'s JSON, `explain`, and the editor), the
/// agents' rules, the plugin's homepage, and the JSON report contract the
/// MCP tools name. A page with a folder of the same name, as
/// `checks/diagnostics.md` has `checks/diagnostics/`, stands for itself and
/// the pages in that folder, and the command reference for its pages
/// wherever they are.
fn addresses() -> Vec<(String, &'static str, Option<String>)> {
    let mut helps = Vec::new();
    let mut cli = crate::cli::command();
    helps.push((cli.render_long_help().to_string(), None));
    // Rendering builds the command, which adds clap's own `help` subcommand.
    for command in cli.get_subcommands_mut().filter(|c| c.get_name() != "help") {
        let name = command.get_name().to_owned();
        helps.push((
            command.render_long_help().to_string(),
            Some(format!("`ascribe {name}`")),
        ));
        // A hidden command, run only by Ascribe itself, isn't documented.
        for sub in command
            .get_subcommands_mut()
            .filter(|c| c.get_name() != "help" && !c.is_hide_set())
        {
            helps.push((
                sub.render_long_help().to_string(),
                Some(format!("`ascribe {name} {}`", sub.get_name())),
            ));
        }
    }
    let mut out: Vec<_> = helps
        .into_iter()
        .map(|(help, heading)| {
            let link = help
                .lines()
                .find_map(|line| line.strip_prefix("Documentation: "))
                .expect("each command's help ends with a link to its documentation");
            (link.to_owned(), COMMAND_REFERENCE, heading)
        })
        .collect();
    // Retired codes keep their entries, and their permalinks.
    for entry in ascribe_check::Registry::global().entries() {
        out.push((
            entry.docs(),
            "checks/diagnostics.md",
            Some(format!("{} `{}`", entry.code, entry.slug)),
        ));
    }
    out.push((
        ascribe_query::rules::directive_reference_url(),
        "writing/directives.md",
        None,
    ));
    out.push((
        crate::agents::plugin::HOMEPAGE.to_owned(),
        "agents/index.md",
        None,
    ));
    out.push((
        crate::mcp::CONTRACT.to_owned(),
        "contracts/json-reports.md",
        None,
    ));
    out
}

/// The docs' address and permalink path in the binary are `docs/ascribe.toml`'s.
#[test]
fn the_docs_site_is_where_the_binary_links() {
    let model = ascribe_model::load(repo().join("docs/ascribe.toml")).unwrap();
    assert_eq!(
        model
            .consumer
            .site
            .as_deref()
            .map(|s| s.trim_end_matches('/')),
        Some(crate::cli::DOCS_SITE),
        "docs_site! in crates/ascribe-core/src/lib.rs isn't [consumer] site in docs/ascribe.toml"
    );
    assert_eq!(
        format!(
            "{}{}",
            crate::cli::DOCS_SITE,
            model.consumer.permalink_path()
        ),
        crate::cli::DOCS_PERMALINKS,
        "docs_permalinks! in crates/ascribe-core/src/lib.rs isn't [consumer] permalink-path in docs/ascribe.toml"
    );
    assert_eq!(
        model.consumer.trailing_slash,
        ascribe_model::TrailingSlash::Always,
        "docs_permalink! in crates/ascribe-core/src/lib.rs ends each address in `/`, as trailing-slash = \"always\" does"
    );
}

/// The command reference's own page. Its other pages are each in the folder
/// of the feature its commands are for.
const COMMAND_REFERENCE: &str = "command/index.md";

/// The command reference's pages: the pages that include a fragment generated
/// from the help text, or that a command's permalink leads to, as `ascribe
/// lsp`'s on the editor's page, which has no options to generate; its own
/// page first.
fn command_reference() -> Vec<String> {
    let permalinks = std::fs::read_to_string(repo().join("docs/ascribe.permalinks")).unwrap();
    let permalinked: Vec<&str> = permalinks
        .lines()
        .filter(|l| l.starts_with("cli-"))
        .filter_map(|l| l.split_once(" = \"")?.1.split(['#', '"']).next())
        .collect();
    fn walk(folder: &str, permalinked: &[&str], pages: &mut Vec<String>) {
        let dir = repo().join("docs/content").join(folder);
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| !name.starts_with('_'))
            .collect();
        names.sort();
        for name in names {
            let path = format!("{folder}{name}");
            if repo().join("docs/content").join(&path).is_dir() {
                walk(&format!("{path}/"), permalinked, pages);
            } else if name.ends_with(".md")
                && path != COMMAND_REFERENCE
                && (permalinked.contains(&path.as_str())
                    || std::fs::read_to_string(repo().join("docs/content").join(&path))
                        .unwrap()
                        .lines()
                        .any(|l| l.starts_with("@include: ") && l.contains("/_generated/cli-")))
            {
                pages.push(path);
            }
        }
    }
    let mut pages = vec![COMMAND_REFERENCE.to_owned()];
    walk("", &permalinked, &mut pages);
    pages
}

/// The pages `page` stands for: itself, and the pages in the folder of its
/// name; or, for the command reference, its pages.
fn pages_of(page: &str) -> Vec<String> {
    if page == COMMAND_REFERENCE {
        return command_reference();
    }
    let mut pages = vec![page.to_owned()];
    let folder = page.trim_end_matches(".md");
    if let Ok(dir) = std::fs::read_dir(repo().join("docs/content").join(folder)) {
        let mut more: Vec<String> = dir
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".md"))
            .map(|name| format!("{folder}/{name}"))
            .collect();
        more.sort();
        pages.extend(more);
    }
    pages
}

/// Which of the pages `page` stands for publishes what's declared in
/// `file`: the page that is `file`, or the one that includes it.
fn publishing_page(page: &str, file: &str) -> Option<String> {
    let file_name = file.rsplit('/').next().unwrap_or(file);
    pages_of(page).into_iter().find(|candidate| {
        candidate == file
            || std::fs::read_to_string(repo().join("docs/content").join(candidate))
                .unwrap()
                .lines()
                .any(|l| l.starts_with("@include: ") && l.ends_with(&format!("/{file_name}")))
    })
}

/// The docs' committed list of permalinks, `docs/ascribe.permalinks`.
fn docs_list() -> String {
    std::fs::read_to_string(repo().join("docs").join(ascribe_model::PERMALINKS_FILE)).unwrap()
}

/// Every address the binary builds is a permalink in the docs' committed
/// list, checked by what `ascribe permalinks verify` runs, as a product's CI
/// would; and it leads to the right page and heading: a new command or
/// diagnostic needs one in the docs. A command's is `cli-` and its name,
/// with a `-` for each space; a diagnostic's is its code.
#[test]
fn every_address_the_binary_builds_is_a_permalink_of_the_docs() {
    let text = docs_list();
    let list = ascribe_model::PermalinkList::parse(&text, ascribe_core::FileId::new(0)).unwrap();
    let addresses = addresses();
    let names: Vec<String> = addresses
        .iter()
        .map(|(address, _, _)| {
            address
                .strip_prefix(crate::cli::DOCS_PERMALINKS)
                .and_then(|rest| rest.strip_suffix('/'))
                .unwrap_or_else(|| {
                    panic!(
                        "{address}: not a permalink of the docs, ending in `/` (trailing-slash = \"always\")"
                    )
                })
                .to_owned()
        })
        .collect();
    let verified =
        ascribe_query::permalinks::verify(&text, "docs/ascribe.permalinks", &names).unwrap();
    assert!(
        verified.missing.is_empty(),
        "the docs' list has no permalink {:?}: mark the heading, then run `ascribe permalinks sync --config docs`",
        verified.missing
    );
    let mut checked = 0;
    for ((address, page, heading), name) in addresses.iter().zip(&names) {
        let place = &list.permalinks[name].place;
        let (file, id) = match place.split_once('#') {
            Some((file, id)) => (file, Some(id)),
            None => (place.as_str(), None),
        };
        // A heading in a fragment the page includes, as the diagnostics
        // reference's are.
        assert!(
            publishing_page(page, file).is_some(),
            "{address}: declared in {file}, which isn't {} or a fragment one includes",
            pages_of(page).join(", ")
        );
        let Some(heading) = heading else {
            assert_eq!(id, None, "{address}: leads to a heading, not the page");
            checked += 1;
            continue;
        };
        assert_eq!(id, Some(name.as_str()), "{address}");
        let expected = match heading.strip_prefix("`ascribe ") {
            Some(command) => format!("cli-{}", command.trim_end_matches('`').replace(' ', "-")),
            None => heading.split(' ').next().unwrap_or_default().to_owned(),
        };
        assert_eq!(*name, expected, "{address}: the permalink for {heading}");
        // The heading the `@id` is under, in the file it's written in.
        let written = std::fs::read_to_string(repo().join("docs/content").join(file)).unwrap();
        let marked = format!(" {heading}\n@id {{permalink=true}}: {name}\n");
        assert!(
            written.contains(&marked),
            "{address}: no heading {heading} marked `{name}` in {file}"
        );
        checked += 1;
    }
    assert!(checked > 150, "only {checked} addresses");
}

/// Each address in the `ascribe.toml` that `ascribe init` writes, and the
/// one its last line gives, is a permalink of the docs, in the committed
/// list: the file is the first of Ascribe's a new user opens, and its links
/// must keep working. Each comment's link leads to a section of the
/// `ascribe.toml` reference.
#[test]
fn the_first_ascribe_toml_links_to_permalinks_of_the_docs() {
    let text = docs_list();
    let list = ascribe_model::PermalinkList::parse(&text, ascribe_core::FileId::new(0)).unwrap();
    let names: Vec<&str> = ascribe_templates::MODEL_TEXT
        .split(crate::cli::DOCS_PERMALINKS)
        .skip(1)
        .map(|rest| rest.split_once('/').unwrap().0)
        .collect();
    assert_eq!(names.len(), 4, "{names:?}");
    for name in &names {
        let place = &list
            .permalinks
            .get(*name)
            .unwrap_or_else(|| {
                panic!("the first ascribe.toml links to {name}, which the docs don't publish")
            })
            .place;
        assert!(
            place.starts_with("content-model/"),
            "{name} leads to {place}, not the ascribe.toml reference"
        );
    }
    let next = ascribe_templates::init::NEXT
        .strip_prefix(crate::cli::DOCS_PERMALINKS)
        .and_then(|rest| rest.strip_suffix('/'))
        .unwrap();
    assert_eq!(list.permalinks[next].place, "astro/index.md");
}

/// The docs' committed list is what `ascribe permalinks sync` writes for
/// them, so a permalink they add is in it, and so is every one they had.
#[test]
fn the_docs_list_of_permalinks_is_current() {
    let project = ascribe_check::Project::load(&repo().join("docs/ascribe.toml")).unwrap();
    let synced = ascribe_query::permalinks::sync(
        &project.index(),
        project.permalinks(),
        project.permalinks_text(),
    )
    .unwrap();
    assert!(
        !synced.changed,
        "docs/ascribe.permalinks is behind the docs (added {:?}, moved {:?}): run `ascribe permalinks sync --config docs`",
        synced.added,
        synced
            .moved
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>()
    );
    assert!(
        synced.removed.is_empty(),
        "docs/ascribe.permalinks has permalinks the docs removed: {:?}",
        synced.removed
    );
}

/// Building the docs under another base path, as when they move to
/// `/docs/`, changes no address the binary builds: each permalink is still
/// published, at the same path.
#[test]
fn the_binary_s_addresses_dont_move_with_the_docs() {
    let moved = docs_permalinks("/docs/");
    for (address, page, _) in addresses() {
        let name = address
            .strip_prefix(crate::cli::DOCS_PERMALINKS)
            .and_then(|rest| rest.strip_suffix('/'))
            .unwrap();
        let found = moved
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("{address}: not published under /docs/"));
        let on = found.page.to_string();
        assert!(
            pages_of(page).contains(&on),
            "{address}: on {on}, not on {}",
            pages_of(page).join(", ")
        );
        // A folder's `index.md` is at the folder's address.
        let id = on.trim_end_matches(".md").trim_end_matches("/index");
        assert_eq!(found.route, format!("/docs/{id}/"));
    }
    let mut model = ascribe_model::load(repo().join("docs/ascribe.toml")).unwrap();
    "/docs/".clone_into(&mut model.consumer.base_path);
    assert_eq!(
        format!(
            "{}{}",
            crate::cli::DOCS_SITE,
            model.consumer.permalink_path()
        ),
        crate::cli::DOCS_PERMALINKS
    );
}

#[test]
fn the_command_reference_is_current() {
    let fragments = render();
    check_fragments("cli-", &fragments);
    let pages = pages_of(COMMAND_REFERENCE);
    for (name, _) in &fragments {
        assert!(
            pages.iter().any(|page| {
                let depth = "../".repeat(page.matches('/').count());
                std::fs::read_to_string(repo().join("docs/content").join(page))
                    .unwrap()
                    .contains(&format!("@include: {depth}_generated/{name}\n"))
            }),
            "no page of the command reference ({}) includes _generated/{name}",
            pages.join(", ")
        );
    }
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

/// The fragments, by file name: the synopsis, the options every command
/// accepts, and one for each command that has options of its own. A command
/// with subcommands (`sources`) is listed as each of them (`sources fetch`).
fn render() -> Vec<(String, String)> {
    let cli = Cli::command();
    let commands = leaves(&cli);
    let mut fragments = vec![("cli-synopsis.md".to_owned(), synopsis(&commands))];

    let global: Vec<&Arg> = cli.get_arguments().filter(|a| documented(a)).collect();
    fragments.push(("cli-global-options.md".to_owned(), options("", &global)));

    for (name, command) in commands {
        let args: Vec<&Arg> = command
            .get_arguments()
            .filter(|a| documented(a) && !a.is_global_set())
            .collect();
        if !args.is_empty() {
            let file = name.replace(' ', "-");
            fragments.push((format!("cli-{file}-options.md"), options(&name, &args)));
        }
    }
    fragments
}

/// Each command that runs, by its name after `ascribe`: the top-level
/// commands, and the subcommands of one that has them in its place.
fn leaves(cli: &Command) -> Vec<(String, &Command)> {
    let mut out = Vec::new();
    for command in cli.get_subcommands() {
        let subcommands: Vec<&Command> = command
            .get_subcommands()
            .filter(|c| c.get_name() != "help" && !c.is_hide_set())
            .collect();
        if subcommands.is_empty() {
            out.push((command.get_name().to_owned(), command));
        }
        for sub in subcommands {
            out.push((format!("{} {}", command.get_name(), sub.get_name()), sub));
        }
    }
    out
}

/// Whether an argument is documented: every one but `--help` and `--version`,
/// which `clap` adds.
fn documented(arg: &Arg) -> bool {
    !arg.is_hide_set() && !matches!(arg.get_id().as_str(), "help" | "version")
}

/// The synopsis: a line for each command, with its options in order, and a
/// blank line between the groups `--help` lists them in.
fn synopsis(commands: &[(String, &Command)]) -> String {
    let width = commands
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    let mut out = format!("{HEADER}```text\n");
    let mut group = None;
    for (name, command) in commands {
        let top = name.split(' ').next().unwrap_or_default();
        let this = crate::cli::GROUPS
            .iter()
            .position(|(_, names)| names.contains(&top));
        if group.is_some() && group != this {
            out.push('\n');
        }
        group = this;
        let usage: Vec<String> = command
            .get_arguments()
            .filter(|a| documented(a) && !a.is_global_set())
            .map(usage)
            .collect();
        let line = format!("ascribe {name:width$} {}", usage.join(" "));
        let _ = writeln!(out, "{}", line.trim_end());
    }
    out.push_str("ascribe --version\n```\n");
    out
}

/// How an argument is written in the synopsis: `[--build <NAME>]...`,
/// `[--format text|json]`, `[--check]`, `[PATHS]...`, and without brackets
/// when it's required: `TARGET --from <PAGE>`.
fn usage(arg: &Arg) -> String {
    let values: Vec<String> = arg
        .get_possible_values()
        .iter()
        .filter(|v| !v.is_hide_set())
        .map(|v| v.get_name().to_owned())
        .collect();
    let delimited = arg.get_value_delimiter().is_some();
    let value = if !values.is_empty() {
        values.join(if delimited { "," } else { "|" })
    } else {
        format!("<{}>", value_name(arg))
    };
    let repeated = matches!(arg.get_action(), ArgAction::Append) && !delimited;
    let written = match arg.get_long() {
        Some(long) if takes_value(arg) => format!("[--{long} {value}]"),
        Some(long) => format!("[--{long}]"),
        None => format!("[{}]", value_name(arg)),
    };
    let written = if arg.is_required_set() {
        written[1..written.len() - 1].to_owned()
    } else {
        written
    };
    if repeated {
        format!("{written}...")
    } else {
        written
    }
}

/// A list item for each argument: how it's written, then its help, its
/// values, and its default.
fn options(command: &str, args: &[&Arg]) -> String {
    let mut out = HEADER.to_owned();
    for arg in args {
        out.push_str("- ");
        let id = arg.get_id().as_str();
        if let Some((_, _, available)) =
            AVAILABLE.iter().find(|(c, o, _)| (*c, *o) == (command, id))
        {
            out.push_str(&format!("@available: {available}\n  "));
        }
        let written = match arg.get_long() {
            // A value that may be left out is given with `=`.
            Some(long)
                if takes_value(arg) && arg.get_num_args().is_some_and(|n| n.min_values() == 0) =>
            {
                format!("--{long}[=<{}>]", value_name(arg))
            }
            Some(long) if takes_value(arg) => format!("--{long} <{}>", value_name(arg)),
            Some(long) => format!("--{long}"),
            None => usage(arg),
        };
        let help = arg
            .get_long_help()
            .or_else(|| arg.get_help())
            .map(ToString::to_string)
            .unwrap_or_default();
        // The help's first paragraph is what `-h` shows; the reference reads
        // them all as one.
        let help: Vec<String> = help.split("\n\n").map(sentence).collect();
        let _ = write!(out, "`{written}`: {}", escape(&help.join(" ")));

        let defaults: Vec<&str> = arg
            .get_default_values()
            .iter()
            .filter_map(|v| v.to_str())
            .collect();
        let values: Vec<_> = if takes_value(arg) {
            arg.get_possible_values()
                .into_iter()
                .filter(|v| !v.is_hide_set())
                .collect()
        } else {
            Vec::new()
        };
        // Every value is the default: say so once, rather than on each.
        let all = values.len() > 1 && values.iter().all(|v| defaults.contains(&v.get_name()));
        if all {
            out.push_str(" All of them by default.");
        } else if values.is_empty() && takes_value(arg) && !defaults.is_empty() {
            let _ = write!(out, " By default, `{}`.", defaults.join(","));
        }
        out.push('\n');
        for value in values {
            let name = value.get_name();
            let default = if defaults.contains(&name) && !all {
                " (the default)"
            } else {
                ""
            };
            let help = value
                .get_help()
                .map(ToString::to_string)
                .unwrap_or_default();
            let _ = writeln!(out, "  - `{name}`{default}: {}", escape(&sentence(&help)));
        }
    }
    out
}

fn takes_value(arg: &Arg) -> bool {
    matches!(arg.get_action(), ArgAction::Set | ArgAction::Append)
}

fn value_name(arg: &Arg) -> String {
    arg.get_value_names()
        .and_then(|names| names.first())
        .map_or_else(|| arg.get_id().as_str().to_uppercase(), ToString::to_string)
}

/// A paragraph of help text as a sentence: on one line, ending with a period,
/// which `clap` leaves off a summary.
fn sentence(text: &str) -> String {
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() || text.ends_with(['.', '!', '?', ':']) {
        text
    } else {
        format!("{text}.")
    }
}

/// Text with a backslash before each `{` and `<` outside code spans, so a
/// placeholder such as `{value}` or `<version>` reads as itself, and is never
/// taken for one of the docs' phrases or for HTML.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut code = false;
    for c in text.chars() {
        if c == '`' {
            code = !code;
        } else if matches!(c, '{' | '<') && !code {
            out.push('\\');
        }
        out.push(c);
    }
    out
}
