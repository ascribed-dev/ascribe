# Search

An author or an agent asks where a project's pages say something, and gets the answer a reader would: the words as a build publishes them, each reported as the place in the source to edit. `grep` finds `Sign in to {cloud}`; `ascribe search` finds "Sign in to Quill Cloud", and says it's on line 62 of `install-agent.md`, through the phrase `cloud`. The idea, the measurements behind it, and the four questions the maintainer settled are in the [proposal](../proposals/search.md). This is the plan for building it.

## Who it's for

- **An agent about to write a page,** which should ask "where do the docs already explain rate limits?" first, and today can only search text that isn't what readers read.
- **A writer** changing how the docs name something, who needs every place a reader meets the old name, including the ones a phrase or a fragment puts there.
- **A reviewer** who sees a sentence in the preview and wants the line that wrote it.

## What exists today

The proposal's "What exists" still holds. What has changed since it was written, on 11 October:

- **Phrases can differ by release line.** A phrase whose value is a line's (`phrase_lines` in `crates/ascribe-model/src/loader.rs`) reads one way in one versioned build and another way in the next, so one source line can read differently, and match differently, in two builds.
- **Memos exist,** and nothing reads their contents ([decision #536c](../decisions/536c-nothing-reads-a-memo.md)). Resolution removes them, so the reading never has one. Phase 3's scan of the source, which reads files as written, leaves them out too.
- **The navigation is published** in `_ascribe/nav.json`, with each section's and page's label. The proposal's settled decision on fields (decision 3 below) keeps it out of search, as the site's sidebar is.
- **The Quill example's line** is 62 now, not 61.

The pieces search stands on, checked against `main` on 20 October:

- **The resolved page** (`ResolvedBlock` in `crates/ascribe-resolve/src/build/tree.rs`): each block's file and span, the includes it came through (`via`), the phrases substituted in its own inline content (`substitutions`), its links (`links`), its rows, its snippet, and its heading's ids.
- **`ascribe diff`'s block text** (`crates/ascribe-diff/src/tree.rs`, `TreeBuilder`): a resolved page as a tree of nodes, each with its text "as a reader reads it, whitespace collapsed" and an `Anchor` of the source file, its lines, and its includes. Exact to the block.
- **The Vale map** (`crates/ascribe-check/src/prose/extract.rs`, `Prose`): a page's prose as a list of pieces, each `Copied`, `Made`, or `Phrase`, and `Prose::locate`, held by a property test that every character has a place. Exact to the character, but over one parsed file with every arm.
- **The commands that answer questions** (`crates/ascribe-query`, and `crates/ascribe-cli/src/commands/refs.rs` for the pattern): an entry function returning a typed answer, text with a 50-place limit and JSON with 500, `next_command` when cut, and `Place`'s `file`, `line`, and `column` in Unicode characters.
- **Choosing builds:** `ascribe_check::select_builds`, every build when none is named. The editor's build is `[editor] build` (`ContentModel::editor_default_build`).
- **The MCP server's cache** (`crates/ascribe-cli/src/mcp/cache.rs`) keeps a project loaded until the listing of its files changes; its tools are in `crates/ascribe-cli/src/mcp/tools.rs`.
- **The language server's requests** with generated result types: `ascribe/targets`, `ascribe/inventory`, and `ascribe/page` (`crates/ascribe-lsp/src/`), through `crates/ascribe-cli/src/shapes.rs`.
- **Speed budgets** in `tests/corpora/baselines/perf.json`, recorded on the CI runner, with `lsp/` targets of 50 ms.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

From the proposal's Settled list, answered by the maintainer on 11 October 2026:

1. **Search gets its own map, and `diff` moves onto it; Vale keeps its own.** The map is built by the Vale map's method: pieces, each a range of the text with the source span it stands for, and a property test that every character has a place. It's held to the plain output by a test. `diff`'s block text moves onto it as the last phase, under the clean-up rule, or the move waits.
2. **No regular expressions.** Words, a phrase, `--case`, and `--word` are a substring search. The binary links no engine. `--regex` is reserved, and the reference says the way to run a pattern over resolved text is `ascribe build --emit plain` and `grep`.
3. **The title, and the field with `role = "description"`, and no other frontmatter.** No new key on a type.
4. **When nothing matches, search says whether the source has the words as written:** in no page; in the source at `file:line`, published by no build searched; or, with `--build`, published by builds not named, which it lists. The exit code stays `1`, and the JSON carries `source_places`, cut at three.

From the proposal's design:

5. **The command is `ascribe search`, the tool `ascribe_search`, and the request `ascribe/search`.**
6. **A query is words, which must all be in one block in any order, or a quoted phrase, matched as written,** a run of spaces matching any run of white space. Case is ignored unless `--case`; nothing else is folded. A word matches inside a longer one unless `--word`. A match never crosses blocks.
7. **It matches text, not markup:** a link is its text, an image its alt text, emphasis and code spans their contents.
8. **What's searched is what a build's plain output says,** a page once per build, as that build resolves it, plus decision 3's two fields. What no build publishes, the source's own spelling, text the content model generates, text in images, raw HTML, and what a site adds aren't searched, and the reference says so.
9. **A result is a place in the source,** listed once however many pages or builds show it: a fragment's line once with the pages that show it; a match wholly inside a phrase's value at the phrase's line in `ascribe.toml`; a match partly in one at the line that uses it, `through phrase <key>`; a snippet's code at the code file's lines.
10. **The order is fixed:** places in titles, then headings, then the rest, each in path order and in order within a file. No ranking.
11. **Search never lists uses.** A place a fragment, a phrase, or a snippet puts on many pages names the first three and the `ascribe refs` command for the rest.
12. **Text output is compact:** `file:line:` and the block as a reader sees it, cut around the match, then a line of where a reader meets it (the heading and its id, the arm, `through …`, the builds when not all). At most 50 places, with the command that gives the rest. `--summary` gives pages and counts instead.
13. **`--format json` is a document of its own,** under decision 52's rules, in the proposal's shape. `in` is `title`, `description`, `heading`, `text`, `table`, or `code`, and `phrase` for a place at a phrase's line in `ascribe.toml`, which the proposal's list left out. A place the proposal gives as a region's lines has an `end_line`.
14. **Exit codes:** `0` when something matched, `1` when nothing did, `2` when it couldn't run.
15. **No index, and no new dependency.** The command resolves from cold; the MCP server and the language server keep each build's text between queries.
16. **In the editor, one command and a quick pick,** searching the build the status bar names, with a button for every build. No view in the sidebar.
17. **Agents are told:** the rules `ascribe agents sync` writes, the skill, and the `new-page` prompt each say to search for a subject before writing a page about it.

From this plan:

18. **The query is one argument, and paths come after it:** `ascribe search QUERY [PATHS]...`. Several words are quoted together, `ascribe search "rate limit"`, and a phrase is quoted inside, `ascribe search '"rate limit"'`. The proposal's first example, `ascribe search rate limit`, read as words until the paths, can't tell a word from a folder; the command checklist names a command's pages `PATHS...`, and the tool's `query` is one string. A path that doesn't exist is exit code `2`, and when every argument after the first is a plain word, the message says to quote the query.
19. **The reading lives in `ascribe-resolve`, as `ascribe_resolve::reading`:** a resolved page's text as a reader reads it, block by block, with its map back to the source, the matching over it, and the places found, merged across pages and builds. `ascribe-query`, `ascribe-lsp`, and `ascribe-diff` all depend on `ascribe-resolve`, which owns `ResolvedPage`, and the reading needs nothing above it: phrases, links' targets, snippets' code, and arms are all resolved there. No new crate.
20. **A place's `match` and `column` count Unicode characters,** as every column the commands report does (`Place` in `crates/ascribe-query/src/refs.rs`).
21. **With no `--build`, every build is searched, from the first phase,** as the command checklist asks of a command that covers several. The same line found in several builds is one place from the start, so no phase ships a default a later one changes.
22. **The thread searches each page's own file; a fragment's text comes in phase 2.** Until then the reference says so in a sentence. Text left out says so; it never just fails to match.

No decision phase comes first. The thread ships text output, which isn't a contract. The JSON document and the tool are contracts, and ship in phase 4, once phases 1 to 3 have settled what a place is. That phase records them in `project-docs/decisions/`.

## The thread

**The use case:** in `examples/quill`, `ascribe search '"sign in to quill cloud"'` prints `docs/install-agent.md:62:`, says the line is under "Connect to Quill", in the `deployment=cloud` arm, through the phrase `cloud`, and exits with `0`. `grep -ri "sign in to quill cloud" docs` finds nothing.

**The layers it crosses:**

| Layer | Where | What the thread needs from it |
|---|---|---|
| Resolve | `ascribe-resolve` | Each build's resolved pages, read, not changed; the reading and its map, the matching, and the places merged across builds, in order (decision 19) |
| The query | `ascribe-query` | `search`, the entry: the builds and pages searched, and the cut |
| The command | `ascribe-cli` | `ascribe search`, its arguments, its text, its exit codes |
| The docs | `docs/content/command/questions.md` | The command's reference: what it searches and what it can't |

**The seam it tests** is the map from a match in a reader's text back to a source line, through text that isn't the source's: a phrase's value, and a title a link with no text borrows from its target. A match in "Sign in to Quill Cloud" starts in copied text and ends in a value; the place must be the line that uses the phrase, and say so. If that can't be made exact, results fall back to a block's first line, and a place's shape changes. The JSON waits on it (phase 4). The thread also records the cold query's cost on the runner, so the phases after can see what each adds.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| Fragments' text, and grouping a fragment's line across the pages that show it; snippets' code; the title and the description | 2 |
| Paths and `--type`; `--case` and `--word`; `--summary`; what "nothing matched" says about the source | 3 |
| `--format json`, the MCP tool, and telling agents | 4 |
| The editor | 5 |
| `ascribe diff` on the reading | 6 |

## Phases

| Phase | Result |
|---|---|
| [1: The thread](phase-1-thread.md) | `ascribe search` finds words as readers read them in each page's own text, through phrases, in every build, and names the line to edit. |
| [2: Text from other files](phase-2-other-files.md) | A fragment's line is found once with the pages that show it, a snippet's code at the code file's lines, and a page's title and description at their keys. |
| [3: Narrowing, and nothing found](phase-3-narrowing.md) | A search keeps to folders, pages, a type, case, or whole words, summarizes by page, and when nothing matches, says whether the words are written but not published. |
| [4: For agents](phase-4-agents.md) | An agent searches through `ascribe_search` or `--format json`, and is told to before it writes a page. |
| [5: In the editor](phase-5-editor.md) | **Search Pages as Readers See Them** finds words as you type, in unsaved text, and opens the line. |
| [6: `diff` on the reading](phase-6-diff.md) | `ascribe diff` draws its block text from the reading, and its output is byte for byte the same. |

Phase 1 is the thread, through the riskiest seam. After it, the order is by risk: phase 2 takes the map through includes, snippets' trimmed regions, and YAML, each a place it could be wrong; phase 3 adds what a person narrows by, cheap and with little risk; phase 4 ships the contracts, once a place's shape has met every kind of source; phase 5 meets the language server's snapshot and an as-you-type budget. Phase 6 changes no output and goes last, so the reading has settled before a second reader depends on it.

Each phase's result is something a person or an agent can run, but phase 6's: a clean-up under decision 1, which changes nothing anyone sees, and goes last for that reason.

## Carved out, and waiting

- **`--regex`,** until a second use for an engine arrives (decision 2).
- **A persistent index,** and the cache file the proposal's "When an index would be worth it" describes, under the [cache proposal](../proposals/cache.md).
- **Memos in search.** A memo isn't published, so it isn't what a reader sees. A search of memos would be a search of the source, with its own claim.
- **The navigation's labels** (`nav-label`, a section's `label`). A build publishes them, in `nav.json`, but decision 3 keeps search to the two fields every output carries, and a sidebar is the site's.
- **A search view in the sidebar,** if people keep results open (decision 16).

## The other plans

None is running. The [cache proposal](../proposals/cache.md) is where a kept reading would live, if a cold query ever becomes too slow; nothing in this plan waits on it.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **No output changes.** Search writes nothing. Phases 1 to 5 run `node scripts/compare/outputs.ts --base main` and show every project byte for byte the same; phase 6 is a clean-up, labeled `optimization`, and shows it too.
- **The commands' JSON and the tools change only by a recorded decision.** Phase 4 records them in `project-docs/decisions/`. Go through "A new command or option" in `project-docs/checklists.md` in every phase that changes the command, and "A new editor command, action, or setting" in phase 5.
- **Libraries don't print, and don't panic on input.** The reading and the matching return what they found; only `ascribe-cli` writes it. Project files are read through `ascribe_resolve::FileSystem`, the source scan of phase 3 included.
- **Behavior changes come with tests at the layer that proves them** ([writing-tests](../../.claude/skills/writing-tests/SKILL.md)): the reading's map in `ascribe-resolve`, with its property test; the entry in `ascribe-query`; the command's text in `crates/ascribe-cli/tests/output/`. Search adds no diagnostic, so it needs no conformance case.
- **The command's output the docs show is a tested example:** written by `crates/ascribe-cli/tests/all/output.rs`, shown by `@snippet`, and observed again (`cargo run -p ascribe-cli -- observe`).
- **The map of the surface** is regenerated when a phase changes it (`node scripts/surface/inventory.ts`), and its diff is read against the conventions.
- **User-visible changes** update their page under `docs/content/`, marked `@available: next` where they're new, and add an entry in `changes/command/` (or `changes/agents/`, `changes/editor/`, `changes/review/`) in the same phase. Each phase file names its pages.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

- Everything in the proposal's "What this leaves out": search for readers, search by meaning, replace, searching a git revision, and a persistent index.
- Searching memos, if a writer asks for it, as a search of the source.
