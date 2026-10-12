# Phase 5: The editor

Part of [Tested examples](README.md). Needs phase 4.

## Goal

A writer looking at a `@snippet` line sees what it shows and whether that was tested, without building anything. A hover on the line gives the file, its status, and its environment. The stale and unobserved advice appears on the line as the page is edited. The preview's code block has the tested line the site will have.

## Context

- README decision 2: the language server compares, and never runs a test or an environment command.
- `crates/ascribe-lsp/src/context.rs`: what the server knows at a `@snippet` line today, which is its node and nothing shown to the writer.
- `crates/ascribe-lsp/src/compute.rs`: which files' changes make the server look at a page again. A snippet is read again when its page changes, not when its code does, and the editor guide says so.
- `project-docs/checklists.md`: "A new editor command, action, or setting".
- Decision 76, part (d): what the language server asks before running. Nothing here runs, so nothing is asked.

## Design

### Hover

On a `@snippet` line:

- the file, as a link that opens it, and the region when the address names one;
- for an example: "Tested with Node 24.9.0 on Linux", or "Changed after its last test run", or "Not tested", with the names of the observations behind it;
- for a stale or unobserved example in the same repository, the folder to run `ascribe observe` in.

Every value from a config or an observation is escaped for the hover's Markdown.

### Problems

`example-stale` and `example-unobserved` are reported on the `@snippet` line by the server, as `ascribe check` reports them, in the editor's build.

They follow the same rule a snippet's other problems do: they're worked out when the page is looked at again. So this phase widens what makes the server look again, by the least that makes the advice true:

- a change to a file under `examples.observations/` of a source, or to its `ascribe.examples.toml`, re-checks the pages that show that source's examples.

A change to the example's own file still doesn't, as now. That's the brainstorm's "a preview that follows the code", and it isn't this phase: the status after an edit to the code is right the next time the page is touched or the project is checked, and the guide says so.

### No action that runs a test

The editor offers no command, quick fix, or button that runs `ascribe observe`. The advice's quick fix is "Copy the command", which puts `ascribe observe` and the folder on the clipboard. A person runs it in a terminal, where they can see what it starts: README decisions 2 and 17.

### The preview

The page preview draws the tested line as the site does, since it's in the block's info string and the preview uses the same elements.

### In `ascribe.examples.toml`

The server reports the config's own problems in the file as it's edited, with completion for its keys, when the file is at the root of a source some open project names. A config no open project uses is just a TOML file.

## Tasks

1. The hover.
2. The two advice checks on the line, in the editor's build.
3. Observations and the config among the files that make the server look again.
4. "Copy the command".
5. The config's diagnostics and completion.
6. The preview's caption, with a test.
7. The editor guide, including what still doesn't follow the code, an entry in `changes/`, and the map of the surface.

## Out of scope

- A preview or problems that follow edits to the code.
- Go to definition from a page into the code, and a lens in the code saying which pages use it.
- Anything in a code file's editor.

## Acceptance criteria

- Hovering a `@snippet` of the example project's file shows the file and "Tested with …".
- Replacing the committed observation with one that has a different hash puts `example-stale` on the line without the page being edited.
- The extension contributes no command that starts `ascribe observe`, and its tests assert it.
- A hostile label in a config is shown as text in the hover.

## Verify

```sh
cargo test -p ascribe-lsp --locked
pnpm --filter ascribe-vscode test
```

## Commits

1. "Show what a snippet's file was tested with on hover"
2. "Report a stale example in the editor"
3. "Check an examples config as it's edited"
