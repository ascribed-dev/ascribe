# Phase 09: Slugger

**Track:** Resolve · **Start after:** 02 · **Parallel with:** 03, 05, 08, 19 · **Unblocks:** 11

## Goal

Generate heading slugs exactly as the consumer does, so ids Tessera validates are the anchors the published site has. Start with a faithful port of `github-slugger`, which Astro uses.

## Read first

- [SPEC.md](../../SPEC.md): §5.5 (source ids and page ids) and §9.5.
- `crates/tessera-core`'s `Slugger` trait (phase 02).
- The `github-slugger` JavaScript package's source and test fixtures.

## Deliverables

- `crates/tessera-resolve/src/slug/`: a `GithubSlugger` implementing `tessera-core`'s `Slugger` trait, and a registry that returns a slugger by the name the consumer profile uses.

## Tasks

1. **Port.** Reproduce `github-slugger`'s behavior exactly: lowercasing, the set of removed characters (the package uses a large Unicode regular expression; port it faithfully), how spaces become hyphens, and duplicate numbering (`intro`, `intro-1`, `intro-2`). Pin the upstream version you ported and record it in the module's docs.
2. **Scopes.** Duplicate numbering is per scope. Phase 11 uses one scope per source file (source ids); phase 12 uses one scope per expanded page (page ids). Make starting a fresh scope explicit in the API.
3. **Fixtures.** Port the upstream test fixtures and make them all pass. Add cases for emoji, non-Latin scripts, punctuation-only headings, and repeated headings.
4. **Registry.** Return a slugger by name, with `github` as the default.

## Acceptance criteria

- [x] Every ported upstream fixture passes.
- [x] Duplicate headings number exactly as `github-slugger` numbers them, within a scope.
- [x] The implementation records the upstream version it matches.

## Out of scope

- Using the slugger on documents (phases 11 and 12).
- Other consumers' sluggers.

## Notes

- A small mismatch here means links that validate but break on the live site. Match upstream behavior exactly, even when it seems odd, and document any known difference.

## Handoff notes

### What I built

`crates/tessera-resolve/src/slug/`:

- `github.rs`: `GithubSlugger` (implements `tessera_core::Slugger`, `name() == "github"`), `GithubScope` (implements `SlugScope`), and `github_slug(text)`, upstream's standalone `slug()` without numbering.
- `table.rs`: the removed-character table, as scalar-value ranges. Generated.
- `mod.rs`: the registry and the pinned-version constant.
- `generate.mjs`, `fixtures/`: the generator and the fixtures it writes, both from the real package's output.

### Public interfaces

- `tessera_resolve::slug::slugger_by_name(&str) -> Option<Box<dyn Slugger>>` (exact names; `github` only), `default_slugger()`, `DEFAULT_SLUGGER_NAME`, `SLUGGER_NAMES`.
- `Slugger::new_scope()` starts an empty scope; a scope numbers a slug only against slugs it gave out earlier. Phase 11 makes one scope per source file, phase 12 one per expanded page. Call `scope.slug(text)` only for headings without `@id` (SPEC §5.5).
- `GITHUB_SLUGGER_VERSION` is `"2.0.0"`.

### Decisions

- **Pinned `github-slugger` 2.0.0**, the latest release. Its `regex.js` is over UTF-16 code units with explicit surrogate-pair alternatives, so the table is exact per scalar value; the generator tests each scalar against the real regex instead of transcribing it.
- **Numbering is upstream's, quirks included.** Every result is recorded, so `a, a, a-1` gives `a, a-1, a-1-1`. The count is per original slug.
- **Lowercasing** uses `str::to_lowercase`. It matches JavaScript's `toLowerCase` on every scalar and on the context cases in the fixtures (final sigma, `İ`). Both use the full Unicode mappings, but they could diverge when the Rust toolchain's Unicode version and Node's differ on newly added characters; the fixtures pin the current behavior.
- Odd upstream results are kept: the hyphen stays (`a - b` is `a---b`), emoji are removed but U+FE0F stays, a punctuation-only heading has the empty slug (and numbers as `-1`, `-2`).
- The empty slug is returned as `""`, not rejected. Phase 11 should decide what an id-less heading with an empty slug means (see Left open).

### Testing

The fixtures were generated from the real package (`github-slugger` 2.0.0 from npm): 734 single texts (hand-picked Latin, Unicode, emoji, punctuation, and seeded random strings, including random scalars from all of Unicode), 18 duplicate sequences, and every Unicode scalar checked one at a time.

**Upstream's own test fixtures** (`test/fixtures.json` from `Flet/github-slugger` at 2.0.0, 78 entries) are vendored unchanged as `fixtures/upstream.json`. They weren't reachable from the implementing session, so they were added afterwards. All 78 pass, run in order through one scope as upstream's test runs them (`upstream_test_fixtures_pass`).

### Left open

- **Empty slugs.** A heading such as `## ???` or `## 🎉` slugs to `""`, and a second one to `-1`. Such an id can't be linked to usefully; how Astro treats an empty id is unverified. Phase 11 or 12 may want a diagnostic. No spec question raised yet, because SPEC doesn't say and the conservative behavior (report what the consumer does) is implemented.
- **Upgrading upstream.** Rerun `generate.mjs` against the new version and update `GITHUB_SLUGGER_VERSION`; the tests fail if the fixtures' recorded version differs from the constant.

