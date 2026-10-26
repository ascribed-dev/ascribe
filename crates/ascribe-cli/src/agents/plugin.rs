//! The Ascribe plugin, `plugins/ascribe/`: one folder that Claude Code and
//! Copilot's CLI read through `.claude-plugin/`, and VS Code and Cursor
//! through the Agent Plugins manifest, `plugin.json`. Every file in it but
//! its README is written here, from what the binary prints, and a test
//! fails when the committed copy differs.
//!
//! It bundles the language server (Claude Code pushes its diagnostics into
//! the conversation after each edit), the stop hook, the MCP server, the
//! skill, and a command for each named prompt. It has no edit hook: the
//! language server already reports each edit, and a hook would report it
//! twice.
//!
//! The language server, the hook, and the MCP server run `ascribe` through
//! the plugin's launcher, `launcher/ascribe.sh`, which finds the binary the
//! VS Code extension would: `ASCRIBE_PATH`, then the project's
//! `node_modules/.bin/ascribe`, then `ascribe` on the path. A plugin is
//! installed once for every project, so `ascribe` from the path alone would
//! be whatever is installed globally, not the version the project pins.
//!
//! Only its test uses it: the test writes the plugin with
//! `ASCRIBE_BLESS=1`, and fails when the plugin differs.

use serde_json::{Value, json};

use super::settings::STOP_TIMEOUT;
use super::{prompts, skill};

/// The plugin's name, which names its commands: `/ascribe:check`.
const NAME: &str = "ascribe";

/// What the plugin is for, in its manifests: the short form in
/// `design/words.toml`, which is the crate's description, then what the
/// plugin adds.
const DESCRIPTION: &str = concat!(
    env!("CARGO_PKG_DESCRIPTION"),
    " This plugin is Ascribe for AI agents. The Ascribe skill, for every agent: how to write, \
     check, and fix pages. The MCP server's tools, for an agent with no shell. The stop hook, \
     and the language server's diagnostics after each edit, for an agent that doesn't check its \
     own work. Commands for the named prompts."
);

/// The launcher, relative to the plugin's folder. Not in `bin/`, which
/// Claude Code puts on the path and claude.ai won't install a plugin with.
const LAUNCHER: &str = "launcher/ascribe.sh";

/// The launcher's text: a POSIX shell script, run with `sh` so it needs no
/// executable bit, and so it runs on Windows through Git's `sh`, which
/// Claude Code needs there.
const LAUNCHER_TEXT: &str = include_str!("launcher.sh");

/// The launcher's path, as the harness expands it in a command or an
/// argument.
const LAUNCHER_PATH: &str = "${CLAUDE_PLUGIN_ROOT}/launcher/ascribe.sh";

/// The stop hook's command.
const STOP_HOOK: &str =
    "sh \"${CLAUDE_PLUGIN_ROOT}/launcher/ascribe.sh\" agents hook claude-code --event stop";

/// The plugin's homepage: the agents guide.
pub(crate) const HOMEPAGE: &str = ascribe_core::docs_permalink!("agents-guide");

/// The command each named prompt gets, by the prompt's name: `fix` is
/// `/ascribe:check`.
fn command_name(prompt: &str) -> &str {
    match prompt {
        "fix" => "check",
        other => other,
    }
}

/// The plugin's files, relative to its folder, with their text.
fn files() -> Vec<(String, String)> {
    let version = env!("CARGO_PKG_VERSION");
    // The maker in `design/words.toml`, which `scripts/design/words.test.ts`
    // compares with the manifests: the plugin's author and its marketplace's
    // owner.
    let author = json!({ "name": "Bryo Studio", "url": "https://bryostudio.com" });
    let repository = "https://github.com/ascribed-dev/ascribe";
    let mut files = vec![
        (
            ".claude-plugin/plugin.json".to_owned(),
            pretty(&json!({
                "name": NAME,
                "displayName": "Ascribe",
                "version": version,
                "description": DESCRIPTION,
                "author": author,
                "homepage": HOMEPAGE,
                "repository": repository,
                "license": "MPL-2.0",
                "keywords": ["ascribe", "documentation", "markdown", "docs-as-code"],
            })),
        ),
        (
            ".claude-plugin/marketplace.json".to_owned(),
            pretty(&json!({
                "name": NAME,
                "owner": author,
                "plugins": [{
                    "name": NAME,
                    "source": "./",
                    "description": DESCRIPTION,
                }],
            })),
        ),
        (
            "plugin.json".to_owned(),
            pretty(&json!({
                "$schema": "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json",
                "name": NAME,
                "version": version,
                "description": DESCRIPTION,
                "author": author,
                "homepage": HOMEPAGE,
                "repository": repository,
                "license": "MPL-2.0",
                "keywords": ["ascribe", "documentation", "markdown", "docs-as-code"],
            })),
        ),
        (
            ".lsp.json".to_owned(),
            pretty(&json!({
                NAME: {
                    "command": "sh",
                    "args": [LAUNCHER_PATH, "lsp"],
                    "extensionToLanguage": { ".md": "markdown" },
                    "diagnostics": true,
                }
            })),
        ),
        (
            ".mcp.json".to_owned(),
            pretty(&json!({
                "mcpServers": { NAME: { "command": "sh", "args": [LAUNCHER_PATH, "mcp"] } }
            })),
        ),
        (
            "hooks/hooks.json".to_owned(),
            pretty(&json!({
                "hooks": {
                    "Stop": [{
                        "hooks": [{
                            "type": "command",
                            "command": STOP_HOOK,
                            "timeout": STOP_TIMEOUT,
                        }]
                    }]
                }
            })),
        ),
        (LAUNCHER.to_owned(), LAUNCHER_TEXT.to_owned()),
    ];
    for (file, text) in skill::files(None, super::command::PLAIN) {
        files.push((format!("skills/{}/{file}", skill::NAME), text));
    }
    for prompt in prompts::PROMPTS {
        files.push((
            format!("commands/{}.md", command_name(prompt.name)),
            command(prompt),
        ));
    }
    files
}

