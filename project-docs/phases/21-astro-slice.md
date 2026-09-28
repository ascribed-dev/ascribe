# Phase 21: Astro end-to-end slice

**Track:** Web · **Start after:** 19, 20 · **Parallel with:** 26 · **Unblocks:** 16, 22, 25 (to finish)

## Goal

Prove the whole pipeline in a real Astro site as early as possible, with the smallest integration that works. The full editor experience (phase 16 onward) waits for this slice, because the integration failures it catches, such as asset paths, heading ids, and partial variant selection, are the ones snapshot tests of Tessera's own output miss.

## Read first

- [SPEC.md](../../SPEC.md): §9.4–§9.7.
- `project-docs/contracts/site-render.md`, `tests/render/`, and `project-docs/contracts/assets.md` (phase 02).
- Handoff notes from phases 19 and 20.
- The current Astro documentation for integrations, content collections, and markdown plugins.

## Deliverables

- `packages/astro`: a minimal `@tessera/astro`.
- `examples/astro-site`: a plain Astro site (no Starlight) that uses it.
- An end-to-end test in CI on Linux.

## Tasks

1. **Minimal integration.** `@tessera/astro`:
   - Runs `tessera build --emit site` for a configured build before Astro loads content. For this phase, it uses the locally built binary; npm distribution is phase 22.
   - Writes the generated Zod schema (phase 20), and exports a helper that defines the content collection over the build output with that schema.
   - Registers a markdown plugin implementing the site-render contract (heading ids and image attributes), so Astro keeps its own heading and table-of-contents handling. The plugin must pass every `tests/render/` fixture, the same fixtures phase 20's renderer passes.
   - Loads `@tessera/elements` (script and CSS) on pages that render Tessera content.
   - Fails the Astro build when Tessera reports errors.
2. **Sample site.** `examples/astro-site`: a layout that renders a page, its availability badge from frontmatter, and the elements.
3. **The slice's content.** A small project in the sample site covering exactly the risky integrations:
   - a page that includes a fragment whose image sits beside the fragment;
   - a heading with an explicit `@id`, and a link to it from another page;
   - a build with a partial variant selection that reduces one group to one arm and leaves another group as tabs;
   - an availability annotation.
4. **End-to-end test.** Build the site in CI and check the built HTML in a browser: the fragment's image loads (and Astro processed it), the explicit-id link lands on the heading, the reduced group shows one arm's content and the other group works as tabs, and the badge renders.

## Acceptance criteria

- [ ] The Astro plugin passes every `tests/render/` fixture.
- [ ] The end-to-end test passes in CI on Linux, checking each item in task 3.
- [ ] Heading ids in the built HTML match the page ids `tessera check` validated.

## Out of scope

- npm distribution of the binary, dev-mode rebuilding, and other platforms (phase 22).

## Notes

- Verify every Astro API you use against the current documentation and record the Astro version targeted. Don't rely on memory for integration hook names or loader APIs.

## Handoff notes

_To be filled in by the implementing agent._
