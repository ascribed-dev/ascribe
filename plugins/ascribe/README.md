# The Ascribe plugin

Checks [Ascribe](https://github.com/ascribed-dev/ascribe) docs as an AI agent edits them. One folder, for Claude Code, Copilot's CLI, VS Code, and Cursor:

- **The language server** (`.lsp.json`): `ascribe lsp` for `.md` files. Claude Code starts it the first time it edits a page, and reads its diagnostics after each edit, so Claude hears about a broken link as soon as it writes one. It reports the editor's build only (`[editor] build` in `ascribe.toml`).
- **The stop hook** (`hooks/hooks.json`): `ascribe agents hook claude-code --event stop`. Before Claude finishes, it checks every build of each project whose pages changed, and keeps Claude working while there are errors. It asks once per stop.
- **The MCP server** (`.mcp.json`): `ascribe mcp`.
- **The skill** (`skills/ascribe/`): how to use Ascribe, the same for every project. It's the part every agent uses; the MCP server's tools are for an agent with no shell, and the language server and the stop hook for one that doesn't check its own work.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run `ascribe agents prompt fix`, `new-page`, and `review`. `/ascribe:new-page` takes what the page is for, and has Claude choose the page type from `ascribe model` first.

It has no edit hook: the language server already reports each edit, and a hook would report it twice.

## Install

The plugin runs the `ascribe` binary the VS Code extension would, so a project that pins a version gets that version, not whichever is installed globally. Its launcher, `launcher/ascribe.sh`, runs the first of these that it finds:

1. The binary the `ASCRIBE_PATH` environment variable names. If it names nothing that runs, the launcher says so and stops, rather than run another.
2. The project's own, `node_modules/.bin/ascribe`, looked for from the project's folder (`CLAUDE_PROJECT_DIR`, or the working directory) up to the repository's root, so a project in a monorepo finds the one at the root.
3. `ascribe` on your path.

So install it in the project:

```shell
npm install --save-dev @ascribed/cli
```

or, for every project, install it globally (`npm install -g @ascribed/cli`) or put the binary from a [release](https://github.com/ascribed-dev/ascribe/releases) on your path, which starts quicker. The launcher is a POSIX shell script, run with `sh`; on Windows, that's the `sh` that comes with Git, which Claude Code needs there.

Then add it in Claude Code in a terminal (its VS Code extension doesn't offer `/plugin`). Ascribe's repository is the plugin's marketplace, so Claude Code adds it from GitHub, without a copy of the repository:

```shell
/plugin marketplace add ascribed-dev/ascribe
/plugin install ascribe@ascribe
```

The plugin's version is the release's, and it's written for that release's `ascribe`. To match it to the `ascribe` you run, add its release's tag instead, `ascribed-dev/ascribe#v<version>`, where `<version>` is what `ascribe --version` prints. Added without a tag, the plugin is `main`'s, which changes version at each release: get it with `/plugin marketplace update ascribe`, or on its own once you turn on auto-update for the marketplace in `/plugin`, under Marketplaces. From a copy of the repository, add the copy's folder: `/plugin marketplace add ./`.

Use the plugin or a project's own hook entries (`ascribe agents sync --with-hook`), not both, or Claude hears about each problem twice. Claude Code doesn't start a plugin's language server in a cloud session; a project that works there writes its own entries, which Claude Code reads from the repository.

## Fewer diagnostics in the conversation

On a project with many problems, the diagnostics Claude Code pushes after each edit can crowd the conversation. To turn them off, set `"diagnostics": false` in the plugin's `.lsp.json`, in your installed copy:

```json
{
  "ascribe": {
    "command": "sh",
    "args": ["${CLAUDE_PLUGIN_ROOT}/launcher/ascribe.sh", "lsp"],
    "extensionToLanguage": { ".md": "markdown" },
    "diagnostics": false
  }
}
```

The stop hook still checks the whole project before Claude finishes.

## The files

Every file here but this README is written by the `ascribe` binary, and so is the marketplace at the repository's root, `.claude-plugin/marketplace.json`; a test fails when one differs: run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli plugin` to write them again. Its version is the release's, and only the plugin's manifests name it.

See [Agents](https://ascribed-dev.com/go/agents-guide/) in Ascribe's docs.
