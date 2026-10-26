# Working on Ascribe

Ascribe is a markup language and toolchain for documentation written as code: Markdown with `@` directives, a content model in `ascribe.toml`, and one compiler that builds a website, plain Markdown, and JSON. The compiler is a Rust workspace in `crates/` (`ascribe-*`); the npm packages and the VS Code extension are a pnpm workspace in `packages/`.

Read [ARCHITECTURE.md](ARCHITECTURE.md) before changing code: it says which crate owns what, and where the tests are. The README of the crate or package you're changing has the rest. [SPEC.md](SPEC.md) defines the language.

## Before a pull request

Run every check CI runs, which must all pass and treat warnings as errors:

```sh
pnpm check:all
```

It runs each step of `rust.yml` and `js.yml`, in an order that works from a fresh checkout, and stops at the first failure; `scripts/check.ts` lists them, and says why it skips the few it does. It needs `cargo-deny` and `cargo-machete`, and when either is missing it says how to install the versions CI pins. Setup: `corepack enable`. To run one check alone, `pnpm typecheck` needs `@ascribed/cli` and `@ascribed/elements` built first; `pnpm build:all` builds the binary and every JS package. The VS Code integration tests and the Astro end-to-end run in CI only.

## Rules that aren't obvious

- **A clean-up changes no output.** If a change is meant to be a refactor, the outputs, diagnostics, and JSON stay byte for byte the same; `node scripts/compare/outputs.ts --base main` shows it, and CI runs it on pull requests labeled `optimization`. An output change a recorded decision accepts is named file by file with `--accept` (in CI, the label `outputs changed` and a block in the description; see CONTRIBUTING.md), never waved through whole. A bug found on the way gets its own pull request.
- **Contracts don't move.** `SPEC.md`, `docs/content/contracts/`, `packages/elements/CONTRACT.md`, the commands' JSON, and the published packages' APIs change only by a decision recorded with the change, never as a side effect.
- **One home per fact.** Don't copy a list, a name, or a version into a second file. Generate it from the first, or add a test that compares the two.
- **Libraries don't print and don't panic on input.** Only `ascribe-cli` writes to standard output or error, apart from the language server's log, which goes through `crates/ascribe-lsp/src/log.rs`; a library returns what happened, failures as an error type with a stable code (`ascribe_core::Coded`), and lets its caller report it. A lint holds this. `unwrap`, `expect`, and `panic!` are linted; an invariant that holds gets a local `#[allow]` with a comment saying why.
- **Diagnostics live in one file,** `tests/conformance/diagnostics.toml`. A new one is added there and in `crates/ascribe-core/src/diagnostics.rs`, in the same order; a test compares them.
- **Behavior changes come with a conformance case** in `tests/conformance/cases/`.
- **Generated files aren't edited by hand.** A file that says what generates it is rewritten by running its test with `ASCRIBE_BLESS=1`; read the diff before committing it. See ARCHITECTURE.md's list.
- **The docs' tested examples are observed again when they change.** `ascribe.examples.toml` lists the files the docs show and a test proves: the command output in `crates/ascribe-cli/tests/output/` and the example projects those tests check. After re-blessing or editing one, run `cargo run -p ascribe-cli -- observe` in the repository's root and commit `examples.observations/default.toml`; CI's `ascribe observe --check` fails without it. At a terminal it lists the commands `ascribe.examples.toml` names and asks before running them: answer `y`, or pass `--yes` (`cargo run -p ascribe-cli -- observe --yes`) once you've read them. Without a terminal, as in an agent's shell, it doesn't ask.
- **Colors are written in `design/tokens.toml` only,** and a stylesheet change is an output change: the HTML report embeds the stylesheets. [design/README.md](design/README.md) has the steps; a test fails on a color written in a stylesheet.
- **Never accept snapshots blindly.** Review `insta` changes with `cargo insta review`.
- **Read project files through `ascribe_resolve::FileSystem`.** It knows the content root, the boundary, exact-case names, and symbolic links. A new direct `std::fs` read of a project file repeats bugs already fixed there; a read of anything else says why in a comment starting `Outside FileSystem:`, or `crates/ascribe-resolve/tests/all/file_reads.rs` fails.
- **Two renderers must agree.** A change to the site output's markup changes the fixtures in `tests/render/`, and both `render_site_html` and the Astro plugin pass them.
- **`crates/comrak-ascribe` is a fork.** Mark each change `// ASCRIBE:` and list it in `crates/comrak-ascribe/FORK.md`.
- **Docs change with the code.** A change a user would notice updates its page under `docs/content/` and adds its changelog entry as a file of its own in `changes/` ([changes/README.md](changes/README.md)); it doesn't edit `CHANGELOG.md`, which a release gathers the entries into. Don't run a formatter over Markdown.
- **Paths, case, and links differ by platform.** Pull requests run on Linux, and on Windows when the change touches the paths `ci.yml`'s `changes` job lists; macOS and arm64 run after the merge. A change about paths runs the full matrix from its branch (Actions → CI → Run workflow).

## Where to look

- [CONTRIBUTING.md](CONTRIBUTING.md): setup, the checks, documenting a change, and the docs site.
- [RELEASING.md](RELEASING.md): how a release and the nightly canary are made.
- [project-docs/checklists.md](project-docs/checklists.md): what a change to the language, a command, or the site output's markup must touch. Go through the one that fits before opening a pull request.
- [.claude/skills/writing-tests/SKILL.md](.claude/skills/writing-tests/SKILL.md): the test layers and what each proves, where a test of each kind goes, the fixture helpers to build on, and what keeps a test fast. Read it before adding or changing a test.
- [.claude/skills/board/SKILL.md](.claude/skills/board/SKILL.md): working from [the project board](https://github.com/orgs/ascribed-dev/projects/1), and the skills for each role on it: `triage`, `implement`, `review`, and `steward` for driving a pull request.
- [project-docs/surface.md](project-docs/surface.md): everything a person or an agent touches, generated from where each is declared: the commands, the directives, `ascribe.toml`'s sections, the diagnostics, the editor's commands and settings, the tools, and the packages' entries. Read it before adding to any of them; `node scripts/surface/inventory.ts` rewrites it.
- [project-docs/decisions.md](project-docs/decisions.md) and [project-docs/decisions/](project-docs/decisions/README.md): the decisions in force: the numbered ones in the first, and each since as a file of its own in the folder, numbered by its issue. A decision made in an issue or a review is added to the folder in the pull request that acts on it.
- [project-docs/outside.md](project-docs/outside.md): every account, secret, and package Ascribe relies on outside the repository, and who can change each.
- `project-docs/`: plans, each with its decisions. A plan says what was intended; the code and ARCHITECTURE.md say what is.
