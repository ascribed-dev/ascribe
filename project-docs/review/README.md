# Review

Seeing what a pull request changes as readers will see it, and commenting on it there. One comment overlay shows changed blocks and the pull request's review threads on the site preview and in the page preview, and each view links to the others and to the source files. This plan comes from section 14 of the [brainstorm](../brainstorm.md#14-rendered-changes-and-comments).

## Names

Use these in the UI, the docs, and the code's user-facing strings.

- **Review** is the feature: **Ascribe: Start Review**, **Ascribe: Stop Review**.
- **Page preview** is the page alone, rendered by Ascribe as you type, with no site around it and nothing else running. It's the preview the extension has today (`ascribe/preview`).
- **Site preview** is the same page inside the real site, with its layout, navigation, and styles, from the site generator's dev server. It shows in a browser, or in the preview panel.
- In the preview panel, the switch between them reads **Page | Site**, since the panel is already titled "Preview". The commands are **Ascribe: Open Page Preview** and **Ascribe: Open Site Preview**.

- **Unsent** is the word for a comment that's in the review but not submitted. "Pending" appears only in code that talks to GitHub's API.

Review works in both previews; neither is named after it.

## The mockup

[`mockup.html`](mockup.html) is the main reference for what review looks like and how it behaves. Open it in a browser before starting any phase with UI in it, and work through its **Try** buttons and its **State** menu: between them they show every view and state this plan builds.

- **Build what it shows.** Layout, wording, labels, what each control does, and what each state says all come from the mockup. Each phase file names the parts of it that phase builds.
- **It's a sketch, not code to copy.** Its data, GitHub, and VS Code window are faked in one HTML file. Take the design from it, not the implementation, and use VS Code's real parts where it draws stand-ins: the editor title actions, the quick pick, the status bar item, the source editor's comment threads, and notifications in place of its toasts.
- **If the mockup and a phase file disagree, stop and report.** Don't pick one silently. When a decision changes the design, the mockup and the phase files change together.
- **Check your work against it.** A phase with UI isn't done until its screens have been compared with the mockup's, at a wide and a narrow width, in light and dark. Say in the pull request what differs and why.

## Who it's for

A reviewer with a checkout of the pull request's branch: most often an engineer, or a writer reviewing another writer's or an agent's change. They want to read the change as a page, comment where they're reading, and have the comment land in the pull request like any other.

## What exists today

- `ascribe build --emit json` writes each page's resolved tree, where every block has a `source` (`file`, `span`, `lines`, and `via`, the includes it came through). `crates/tessera-emit/README.md` documents it.
- The site output is Markdown with web components. `render_site_html` (`crates/tessera-emit/src/render/`) renders it to HTML for the page preview, and `@ascribed/astro` renders it in Astro's Markdown pipeline. Both pass the fixtures in `tests/render/`, under the [site-render contract](../../docs/contracts/site-render.md).
- The rendered HTML carries no source positions. The page preview maps only headings to lines (`sections` in `crates/tessera-lsp/src/preview.rs`).
- A project is loaded through the `FileSystem` trait (`crates/tessera-resolve/src/fs.rs`), with `DiskFs` and `MemoryFs` implementations.
- Nothing in Ascribe reads git or talks to GitHub. The binary has no HTTP client and no async runtime.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **One overlay, several hosts.** The marks for changed blocks and the comment threads are one browser component (`@ascribed/review`). The page preview and the Astro dev server host it. It gets its data from its host through one interface, so a third host later needs no change to it.
2. **Source anchors are what it attaches to.** In review mode, the site output marks each block with the source file and lines it came from. Outside review mode the output is byte-for-byte what it is today. With anchors, the rendered page is the same page plus the anchors: same elements, same order, same text.
3. **The diff compares resolved pages, per build.** Not files. A change to a fragment, a phrase, or `ascribe.toml` changes pages whose own files didn't change, and the diff shows those pages.
4. **Each layer needs only what it uses.** The diff needs `git` and nothing else. Threads need GitHub. Nothing needs a hosted service. A user without `gh` or a GitHub sign-in still gets the diff.
5. **The binary doesn't talk to GitHub.** It shells out to `git` and links no git, HTTP, or async library. GitHub access is in TypeScript, through the GitHub CLI (`gh api`) or, in the VS Code extension, VS Code's GitHub sign-in. Ascribe stores no token, and no token reaches a web page.
6. **GitHub is the only store.** A thread is an ordinary pull request review thread on a file and line. Ascribe keeps no comments of its own. A comment that GitHub can't anchor to a line (its file isn't in the pull request, or its line is away from the diff's changes) is held in the review's summary with a hidden marker naming its block, so it reaches the pull request's conversation when the review is submitted, and the overlay can put it back.
7. **New threads wait for the reviewer to submit.** A new comment goes into the reviewer's review on GitHub (its API calls this a pending review) and nobody else sees it until they submit. The UI calls these **unsent** comments, never "pending", which readers take to mean "waiting on someone else": "2 unsent comments · only you can see them". A reply to an existing thread can be sent at once (**Reply now**) or added to the review (**Add to review**). While the reviewer has unsent comments, GitHub adds every reply to the review, so **Reply now** is unavailable until the review is submitted or discarded, and the UI says why. Resolving and reopening act at once, and the UI says so.
8. **Review mode is opt-in and quiet.** Nothing runs `git` for a diff or contacts GitHub until the user turns review on. No server starts just for review (the multi-project rule).
9. **Three views, linked.** The site preview, the page preview, and the source files show the same threads. From a block or a thread: **Open source**. From a file: **Open Site Preview** and **Open Page Preview**.

