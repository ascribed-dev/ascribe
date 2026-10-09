//! What `ascribe agents sync --cloud` writes for Copilot's cloud agent,
//! which runs on GitHub with no editor: the setup steps that put `ascribe`
//! on its path, and the MCP server, either as the JSON for the
//! repository's settings or in a custom agent that carries it.
//!
//! The cloud agent reads the skill from `.agents/skills/`, and the hooks
//! from `.github/hooks/`, so those are what `sync` writes anyway.

use serde_json::Value;

use super::markers::{Comment, Markers};
use crate::mcp::TOOLS;

/// The cloud agent's setup steps, from the repository's root.
pub const SETUP_STEPS: &str = ".github/workflows/copilot-setup-steps.yml";

/// The custom agent that carries the MCP server, from the repository's
/// root.
pub const AGENT: &str = ".github/agents/ascribe-docs.md";

/// The job the cloud agent runs: no other name is picked up.
const JOB: &str = "copilot-setup-steps";

/// The actions the steps use, at the major versions this repository's own
/// workflows use (a test compares them).
pub const CHECKOUT: &str = "actions/checkout@v7";
/// See [`CHECKOUT`].
pub const SETUP_NODE: &str = "actions/setup-node@v7";

/// The Node.js the steps install: the oldest `@ascribed/cli` supports (a
/// test compares it with the package's `engines`).
pub const NODE: &str = "24";

/// The MCP server's name, in the settings and the custom agent.
const SERVER: &str = "ascribe";

/// How the steps install `ascribe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Install {
    /// The project doesn't pin `@ascribed/cli`: install this version
    /// globally.
    Global {
        /// The version.
        version: String,
    },
    /// The project pins `@ascribed/cli` in a `package.json`: install its
    /// dependencies, and put their binaries on the path.
    Pinned {
        /// The folder of the `package.json` that pins it, from the
        /// repository's root, `/`-separated; empty for the root.
        package: String,
        /// The folder to install in, from the repository's root (where the
        /// lockfile is), `/`-separated; empty for the root.
        install: String,
        /// The command that installs.
        command: &'static str,
        /// The `node_modules/.bin` that holds `ascribe`, from the
        /// repository's root.
        bin: String,
    },
}

/// The command that installs a folder's dependencies, by its lockfile, in
/// the order they're looked for.
pub const LOCKFILES: &[(&str, &str)] = &[
    ("package-lock.json", "npm ci"),
    (
        "pnpm-lock.yaml",
        "corepack enable && pnpm install --frozen-lockfile",
    ),
    ("yarn.lock", "corepack enable && yarn install"),
];

/// The command that installs a folder's dependencies when it has no
/// lockfile.
pub const NO_LOCKFILE: &str = "npm install";

/// Whether a `package.json`'s text pins `@ascribed/cli`.
pub fn pins_ascribe(package_json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(package_json) else {
        return false;
    };
    ["dependencies", "devDependencies", "optionalDependencies"]
        .iter()
        .any(|key| value[key].get("@ascribed/cli").is_some())
}

/// The steps that install `ascribe`, as YAML list items at no
/// indentation. With `checkout`, the repository is checked out first,
/// which installing a pinned version needs.
fn steps(install: &Install, checkout: bool) -> String {
    let mut out = String::new();
    if checkout {
        out.push_str(&format!(
            "- name: Check out the repository\n  uses: {CHECKOUT}\n"
        ));
    }
    out.push_str(&format!(
        "- name: Set up Node.js\n  uses: {SETUP_NODE}\n  with:\n    node-version: {NODE}\n"
    ));
    match install {
        Install::Global { version } => out.push_str(&format!(
            "- name: Install Ascribe\n  shell: bash\n  run: npm install --global @ascribed/cli@{version}\n"
        )),
        Install::Pinned {
            install,
            command,
            bin,
            ..
        } => {
            out.push_str(&format!(
                "- name: Install Ascribe\n  shell: bash\n  run: |\n    {command}\n    echo \"$GITHUB_WORKSPACE/{bin}\" >> \"$GITHUB_PATH\"\n"
            ));
            if !install.is_empty() {
                out.push_str(&format!("  working-directory: {install}\n"));
            }
        }
    }
    out.push_str("- name: Show Ascribe's version\n  shell: bash\n  run: ascribe --version\n");
    out
}

