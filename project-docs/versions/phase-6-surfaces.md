# Phase 6: Every surface

Part of [Versions](README.md). Needs phase 4.

## Goal

Working in a project with a window is as quick and as quiet as working in one without. The editor shows a page as one line and can switch. A pull request's review lists a changed page once, with the lines it differs in. The tools an agent calls take a line's name. And a build does a page's work once when the page is the same in every line, which most are.

After phase 2 every line is resolved in full and each surface treats a line as one more build. That's correct, and with five live lines it's five times the work and five times the list.

## Context

- The [proposal](../proposals/versions.md): "Names, and work saved".
- Phase 1's decision 6.
- `crates/ascribe-resolve/src/incremental/`: what's kept between builds, and what decides that a page must be resolved again.
- `crates/ascribe-lsp/src/`: the build lens, `[editor] build`, and the preview.
- `crates/ascribe-diff/`: `diff`, `drift`, and the HTML report. `packages/review`: the report's script, and review in `astro dev`.
- `crates/ascribe-mcp/`: the tools, and which take a build.
- `project-docs/checklists.md`: "A new editor command, action, or setting".

## Design

### A page that's the same in every line

A page's output can differ between lines only through what a line decides: content removed, a planned mark, a link that resolved by phase 4's second or third step, the phrase for the line, and its routes. A page none of those touch is resolved once and written to each line with its routes.

- Sameness is decided from the resolved page, not guessed from the source: a page with no mark for the target can still differ, through an include or a link.
- `ascribe build` says how many pages it resolved once.
- The outputs are byte for byte what phase 5 wrote. This is a clean-up in the repository's sense, and the comparison shows it.

A measurement comes first: `examples/quill` and the docs with three lines declared, before and after, in the pull request. If the saving is small, the phase says so and keeps the simpler code.

### Check

A problem on a page that's the same in every line is found once and reported as phase 1's decision 6 has it. `builds_checked` still lists every line. The summary counts a problem once.

### The editor

- **The build lens** lists a versioned build's lines under it, the current one first, and choosing one sets the build for the session.
- **`[editor] build`** takes `site@3.4`, as it has since phase 2.
- **Diagnostics** follow the chosen line, and a problem that's only in another line is still reported, saying which.
- **The preview** draws the chosen line, with its links out marked.
- **A dimmed block:** content that isn't in the chosen line is shown as content a filter build removes is shown today.
- **Hover** on a version in a mark says which line it's in and the line's status, and on a `replaces` line, the page.

### Review and diff

- `ascribe diff` and the HTML report list a changed page once, with the lines it changed in: "in every line", or the lines by name.
- The report's page view has the list of lines, and switching shows that line's rendering of the change.
- Review in `astro dev` covers the line being viewed.
- `ascribe drift` is about sources and needs no change beyond naming a line where it names a build.

### The tools

Every tool and command that takes a build takes a line's name, and says the lines in its answer where it lists builds: `ascribe render`, `outline`, `link`, `refs`, `model`, and the MCP tools that wrap them. `ascribe model` reports `[releases]`, with each line's status.

## Tasks

1. Measure. Put the numbers in the pull request.
2. Resolve once, behind the comparison that shows no output changed. Its own pull request, labeled `optimization`.
3. `check`'s counts and text for a problem in every line.
4. The build lens, the preview, the dimmed block, and the hovers.
5. `diff` and the report listing a page once, and the report's list of lines.
6. The tools and commands that take a build.
7. The editor's and the tools' pages in the docs, entries in `changes/`, and the map of the surface.

## Out of scope

- A reader's switcher in the site. Phase 5.
- Anything about an archived line in the editor.

## Acceptance criteria

- The pull request for task 2 changes no output in any project, and says what it saved.
- In `examples/quill`, the editor's lens offers both lines, and the page new in 3.5 is dimmed whole when 3.4 is chosen.
- A change to a shared page shows in the review report once, "in every line"; a change inside a block marked 3.5 shows in one.
- `ascribe render install.md --build site@3.4` renders that line, and the `ascribe_render` tool does the same.
- `ascribe check`'s summary counts a problem on a shared page once.

## Verify

```sh
cargo test --workspace --locked
pnpm test
node scripts/compare/outputs.ts --base main
```

## Commits

1. "Resolve a page once when every line has it the same"
2. "Count a problem in every line once"
3. "Choose a line in the editor"
4. "List a changed page once in review, with its lines"
5. "Take a line's name wherever a build is named"
