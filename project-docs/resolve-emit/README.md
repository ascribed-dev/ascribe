# A page in the site output, from the command line

`ascribe resolve PAGE --emit site` writes one page as the site output writes it, Ascribe's elements and all, without building the project, and an agent gets the same from `ascribe_resolve`. The idea, the design, and the three questions the maintainer settled are in the [proposal](../proposals/resolve-emit.md). This is the plan for building it.

## Who it's for

- **An agent** checking what a reader of the site gets, not only a reader of the Markdown version (phase 2, for one without a shell).
- **A writer or a reviewer** asking what a directive becomes, without building and opening the site.
- **A test of a site's integration** that starts from one page, its frontmatter included.

## What exists today

The proposal's "What exists" holds. The pieces this plan stands on, checked against `main` on 24 October:

- **The command:** `crates/ascribe-cli/src/commands/resolve.rs`, `Args` (`page`, `build`, `frontmatter`, `format`) and `Answer` (`Page`, `Help` for `go:<name>`). It calls `ascribe_query::resolve(project, project_root, path, build, frontmatter)` in `crates/ascribe-query/src/resolve.rs`, which returns `Resolved` (`schema_version`, `ascribe_version`, `page`, `build`, `not_published`, `route`, `text`).
- **One page in the site output:** `crates/ascribe-lsp/src/preview.rs` does it for the preview: `project.for_build(build)`, `resolve_page(path, build, &AstroRouter::from_consumer(...))`, then `ascribe_emit::emit_page(&SiteEmitter::new(model), &EmitContext::new(project, root, build), &page)`. The preview sets `with_anchors(true)`; this doesn't.
- **The schema:** `Resolved` derives `schemars::JsonSchema` under `ascribe-query`'s `json-schema` feature; `crates/ascribe-cli/src/shapes.rs` writes `schemas/resolve.schema.json` and the generated page `docs/content/_generated/json-resolve.md` from it.
- **The tool:** `crates/ascribe-cli/src/mcp/tools.rs`, the `ascribe_resolve` spec, its description and input schema. `crates/ascribe-cli/src/agents/skill.rs` lists `ascribe resolve <page> --build <name>` in the skill.
- **The tests:** `crates/ascribe-cli/tests/output/resolve.txt` and `resolve.json`, and `resolve-permalink.*` for `go:`; `crates/ascribe-cli/tests/all/mcp.rs` for the tools.
- **The docs:** `docs/content/command/questions.md`'s `ascribe resolve` (`@id {permalink=true}: cli-resolve`), its options generated into `docs/content/_generated/cli-resolve-options.md`; `docs/content/contracts/json-reports/questions.md`; `docs/content/agents/commands.md`.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

The proposal's Settled 1 to 3 and its design hold. From this plan:

1. **The site form is written where the plain form is:** `ascribe_query::resolve` takes which output to write, and the site form calls `emit_page` with the build's `SiteEmitter`, as the preview does, with no anchors. Nothing renders a page a third way.
2. **`--emit` is an `Option`,** so `go:<name>` can refuse one that's given while the default stays `plain`. A value that isn't `plain` or `site`, a list among them, is clap's usage error, exit `2`, naming the values.
3. **`emit` in `Resolved` is an enum,** serialized `"plain"` or `"site"`, so a later `json` is a value added. Its doc comment becomes the schema's description.
4. **The example the docs test** is quill's `install-agent.md` with `--emit site`, cut to its Connect section, listed in `ascribe.examples.toml` and observed again.
5. **Two phases, each one pull request:** the command first, since everything else reads its JSON; then the tool and the agents' rules, which add a contract of their own (decision 56).

The contracts, and the phase that records each in `project-docs/decisions/`: `ascribe resolve`'s `--emit` and `resolve.schema.json`'s `emit` (phase 1); `ascribe_resolve`'s `emit` (phase 2).

## No thread

One layer, over a path in use: the proposal's "No thread" holds.

## Phases

| Phase | Result |
|---|---|
| [1: The command](phase-1-command.md) | `ascribe resolve PAGE --emit site` writes the page as the site output does, and its JSON says which output it is. |
| [2: For agents](phase-2-agents.md) | `ascribe_resolve` with `emit: "site"` gives an agent the same, and the agents' rules say when to ask for it. |

## Rules for every phase

- Branch before committing; never commit to `main`.
- Read the current code before the phase file's pointers. If the phase file and the code disagree, or a decision above can't be met, stop and report instead of choosing silently.
- **No output of a build changes.** This writes what the site output already writes. `node scripts/compare/outputs.ts --base main` shows no change.
- **Contracts change only by a recorded decision,** in the phase that changes them.
- **Libraries don't print.** The page is returned; only `ascribe-cli` writes.
- **Tests at the layer that proves them** ([writing-tests](../../.claude/skills/writing-tests/SKILL.md)): the command's text and JSON in `crates/ascribe-cli/tests/output/`, the tool in `crates/ascribe-cli/tests/all/mcp.rs`.
- **Generated files** (`schemas/`, `docs/content/_generated/`) are rewritten by their test with `ASCRIBE_BLESS=1`, and their diff read.
- **The map of the surface** is regenerated (`node scripts/surface/inventory.ts`), and its diff read against the conventions.
- **User-visible changes** update their page under `docs/content/`, marked `@available: ascribe planned` where they're new, and add an entry in `changes/`.
- No phase history in code or docs. Describe what the code does now.
- Before finishing a phase, all of these pass:

  ```sh
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
  pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
  ```

## Later, not in this plan

The proposal's "What this leaves out": HTML, `--emit json`, source anchors, and copying a page's assets.
