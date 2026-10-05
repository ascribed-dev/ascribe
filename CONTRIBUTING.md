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
pnpm format:check && pnpm lint && pnpm typecheck && pnpm test
```

- Behavior changes come with a conformance case in [tests/conformance](tests/conformance). Diagnostics are defined only in `tests/conformance/diagnostics.toml`; `docs/content/reference/diagnostics.md` is generated from it, and a test fails when it's stale.
- Changes to `crates/comrak-tessera` follow [FORK.md](crates/comrak-tessera/FORK.md): mark each one `// TESSERA:` and update the table.
- Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Dependencies come from Dependabot. Keep the toolchain, `.nvmrc`, and lockfiles current rather than pinning old versions.

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

## If you move a checkout

Some tests embed the checkout's absolute path at compile time (`env!("CARGO_MANIFEST_DIR")`), and Cargo doesn't notice when the directory moves. If tests fail with `NotFound` on a path from the old location, run `cargo clean`.

## License

By contributing, you agree that your contributions are licensed under the [MPL-2.0](LICENSE).
