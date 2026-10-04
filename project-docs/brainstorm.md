# Brainstorm: what Ascribe could do next

Ideas for Ascribe after 0.1.1 (written 2026-10-02). This is exploration, not a plan: nothing here is decided. Each idea says what it is, how it fits Ascribe, what's hard about it, and the questions to answer before designing it. The suggested order and the open questions are at the end.

Three groups:

- [Product features](#product-features): code snippets from tested code, editable previews, OpenAPI, editor UI (an actions bar, a sidebar, and more), prose linting with Vale, and other CMS features.
- [Agents](#agents): publishing docs that agents can read (the Web Documentation Delivery Spec), and helping agents that write docs (diagnostics, the CLI, an MCP server, GitHub Copilot, and Claude Code).
- [Review](#review): seeing what a pull request changes as readers will see it, and commenting on it there.

Ascribe's own docs, and the drift checks built on them (generated reference, coverage, and snippets from section 1), are planned in [docs/](docs/README.md).

## What the ideas are judged against

- **Markdown stays the source of truth.** SPEC principle 1: source readability wins. A feature that writes source must write the source a person would.
- **Everything is checkable.** The content model declares what's allowed, and mistakes are diagnostics, in the editor as you type and in CI.
- **One model, many outputs.** Builds, variants, and availability make different documentation sets from the same source.
- **Projects are self-contained.** A project's files stay inside its folder, and a workspace can hold several projects.
- **The language server is the engine.** The editor, the preview, and the CLI share it, so a capability added to the server reaches all of them.

---

# Product features

## 1. Code snippets from tested code (Bluehawk, built in)

### What Bluehawk does today

[Bluehawk](https://github.com/mongodb-university/Bluehawk) (MongoDB) extracts code examples from real, tested source files. Authors annotate the code with tags in comments:

```swift
import RealmSwift

// :snippet-start: person-model
class Person: Object {
    @Persisted var name: String = ""
    @Persisted var age: Int? // :emphasize:
}
// :snippet-end:
```

Its tags include `snippet`, `remove` (and the one-line `// :remove:`), `replace` (with a JSON attribute list), `uncomment`, `state` / `state-uncomment` / `state-remove` (versions of one example, such as a tutorial's start and finish), and `emphasize`. `bluehawk snip` writes each snippet to its own file (`Models.snippet.person-model.swift`), which the docs then include. `bluehawk check` validates the tags, and `bluehawk copy` makes tutorial branches. The tests are the project's own; Bluehawk only extracts.

### What Ascribe would add

Ascribe can drop the middle step: a page refers to the snippet in the source file, and Ascribe extracts it when it checks, builds, or previews.

```markdown
@snippet: ../examples/swift/Models.swift#person-model
```

- **No generated files.** Nothing to regenerate, commit, or let drift: Bluehawk's main source of stale examples.
- **Checked like everything else.** A missing snippet, unbalanced `-start`/`-end` tags, an unknown state, or a malformed `replace` list are diagnostics. The language server can report them in the tagged code file too, not only in the page.
- **Editor navigation.** Go to definition from the page jumps to the snippet in the code; hover shows the extracted code; a CodeLens in the code says "Used by 3 pages".
- **A live preview.** Editing the Swift file updates the preview of every page that uses the snippet.
- **States meet builds and variants.** Bluehawk's states map onto Ascribe's model: `@snippet {state=start}` inside a `@variant` arm, or a build that picks a state for a whole tutorial.

### Hard parts and questions

- **Compatibility.** Reading Bluehawk's tag syntax as it is gives MongoDB-style repositories a migration path. Its GitHub license is shown as "other" (unrecognized), so reimplement the syntax rather than copy code, after checking the license.
- **Where code may live.** Projects now keep their files inside their own folder, and code usually sits beside the docs (`service/` in `examples/monorepo`). The content model needs an explicit allow-list, such as `[snippets] roots = ["../service", "../examples"]`, checked like the other boundaries.
- **Comment syntax per language.** Bluehawk knows each language's comment and string syntax so tags inside strings aren't tags. Ascribe needs the same table, and a way to extend it.
- **Out of scope:** running the code. Tests stay the project's own; CI runs them.
- **Directive design:** a new `@snippet`, or `@include` taught to read code? Attributes for language, line highlighting (`emphasize`), and dedenting.

## 2. Editable previews

### The idea

Edit in the preview instead of in Markdown, with changes written back to the source file.

### The risk

A general WYSIWYG editor that serializes back to Markdown tends to produce source no one wants to read or review: escaped characters, reflowed lines, reordered attributes. That breaks principle 1. So build it in layers, each useful on its own, and stop before a full rich-text editor unless writers ask for one.

### Layers

1. **Structural editing.** Click an element in the preview to change it with a small control:
   - a note's type, from a menu;
   - the order of steps or tabs, by dragging;
   - an availability badge, in a small form ("cloud, self-hosted beta 2.4");
   - whether a `@details` is open, a tab's label, an image's width.

   Each action becomes a source edit sent through VS Code, so undo, the dirty marker, and git behave as for typing. Most of the parts exist: the preview already maps elements to source positions (it scrolls to the cursor), the server already makes edits (code actions, rename), and `fmt` keeps the result canonical.
2. **Insert from a palette.** Add a note, steps, a tab group, an include, or a snippet from a menu in the preview, written in canonical form at the right place. This removes most of the need to remember directive syntax.
3. **Text editing in place.** Double-click a paragraph, heading, or list item to edit its text, limited to plain text and inline marks, mapped back to that block's source range.
4. **A full rich-text editor** (ProseMirror or similar). Only if writers need it; the hard part is a lossless mapping between the editor's model and Ascribe's syntax tree.

### Questions

- Who would use it: writers who avoid Markdown, or everyone, for structure?
- How should a preview edit and a concurrent edit in the text editor reconcile? (VS Code's document versions help; the server already rejects stale versions.)

## 3. OpenAPI

### Levels

1. **Checked links into the spec.** `[Get a user](../api/openapi.yaml#getUser)`, checked like heading links, with completion of operation ids and hover showing the operation. Cheap and immediately useful.
2. **Embeds.** `@openapi-operation: getUser` in a hand-written page renders the method, path, parameters, request and response schemas, and examples. Writers keep the narrative; the spec supplies the facts.
3. **Generated reference pages.** Virtual pages, one per tag or operation, built from the spec at build time and never written to disk, with hooks for authored prose around them.

### What Ascribe could do that other tools don't

- **Check examples against schemas.** JSON in a page's code blocks that claims to be a request or response is validated against the spec, so a stale example is an error.
- **Coverage.** Report operations no page documents.
- **Versions.** Map API versions onto builds and variants; availability from a spec extension such as `x-ascribe-available`.

### Questions

- Embedded rendering needs elements in `@ascribed/elements` (operation, schema, example), and an Astro story.
- Start with OpenAPI 3.1 only? GraphQL and AsyncAPI would follow the same design later.
- Generated pages raise navigation and URL questions (see [navigation](#navigation-and-site-structure)).

## 4. Editor UI: an actions bar, a sidebar, and more

Planned in [editor-ui/](editor-ui/README.md), in nine phases.

### An actions bar for where the cursor is

One key opens a menu at the top center of the window, where the Command Palette appears, listing only the actions that apply to the cursor or selection right now. It's for writers more than power users: plain-language titles, a one-line description on each, typing to filter, and short wizards in the same box when an action needs input. It's built on VS Code's quick pick (`vscode.window.createQuickPick`), bound to a key with a `keybindings` contribution and active only in Ascribe projects.

**Every action has another path.** Each action is defined once, in one registry: its title, description, the contexts it applies to, and what it does. That definition feeds:

- the Command Palette (`Ascribe: Wrap in a note`), which users can also bind to keys;
- the lightbulb (`Cmd+.`), where an action is a natural fix or refactor;
- an **Ascribe** submenu in the editor's context menu;
- the actions bar.

The bar is a filtered view of commands that exist anyway, never the only way to reach one. A test checks that every registered action has a palette command.

**Why not only the lightbulb.** `Cmd+.` is context sensitive too, but it's built for developers: it mixes every extension's fixes and refactors, its titles are terse, it has no groups or descriptions, and it can't ask follow-up questions. The bar is the writer-friendly front end to the same actions.

**What it offers.** The language server works out the context from the parsed page (a custom request, like `ascribe/preview`):

| Where the cursor is | Actions |
|---|---|
| On a problem | Its fixes, listed first |
| Selected prose | Wrap in a note (pick its type); wrap in expandable details; make it a link (pick a page or section); make it a phrase; add it to the glossary |
| An empty line | Insert a note, steps, tabs (pick a dimension and its values), expandable details, a fragment, an image (pick a file, then write alt text), or a project widget (its attributes as prompts) |
| A list | Make it a procedure (`@steps`) |
| A heading | Give it a fixed id; copy a link to this section; mark where it's available |
| A note | Change its type; turn it into expandable details; remove the note but keep its text |
| A tab group | Add a tab; remove this tab |
| A link | Change where it points; use the target's title as its text |
| An image | Set its width; edit its alt text |

**For writers.**

- Plain words before syntax: "Wrap in a note", with `@note` in the description.
- The detail line previews what will be inserted.
- Wizards ask ("Which dimension?", "Which values?") instead of expecting the syntax.
- Every edit is in canonical form, and undoes in one step.

**Out of scope for now:** anything beyond the editor, such as running checks or builds, or opening the published page; and modes for power users, such as prefixes that search pages or headings.

### A sidebar

An **Ascribe** view container in the activity bar, starting with tree views rather than a form editor for `ascribe.toml`:

- **Projects:** each project, its server's state, its editor build; buttons to restart it or open its output. This builds directly on the multi-project work.
- **Used by:** for the current page or fragment, what links to it and what includes it.
- **Pages:** grouped by type, with fragments and what includes them, and orphaned pages flagged.
- **Content model:** types, dimensions, phrases, features, glossary terms, widgets, and builds, each with a count of the pages that use it, and a jump to its declaration.

Actions on the content model belong in the actions bar's registry too, so they appear in the palette and the bar:

- **Make the selection a phrase**, replacing its other occurrences.
- **Add the selection to the glossary.**
- **Promote a feature** ("Promote audit-log to GA"), edited in `ascribe.toml`.
- **Rename a dimension value or phrase key** across every page.

Edits to `ascribe.toml` must keep its comments, so the server makes them with a round-trip TOML editor (`toml_edit`). A full form editor for `ascribe.toml` can come later, if at all: with the sidebar and actions, it matters much less.

### A status bar item

The active file's project and editor build ("Ascribe: docs · site"). Clicking it switches the editor's build or opens the project's output. In a multi-project workspace it answers, at a glance, which project a file belongs to.

### A build lens

Choose a build, and the editor dims what that build leaves out: other variant arms, and sections whose availability excludes it. "What will self-hosted readers see?" is then answered in the source, without opening the preview. It uses editor decorations and data the server already computes for builds.

### A getting-started walkthrough

VS Code's walkthrough contribution, shown after install: create an `ascribe.toml`, write a page, open the preview, add `ascribe check` to CI. Cheap, and it helps new teams start.

## 5. Prose linting with Vale

### Why Vale, not rules of our own

[Vale](https://vale.sh) is the established prose linter for documentation: a project's `.vale.ini` picks styles (Microsoft, Google, write-good, or the team's own), and rules are YAML files (existence, substitution, occurrence, capitalization, spelling, and more). Teams already have Vale styles. Building a rule language of our own would duplicate it and split the ecosystem, so Ascribe should run Vale, not replace it.

### What goes wrong today

Running Vale on Ascribe source works badly:

- **Directive lines are read as prose.** `@note {type=tip}`, `@variant {pm=npm}:`, and `@end` get flagged, unless each project writes `BlockIgnores` and `TokenIgnores` patterns in `.vale.ini`.
- **Phrases are unresolved.** `{product}` isn't "Lantern", so spelling and capitalization rules see the wrong text.
- **Two sets of squiggles.** Vale's own language server and Ascribe's both report on the same file, with no shared idea of what's prose.

### What Ascribe would do

Ascribe knows exactly which text is prose: it parses directives, attributes, code, phrases, and includes. So it can give Vale the prose and map the results back:

1. **Extract** each page's prose, with phrases substituted, and a map from every character back to its place in the source.
2. **Run Vale** on it (`vale --output=JSON`, reading the project's own `.vale.ini` and styles).
3. **Report** each alert as an Ascribe diagnostic at the right place in the source, named after its Vale rule (`Microsoft.Contractions`), with Vale's suggested replacement as a quick fix. One set of squiggles, one Problems list, and the same results in `ascribe check` for CI.

### Feeding Vale what Ascribe knows

- **Vocabulary.** Phrase values, glossary terms, and dimension labels are the project's own words. Ascribe can write them into a Vale vocabulary (`accept.txt`), so `Lantern Cloud` is never a spelling error.
- **Per-build linting.** Lint the text a build's readers see, with variants and availability resolved, if a team wants rules to apply to output rather than source.

### Questions

- **Ship Vale or require it?** Vale is a Go binary under the MIT license. Bundling it in the npm platform packages and the VS Code extension makes linting work with no setup; requiring an install keeps packages smaller and the version the team's choice. Probably: use the project's Vale if there is one, configured as `[lint.vale]` in `ascribe.toml`.
- **When to run.** Vale is fast, but not keystroke-fast across a project. On save and on open in the editor, and in full in `ascribe check`.
- **Severity.** Map Vale's `suggestion`, `warning`, and `error` onto Ascribe's levels, and decide whether Vale warnings count under `--deny-warnings`.

## 6. Other CMS features

In order of how naturally each builds on what exists.

### Review dates and a content inventory

`owner` and `review-by` fields, with a warning when a page is overdue; `examples/monorepo`'s security policies already model this. From there, an inventory: pages by owner and type, overdue pages, orphans, and unused fragments, phrases, glossary terms, and features. It could be a report from the CLI (`ascribe inventory`) and a view in the extension.

### Navigation and site structure

Ascribe doesn't define sidebars, ordering, or tables of contents; the site generator does. If that's a gap rather than a choice, a checkable navigation file would catch missing pages, orphans, and inconsistent order across builds, would give generated pages (OpenAPI) a place to go, and would give `llms.txt` (section 7) its sections.

### Redirects

Renaming or moving a page already updates the links to it. It could also record a redirect from the old published URL, emitted with the site, so external links and bookmarks keep working. Agents need this too: the Web Documentation Delivery Spec asks for same-host HTTP redirects when content moves (section 7).

### Differences between builds

"What does the self-hosted build leave out compared with the site build?" A report of pages, sections, and variant arms per build, for release reviews. The build machinery already computes the answer.

### Translation

Per-page translation status from a hash of the source: a translation is marked stale when the source page changes, with the changed sections listed.

### Others worth noting

External link checking (opt-in, for CI); a PDF or print output; image checks (unused images, large files).

---

# Agents

Sections 8 to 13 are planned in [agents/](agents/README.md), in eleven phases, checked against a [research report](../reports/Agent%20first%20interfaces%20for%20docs%20tools.md), with one idea added on 2026-10-04: a **Prompt agent** action on problems, review comments, and changed pages, which builds a prompt for the user's own agent. Section 7 isn't in that plan.

Agents meet documentation in two ways: they **read** published docs while they code, and they **write** docs in a repository. Ascribe can help with both, and it's unusually well placed for the first, because it already builds a resolved, plain-Markdown version of every page.

## 7. Docs that agents can read: the Web Documentation Delivery Spec

### The spec

The [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) (draft 0.6.0, 2026-09-13, by Dachary Carey and community contributors) says how documentation sites should serve coding agents such as Claude Code, Cursor, and GitHub Copilot when they fetch docs during development. It defines 28 checks in seven categories, with a reference checker, [`afdocs`](https://github.com/agent-ecosystem/afdocs). The essentials:

- **`llms.txt`** at the site's root, under 50,000 characters, listing the pages: an H1 with the site's name, a blockquote summary, and H2 sections of `[Title](url): description` links. Large sites nest: the root `llms.txt` links to one per section. The spec calls this "the single highest-impact action".
- **A Markdown version of every page**, at a `.md` URL (`/docs/api.md` beside `/docs/api`), by content negotiation (`Accept: text/markdown`), or both. `llms.txt` should link to the Markdown versions.
- **A pointer to `llms.txt` on every page:** a visually hidden element near the top of the HTML ("For AI agents: a documentation index is available at /llms.txt", not `display: none`), and a blockquote at the top of the Markdown version, with an absolute URL.
- **Pages under 50,000 characters** of Markdown, since agent platforms truncate silently, with prose before bulk content so truncation loses data rather than explanation.
- **Content structure:** tabbed content split into separate pages or requestable per variant; variant context in headings ("Step 1 (Python)", not "Step 1"); valid code fences; absolute links in Markdown.
- **Hosting:** honest status codes, same-host redirects, cache headers, no bot protection on public docs.

### How Ascribe meets it

Most of the spec is about generated output, which Ascribe owns:

| The spec asks for | Ascribe today | What to add |
|---|---|---|
| A Markdown version of each page | The `plain` output: resolved CommonMark, "for LLM consumption" (SPEC §9.4) | Publish it beside the HTML at `.md` URLs, through `@ascribed/astro` |
| Absolute links in Markdown | Done when `[consumer] site` is set | Make `site` required, or warn, when publishing for agents |
| Valid code fences | Done: the output is generated, never hand-written | — |
| `llms.txt` and nested `llms.txt` | Ascribe knows every page's title, description, and route, per build | Emit `llms.txt` with each build, split into nested files when it would pass 50,000 characters; sections from [navigation](#navigation-and-site-structure), or from folders until then |
| A pointer to `llms.txt` on each page | — | A hidden element in the site output and a blockquote in the plain output, from the build |
| Pages under 50,000 characters | — | A check: the plain output of a page, per build, over the limit is a warning in `ascribe check` and the editor |
| Tabbed content an agent can read | Variants become labeled sections, with bold labels | Variant labels as headings, with context ("Install the CLI (macOS)"); and per-variant Markdown, since a build already selects variants: `cloud` and `self-hosted` builds each have their own pages and their own `llms.txt` |
| `llms.txt` covers every page; Markdown matches HTML | — | True by construction: both come from one resolved page |
| Same-host redirects | — | The [redirects](#redirects) idea, emitted with the site |
| Status codes, caching, bot protection, authentication | — | Hosting, out of Ascribe's reach; the docs can say what to set |

So for an Ascribe site, most of the spec could pass by default, and a docs team gets agent-ready output by upgrading Ascribe rather than by building a second pipeline.

### Testing it with `afdocs`

[`afdocs`](https://github.com/agent-ecosystem/afdocs) is the spec's checker (MIT, npm package `afdocs`, Node.js 22 or later; v0.22.2 implements spec 0.6.0). It fetches a site over HTTP, discovers pages from `llms.txt` and the sitemap, samples them, and runs the 28 checks:

```sh
npx afdocs check https://docs.example.com --format scorecard
```

- **Output** is `text`, `scorecard` (an overall score, scores per category, and a fix for each failing check), or `json` (with `--score`), for scripts.
- **Sampling:** `--sampling deterministic` gives the same pages on every run, which CI needs; `--urls` or a `pages` list checks chosen pages.
- **Configuration** lives in `agent-docs.config.yml` (the URL, `checks` or `skipChecks`, sampling, and pages).
- **CI helpers for vitest:** `describeAgentDocsPerCheck()` from `afdocs/helpers` makes each check a test case; a failing check fails the test, and a warning doesn't.
- **A build, not only the live site.** Its CI guide recommends checking the built site on pull requests: build, serve the output, and check `localhost`, with `--canonical-origin` mapping the production URLs in `llms.txt` and the sitemap onto the local server, and `skipChecks` for what only the real host can pass (`content-negotiation`, `cache-header-hygiene`).

How Ascribe would use it:

1. **In Ascribe's own CI.** The Astro end-to-end job already builds `examples/astro-site`. Add a step that serves the build (`astro preview`) and runs `afdocs` against it through the vitest helpers, which match the repository's existing vitest setup. Every check Ascribe claims to pass becomes a test, so a regression in `llms.txt`, the `.md` pages, or the page-size limit fails CI.
2. **As a measure of progress.** Run it once on today's `examples/astro-site` to get a baseline score and the list of failing checks; that list is the work for this section.
3. **For Ascribe's users.** Document the same recipe in `docs/astro.md`: a config file, a test file, and the CI step, with the hosting checks named so teams know which ones are theirs.

`afdocs` is 0.x: its check ids, flags, and output may change between minor versions. Pin its version, and update it deliberately alongside the spec version Ascribe targets.

### Questions

- **Content negotiation** needs the web server's help. Ascribe can emit the configuration (an Astro middleware, or headers files for common hosts), but not apply it.
- **Variant URLs.** Per-build pages cover editions and versions. A dimension such as `platform`, which the site shows as tabs, either stays as labeled sections in one page, or gets per-value pages (`getting-started.macos.md`). `afdocs`'s `tabbed-content-serialization` check, run on both, can help decide.
- **Follow the spec's changes.** It's a draft (0.6.0); its version should be pinned in the docs and its changelog watched.

## 8. Diagnostics agents can use

The rest of this group is about agents that write documentation. The goal is one feedback loop agents can run themselves: **write, check, fix**, with Ascribe's knowledge of the content model, the ids, the phrases, and the diagnostics. Each harness gets that loop through its own channels.

### What already works

GitHub Copilot's agent mode and Claude Code's VS Code extension both read VS Code's diagnostics, so what the language server publishes already reaches agents. Good diagnostics are agent features.

### Improvements

- **Say the fix, not only the fault.** "`./missing-page.md` doesn't exist; did you mean `guides/missing-page.md`?"
- **Link each code to its documentation** with LSP's `codeDescription` (`docs/diagnostics.md#…`).
- **Expose fixes as code actions.** Agents apply them reliably.

### A gap to close first

With `ascribe.startServers: "onDemand"`, a project's server starts when someone opens one of its files. An agent that edits files on disk without opening them gets no diagnostics, and concludes the page is clean. Start a project's server when one of its files changes on disk too; the extension already watches for `ascribe.toml` changes, so this is a small addition (and related to [#56](https://github.com/ascribed-dev/ascribe/issues/56)).

## 9. A CLI for agents

Every harness can run shell commands, so this layer reaches all of them. `ascribe check --format json` exists; these are missing:

| Command | What it gives an agent |
|---|---|
| `ascribe check path/to/page.md` | One file checked in its project's context, for a fast loop. |
| `ascribe check --stdin --path guides/new.md` | Text checked as if it were that file, before it's written. |
| `ascribe explain ASC036` | A diagnostic's meaning, an example, and the fix. |
| `ascribe model --format json` | The resolved content model: page types and their required frontmatter, dimensions and values, phrases, features, glossary, widgets, builds. What an agent most needs to write a valid page. |
| `ascribe outline page.md` | Headings and their ids, so `page.md#id` links are right the first time. |
| `ascribe render page.md --build self-hosted --format text` | What a reader of that build sees: variants resolved, phrases substituted. |

Each is a thin wrapper over something the server already computes. Output should be compact, since it's spent from an agent's context.

## 10. An MCP server: `ascribe mcp`

The same capabilities as typed tools, which both harnesses prefer to shell commands.

- **Stateless and multi-project.** Every tool takes a path and finds the nearest `ascribe.toml`, so one server handles a monorepo.
- **Read-only by default.** Tools that would change files return edits rather than apply them; the harness's own edit tools, with their approval prompts, write.
- **Tools:** `check` (files, or unsaved text at a path); `explain`; `model`; `outline`; `resolve_link` ("does `keys.md#rotate-keys` exist, and what's its title?"); `references` (where a page, id, phrase, fragment, or feature is used); `render`; `format`; `rename` (edits, as a dry run).
- **Resources:** a directive cheat sheet, the content model, and spec sections, so an agent reads the rules when it needs them instead of guessing.
- **Prompts:** "new page of type X", with the frontmatter the type requires.

## 11. GitHub Copilot

### In VS Code

The extension can use three channels:

1. **Register the MCP server from the extension** (`contributes.mcpServerDefinitionProviders` and `vscode.lm.registerMcpServerDefinitionProvider`). Copilot's agent mode then has `ascribe mcp` with no setup, using the binary the extension already resolves per project.
2. **Language model tools** (`contributes.languageModelTools` and `vscode.lm.registerTool`). Unlike MCP tools, these run inside the extension, so they see unsaved buffers and the running language servers: the right home for "check what's in the editor now" and "render the preview". Writers can reference them in chat (`#ascribeCheck`).
3. **Instruction files Copilot reads on its own:** `.github/copilot-instructions.md`; path-scoped `.github/instructions/ascribe.instructions.md` with `applyTo: "docs/**/*.md"`, so the rules load only when editing docs; and prompt files such as `.github/prompts/new-page.prompt.md`.

A chat participant (`@ascribe`) is possible but works only in Copilot Chat; skip it unless there's demand.

### Copilot's cloud coding agent

It runs on GitHub without VS Code. The MCP server (configured in the repository's settings) and a `copilot-setup-steps.yml` that installs `@ascribed/cli` give it the same loop.

## 12. Claude Code (CLI and its VS Code extension)

- **A hook for automatic feedback.** A `PostToolUse` hook on `Edit|Write` for Markdown files runs `ascribe check <file> --format json` and returns any errors to Claude, so every edit is checked without Claude remembering to. The most effective single item for Claude Code, and it needs only the per-file check in [section 9](#9-a-cli-for-agents).
- **A plugin** bundling a skill ("Writing Ascribe documentation", loaded only when relevant), the MCP server, the hook, and commands such as `/ascribe:new-page` and `/ascribe:check`. Installed in one step from a plugin marketplace; the same plugin works in the CLI and in the VS Code extension.
- **Project files** for teams without the plugin: `.mcp.json` at the repository root, and `CLAUDE.md`.

## 13. One source for agent instructions

`AGENTS.md`, `CLAUDE.md`, Copilot's instruction files, and a skill all want the same guidance. Generate them from one source, the project's content model plus Ascribe's directive reference:

```sh
ascribe init --agents        # writes or updates AGENTS.md, CLAUDE.md, .github/instructions/…
```

- The generated part sits between markers, so it can be regenerated without touching what the team wrote.
- It's short: the page types and their required frontmatter, the phrases to use instead of literal names, the directives this project uses, and "run `ascribe check` before you finish".
- It's per project: an agent in the security handbook learns about `review: quarterly|yearly`, not the docs project's `since`.

---

# Review

## 14. Rendered changes and comments

Added 2026-10-03, after two research reports: [Docs as code pain points](../reports/Docs%20as%20code%20pain%20points.md) and [Ascribe fit for docs pain points](../reports/Ascribe%20fit%20for%20docs%20pain%20points.md). Planned in [review/](review/README.md), in eight phases.

### The problem

Reviewing docs in a pull request means choosing between the rendered page and the comments. GitHub's rendered view of a Markdown change takes no comments, and its source view shows lines, not what a reader sees. Preview deployments show the page but not what changed. Reviewers of AI-written changes feel this most, and nothing above addresses it.

### The idea

One comment overlay, with three views of the same review:

- **The site preview.** The real page, in the site's own layout, from the site generator's dev server. Changed blocks are marked, and the pull request's review threads sit beside the blocks they refer to.
- **The page preview.** The same marks and threads on Ascribe's own instant render of the page alone, in VS Code.
- **The source files.** The threads on the lines they were made on, as today.

Each view links to the other two: "open source" from a block or a thread, "open site preview" and "open page preview" from a file.

### What Ascribe adds that a Markdown diff can't

- **Pages, not files.** "This pull request changes 7 pages, 3 of them through a fragment." A change to a fragment, a phrase, or `ascribe.toml` changes pages whose own files didn't change, and only a tool that resolves the project can say which.
- **Per build.** "What does this change for self-hosted readers?"
- **The real site.** Comments in the context a reader will have, not on a bare render.

### How

- **Source anchors.** In a review mode, the site output marks each block with the source file and lines it came from, through includes. Whatever renders the output carries the anchors into the page. They are what the overlay attaches to, and they don't depend on the site generator.
- **`ascribe diff`.** Resolves the project at a base revision (read through `git`) and now, compares the resolved pages per build, and reports changed pages and blocks. It needs `git` and nothing else, and it can write a static HTML report for CI.
- **Threads from GitHub.** Review threads are read and written through the GitHub CLI (`gh`), or VS Code's GitHub sign-in in the extension. GitHub stays the only store: a comment made on a rendered block is an ordinary review comment on the file and line it came from.
- **Hosts for the overlay.** The page preview in VS Code, and Astro's dev toolbar through `@ascribed/astro`.

### Hard parts and questions

- GitHub anchors a comment only to a file the pull request changed. A comment on a page that changed only through a fragment has to go on the fragment, or become a pull request comment that remembers its block.
- A site's layout or components can drop the anchors. The overlay has to say when a thread has nowhere to attach, and list it anyway.
- A reviewer needs a checkout and the site running. Reviewers without a GitHub account, or without a checkout, need a hosted service: deployed previews, sign-in, and stored comments. That's the one place a paid tier could fit, and the research says it's premature.
- GitHub only, or GitLab too?
- VS Code has an experimental rendered Markdown diff (August 2026), without comments. If it gains them, the page preview matters less; the site preview and resolved content still would.

---

# Suggested order

1. **Agent-friendly diagnostics and CLI** (sections 8 and 9): start servers on file changes, clearer messages with links, per-file and stdin checks, `explain`, `model`. Cheap, and every harness benefits.
2. **Docs agents can read** (section 7): first a baseline `afdocs` run on `examples/astro-site`, then `llms.txt`, `.md` pages, the per-page pointer, and the page-size check, with `afdocs` in CI to hold the result. Mostly new outputs from what the builds already compute, and it serves every reader of an Ascribe site, not only its authors.
3. **The Claude Code hook and `ascribe init --agents`** (sections 12 and 13): the tightest writing loop for the least work, built on step 1.
4. **Review** (section 14): source anchors and `ascribe diff` first, since the static report and both overlays build on them. The research ranks review as the gap that drift and AI-written changes both drain into.
5. **Vale** (section 5): prose linting with the rules teams already have, without directive noise.
6. **Code snippets** (section 1): the most valuable product feature for docs-as-code teams, and better than Bluehawk because Ascribe can check it.
7. **`ascribe mcp`** (section 10), read-only tools first.
8. **Editor UI** (section 4): the actions bar first (writers gain the most), then the status bar item, the Projects and Used-by views, and the build lens.
9. **VS Code integration for Copilot** (section 11): registering the MCP server, then language model tools.
10. **Structural editing in the preview** (section 2, layers 1 and 2).
11. **OpenAPI links and embeds** (section 3).
12. **A Claude Code plugin and Copilot prompt files**, packaging what exists.

Harness and spec details change quickly: VS Code's MCP and language model APIs, Copilot's instruction formats, Claude Code's hook and plugin formats, and the Web Documentation Delivery Spec (a draft). Check each against current documentation when designing that phase.

# Open questions

- **Who are the main users?** Writers on a docs team favor the preview editing, Vale, and CMS features; engineers documenting their own code favor snippets, OpenAPI, review, and agent support. The research found engineers feel drift most and do the reviewing.
- **Bluehawk compatibility:** read existing Bluehawk-tagged code as is, or design new tags?
- **Vale:** bundle it, or use the project's own install?
- **The actions bar's key:** one that's free on macOS, Windows, and Linux, and doesn't clash with VS Code's own (`Cmd+.` is the lightbulb).
- **Navigation:** deliberately left to the site generator, or a gap? `llms.txt` sections and OpenAPI pages both want an answer.
- **Variants for agents:** labeled sections in one Markdown page, or a page per variant value?
- **Which agent harnesses come first:** Copilot, Claude Code, or both?
- **Should Ascribe ever apply edits for an agent** (`rename` applying its changes), or always return edits for the harness to apply with its own approvals?
- **Review beyond a checkout:** is a hosted review service (guest reviewers, deployed previews) ever in scope, and would it be paid?
- **Review hosts:** GitHub only, or GitLab too?