/// The workflow for a repository that has none: the job, checking out the
/// repository, and the block.
fn new_workflow(markers: &Markers<'_>, install: &Install) -> String {
    let header = format!(
        "# The environment Copilot's cloud agent works in. GitHub runs this job
# before the agent starts, from the default branch, and only under the
# name {JOB}.
name: Copilot setup steps

on:
  workflow_dispatch:
  push:
    paths:
      - {SETUP_STEPS}
  pull_request:
    paths:
      - {SETUP_STEPS}

jobs:
  {JOB}:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - name: Check out the repository
        uses: {CHECKOUT}
"
    );
    let mut out = header;
    let indent = "      ";
    out.push_str(&format!("{indent}{}\n", markers.start()));
    out.push_str(&format!("{indent}{}\n", markers.end()));
    // A block between markers that were just written always places.
    markers
        .place(Some(&out), &steps(install, false))
        .unwrap_or(out)
}

/// The setup steps' workflow, from its text now: the block between its
/// markers, or added at the end of the `copilot-setup-steps` job's steps,
/// or a new workflow when there's none. An error says what's wrong with
/// the file.
pub fn setup_steps(
    old: Option<&str>,
    markers: &Markers<'_>,
    install: &Install,
) -> Result<String, String> {
    debug_assert_eq!(markers.comment, Comment::Hash);
    let Some(old) = old else {
        return Ok(new_workflow(markers, install));
    };
    parse_job(old)?;
    let has_markers = old
        .lines()
        .any(|line| markers.is_start(line.trim()) || markers.is_end(line.trim()));
    let text = if has_markers {
        old.to_owned()
    } else {
        insert_markers(old, markers)?
    };
    let checkout = matches!(install, Install::Pinned { .. })
        && !outside_block(&text, markers).contains("actions/checkout");
    let placed = markers
        .place(Some(&text), &steps(install, checkout))
        .map_err(|damage| format!("{damage}; fix its `{}` markers by hand", markers.name))?;
    parse_job(&placed)?;
    Ok(placed)
}

/// Checks that `text` is YAML with a `copilot-setup-steps` job whose steps
/// are a list.
fn parse_job(text: &str) -> Result<(), String> {
    let value: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(text).map_err(|e| format!("it isn't YAML: {e}"))?;
    let job = value
        .get("jobs")
        .and_then(|jobs| jobs.get(JOB))
        .ok_or_else(|| format!("it has no `{JOB}` job"))?;
    match job.get("steps") {
        Some(serde_yaml_ng::Value::Sequence(_)) => Ok(()),
        _ => Err(format!("its `{JOB}` job has no list of steps")),
    }
}

