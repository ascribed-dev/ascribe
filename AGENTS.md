# Working on Ascribe

Ascribe is a markup language and toolchain for documentation written as code: Markdown with `@` directives, a content model in `ascribe.toml`, and one compiler that builds a website, plain Markdown, and JSON. The compiler is a Rust workspace in `crates/` (`ascribe-*`); the npm packages and the VS Code extension are a pnpm workspace in `packages/`.

Read [ARCHITECTURE.md](ARCHITECTURE.md) before changing code: it says which crate owns what, and where the tests are. The README of the crate or package you're changing has the rest. [SPEC.md](SPEC.md) defines the language.

## Before a pull request

Every one of these must pass; CI runs them all and treats warnings as errors.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

Setup: `corepack enable && pnpm install`. `pnpm typecheck` needs `@ascribed/cli` and `@ascribed/elements` built first; `pnpm build:all` builds the binary and every JS package. The VS Code integration tests need VS Code and run in CI only.

## Rules that aren't obvious

- **A clean-up changes no output.** If a change is meant to be a refactor, the outputs, diagnostics, and JSON stay byte for byte the same; `node scripts/compare/outputs.ts --base main` shows it, and CI runs it on pull requests labeled `optimization`. An output change a recorded decision accepts is named file by file with `--accept` (in CI, the label `outputs changed` and a block in the description; see CONTRIBUTING.md), never waved through whole. A bug found on the way gets its own pull request.
- **Contracts don't move.** `SPEC.md`, `docs/content/contracts/`, `packages/elements/CONTRACT.md`, the commands' JSON, and the published packages' APIs change only by a decision recorded with the change, never as a side effect.
- **One home per fact.** Don't copy a list, a name, or a version into a second file. Generate it from the first, or add a test that compares the two.
- **Libraries don't print and don't panic on input.** Only `ascribe-cli` writes to standard output or error, apart from the language server's log, which goes through `crates/ascribe-lsp/src/log.rs`; a library returns what happened, failures as an error type with a stable code (`ascribe_core::Coded`), and lets its caller report it. A lint holds this. `unwrap`, `expect`, and `panic!` are linted; an invariant that holds gets a local `#[allow]` with a comment saying why.
- **Diagnostics live in one file,** `tests/conformance/diagnostics.toml`. A new one is added there and in `crates/ascribe-core/src/diagnostics.rs`, in the same order; a test compares them.
- **Behavior changes come with a conformance case** in `tests/conformance/cases/`.
- **Generated files aren't edited by hand.** A file that says what generates it is rewritten by running its test with `ASCRIBE_BLESS=1`; read the diff before committing it. See ARCHITECTURE.md's list.
- **Colors are written in `design/tokens.toml` only,** and a stylesheet change is an output change: the HTML report embeds the stylesheets. [design/README.md](design/README.md) has the steps; a test fails on a color written in a stylesheet.
- **Never accept snapshots blindly.** Review `insta` changes with `cargo insta review`.
- **Read project files through `ascribe_resolve::FileSystem`.** It knows the content root, the boundary, exact-case names, and symbolic links. A new direct `std::fs` read of a project file repeats bugs already fixed there; a read of anything else says why in a comment starting `Outside FileSystem:`, or `crates/ascribe-resolve/tests/all/file_reads.rs` fails.
- **Two renderers must agree.** A change to the site output's markup changes the fixtures in `tests/render/`, and both `render_site_html` and the Astro plugin pass them.
- **`crates/comrak-ascribe` is a fork.** Mark each change `// ASCRIBE:` and list it in `crates/comrak-ascribe/FORK.md`.
- **Docs change with the code.** A change a user would notice updates its page under `docs/content/` and the unreleased section of `CHANGELOG.md`. Don't run a formatter over Markdown.
- **Paths, case, and links differ by platform.** Pull requests run on Linux and Windows; macOS and arm64 run after the merge. A change about paths runs the full matrix from its branch (Actions → CI → Run workflow).

## Where to look

- [CONTRIBUTING.md](CONTRIBUTING.md): setup, the checks, documenting a change, and the docs site.
- [RELEASING.md](RELEASING.md): how a release and the nightly canary are made.
- [project-docs/checklists.md](project-docs/checklists.md): what a change to the language, a command, or the site output's markup must touch. Go through the one that fits before opening a pull request.
- [project-docs/decisions.md](project-docs/decisions.md): the decisions in force. A decision made in an issue or a review is added there in the pull request that acts on it.
- [project-docs/outside.md](project-docs/outside.md): every account, secret, and package Ascribe relies on outside the repository, and who can change each. [project-docs/lints.md](project-docs/lints.md): the lints tried, and why each was kept or not.
- `project-docs/`: plans, each with its decisions. A plan says what was intended; the code and ARCHITECTURE.md say what is.
