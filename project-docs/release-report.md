# Release report: Ascribe 0.1.0

Prepared 2026-09-30 for the first release. Everything here was run on one machine, an Apple-silicon Mac (macOS, arm64). Nothing ran in GitHub Actions: CI is manual-only, and no Actions minutes were available. Linux and Windows are unverified until the release workflow's dry run, or `rust.yml` and `js.yml`, run by hand.

## Summary

| | Status | What's left |
|---|---|---|
| Gate: no unintentional skips | **Passes** | Nothing |
| Gate: diagnostic parity | **Passes on macOS arm64** | Run `rust.yml` by hand for Linux and Windows |
| Gate: end to end | **Passes on macOS arm64** | Run `js.yml` by hand for the other four platforms |
| Gate: performance | **Passes**, every target met | Nothing |
| Open questions | Q171 to Q174 **await your approval** | Approve, or amend |
| Release workflows | Ready; **dry run passes locally for darwin-arm64** | The workflow's own dry run, in Actions |
| Extension packages | **darwin-arm64 installs and works** with `examples/quill` | The other three: the workflow's smoke jobs |
| Documentation | Written | Your review |
| Release checklist | [RELEASING.md](../RELEASING.md) | Your review |
| Names | All available or already yours | Nothing |

Two things block publishing, and both are yours to decide: **the repository is private**, and npm refuses provenance from a private repository; and **the `release` environment doesn't exist yet**. [RELEASING.md](../RELEASING.md), "Before the first release", covers both.

## Release gates

### No unintentional skips