/// `text` without the lines of the block, markers and all.
fn outside_block(text: &str, markers: &Markers<'_>) -> String {
    let mut inside = false;
    let mut out = String::new();
    for line in text.lines() {
        if markers.is_start(line.trim()) {
            inside = true;
        } else if markers.is_end(line.trim()) {
            inside = false;
        } else if !inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// How far a line is indented.
fn indent_of(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// Whether a line means nothing to YAML's structure: blank, or a comment.
fn is_blank(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.is_empty() || trimmed.starts_with('#')
}

/// Whether a line ends a block `sync` wrote, of this project or another.
fn is_block_end(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("# ascribe:copilot") && trimmed.ends_with(" end")
}

/// A YAML line's key, when it's `key:` with nothing after it but a
/// comment.
fn bare_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let (key, rest) = trimmed.split_once(':')?;
    let rest = rest.trim();
    (rest.is_empty() || rest.starts_with('#')).then_some(key.trim())
}

/// `text` with empty markers after the last of the `copilot-setup-steps`
/// job's steps, indented as its steps are. The workflow is read line by
/// line, so only the block form YAML is written in is understood.
fn insert_markers(text: &str, markers: &Markers<'_>) -> Result<String, String> {
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = text.lines().collect();
    let unclear = || {
        format!(
            "can't find where the `{JOB}` job's steps end; add the markers `{}` and `{}` there by hand",
            markers.start(),
            markers.end()
        )
    };
    // The end of the block that starts at `from`, whose first line is
    // indented by `indent`: the first line after it indented no more.
    let end_of = |from: usize, indent: usize| {
        (from + 1..lines.len())
            .find(|&i| !is_blank(lines[i]) && indent_of(lines[i]) <= indent)
            .unwrap_or(lines.len())
    };
    let jobs = (0..lines.len())
        .find(|&i| indent_of(lines[i]) == 0 && bare_key(lines[i]) == Some("jobs"))
        .ok_or_else(unclear)?;
    let jobs_end = end_of(jobs, 0);
    let job = (jobs + 1..jobs_end)
        .find(|&i| bare_key(lines[i]) == Some(JOB))
        .ok_or_else(unclear)?;
    let job_end = end_of(job, indent_of(lines[job]));
    let keys = (job + 1..job_end)
        .find(|&i| !is_blank(lines[i]))
        .map(|i| indent_of(lines[i]))
        .ok_or_else(unclear)?;
    let steps = (job + 1..job_end)
        .find(|&i| indent_of(lines[i]) == keys && bare_key(lines[i]) == Some("steps"))
        .ok_or_else(unclear)?;
    let steps_indent = indent_of(lines[steps]);
    // The steps are list items, which YAML lets sit at the key's own
    // indentation.
    let first = (steps + 1..job_end)
        .find(|&i| !is_blank(lines[i]))
        .filter(|&i| lines[i].trim_start().starts_with('-'))
        .ok_or_else(unclear)?;
    let item_indent = indent_of(lines[first]);
    if item_indent < steps_indent {
        return Err(unclear());
    }
    let end = (first + 1..job_end)
        .find(|&i| {
            !is_blank(lines[i])
                && (indent_of(lines[i]) < item_indent
                    || (indent_of(lines[i]) == item_indent
                        && !lines[i].trim_start().starts_with('-')))
        })
        .unwrap_or(job_end);
    // After the last step's last line, or another project's block, but
    // not the blank lines and comments that follow.
    let at = (first..end)
        .rev()
        .find(|&i| !is_blank(lines[i]) || is_block_end(lines[i]))
        .map_or(end, |i| i + 1);
    let indent = " ".repeat(item_indent);
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        if i == at {
            out.push_str(&format!("{indent}{}{newline}", markers.start()));
            out.push_str(&format!("{indent}{}{newline}", markers.end()));
        }
        out.push_str(line);
        out.push_str(newline);
    }
    if at == lines.len() {
        out.push_str(&format!("{indent}{}{newline}", markers.start()));
        out.push_str(&format!("{indent}{}{newline}", markers.end()));
    }
    if !text.ends_with('\n') && at != lines.len() {
        out.truncate(out.len() - newline.len());
    }
    Ok(out)
}

/// Each of the MCP server's tools, quoted: the cloud agent uses them
/// without asking, and they only read.
fn tools() -> Vec<String> {
    TOOLS
        .iter()
        .map(|tool| Value::from(tool.name).to_string())
        .collect()
}

/// The JSON for the repository's MCP settings: a local server running
/// `ascribe mcp`, with each of its tools allowed. Written out by hand to
/// keep its keys in the order the settings' documentation shows.
pub fn mcp_settings() -> String {
    let tools = tools()
        .iter()
        .map(|tool| format!("        {tool}"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "{{
  \"mcpServers\": {{
    \"{SERVER}\": {{
      \"type\": \"local\",
      \"command\": \"ascribe\",
      \"args\": [\"mcp\"],
      \"tools\": [
{tools}
      ]
    }}
  }}
}}
"
    )
}

/// Where the repository's MCP settings are, for the person pasting them.
pub const MCP_SETTINGS_PLACE: &str =
    "the repository's Settings → Copilot → MCP servers, under \"MCP configuration\"";