## The pieces

| Piece | Where | What it does |
|---|---|---|
| Source anchors | `tessera-emit`, `@ascribed/astro`, the contract | Marks each rendered block with where it came from |
| `ascribe diff` | A new crate, `tessera-diff`, and `tessera-cli` | Changed pages and blocks between a base revision and now, as JSON or a static HTML report |
| `@ascribed/review` (Node part) | `packages/review` | Reads and writes review threads on GitHub, and places them on blocks |
| `@ascribed/review` (browser part) | `packages/review` | The overlay |
| Hosts | `packages/vscode`, `packages/astro` | Give the overlay its data and a place to run |

## Phases

Each phase leaves the repository green and can be its own pull request. A phase can start once the phases it needs are merged.

| Phase | Result | Needs phases |
|---|---|---|
| [1: Source anchors](phase-1-anchors.md) | In review mode, every rendered block says which source lines it came from, in the page preview and in Astro. The preview scrolls with the editor by block. | Nothing |
| [2: `ascribe diff`](phase-2-diff.md) | Changed pages and blocks between a base revision and the working tree, per build, as JSON. | Nothing |
| [3: The static report](phase-3-report.md) | `ascribe diff --format html`: one file showing every changed page rendered, with changes marked. A CI recipe uploads it. | 1, 2 |
| [4: Changes in the page preview](phase-4-preview-changes.md) | The page preview marks what changed against a base, and lists the changed pages. | 1, 2, 3 |
| [5: Review threads](phase-5-threads.md) | `@ascribed/review` reads a pull request's threads, places them on blocks, and posts comments into a pending review. | 2 |
| [6: The overlay, in the page preview](phase-6-overlay.md) | Threads beside blocks in the page preview: read, reply, resolve, comment, submit. Threads in the source editor too. | 4, 5 |
| [7: The site preview](phase-7-site-preview.md) | The overlay in Astro's dev toolbar, and switching between the site preview, the page preview, and the source. | 6 |
| [8: Docs and a full pass](phase-8-docs.md) | The review guide, and the whole flow tried by hand on a real pull request. | 1 to 7 |

### What can run at the same time

Two pairs, and no others:

