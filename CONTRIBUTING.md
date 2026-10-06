# Contributing to Ascribe

Thanks for helping. Open an issue before starting anything large, so the change fits the design.

## Setup

You need the Rust toolchain in [rust-toolchain.toml](rust-toolchain.toml) (`rustup` installs it on first use) and Node.js 24 or later with [pnpm](https://pnpm.io) (`corepack enable`).

```sh
corepack enable && pnpm install
cargo test --workspace --locked
pnpm -r test
```

The [README](README.md) describes the layout. [packages/vscode/DEVELOPMENT.md](packages/vscode/DEVELOPMENT.md) covers the extension.

## Before you open a pull request

CI runs all of these, and every check must pass:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo deny check && cargo machete
pnpm format:check && pnpm lint && pnpm exec knip && pnpm typecheck && pnpm test
```

- Behavior changes come with a conformance case in [tests/conformance](tests/conformance). Diagnostics are defined only in `tests/conformance/diagnostics.toml`; the diagnostics reference is generated from it.
- Changes to `crates/comrak-tessera` follow [FORK.md](crates/comrak-tessera/FORK.md): mark each one `// TESSERA:` and update the table.
- Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Dependencies come from Dependabot. Keep the toolchain, `.nvmrc`, and lockfiles current rather than pinning old versions.
- Dependencies are audited. `cargo deny` ([deny.toml](deny.toml)) allows the licenses listed there and crates.io only, fails on a security advisory, and warns on a second version of a crate. `cargo machete` and `knip` ([knip.jsonc](knip.jsonc)) fail on an unused dependency, and `knip` on an unused file or export too. Where one is wrong, ignore the name in its configuration with a comment saying why. Install the Rust tools with `cargo install --locked cargo-deny@0.20.2 cargo-machete@0.9.2`, the versions `rust.yml` pins.

A pull request runs them on Linux and Windows, and `main` runs them on Linux and macOS. Every platform, with the Astro end-to-end on each, runs every night. So a break that shows only on macOS or arm64 appears after the merge, not on the pull request: if your change is about paths, case, or links on one of those, start a full run from your branch (Actions → CI → Run workflow). A change to `project-docs/`, `reports/`, or `research_notes/` alone runs only the formatter.

## Documenting a change

The user docs are in `docs/`, an Ascribe project, and change in the same pull request as the code they describe. A change someone using Ascribe would notice updates the page that describes it, under `docs/content/`, and adds a line to the unreleased section of [CHANGELOG.md](CHANGELOG.md). What no release has yet is marked `@available: next` on its page or section. Every page must pass the checks CI runs on it (in `site.yml`):

```sh
cargo build -p tessera-cli
./target/debug/ascribe check --deny-warnings --config docs
./target/debug/ascribe fmt --check --config docs
```

A page's code examples aren't copies: each is taken with [`@snippet`](docs/content/reference/directives.md#snippet) from a file that's built, tested, or run, so it changes when the file does. `[sources.code]` in `docs/ascribe.toml` lists the folders a page may take one from: `examples/`, the workflows in `.github/workflows/`, and the command output in `crates/tessera-cli/tests/output/`. To show part of a file, mark it with tags in its comments, and give the page its address:

```toml
# :snippet-start: project
[project]
content-root = "docs"
# :snippet-end:
```

```markdown
@snippet: code:examples/content-models/full.toml#project
```

A line ending in `# :remove:`, or the lines between `# :remove-start:` and `# :remove-end:`, are left out of the example; `.github/workflows/drift.yml` uses them for what this repository adds to the job its guide gives users. The command output in `crates/tessera-cli/tests/output/` is written by `crates/tessera-cli/tests/output.rs`, which fails when a command's output changes; run it with `ASCRIBE_BLESS=1 cargo test -p tessera-cli --test output` to rewrite the files, and read the diff. An example with no file behind it, a few lines showing syntax, stays in the page as a code block.

On a pull request, the **Drift** workflow's summary lists the pages whose examples the change touched: first those whose words around the example didn't change, which are the ones to reread, then those that changed along with it. It never fails and never comments. When it lists a page, read the sentences around the example against the new code, and fix the page in the same pull request if they no longer hold. An empty summary means no example changed.

## The docs site

The user docs in `docs/` are an Ascribe project, and `site/` publishes them with Astro. `site/` isn't in the pnpm workspace: it installs Ascribe from npm with its own lockfile, as a user's site does. To see a change to the docs, or to Ascribe and its docs together, build the site with this checkout's Ascribe:

```sh
cargo build -p tessera-cli
cd site
npm ci
npm run build:checkout   # this checkout's binary and packages, installed without saving
npm run preview          # the built site, with search
npm test                 # navigation, links, and the site in Chromium
```

For `astro dev`, which rebuilds as you edit pages, run `ASCRIBE_BIN=../target/debug/ascribe npm run dev` after `build:checkout`; without `ASCRIBE_BIN`, it runs the binary from npm. `npm ci` puts back the packages from npm. [site/README.md](site/README.md) has the rest.

The site is published at <https://ascribed-dev.com>. A pull request that changes `docs/`, `site/`, Ascribe, or a file the docs take examples from gets a preview of the site built with its own Ascribe, linked from the **Site** workflow's summary, and one that changes `docs/`, or a file its pages take examples from, gets a review report of its pages from the **Review** workflow.

## Generated docs

Parts of the user docs repeat what the code already says, so they're generated from it, as fragments in `docs/content/_generated/` that pages include: the diagnostics, from `tests/conformance/diagnostics.toml`; each command's options, from the `clap` help text in `crates/tessera-cli/src/`; and the extension's settings and commands, from `packages/vscode/package.json`. Each fragment says at its top what generates it. A test beside each source fails when its fragments are stale; change the source, not the fragment, then run the test with `ASCRIBE_BLESS=1` to rewrite them (`ASCRIBE_BLESS=1 cargo test -p tessera-conformance --test docs`, `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`, or `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`). The phrases in `docs/ascribe.toml` whose values are in another file, such as `version`, are checked against it too.

## If you move a checkout

Some tests embed the checkout's absolute path at compile time (`env!("CARGO_MANIFEST_DIR")`), and Cargo doesn't notice when the directory moves. If tests fail with `NotFound` on a path from the old location, run `cargo clean`.

## License

By contributing, you agree that your contributions are licensed under the [MPL-2.0](LICENSE).
