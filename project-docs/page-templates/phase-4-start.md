# Phase 4: Starting a page

Part of [Page templates](README.md). Requires phase 2; better after phase 3. The binary's prompts, the language server, the extension, and the agents' rules.

## Goal

Nobody copies a file by hand:

- **In the editor,** an action: pick a type, give a title, and the page is made from the type's template in the folder the type covers, with the cursor at the first thing to fill in.
- **For an agent,** the `new-page` prompt has the template's body and says to read the guide first, by its path.
- **In the rules `ascribe agents sync` writes,** each type has a line saying what it's for.

## Context

- The proposal: "Starting a page".
- `crates/ascribe-cli/src/agents/prompts.rs`: the `new-page` prompt, its arguments, and what it writes today.
- The plugin's command `ascribe:new-page`, and the surface audit's finding 59: the agent "couldn't run it until it knew the type, and to know the type it had to find the project and read its model". Check whether its issue has landed, and build on what's there.
- `crates/ascribe-query/src/rules.rs`: the rules, and their budget of 4,000 characters (decisions 51 and 56).
- The extension's actions: `packages/vscode/src/actions/registry.ts`, the actions bar, and how an action that isn't about the cursor's place is offered. The Explorer's context menu, and the **Pages** view.
- `ascribe/edit` and `ascribe/context`, in `crates/ascribe-lsp/`: what the server already does to change files, and whether creating one is something it has done.
- `project-docs/checklists.md`, "A new editor command, action, or setting": titles, labels, and where a choice lives.

## Design

### Where a page starts

One function in a library crate: given a type, a title, and optionally a path, it returns the new file's path and text. The text is the type's template with its `title` set to the title given, and nothing else changed. The title is a frontmatter field (README decision 21), so filling it is setting one value, quoted as YAML needs. The prompt, the server, and a command all call it.

The path, when none is given: the first folder the type's `files` covers, and a file name slugged from the title, as a route's segments are. A file that exists is never overwritten.

### In the editor

- An action, **New page**, offered where it makes sense without a cursor in a page: the palette, the Explorer's menu on a folder, and the **Pages** view. It asks for the type (each with its one-line purpose, from the guide's summary), then the title.
- Offered from a folder, only the types whose `files` cover that folder are listed.
- The page opens with the cursor on the first thing left to fill in, selected, so typing replaces it. With a title given, that's usually the description, in the frontmatter, and then the body's first placeholder. A next-placeholder action steps through the rest. Both are found by the same reading the check uses, frontmatter fields and body alike, and a placeholder that runs across blocks is selected whole.
- A type with no template makes a page with its frontmatter and nothing else, so the action works in every project.
- The server makes the file, since it knows the model and the template, and the extension opens it. This needs no request of Ascribe's own: a `WorkspaceEdit` carries `CreateFile` operations, and the server's existing edit path can return one. Check that the path does, and that VS Code applies a created file with its text in one step.

### For an agent

`ascribe agents prompt new-page` gains the template's body for a type that has one, and one instruction before it: read the guide at its path, and Ascribe's notes, before writing. The guide isn't put in the prompt: it's long, and a file an agent reads itself arrives whole.

The prompt's check command is written for the folder it was run in (the surface audit's finding 49).

The MCP prompt and the plugin's command follow, since they're the same text.

### In the rules

Each type's line in `AGENTS.md` says what it's for, in a few words. For a type from the set, that's their index's `summary` of it, which `templates.lock` has carried since phase 2. For any type, a `purpose` it states itself comes first. Within the budget: if the lines don't fit, they're the first thing cut, and the rules say where the full list is.

So the `purpose` key is for a project's own types, and for a team that wants to say what one of the six means to it. It's a new key in the content model, added by a recorded decision. The editor's list of types shows the same line.

### At the command line

A person at a terminal can start a page too: the same function, as a command. Named by the checklist: it makes a page, so it isn't under `templates`, and it isn't under `agents`. Decide the name here, against the map of the surface, and record it. If no name fits well, leave the command out and say why: the editor and the prompt cover the two users the audit found.

## Tasks

1. The function, with tests: a type with a template, one without, a title that slugs to an existing file, a path given.
2. The prompt, the MCP prompt, and the plugin's command, regenerated.
3. The rules' line for each type, with the budget's test.
4. The server's part, the action, the next-placeholder action, and their tests, with the extension's integration test for the whole path.
5. The command, or the recorded reason for none.
6. The editor guide, the agents guide, the page on templates, the command reference, `CHANGELOG.md`, and the map of the surface.

## Out of scope

Choosing a type for the writer. Filling a page from a description. Templates for fragments.

## Acceptance criteria

- In VS Code, in a project with the six types, **New page** on the `how-to` folder offers the how-to, makes the file, and opens it with the first placeholder selected. The integration test does exactly this.
- `ascribe agents prompt new-page --arg type=how-to --arg title="Rotate keys"` contains the template's body and the guide's path, and its check command works from the folder it was run in.
- A session in the audit's form, run once: an agent with the plugin asked for a how-to reads the guide and writes a page with no placeholder left. Its transcript is kept with the pull request, and what it did is said plainly, whatever it was.
- The rules for a project with six types are under their budget.

## Verify

```sh
cargo test --workspace --locked
pnpm --filter ascribe-vscode test
pnpm test
```

## Commits

1. "Start a page from its type's template"
2. "Give an agent the template, and send it to the guide"
3. "Say what each page type is for in the rules for agents"
4. "Start a page from the editor"