- **Phases 1 and 2.** They share only the [anchor grammar](#the-anchor-grammar), which is fixed below so neither waits for the other.
- **Phases 3 and 5**, once 1 and 2 are merged. They share only the [package setup](#the-package-setup), which whichever starts first does.

Phases 4, 6, 7, and 8 run one at a time, in order. Phases 6 and 7 are not split: both rework the extension's preview code, and running them together costs more in conflicts than it saves.

When two phases run at once:

- Each works on its own branch (or worktree), never the same checkout.
- Merge one, then rebase the other onto it before its final checks. Expect small conflicts in the files both touch: `CHANGELOG.md`, `docs/cli.md`, `docs/review.md`, `Cargo.toml` and `Cargo.lock`, and `pnpm-lock.yaml`. Resolve them by keeping both sides' additions.
- Each phase is still reviewed by itself before it's merged.

### The anchor grammar

Phases 1, 2, 3, and 5 all read or write source anchors. This is the format; phase 1 copies it into the site-render contract.

- **A block's source** is `<path>:<first>-<last>`. `<path>` is the content path of the file the block's text is written in, with `/` between segments on every platform, and each segment percent-encoded except for ASCII letters, digits, `-`, `.`, `_`, and `~`. `<first>` and `<last>` are line numbers counted from 1, in decimal, with `<last>` never less than `<first>`. A one-line block still writes both: `guides/install.md:12-12`.
- **An include** is `<path>:<line>`: the file holding the `@include` and its line.
- **In HTML,** `data-ascribe-source` holds the block's source. When the block came through includes, `data-ascribe-via` holds them, outermost first, separated by single spaces. A block written in the page itself has no `data-ascribe-via`.
- **In JSON** (phase 2's output), `source` is the same string and `via` is an array of include strings, empty when there are none.

To parse one, split at the last `:`; the path can't contain an unencoded `:`.

**Finding a block by its anchor.** Not every block renders as an element of its own: a paragraph in a tight list item has no `<p>`, so the item carries the anchor (site-render contract §7.2), while phase 2 reports the paragraph's own lines. So a consumer looking for a block's element (phase 3's marks, phase 5's overlay) uses the element with exactly its anchor, and when there is none, the smallest anchored element with the same file and `via` whose lines contain it.

### The package setup

`packages/review` is used by phases 3, 5, 6, and 7. Whichever of phases 3 and 5 starts first creates it, as its first commit, "Add @ascribed/review to the workspace and the release"; the other builds on that commit. The setup is:

- one published package, `@ascribed/review`, with separate entry points so a host bundles only what it uses: `./marks` and `./overlay` for the browser, `./github` and `./place` for Node;
- its build, lint, typecheck, and vitest configuration, matching `packages/astro`;
- its place in the release scripts (`scripts/release/`, with their tests) and in `RELEASING.md` if packages are listed there, since every published package is versioned in lock step;
- a `README.md` that says what each entry point is for.

It adds no behavior: each entry point starts empty.

Phases 1 to 4 need no GitHub access and are useful without the rest.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers: line numbers drift. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- Match the surrounding code's style, comment density, and naming. Libraries don't panic on user input; `unwrap` and `expect` are linted.
- **External APIs change.** GitHub's REST and GraphQL APIs, `gh`'s output, Astro's dev toolbar and integration hooks, and VS Code's comments and authentication APIs: check each against its current documentation before designing against it, and say in the pull request what you checked.
- **No network in tests.** GitHub is tested against recorded responses, and `gh` against a fake executable on the path. `git` is real: tests make a temporary repository.
- **Server work:** a new request gets its own module like `crates/tessera-lsp/src/preview.rs`, a handler in `server.rs`, scenario tests in `crates/tessera-lsp/tests/`, and a section in `crates/tessera-lsp/README.md`.
- **Extension work:** unit tests with vitest (`packages/vscode/test/unit/`), integration tests (`packages/vscode/test/integration/suite/`) against the real server. Everything goes to the server of the project that owns the file (`ProjectRegistry.serverFor`).
- **UI matches the mockup** ([The mockup](#the-mockup)): its layout, wording, and states, unless the phase file says otherwise.
- **User-visible changes** update the docs named in the phase and add a line to the unreleased section of `CHANGELOG.md`.
- Tests must be correct on Windows: no hard-coded `/` in filesystem paths, `file:///C:/…` URIs with three slashes, drive-letter case folded when comparing, and `git` paths (always `/`) converted before use as filesystem paths.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  cargo build -p tessera-cli && ASCRIBE_BIN=$PWD/target/debug/ascribe pnpm --filter ascribe-vscode test:integration
  ```

  (`corepack pnpm` where `pnpm` isn't on the path.)

## How this sits with the Editor UI plan

The two plans are independent and can go in either order. They meet in two places:

- Phase 4 here lists changed pages in a quick pick. If [Editor UI](../editor-ui/README.md) phase 7's Pages view exists by then, add a **Changed** group to it as well.
- Phase 4 here adds a status bar item of its own for review ("Review: #128 ← main", or "Review: off"), as the mockup shows. If Editor UI phase 6's project item exists, the review item sits beside it; neither replaces the other.

## Later, not in this plan

- **Reviewers without a checkout or a GitHub account.** That needs deployed previews, sign-in, and stored comments: a hosted service. The overlay's data interface (decision 1) is what it would plug into.
- **Suggested changes** from the preview (an edit proposed as a GitHub suggestion). It would build on the Editor UI plan's edits.
- **GitLab** and other hosts. Keep GitHub behind the interface phase 5 defines.
- **Other site generators.** The anchors don't depend on Astro, but only Astro has an integration to host the overlay.
