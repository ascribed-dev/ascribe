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

- Behavior changes come with a conformance case in [tests/conformance](tests/conformance). Diagnostics are defined only in `tests/conformance/diagnostics.toml`; `docs/diagnostics.md` is generated from it, and a test fails when it's stale.
- Changes to `crates/comrak-tessera` follow [FORK.md](crates/comrak-tessera/FORK.md): mark each one `// TESSERA:` and update the table.
- Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Dependencies come from Dependabot. Keep the toolchain, `.nvmrc`, and lockfiles current rather than pinning old versions.

## If you move a checkout

Some tests embed the checkout's absolute path at compile time (`env!("CARGO_MANIFEST_DIR")`), and Cargo doesn't notice when the directory moves. If tests fail with `NotFound` on a path from the old location, run `cargo clean`.

## License

By contributing, you agree that your contributions are licensed under the [MPL-2.0](LICENSE).