/// `.github/agents/ascribe-docs.md`: a custom agent for documentation work
/// that carries the MCP server, for the cloud agent only (VS Code reads the
/// server from its own settings).
pub fn agent_md() -> String {
    format!(
        "---
name: ascribe-docs
description: Writes and fixes this repository's Ascribe documentation, checking each change with Ascribe.
target: github-copilot
mcp-servers:
  {SERVER}:
    type: local
    command: ascribe
    args: [\"mcp\"]
    tools: [{}]
---

<!-- Generated by `ascribe agents sync`. Run it again rather than editing this file. -->

Write and fix documentation in this repository's Ascribe projects.

- Follow the project's rules in the `AGENTS.md` beside its `ascribe.toml`, and use the `ascribe` skill for Ascribe's syntax and commands.
- Ask the `{SERVER}` MCP server's tools, or the `ascribe` commands, about the content model, a page's outline, links, and where a page is used, rather than guessing.
- After each change, run `ascribe check` on the files you changed, and fix what it reports. Before you finish, run `ascribe check` on the whole project.
",
        tools().join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKERS: Markers<'static> = Markers {
        name: "ascribe:copilot",
        note: "generated",
        comment: Comment::Hash,
    };

    fn global() -> Install {
        Install::Global {
            version: "1.2.3".to_owned(),
        }
    }

    fn pinned() -> Install {
        Install::Pinned {
            package: "docs".to_owned(),
            install: String::new(),
            command: "npm ci",
            bin: "docs/node_modules/.bin".to_owned(),
        }
    }

    #[test]
    fn a_new_workflow_is_valid_and_places_again_unchanged() {
        let text = setup_steps(None, &MARKERS, &global()).unwrap_or_default();
        assert!(parse_job(&text).is_ok(), "{text}");
        assert!(
            text.contains("      - name: Install Ascribe\n        shell: bash\n        run: npm install --global @ascribed/cli@1.2.3\n"),
            "{text}"
        );
        assert_eq!(
            setup_steps(Some(&text), &MARKERS, &global()),
            Ok(text.clone())
        );
        // A pinned install has the checkout it needs already.
        let pinned = setup_steps(Some(&text), &MARKERS, &pinned()).unwrap_or_default();
        assert_eq!(pinned.matches("actions/checkout").count(), 1, "{pinned}");
        assert!(pinned.contains("          npm ci\n"), "{pinned}");
    }

    #[test]
    fn the_block_goes_after_the_last_step() {
        let old = "\
on: workflow_dispatch
jobs:
  other:
    runs-on: ubuntu-latest
    steps:
      - run: other
  copilot-setup-steps:
    runs-on: ubuntu-latest
    steps:
    - name: Mine
      run: |
        make setup

    # A comment about what follows.
    timeout-minutes: 30
";
        let text = setup_steps(Some(old), &MARKERS, &global()).unwrap_or_default();
        assert!(
            text.contains("        make setup\n    # ascribe:copilot start (generated)\n    - name: Set up Node.js\n"),
            "{text}"
        );
        assert!(
            text.ends_with("    # ascribe:copilot end\n\n    # A comment about what follows.\n    timeout-minutes: 30\n"),
            "{text}"
        );
        assert!(text.starts_with("on: workflow_dispatch\njobs:\n  other:\n"));
        assert_eq!(
            setup_steps(Some(&text), &MARKERS, &global()),
            Ok(text.clone())
        );
    }

    #[test]
    fn steps_at_the_end_of_the_file_without_a_newline() {
        let old = "jobs:\n  copilot-setup-steps:\n    steps:\n      - run: a";
        let text = setup_steps(Some(old), &MARKERS, &global()).unwrap_or_default();
        assert!(
            text.starts_with("jobs:\n  copilot-setup-steps:\n    steps:\n      - run: a\n      # ascribe:copilot start"),
            "{text}"
        );
        assert!(parse_job(&text).is_ok());
    }

    #[test]
    fn a_pinned_install_checks_out_when_nothing_else_does() {
        let old = "jobs:\n  copilot-setup-steps:\n    steps:\n      - run: a\n";
        let text = setup_steps(Some(old), &MARKERS, &pinned()).unwrap_or_default();
        assert!(
            text.contains(&format!(
                "- name: Check out the repository\n        uses: {CHECKOUT}\n"
            )),
            "{text}"
        );
        assert_eq!(
            setup_steps(Some(&text), &MARKERS, &pinned()),
            Ok(text.clone())
        );
    }

    #[test]
    fn a_workflow_it_cant_place_in_is_refused() {
        let refused = |old: &str| setup_steps(Some(old), &MARKERS, &global()).unwrap_err();
        assert!(refused("jobs: [").starts_with("it isn't YAML"));
        assert_eq!(
            refused("jobs:\n  build:\n    steps: []\n"),
            "it has no `copilot-setup-steps` job"
        );
        assert_eq!(
            refused("jobs:\n  copilot-setup-steps:\n    runs-on: x\n"),
            "its `copilot-setup-steps` job has no list of steps"
        );
        assert!(
            refused("jobs:\n  copilot-setup-steps:\n    steps: [{run: a}]\n")
                .starts_with("can't find where")
        );
        assert!(
            refused("jobs:\n  copilot-setup-steps:\n    steps:\n      - run: a\n      # ascribe:copilot end\n")
                .contains("markers by hand")
        );
    }

    #[test]
    fn the_package_json_pins_ascribe() {
        assert!(pins_ascribe(
            r#"{"devDependencies": {"@ascribed/cli": "^1.0.0"}}"#
        ));
        assert!(!pins_ascribe(r#"{"dependencies": {"astro": "5"}}"#));
        assert!(!pins_ascribe("not json"));
    }

    #[test]
    fn the_settings_and_the_agent_list_every_tool() {
        let settings: Value = serde_json::from_str(&mcp_settings()).unwrap_or_default();
        let server = &settings["mcpServers"]["ascribe"];
        assert_eq!(server["type"], "local");
        assert_eq!(server["tools"].as_array().map(Vec::len), Some(TOOLS.len()));
        let agent = agent_md();
        let front = agent.split("---\n").nth(1).unwrap_or_default();
        let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(front).unwrap_or_default();
        let tools = &value["mcp-servers"]["ascribe"]["tools"];
        assert_eq!(tools.as_sequence().map(Vec::len), Some(TOOLS.len()));
        assert_eq!(value["mcp-servers"]["ascribe"]["args"][0], "mcp");
    }
}
