# Phase 22: Astro integration and npm distribution

**Track:** Web · **Start after:** 21 · **Parallel with:** 16, 25 · **Unblocks:** 27 · **Human checkpoint: done. The owner claimed the npm org `@ascribed` (and the GitHub org `ascribed-dev`) on 2026-09-29.**

## Goal

Turn the phase 21 slice into a complete integration, and deliver the `ascribe` binary through npm for every platform.

## Read first

- [PLAN.md](../PLAN.md): Key decisions (distribution), Astro integration and elements.
- Handoff notes from phase 21.
- The current Astro documentation for integrations.

## Deliverables

- `packages/cli`: `@ascribed/cli` and its per-platform binary packages.
- A complete `@ascribed/astro` and `examples/astro-site`.
- CI jobs that cross-compile the binary and run the end-to-end test on every platform.

## Tasks

1. **Binary packages.** Follow the pattern esbuild and Biome use: one package per platform (at least macOS arm64 and x64, Linux x64 and arm64, Windows x64), each containing the binary, declared as optional dependencies of `@ascribed/cli`. `@ascribed/cli` provides a small `ascribe` shim that finds the right binary without a postinstall script. CI cross-compiles release binaries for every platform.
2. **Integration, completed.**
   - Uses the project's `@ascribed/cli` binary.
   - In `astro dev`, rebuilds when Ascribe sources, assets, or `ascribe.toml` change, and refreshes the page.
   - Reports Ascribe's diagnostics in Astro's terminal output in a readable form.
   - Lets the site choose which Ascribe build to use.
3. **Cross-platform CI.** Run phase 21's end-to-end test on Linux, macOS, and Windows, using binaries from the npm packages built in CI.

## Acceptance criteria

- [ ] Installing `@ascribed/cli` from locally packed tarballs on each platform runs `ascribe --version`, with no postinstall script.
- [ ] The end-to-end test passes on all three platforms.
- [ ] Editing an Ascribe file, an asset, or `ascribe.toml` in `astro dev` rebuilds and refreshes the page.

## Out of scope

- Publishing to npm (phase 27).

## Handoff notes

### Implementation and interfaces

- `@ascribed/cli` has a Node bin shim and `@ascribed/cli/binary` exports
  `resolveBinary(): string`. It selects one optional native package by OS/CPU:
  `@ascribed/cli-{darwin-arm64,darwin-x64,linux-arm64,linux-x64,win32-x64}`.
  Each native package declares `os` and `cpu` and contains `bin/ascribe`
  (Windows: `bin/ascribe.exe`). No install-time scripts or downloads are used.
  `packages/cli/scripts/stage-native.mjs` copies the chosen release binary
  using `ASCRIBE_BIN_<TARGET>` before packing; see `packages/cli/README.md`.
- `@ascribed/astro` defaults to the installed CLI's `resolveBinary()`;
  explicit `binary` and `ASCRIBE_BIN` overrides remain for development.
  Its `astro:server:setup` watcher rebuilds on source, fragment, asset, and
  model edits, serializes builds, refreshes the Astro collection, then sends
  a full reload. Failed builds log compiler diagnostics and return HTTP 503
  instead of silently serving stale pages; a later edit retries. The generated
  output directory is excluded from the source watcher. Existing collection,
  schema, plugin, elements, and preview/site-render interfaces are unchanged.
- The example depends on `@ascribed/cli`, and its real-browser e2e tests cover
  source, linked asset, and `ascribe.toml` edits and failure/recovery.
- **Shared integration review:** `pnpm-workspace.yaml`, `pnpm-lock.yaml`, and
  `.github/workflows/js.yml` changed. The manual-only `astro` matrix uses
  native release builds on Ubuntu x64/arm64, macOS x64/arm64, and Windows x64;
  it stages and packs the matching optional package and CLI, installs the two
  tarballs with `npm --offline --ignore-scripts`, checks `ascribe --version`,
  and runs the original phase 21 browser suite with `ASCRIBE_BIN` resolved
  from that installed package. Native packages use `npm pack` to preserve the
  executable mode; the CLI uses `pnpm pack` to replace workspace protocol
  references with publishable versions. The general JS check builds the CLI
  and elements packages before typechecking Astro and VS Code, which import
  their generated declarations. The Linux x64 job retains phase 25 parity.
  No Rust, contract, SPEC, or phase 24 files changed.

### Verification and open gates

On a macOS arm64 host (Node 24.21, pinned pnpm 11.24), with a staged debug
binary from `cargo build -p tessera-cli --locked`:

```sh
corepack pnpm install --no-frozen-lockfile
corepack pnpm --filter @ascribed/cli build
ASCRIBE_BIN_DARWIN_ARM64="$PWD/target/debug/ascribe" corepack pnpm --filter @ascribed/cli stage-native darwin-arm64
node packages/cli/dist/index.js --version                    # ascribe 0.0.0
corepack pnpm --filter @ascribed/astro build
corepack pnpm --filter @ascribed/elements build
corepack pnpm --filter @ascribed/cli test                      # 10 passed
corepack pnpm --filter @ascribed/astro test                    # 49 passed
corepack pnpm --filter @ascribed/example-astro-site test:e2e  # 19 passed
corepack pnpm -r lint                                         # passed
corepack pnpm -r typecheck                                    # passed
corepack pnpm format:check                                    # passed
cargo fmt --all --check                                       # passed
cargo clippy --workspace --all-targets --locked -- -D warnings # passed
cargo test --workspace --locked --quiet                       # passed; conformance 366/366, 0 skipped
```

Pack dry runs list `dist/index.js` in the CLI archive and `bin/ascribe` in the
staged macOS arm64 native archive. The macOS arm64 native and CLI tarballs were
packed with `npm pack` and `pnpm pack`, respectively, then installed with
`npm --offline --ignore-scripts`; the installed `ascribe --version` succeeded.
Use `npm pack` for the native archive: `pnpm pack` normalizes the executable to
mode 0644, which causes `EACCES` after installation. Use `pnpm pack` for the
CLI archive because `npm pack` leaves its `workspace:*` dependencies
uninstallable by npm. The local run exercised both package managers with the
native archive from `npm pack` and the CLI archive from `pnpm pack`. The real
Astro browser suite (19 tests) and production build also passed with
`ASCRIBE_BIN` pointing to the extracted binary from the local tarball install.

The JavaScript workflow has not been dispatched, so Linux, Windows, and macOS
x64 e2e results remain **pending**. With owner authorization, manually run the
**JavaScript** workflow (`.github/workflows/js.yml`) on this PR's branch in
Actions (or `gh workflow run js.yml --ref phase/22-astro-and-npm`), inspect all
five `Astro npm end-to-end` jobs, and only then check the platform acceptance
criteria. Do not dispatch without authorization.

Changing `[project] output-dir` in a running `astro dev` requires a restart:
the existing collection loader and generated schema import have fixed paths;
the integration reports an error instead of serving stale output. Releasing
or publishing any package remains phase 27's human-controlled work.