`cargo test -p tessera-conformance`: **366 passed, 0 failed, 0 skipped.** `tests/conformance/SKIPS.toml` is now empty. Its one entry skipped the `output` tag, which no case carries (the emitters' outputs are tested in `tessera-emit`), so removing it changes nothing that runs.

### Diagnostic parity

On macOS arm64:

- `crates/tessera-cli/tests/lsp_parity.rs`: for every build of `examples/quill`, and of a fixture with a problem in every build, `ascribe lsp` (a real process over stdio) publishes exactly what `ascribe check --build <name> --format json` reports: codes, files, spans, severities, messages, and related places. Passes.
- `crates/tessera-cli/tests/build.rs`: `ascribe build` reports what `ascribe check` reports. Passes.
- The VS Code extension's `quill` integration suite, against the real server: 7 passing.

The gate asks for every platform. `rust.yml` runs these tests on Ubuntu, macOS, and Windows, and waits for a manual run.

### End to end

On macOS arm64, with the native package staged as npm installs it:

- `examples/astro-site` end-to-end tests (`vitest run test/e2e`): **20 passed**, including a new test that `astro dev` rebuilds when a file outside the content root changes (below).
- VS Code integration suites: activation 2, stub 8, quill 7, preview 9, all passing.
- Preview parity with the built Astro site (`test:parity`): 16 passed.

The gate asks for every platform in CI: `js.yml`'s `astro` matrix (Linux x64 and arm64, macOS arm64, Windows x64) waits for a manual run. The release workflow's `smoke` jobs also install the packed CLI and the extension on all four platforms.

### Performance

`cargo bench` for the four benchmarks, then `corpora compare` against `tests/corpora/baselines/perf.json` (3x plus 25 ms, and the targets): **every metric ok.** Medians:

| Target | Result |
|---|---|
| `ascribe check` of a 3,000-page project, a few seconds | synthetic: **0.39 s**; converted Elastic sample (3,008 pages, 21 MB): **2.07 s** |
| Diagnostics after a keystroke, about 50 ms | page **1.0 ms**; fragment included by 30 pages **6.7 ms** (3,000 pages) |
| Completion, about 50 ms | **2.6 ms** at most (a link by page title, p95) |

The Elastic miss recorded in `tests/corpora/RESULTS.md` (7.4 s on the phase's container) is gone: the follow-ups made it 2.0 s. These are laptop numbers, faster than that container; the gate is met either way.

## Open questions

Q171 to Q174 are open, recorded at the review of phase 22, and each describes what's implemented:

- **Q171:** the npm packages that carry the binary: `@ascribed/cli-<os>-<cpu>`, with a launcher in `@ascribed/cli`.
- **Q172:** where the Astro integration finds the binary: `binary`, then `ASCRIBE_BIN`, then the installed `@ascribed/cli`.
- **Q173:** how `astro dev` rebuilds, and what a failed rebuild does. **Its known gap is now closed:** the integration reads the site output's manifest after each build and watches each asset outside the content root (`packages/astro/src/dev.ts`, with a unit test and an end-to-end test that fails without the change).
- **Q174:** how CI builds the platform packages: natively on each platform, packed with `npm pack` and `pnpm pack`.

Recommendation: approve all four as implemented. They concern what's published, so the checklist's first step is their approval.

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

One version, **0.1.0**, set by `node scripts/release/version.mjs 0.1.0` in `Cargo.toml` (every crate inherits it, so `ascribe --version` is `ascribe 0.1.0`), `Cargo.lock`, all seven npm packages, the extension, and the extension's `ascribe.minServerVersion`. 0.1.0 matches the specification's version, `spec = "0.1"`; one command changes it if you prefer 1.0.0. `version.mjs --check` verifies that every file agrees, that the changelog has the version, and, in the workflow, that the tag matches. The changelog is [CHANGELOG.md](../CHANGELOG.md); its 0.1.0 section is dated when you release.

## Packages

- **npm:** `private` is removed from the seven published packages. Each gains `repository` (which npm provenance checks against the building repository), `homepage`, `bugs`, and `publishConfig.access = public`. The platform packages ship only their binary (`files`), and the Linux ones declare `libc: glibc`, so npm doesn't install a glibc binary on Alpine. Packed tarballs are checked for the version, resolved `workspace:` ranges, a LICENSE, and an executable binary. The packed CLI installs offline and checks `examples/quill` cleanly.
- **Extension:** one VSIX per platform (`vsce package --target`), each holding only its own binary under `bin/<platform>-<arch>/`, plus the bundle, webview files, grammars, README, changelog, and license (`.vscodeignore` allowlists them). The extension stays `private` on npm, which it isn't published to.
- **Extension smoke test** (`scripts/release/smoke-vsix.mjs`): installs a VSIX into a fresh VS Code, opens a copy of `examples/quill` with a broken page, and checks that the server runs from the **bundled** binary and reports `ASC001`; then adds a project `node_modules/.bin/ascribe` and checks that the extension **prefers the project's binary**. Passes for darwin-arm64.

## Release workflow

[`.github/workflows/release.yml`](../.github/workflows/release.yml), manual only (`workflow_dispatch`), like the other workflows:

1. `version`: the version is consistent, and matches the tag when publishing.
2. `build` (4 platforms, natively): `cargo build --release --locked`; checks `--version` and `ascribe check` on `examples/quill`. Windows links the C runtime statically; Linux builds on Ubuntu 22.04 (glibc 2.35).
3. `pack`: `scripts/release/pack.mjs`, then `publish.mjs --dry-run` for npm and the Marketplace.
4. `smoke` (4 platforms): installs the packed CLI and checks `examples/quill`; runs the extension smoke test.
5. With `publish` checked, each waiting for approval in the `release` environment: `npm` (with provenance), `marketplace`, and `github` (a **draft** release with the archives, the VSIX files, and `SHA256SUMS`). Every step skips what's already published, so a failed run can be repeated.

Local evidence: `actionlint` passes on every workflow. The scripts the jobs run (`pack.mjs`, `publish.mjs --dry-run`, `smoke-vsix.mjs`) ran here for darwin-arm64. The workflow itself hasn't run: its first run is the dry run in RELEASING.md, step 5.

## Documentation

New, in `docs/`: [getting started](../docs/getting-started.md) (installing, `ascribe.toml`, a first page, checking, building, formatting, Astro, CI), the [directive reference](../docs/directives.md), the [`ascribe.toml` reference](../docs/content-model.md) (from `content-model.md`, without the planning parts), the [command reference](../docs/cli.md), the [diagnostics reference](../docs/diagnostics.md), [editing](../docs/editor.md), and [Astro](../docs/astro.md). A root [README](../README.md) is new too.

The diagnostics reference is generated from the registry, which gained a `fix` for each of the 126 diagnostics; `tests/conformance/tests/docs.rs` fails when the page is out of date, and requires every diagnostic to have a fix. The getting-started walkthrough and the README's example were run against the release binary as written.

The package READMEs, which npm and the Marketplace show, are rewritten for users; the extension's contributor notes moved to `packages/vscode/DEVELOPMENT.md`.

## Known limitations

- The binaries aren't signed with an Apple Developer ID or a Windows Authenticode certificate. It matters only for binaries downloaded from the GitHub release with a browser; npm and the extension aren't affected. RELEASING.md says how users clear the macOS quarantine.
- Intel Macs aren't supported (dropped at the owner's request): no `darwin-x64` npm package or extension package.
- Linux needs glibc 2.35 or later; musl systems such as Alpine aren't supported.
- Linux and Windows are verified only by CI runs that haven't happened yet.
