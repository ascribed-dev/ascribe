# Phase 19: Element library

**Track:** Web · **Start after:** 02 · **Parallel with:** everything from wave C onward · **Unblocks:** 21, 25

## Goal

Build `@tessera/elements`: the custom elements the site output uses, implementing `packages/elements/CONTRACT.md` exactly. They must work in any site, render meaningfully without JavaScript, and be accessible.

## Read first

- `packages/elements/CONTRACT.md` (phase 02). It's the specification for this phase.
- [SPEC.md](../../SPEC.md): §9.4 and §9.7.
- [PLAN.md](../PLAN.md): Astro integration and elements.

## Deliverables

- `packages/elements`: the elements, their CSS, and their tests.

## Tasks

1. **Elements.** Plain custom elements with no framework, rendering into the light DOM (SPEC §9.7), exactly as the contract defines: `tessera-note`, `tessera-steps`, `tessera-tabs` with `tessera-tab`, and `tessera-availability`.
   - `tessera-note`, `tessera-steps`, and `tessera-availability` need CSS only, and their content renders meaningfully even if the script never loads.
   - `tessera-tabs` is the only element with behavior. Without JavaScript, every tab shows with its label. With it, the element follows the WAI-ARIA tabs pattern (roles, keyboard navigation). Tabs with the same `sync` value stay in step across the page, and the reader's choice is remembered across pages with `localStorage` (guarded, because storage can be unavailable).
2. **Theming.** CSS custom properties for colors, spacing, borders, and type, with a documented list and sensible defaults. Note types map to colors through properties, and projects can add their own types.
3. **Packaging.** An ES module entry that registers the elements, a CSS file, and a way to import the CSS without the script.
4. **Tests.** Browser tests (for example Playwright or Web Test Runner) covering rendering without the script, tab keyboard navigation, cross-group syncing, remembered choices, and an accessibility audit (for example `axe-core`).

## Acceptance criteria

- [ ] Every element and attribute in `CONTRACT.md` is implemented, and none beyond it.
- [ ] Every element renders its content without the script loaded.
- [ ] Tabs pass keyboard and accessibility tests, and sync across groups with the same `sync` value.
- [ ] `pnpm --filter @tessera/elements test` passes in CI.

## Out of scope

- Emitting these elements from Tessera source (phase 20).
- Changing the contract. Raise needed changes through the contract process.

## Handoff notes

_To be filled in by the implementing agent._