/// A command's file: it runs `ascribe agents prompt` for its prompt, with
/// the arguments the person gave, since the prompt is about the project as
/// it is when it's used. `new-page` takes the request instead, since the
/// person doesn't know the page type until the agent reads the model.
fn command(prompt: &prompts::Named) -> String {
    if prompt.name == "new-page" {
        return new_page_command(prompt);
    }
    let hint: Vec<String> = prompt
        .arguments
        .iter()
        .map(|(name, _, required)| {
            if *required {
                format!("<{name}>")
            } else {
                format!("[{name}]")
            }
        })
        .collect();
    let mut out = format!(
        "---\ndescription: {}\nargument-hint: {}\n---\n\n",
        yaml_string(prompt.description),
        yaml_string(&hint.join(" "))
    );
    out.push_str(&format!(
        "Run `ascribe agents prompt {}{}`, and do what the prompt it prints asks.\n\n",
        prompt.name,
        arguments(prompt)
    ));
    if !prompt.arguments.is_empty() {
        out.push_str("Take the arguments from this, in this order, and leave out an optional one that isn't given: $ARGUMENTS\n\n");
        for (name, what, _) in prompt.arguments {
            out.push_str(&format!("- `{name}`: {what}\n"));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "Without a shell, use the prompt `{}` of Ascribe's MCP server instead.\n",
        prompt.name
    ));
    out
}

/// `new-page`'s file: the request, the page type from `ascribe model`, then
/// the prompt.
fn new_page_command(prompt: &prompts::Named) -> String {
    let mut out = format!(
        "---\ndescription: {}\nargument-hint: {}\n---\n\n",
        yaml_string(
            "Write a new page from a request: the page type that fits it, where its file goes, \
             the guide to read first, and the type's template to start from."
        ),
        yaml_string("<what the page is for>")
    );
    out.push_str("The page to write: $ARGUMENTS\n\n");
    out.push_str(
        "1. Run `ascribe model --section types`, and choose the page type that fits the \
         request. Choose the page's title, and, when the request says where the page goes, a \
         file or folder for it.\n",
    );
    out.push_str(&format!(
        "2. Run `ascribe agents prompt {}{}`, and do what the prompt it prints asks.\n\n",
        prompt.name,
        arguments(prompt)
    ));
    out.push_str(&format!(
        "Without a shell, call the MCP server's tool `ascribe_model` for the page types, then \
         use its prompt `{}`.\n",
        prompt.name
    ));
    out
}

/// A prompt's arguments as `ascribe agents prompt` takes them, with a
/// space before each.
fn arguments(prompt: &prompts::Named) -> String {
    prompt
        .arguments
        .iter()
        .map(|(name, _, required)| {
            if *required {
                format!(" --arg {name}=<{name}>")
            } else {
                format!(" [--arg {name}=<{name}>]")
            }
        })
        .collect()
}

/// JSON as the plugin's files hold it: two-space indents, and a line end.
fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default() + "\n"
}

