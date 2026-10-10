# The Ascribe plugin

Checks [Ascribe](https://github.com/ascribed-dev/ascribe) docs as an AI agent edits them. One folder, for Claude Code, Copilot's CLI, VS Code, and Cursor:

- **The language server** (`.lsp.json`): `ascribe lsp` for `.md` files. Claude Code starts it the first time it edits a page, and reads its diagnostics after each edit, so Claude hears about a broken link as soon as it writes one. It reports the editor's build only (`[editor] build` in `ascribe.toml`).
- **The stop hook** (`hooks/hooks.json`): `ascribe agents hook claude-code --event stop`. Before Claude finishes, it checks every build of each project whose pages changed, and keeps Claude working while there are errors. It asks once per stop.
- **The MCP server** (`.mcp.json`): `ascribe mcp`.
- **The skill** (`skills/ascribe/`): how to use Ascribe, the same for every project.
- **Commands:** `/ascribe:check`, `/ascribe:new-page`, and `/ascribe:review`, which run `ascribe agents prompt fix`, `new-page`, and `review`.

It has no edit hook: the language server already reports each edit, and a hook would report it twice.

## Install

The plugin runs `ascribe` from your path. Install it first:

```shell
npm install -g @ascribed/cli
```

or put the binary from a [release](https://github.com/ascribed-dev/ascribe/releases) on your path, which starts quicker.

Then, in Claude Code in a terminal, from a copy of the repository (its VS Code extension doesn't offer `/plugin`):

```shell
/plugin marketplace add ./plugins/ascribe
/plugin install ascribe@ascribe
```

Use the plugin or a project's own hook entries (`ascribe agents sync --with-hook`), not both, or Claude hears about each problem twice. Claude Code doesn't start a plugin's language server in a cloud session; a project that works there writes its own entries, which Claude Code reads from the repository.

## Fewer diagnostics in the conversation

On a project with many problems, the diagnostics Claude Code pushes after each edit can crowd the conversation. To turn them off, set `"diagnostics": false` in the plugin's `.lsp.json`, in your installed copy:

```json
{
  "ascribe": {
    "command": "ascribe",
    "args": ["lsp"],
    "extensionToLanguage": { ".md": "markdown" },
    "diagnostics": false
  }
}
```

The stop hook still checks the whole project before Claude finishes.

## The files

Every file here but this README is written by the `ascribe` binary, and a test fails when one differs: run `ASCRIBE_BLESS=1 cargo test -p ascribe-cli plugin` to write them again. Its version is the release's.

See [Agents](https://ascribed-dev.com/guides/agents/) in Ascribe's docs.
