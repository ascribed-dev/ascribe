# Release report: Ascribe 0.1.0

Prepared 2026-09-30 for the first release, and updated the same day after the repository went public and CI was re-enabled. The gate results below were first run on one machine, an Apple-silicon Mac (macOS, arm64). Since then, [PR #39](https://github.com/ascribed-dev/ascribe/pull/39) re-enabled CI and brought the toolchains and dependencies up to date, and its checks pass on macOS, Linux (x64 and arm64), and Windows: clippy and tests, the JavaScript checks, the VS Code integration tests, and the Astro end-to-end suite on all four platforms. That replaces the owner's earlier decision to accept Linux and Windows unverified. The release workflow's own dry run hasn't run yet; it builds and smoke-tests every platform, and a failure there stops the release before anything is published.

## Summary

| | Status | What's left |
|---|---|---|
| Gate: no unintentional skips | **Passes** | Nothing |
| Gate: diagnostic parity | **Passes** on macOS arm64 locally, and on Linux, macOS, and Windows in CI (PR #39) | Nothing |
| Gate: end to end | **Passes** on macOS arm64 locally, and on Linux x64 and arm64, macOS arm64, and Windows x64 in CI (PR #39) | Nothing |
| Gate: performance | **Passes**, every target met | Nothing |
| Open questions | **None open.** Q171 to Q174 resolved as implemented, on the owner's delegation | Nothing |
| Release workflows | Ready; **dry run passes locally for darwin-arm64**; moved to Ubuntu 24.04, macOS 26, and Windows 2025 runners | The workflow's own dry run, in Actions. Nothing is tagged until it passes on every platform |
| Extension packages | **darwin-arm64 installs and works** with `examples/quill` | The other three are smoke-tested by the release run itself |
| Documentation | Written | Your review |
| Release checklist | [RELEASING.md](../RELEASING.md) | Your review |
| Names | All available or already yours | Nothing |

Publishing needs the following from you. **The repository is public now**, which npm provenance requires. **The `release` environment** must exist before the publish jobs can run; [RELEASING.md](../RELEASING.md), "Before the first release", covers it. Before merging PR #39, make the `rust.yml` and `js.yml` checks required in branch protection.

## Release gates

### No unintentional skips

`cargo test -p tessera-conformance`: **366 passed, 0 failed, 0 skipped.** `tests/conformance/SKIPS.toml` is now empty. Its one entry skipped the `output` tag, which no case carries (the emitters' outputs are tested in `tessera-emit`), so removing it changes nothing that runs.

### Diagnostic parity

On macOS arm64:

- `crates/tessera-cli/tests/lsp_parity.rs`: for every build of `examples/quill`, and of a fixture with a problem in every build, `ascribe lsp` (a real process over stdio) publishes exactly what `ascribe check --build <name> --format json` reports: codes, files, spans, severities, messages, and related places. Passes.
- `crates/tessera-cli/tests/build.rs`: `ascribe build` reports what `ascribe check` reports. Passes.
- The VS Code extension's `quill` integration suite, against the real server: 7 passing.

The gate asks for every platform. `rust.yml` runs these tests on Ubuntu, macOS, and Windows, and all three pass on PR #39. Getting Windows to pass took fixes to the tests only (paths and URIs in the `tessera-lsp` and `lsp_parity` tests); the server was correct.

### End to end

On macOS arm64, with the native package staged as npm installs it:

- `examples/astro-site` end-to-end tests (`vitest run test/e2e`): **20 passed**, including a new test that `astro dev` rebuilds when a file outside the content root changes (below).
- VS Code integration suites: activation 2, stub 8, quill 7, preview 9, all passing.
- Preview parity with the built Astro site (`test:parity`): 16 passed.

The gate asks for every platform in CI: `js.yml`'s `astro` matrix covers Linux x64 and arm64, macOS arm64, and Windows x64, and all four pass on PR #39. The first CI run found one test that failed on Linux and sometimes on Windows, and had failed the same way on `main`: the dev-recovery test wrote its fix a few milliseconds after the failed rebuild, and the file watcher dropped the second change. It now rewrites the file until it's rebuilt, like the other watch tests. The release workflow's `smoke` jobs also install the packed CLI and the extension on all four platforms before anything is published.

### Performance

`cargo bench` for the four benchmarks, then `corpora compare` against `tests/corpora/baselines/perf.json` (3x plus 25 ms, and the targets): **every metric ok.** Medians:

| Target | Result |
|---|---|
| `ascribe check` of a 3,000-page project, a few seconds | synthetic: **0.39 s**; converted Elastic sample (3,008 pages, 21 MB): **2.07 s** |
| Diagnostics after a keystroke, about 50 ms | page **1.0 ms**; fragment included by 30 pages **6.7 ms** (3,000 pages) |
| Completion, about 50 ms | **2.6 ms** at most (a link by page title, p95) |

The Elastic miss recorded in `tests/corpora/RESULTS.md` (7.4 s on the phase's container) is gone: the follow-ups made it 2.0 s. These are laptop numbers, faster than that container; the gate is met either way.

## Open questions

None. Q171 to Q174, recorded at the review of phase 22, are resolved as implemented, on the repository owner's delegation (2026-09-30):

- **Q171:** the npm packages that carry the binary: `@ascribed/cli-<os>-<cpu>` for four platforms, with a launcher in `@ascribed/cli`.
- **Q172:** where the Astro integration finds the binary: `binary`, then `ASCRIBE_BIN`, then the installed `@ascribed/cli`.
- **Q173:** how `astro dev` rebuilds, and what a failed rebuild does. Its known gap is closed: the integration reads the site output's manifest after each build and watches each asset outside the content root (`packages/astro/src/dev.ts`, with a unit test and an end-to-end test that fails without the change).
- **Q174:** how CI builds the platform packages: natively on each platform, packed with `npm pack` and `pnpm pack`.

## Names

Checked 2026-09-30. Nothing was registered.

| Name | Status |
|---|---|
| npm organization `ascribed` | Exists (yours); no packages published |
| `@ascribed/cli`, `@ascribed/astro`, `@ascribed/elements`, `@ascribed/cli-{darwin-arm64,linux-arm64,linux-x64,win32-x64}` | Unpublished (`npm view` gives E404): available to the organization |
| GitHub organization `ascribed-dev` | Exists (yours); no public repositories |
| VS Code Marketplace publisher `Ascribe` | Yours (claimed 2026-09-30) |
| Extension `Ascribe.ascribe-vscode` | Not on the Marketplace; no Marketplace extension is named or found by "Ascribe" |

## Versioning

One version, **0.1.0**, set by `node scripts/release/version.ts 0.1.0` in `Cargo.toml` (every crate inherits it, so `ascribe --version` is `ascribe 0.1.0`), `Cargo.lock`, all seven npm packages, the extension, and the extension's `ascribe.minServerVersion`. 0.1.0 matches the specification's version, `spec = "0.1"`; one command changes it if you prefer 1.0.0. `version.ts --check` verifies that every file agrees, that the changelog has the version, and, in the workflow, that the tag matches. The changelog is [CHANGELOG.md](../CHANGELOG.md); its 0.1.0 section is dated when you release.

## Packages

- **Third-party notices:** `scripts/release/notices.ts` generates `THIRD-PARTY-NOTICES` from the crates compiled into the binary, grouped by license text. The GitHub archives, the platform npm packages, and the extension packages include it, and `pack.ts` fails if a package is missing it.
- **npm:** `private` is removed from the seven published packages. Each gains `repository` (which npm provenance checks against the building repository), `homepage`, `bugs`, and `publishConfig.access = public`. The platform packages ship only their binary (`files`), and the Linux ones declare `libc: glibc`, so npm doesn't install a glibc binary on Alpine. Packed tarballs are checked for the version, resolved `workspace:` ranges, a LICENSE, and an executable binary. The packed CLI installs offline and checks `examples/quill` cleanly.
- **Extension:** one VSIX per platform (`vsce package --target`), each holding only its own binary under `bin/<platform>-<arch>/`, plus the bundle, webview files, grammars, README, changelog, and license (`.vscodeignore` allowlists them). The extension stays `private` on npm, which it isn't published to.
- **Extension smoke test** (`scripts/release/smoke-vsix.ts`): installs a VSIX into a fresh VS Code, opens a copy of `examples/quill` with a broken page, and checks that the server runs from the **bundled** binary and reports `ASC001`; then adds a project `node_modules/.bin/ascribe` and checks that the extension **prefers the project's binary**. Passes for darwin-arm64.

## Release workflow

[`.github/workflows/release.yml`](../.github/workflows/release.yml), manual only (`workflow_dispatch`); the Rust and JavaScript workflows run on pull requests and pushes to `main`, and the corpora workflow runs weekly. Every action in every workflow is pinned to a commit SHA, and Dependabot watches Cargo, npm, and Actions.

1. `version`: the version is consistent, and matches the tag when publishing.
2. `build` (4 platforms, natively): `cargo build --release --locked`; checks `--version` and `ascribe check` on `examples/quill`. Windows links the C runtime statically; Linux builds on Ubuntu 24.04 (glibc 2.39), macOS on macOS 26, and Windows on Windows Server 2025.
3. `pack`: `scripts/release/pack.ts`, then `publish.ts --dry-run` for npm and the Marketplace.
4. `smoke` (4 platforms): installs the packed CLI and checks `examples/quill`; runs the extension smoke test.
5. With `publish` checked, each waiting for approval in the `release` environment: `npm` (with provenance), `marketplace`, and `github` (a **draft** release with the archives, the VSIX files, and `SHA256SUMS`). Every step skips what's already published, so a failed run can be repeated.

Local evidence: `actionlint` passes on every workflow. The scripts the jobs run (`pack.ts`, `publish.ts --dry-run`, `smoke-vsix.ts`) ran here for darwin-arm64. The workflow itself hasn't run: its first run is the dry run in RELEASING.md, step 5.

## Documentation

New, in `docs/`: [getting started](../docs/getting-started.md) (installing, `ascribe.toml`, a first page, checking, building, formatting, Astro, CI), the [directive reference](../docs/directives.md), the [`ascribe.toml` reference](../docs/content-model.md) (from `content-model.md`, without the planning parts), the [command reference](../docs/cli.md), the [diagnostics reference](../docs/diagnostics.md), [editing](../docs/editor.md), and [Astro](../docs/astro.md). A root [README](../README.md) is new too.

The diagnostics reference is generated from the registry, which gained a `fix` for each of the 126 diagnostics; `tests/conformance/tests/docs.rs` fails when the page is out of date, and requires every diagnostic to have a fix. The getting-started walkthrough and the README's example were run against the release binary as written.

The package READMEs, which npm and the Marketplace show, are rewritten for users; the extension's contributor notes moved to `packages/vscode/DEVELOPMENT.md`.

## Known limitations

- The binaries aren't signed with an Apple Developer ID or a Windows Authenticode certificate. It matters only for binaries downloaded from the GitHub release with a browser; npm and the extension aren't affected. RELEASING.md says how users clear the macOS quarantine.
- Intel Macs aren't supported (dropped at the owner's request): no `darwin-x64` npm package or extension package.
- Linux needs glibc 2.39 or later; musl systems such as Alpine aren't supported.
- Node.js 24 or later is required, VS Code 1.138 or later for the extension, and Rust 1.98 to build from source. There's no support for older versions.
- The JavaScript packages are on TypeScript 7.0. typescript-eslint doesn't support it, so linting is Oxlint, with its type-aware rules, instead of ESLint. `html5ever` stays at 0.39 (a dev-dependency) because `markup5ever_rcdom` has no release for 0.40.
