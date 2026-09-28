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

- [ ] Every ported upstream fixture passes.
- [ ] Duplicate headings number exactly as `github-slugger` numbers them, within a scope.
- [ ] The implementation records the upstream version it matches.

## Out of scope

- Using the slugger on documents (phases 11 and 12).
- Other consumers' sluggers.

## Notes

- A small mismatch here means links that validate but break on the live site. Match upstream behavior exactly, even when it seems odd, and document any known difference.

## Handoff notes

_To be filled in by the implementing agent._
