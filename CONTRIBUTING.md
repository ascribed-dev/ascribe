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

- Behavior changes come with a conformance case in [tests/conformance](tests/conformance). Diagnostics are defined only in `tests/conformance/diagnostics.toml`; the diagnostics reference is generated from it.
- Changes to `crates/comrak-tessera` follow [FORK.md](crates/comrak-tessera/FORK.md): mark each one `// TESSERA:` and update the table.
- Libraries don't panic on user input; `unwrap` and `expect` are linted.
- Dependencies come from Dependabot. Keep the toolchain, `.nvmrc`, and lockfiles current rather than pinning old versions.

## Generated docs

Parts of the user docs repeat what the code already says, so they're generated from it, as fragments in `docs/content/_generated/` that pages include: the diagnostics, from `tests/conformance/diagnostics.toml`; each command's options, from the `clap` help text in `crates/tessera-cli/src/`; and the extension's settings and commands, from `packages/vscode/package.json`. Each fragment says at its top what generates it. A test beside each source fails when its fragments are stale; change the source, not the fragment, then run the test with `ASCRIBE_BLESS=1` to rewrite them (`ASCRIBE_BLESS=1 cargo test -p tessera-conformance --test docs`, `ASCRIBE_BLESS=1 cargo test -p tessera-cli docs`, or `ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts`). The phrases in `docs/ascribe.toml` whose values are in another file, such as `version`, are checked against it too.

## If you move a checkout

Some tests embed the checkout's absolute path at compile time (`env!("CARGO_MANIFEST_DIR")`), and Cargo doesn't notice when the directory moves. If tests fail with `NotFound` on a path from the old location, run `cargo clean`.

## License

By contributing, you agree that your contributions are licensed under the [MPL-2.0](LICENSE).
