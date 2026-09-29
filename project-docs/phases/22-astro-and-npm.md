# Phase 22: Astro integration and npm distribution

**Track:** Web · **Start after:** 21 · **Parallel with:** 16, 25 · **Unblocks:** 27 · **Human checkpoint before starting: confirm the `@tessera` npm scope**

## Goal

Turn the phase 21 slice into a complete integration, and deliver the `tessera` binary through npm for every platform.

## Read first

- [PLAN.md](../PLAN.md): Key decisions (distribution), Astro integration and elements.
- Handoff notes from phase 21.
- The current Astro documentation for integrations.

## Deliverables

- `packages/cli`: `@ascribed/cli` and its per-platform binary packages.
- A complete `@ascribed/astro` and `examples/astro-site`.
- CI jobs that cross-compile the binary and run the end-to-end test on every platform.

## Tasks

1. **Binary packages.** Follow the pattern esbuild and Biome use: one package per platform (at least macOS arm64 and x64, Linux x64 and arm64, Windows x64), each containing the binary, declared as optional dependencies of `@ascribed/cli`. `@ascribed/cli` provides a small `tessera` shim that finds the right binary without a postinstall script. CI cross-compiles release binaries for every platform.
2. **Integration, completed.**
   - Uses the project's `@ascribed/cli` binary.
   - In `astro dev`, rebuilds when Tessera sources, assets, or `ascribe.toml` change, and refreshes the page.
   - Reports Tessera's diagnostics in Astro's terminal output in a readable form.
   - Lets the site choose which Tessera build to use.
3. **Cross-platform CI.** Run phase 21's end-to-end test on Linux, macOS, and Windows, using binaries from the npm packages built in CI.

## Acceptance criteria

- [ ] Installing `@ascribed/cli` from locally packed tarballs on each platform runs `tessera --version`, with no postinstall script.
- [ ] The end-to-end test passes on all three platforms.
- [ ] Editing a Tessera file, an asset, or `ascribe.toml` in `astro dev` rebuilds and refreshes the page.

## Out of scope

- Publishing to npm (phase 27).

## Handoff notes

_To be filled in by the implementing agent._
