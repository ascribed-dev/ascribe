# Phase 3: Trails

Part of [Optimization](README.md). Needs phase 1 (done). Docs, plus three lint trials.

**Runs with:** phase 2, at the same time. Parts A, B, and C touch different files and can run at once. Part C's lint changes, where a lint is kept, must merge before phase 5 starts or after it ends, never during: both touch many Rust files.

## Goal

Someone starting cold, a person or an agent, can find out how the system is shaped, what the rules are, and what a change has to touch, without reading old plans. And three rules that would otherwise be sentences in a guide are tried as lints.

Ninety pull requests merged in eight days, most from implementors with no memory of the last one. That pace is why the repeats in the [inventory](inventory.md) appear, and this phase is the cheapest fix for it.

## Context

- The [inventory](inventory.md): "Shape" (the crate graph, the surfaces, the two renderers), "Command paths", "Guides for implementors", and "Rules in force".
- `CONTRIBUTING.md` (89 lines), `RELEASING.md`, `README.md`, and the four crate READMEs that exist (`cli`, `diff`, `emit`, `lsp`): match their voice.
- `project-docs/`: each plan's `README.md` has a "Decisions" section. `project-docs/review/` and `project-docs/docs/` are finished or nearly so; `agents/` and `editor-ui/` aren't started.
- `project-docs/agents/phase-4-instructions.md`: that plan will later generate an `AGENTS.md` block for _projects that use Ascribe_. It says nothing may be generated into this repository's root. A hand-written root file is a different thing and doesn't conflict, but leave room: don't use the marker comments that plan reserves (`<!-- ascribe:agents start`).
- `Cargo.toml`, `[workspace.lints]`, and `clippy.toml`: where lints are set. `crates/comrak-tessera` doesn't use the workspace lints and must stay untouched.
- `.github/workflows/js.yml`: the only place the JS build order is written down.

## Design

### Part A: the map

One pull request, docs only.

- **`ARCHITECTURE.md`** at the root: what each crate and package is for, in one or two sentences each; the dependency graph; the surfaces and how each reaches the core; where a project is loaded and where files are read; the two HTML renderers and the tests that compare them; the outputs and the contract each follows; where tests of each kind live and what `ASCRIBE_BLESS=1` does. Describe what is, in the present tense. Link to code by path. No history.
- **`AGENTS.md`** at the root, short (under 80 lines): what the repository is, the commands that must pass before a pull request, the rules that aren't obvious (a clean-up changes no output; contracts don't move; one home per fact; libraries don't print and don't panic on input), and pointers to `ARCHITECTURE.md`, `CONTRIBUTING.md`, and the change checklist. It's the first file an agent reads, so every line must change what an agent does.
- **READMEs** for the seven crates without one: `tessera-check`, `core`, `fmt`, `model`, `resolve`, `sources`, `syntax`. Each: what it's for, its main types and entry points, what it must not depend on, and its tests. `tessera-resolve` first, since it's the largest and owns the file system.
- **Building the JS packages from a fresh checkout:** `pnpm -r build` fails today, because `examples/astro-site` needs a native binary staged first. Add a root script (`pnpm build:all`, or the name that fits `package.json`) that does the steps `js.yml` does, in order, and document it in `CONTRIBUTING.md`. If that needs more than a script, document the order and file an issue.

### Part B: decisions and checklists

One pull request, docs only.

- **`project-docs/decisions.md`:** one list of decisions still in force, each with a number, a sentence, the reason, and where it came from (a plan's decision, an issue, a pull request). Seed it from the "Decisions" sections of the four plans, keeping only what binds future work, and from issues closed with a decision. From now on, a decision made in an issue or a review is added here in the pull request that acts on it; say so in `CONTRIBUTING.md`.
- **`project-docs/checklists.md`:** what a change must touch, as lists a reviewer can tick.
  - A change to the language: `SPEC.md`; parsing (`tessera-syntax`); the checks and their diagnostics; each output (`site`, `plain`, `json`) and its contract; the language server's features one by one (diagnostics, completion, hover, definition, rename, links, tokens, quick fixes, preview); `fmt`; the VS Code grammar; the docs reference; conformance cases; `CHANGELOG.md`.
  - A new command or option: the CLI; the docs' command reference (generated); the JSON shape and its schema; exit codes; the agents plan's table of tools, once it exists.
  - A change to the site output's markup: `tessera-emit`; `@ascribed/elements`; `@ascribed/astro`; `@ascribed/review`'s marks; the editor preview; the HTML report; the contract page.

  Build each list from the code, not from memory: for the language server, list the capabilities it registers.

- **`.github/pull_request_template.md`:** one line pointing at the checklists.
- **Plans marked.** At the top of each finished plan's `README.md`, a line saying it's finished, when, and where the current truth lives (`ARCHITECTURE.md`, the docs, the spec). Don't edit the plans' bodies.

### Part C: three lints, tried

One pull request per lint that's kept, and one report for all three.

For each lint: turn it on in a scratch branch, count what it flags per crate, read a sample, and decide. Put the counts and the decision in the pull request and in `project-docs/optimization/lints.md`.

1. **`unreachable_pub`** (rustc). An item that nothing outside its crate uses shouldn't be `pub`; `tessera-resolve` has 134 public functions, each of which must carry documentation. If kept: make the flagged items `pub(crate)`, and remove documentation only where it restated the name.
2. **`clippy::too_many_lines`**, with `too-many-lines-threshold` in `clippy.toml`. Choose the threshold from the distribution, so that it flags the worst few, not hundreds. If kept: allow the existing offenders by name with a comment, so the lint stops new ones; phase 5 splits them only where it's already working.
3. **`clippy::indexing_slicing`.** In the spirit of the rule against `unwrap`: input must not cause a panic. It may flag hundreds of safe lines. Keep it only if the sample shows real risks; otherwise record why not.

A lint that's kept is set to `warn` in `[workspace.lints]`, as the others are; CI already fails on warnings.

## Tasks

1. Part A: `ARCHITECTURE.md`, `AGENTS.md`, seven READMEs, the JS build script and its documentation.
2. Part B: `decisions.md`, `checklists.md`, the template line, the plans marked.
3. Part C: the trial report, and a pull request for each lint kept.

## Out of scope

- Changing code to fit a rule, beyond what a kept lint requires.
- A workaround register: the inventory found no open workarounds.
- Rewriting `CONTRIBUTING.md` or the plans.
- A general lint pass. The usual rules are already on.

## Acceptance criteria

- Every crate and package has a README, and `ARCHITECTURE.md` names each one.
- Every path and command named in the new files exists; a test or a script checks the paths (`scripts/docs-site/links.test.ts` shows the pattern).
- From a fresh checkout, the documented command builds every JS package.
- `checklists.md`'s language-server list matches the capabilities the server registers.
- Each of the three lints has a recorded count and a decision.

## Stop and report if

- A kept lint would change more than about 300 lines in one crate: propose doing it crate by crate instead.
- Writing the map shows two parts of the code disagreeing about who owns something. Record it as a finding; don't settle it here.

## Verify

```sh
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

## Commits

1. "Describe how Ascribe is put together"
2. "Add READMEs for the crates that had none"
3. "Build every JS package with one command"
4. "Keep decisions and change checklists in one place"
5. "Mark the finished plans"
6. One per lint kept, such as "Stop exporting what only one crate uses"
