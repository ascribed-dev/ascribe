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
//! Only its test uses it: the test writes the plugin with
//! `ASCRIBE_BLESS=1`, and fails when the plugin differs.

use serde_json::{Value, json};

use super::settings::{STOP_TIMEOUT, mcp_servers};
use super::{prompts, skill};

/// The plugin's name, which names its commands: `/ascribe:check`.
const NAME: &str = "ascribe";

/// What the plugin is for, in its manifests.
const DESCRIPTION: &str = "Check Ascribe docs as an agent edits them: the language server's \
     diagnostics after each edit, a full check before the agent finishes, the MCP server, the \
     Ascribe skill, and commands for the named prompts.";

/// The stop hook's command.
const STOP_HOOK: &str = "ascribe agents hook claude-code --event stop";

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
    let author = json!({ "name": "Ascribe", "url": "https://github.com/ascribed-dev" });
    let repository = "https://github.com/ascribed-dev/ascribe";
    let homepage = format!("{}/guides/agents/", ascribe_core::docs_site!());
    let mut files = vec![
        (
            ".claude-plugin/plugin.json".to_owned(),
            pretty(&json!({
                "name": NAME,
                "displayName": "Ascribe",
                "version": version,
                "description": DESCRIPTION,
                "author": author,
                "homepage": homepage,
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
                "homepage": homepage,
                "repository": repository,
                "license": "MPL-2.0",
                "keywords": ["ascribe", "documentation", "markdown", "docs-as-code"],
            })),
        ),
        (
            ".lsp.json".to_owned(),
            pretty(&json!({
                NAME: {
                    "command": "ascribe",
                    "args": ["lsp"],
                    "extensionToLanguage": { ".md": "markdown" },
                    "diagnostics": true,
                }
            })),
        ),
        (".mcp.json".to_owned(), pretty(&mcp_servers("ascribe"))),
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
    ];
    for (file, text) in skill::files(None) {
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
/// it is when it's used.
fn command(prompt: &prompts::Named) -> String {
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
    let words: Vec<String> = prompt
        .arguments
        .iter()
        .map(|(name, _, required)| {
            if *required {
                format!(" --arg {name}=<{name}>")
            } else {
                format!(" [--arg {name}=<{name}>]")
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
        words.concat()
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
        assert_eq!(read(".lsp.json")[NAME]["args"][0], "lsp");
        assert_eq!(read(".mcp.json")["mcpServers"]["ascribe"]["args"][0], "mcp");
        let stop = &read("hooks/hooks.json")["hooks"]["Stop"][0]["hooks"][0];
        assert_eq!(stop["command"], STOP_HOOK);
        assert!(folder().join("skills/ascribe/SKILL.md").is_file());
        for command in ["check", "new-page", "review"] {
            assert!(folder().join(format!("commands/{command}.md")).is_file());
        }
    }

    #[test]
    fn a_command_runs_its_prompt() {
        let new_page = prompts::find("new-page").unwrap();
        let text = command(new_page);
        assert!(
            text.starts_with("---\ndescription: \"Write a new page"),
            "{text}"
        );
        assert!(
            text.contains("argument-hint: \"<type> <title> [path]\""),
            "{text}"
        );
        assert!(
            text.contains(
                "Run `ascribe agents prompt new-page --arg type=<type> --arg title=<title> \
                 [--arg path=<path>]`"
            ),
            "{text}"
        );
    }
}