/// A YAML double-quoted string.
fn yaml_string(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    const BLESS: &str = "ASCRIBE_BLESS=1 cargo test -p ascribe-cli plugin";

    fn folder() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../plugins/ascribe")
    }

    /// Every file but the README, relative to the plugin's folder.
    fn committed() -> Vec<String> {
        fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
            for entry in fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, base, out);
                } else {
                    let rel = path.strip_prefix(base).unwrap();
                    out.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        let mut out = Vec::new();
        walk(&folder(), &folder(), &mut out);
        out.retain(|f| f != "README.md");
        out.sort();
        out
    }

    #[test]
    fn the_plugin_is_the_binarys() {
        let bless = std::env::var_os("ASCRIBE_BLESS").is_some();
        let files = files();
        for (name, text) in &files {
            let path = folder().join(name);
            if bless {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                fs::write(&path, text).unwrap();
                continue;
            }
            let on_disk = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("plugins/ascribe/{name} is missing; run `{BLESS}`"))
                .replace("\r\n", "\n");
            assert!(
                &on_disk == text,
                "plugins/ascribe/{name} isn't what the binary writes; run `{BLESS}`"
            );
        }
        let mut expected: Vec<String> = files.into_iter().map(|(name, _)| name).collect();
        expected.sort();
        if bless {
            for extra in committed().iter().filter(|f| !expected.contains(f)) {
                fs::remove_file(folder().join(extra)).unwrap();
            }
        }
        assert_eq!(
            committed(),
            expected,
            "plugins/ascribe/ has files the binary doesn't write; run `{BLESS}`"
        );
        assert!(folder().join("README.md").is_file());
    }

    /// Both manifests parse, and the files they and the plugin's layout
    /// name are there.
    #[test]
    fn the_manifests_name_files_that_exist() {
        let read = |name: &str| -> Value {
            serde_json::from_str(&fs::read_to_string(folder().join(name)).unwrap()).unwrap()
        };
        let claude = read(".claude-plugin/plugin.json");
        let agent_plugins = read("plugin.json");
        for manifest in [&claude, &agent_plugins] {
            assert_eq!(manifest["name"], NAME);
            assert_eq!(manifest["version"], env!("CARGO_PKG_VERSION"));
        }
        let marketplace = read(".claude-plugin/marketplace.json");
        let entry = &marketplace["plugins"][0];
        assert_eq!(entry["name"], NAME);
        let source = folder().join(entry["source"].as_str().unwrap());
        assert!(source.join(".claude-plugin/plugin.json").is_file());
        assert_eq!(read(".lsp.json")[NAME]["args"][0], LAUNCHER_PATH);
        assert_eq!(read(".lsp.json")[NAME]["args"][1], "lsp");
        let mcp = &read(".mcp.json")["mcpServers"]["ascribe"];
        assert_eq!(mcp["args"][0], LAUNCHER_PATH);
        assert_eq!(mcp["args"][1], "mcp");
        let stop = &read("hooks/hooks.json")["hooks"]["Stop"][0]["hooks"][0];
        assert_eq!(stop["command"], STOP_HOOK);
        assert!(STOP_HOOK.contains(LAUNCHER_PATH));
        assert_eq!(LAUNCHER_PATH, format!("${{CLAUDE_PLUGIN_ROOT}}/{LAUNCHER}"));
        assert!(folder().join(LAUNCHER).is_file());
        assert!(folder().join("skills/ascribe/SKILL.md").is_file());
        for command in ["check", "new-page", "review"] {
            assert!(folder().join(format!("commands/{command}.md")).is_file());
        }
    }

    #[test]
    fn a_command_runs_its_prompt() {
        let review = prompts::find("review").unwrap();
        let text = command(review);
        assert!(text.contains("argument-hint: \"[path] [base]\""), "{text}");
        assert!(
            text.contains(
                "Run `ascribe agents prompt review [--arg path=<path>] [--arg base=<base>]`"
            ),
            "{text}"
        );
    }

    /// `new-page` takes the request, and the agent finds the type before
    /// it runs the prompt.
    #[test]
    fn new_page_takes_the_request() {
        let new_page = prompts::find("new-page").unwrap();
        let text = command(new_page);
        assert!(
            text.contains("argument-hint: \"<what the page is for>\""),
            "{text}"
        );
        let model = text.find("ascribe model --section types").unwrap();
        let prompt = text
            .find(
                "Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> \
                 [--arg path=<path>]`",
            )
            .unwrap();
        assert!(model < prompt, "{text}");
    }

    /// The launcher runs the binary the VS Code extension would, in its
    /// order: `ASCRIBE_PATH`, then the project's `node_modules/.bin/ascribe`
    /// from the project's folder up to the repository's root, then
    /// `ascribe` on the path. Each binary here is a script that prints its
    /// name and arguments. Unix only: on Windows the launcher runs in Git's
    /// `sh`, which a test can't count on finding.
    #[cfg(unix)]
    mod launcher {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::path::Path;
        use std::process::{Command, Output};

        use super::super::LAUNCHER_TEXT;

        /// A binary at `path` that prints `name` and its arguments.
        fn fake(path: &Path, name: &str) {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, format!("#!/bin/sh\necho {name} \"$@\"\n")).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }

        /// A temporary folder with the launcher, a global `ascribe` in
        /// `global/`, and a repository, `repo/`, whose project is
        /// `repo/docs/`.
        fn world() -> tempfile::TempDir {
            let dir = tempfile::tempdir().unwrap();
            fs::write(dir.path().join("ascribe.sh"), LAUNCHER_TEXT).unwrap();
            fake(&dir.path().join("global/ascribe"), "global");
            fs::create_dir_all(dir.path().join("repo/.git")).unwrap();
            fs::create_dir_all(dir.path().join("repo/docs")).unwrap();
            dir
        }

        /// Runs the launcher with `lsp`, from `cwd`, with `env` set and
        /// `global/` first on the path.
        fn run(dir: &Path, cwd: &Path, env: &[(&str, &Path)]) -> Output {
            let path = format!("{}:/usr/bin:/bin", dir.join("global").display());
            let mut command = Command::new("sh");
            command
                .arg(dir.join("ascribe.sh"))
                .arg("lsp")
                .current_dir(cwd)
                .env("PATH", path)
                .env_remove("ASCRIBE_PATH")
                .env_remove("CLAUDE_PROJECT_DIR");
            for (key, value) in env {
                command.env(key, value);
            }
            command.output().unwrap()
        }

        fn ran(output: &Output) -> String {
            assert!(output.status.success(), "{output:?}");
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }

        #[test]
        fn ascribe_path_comes_first() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("repo/docs/node_modules/.bin/ascribe"), "project");
            fake(&root.join("pinned/ascribe"), "setting");
            let pinned = root.join("pinned/ascribe");
            let out = run(root, &root.join("repo/docs"), &[("ASCRIBE_PATH", &pinned)]);
            assert_eq!(ran(&out), "setting lsp");
        }

        #[test]
        fn an_ascribe_path_that_isnt_a_binary_is_an_error_and_doesnt_fall_through() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("repo/docs/node_modules/.bin/ascribe"), "project");
            let missing = root.join("nowhere/ascribe");
            let out = run(root, &root.join("repo/docs"), &[("ASCRIBE_PATH", &missing)]);
            assert_eq!(out.status.code(), Some(127), "{out:?}");
            assert!(out.stdout.is_empty(), "{out:?}");
            assert!(String::from_utf8_lossy(&out.stderr).contains("ASCRIBE_PATH"));
        }

        #[test]
        fn the_projects_binary_comes_before_the_one_on_the_path() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("repo/docs/node_modules/.bin/ascribe"), "project");
            assert_eq!(ran(&run(root, &root.join("repo/docs"), &[])), "project lsp");
        }

        /// A project in a monorepo finds the binary installed at the
        /// repository's root, as the extension does.
        #[test]
        fn the_projects_binary_is_looked_for_up_to_the_repositorys_root() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("repo/node_modules/.bin/ascribe"), "root");
            assert_eq!(ran(&run(root, &root.join("repo/docs"), &[])), "root lsp");
        }

        #[test]
        fn a_binary_above_the_repository_isnt_the_projects() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("node_modules/.bin/ascribe"), "outside");
            assert_eq!(ran(&run(root, &root.join("repo/docs"), &[])), "global lsp");
        }

        /// Claude Code runs a hook from wherever its session is, and says
        /// where the project is in `CLAUDE_PROJECT_DIR`.
        #[test]
        fn claude_project_dir_is_where_the_project_is_looked_for() {
            let dir = world();
            let root = dir.path();
            fake(&root.join("repo/docs/node_modules/.bin/ascribe"), "project");
            let project = root.join("repo/docs");
            let out = run(root, root, &[("CLAUDE_PROJECT_DIR", &project)]);
            assert_eq!(ran(&out), "project lsp");
        }

        #[test]
        fn without_a_project_binary_the_one_on_the_path_runs() {
            let dir = world();
            let root = dir.path();
            assert_eq!(ran(&run(root, &root.join("repo/docs"), &[])), "global lsp");
        }

        #[test]
        fn with_no_binary_at_all_it_says_how_to_install_one() {
            let dir = world();
            let root = dir.path();
            fs::remove_file(root.join("global/ascribe")).unwrap();
            let out = run(root, &root.join("repo/docs"), &[]);
            assert_eq!(out.status.code(), Some(127), "{out:?}");
            assert!(String::from_utf8_lossy(&out.stderr).contains("@ascribed/cli"));
        }
    }
}
