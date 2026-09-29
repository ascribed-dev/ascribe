# Phase 19: Element library

**Track:** Web · **Start after:** 02 · **Parallel with:** everything from wave C onward · **Unblocks:** 21, 25

## Goal

Build `@ascribed/elements`: the custom elements the site output uses, implementing `packages/elements/CONTRACT.md` exactly. They must work in any site, render meaningfully without JavaScript, and be accessible.

## Read first

- `packages/elements/CONTRACT.md` (phase 02). It's the specification for this phase.
- [SPEC.md](../../SPEC.md): §9.4 and §9.7.
- [PLAN.md](../PLAN.md): Astro integration and elements.

## Deliverables

- `packages/elements`: the elements, their CSS, and their tests.

## Tasks

1. **Elements.** Plain custom elements with no framework, rendering into the light DOM (SPEC §9.7), exactly as the contract defines: `ascribe-note`, `ascribe-steps`, `ascribe-tabs` with `ascribe-tab`, and `ascribe-availability`.
   - `ascribe-note`, `ascribe-steps`, and `ascribe-availability` need CSS only, and their content renders meaningfully even if the script never loads.
   - `ascribe-tabs` is the only element with behavior. Without JavaScript, every tab shows with its label. With it, the element follows the WAI-ARIA tabs pattern (roles, keyboard navigation). Tabs with the same `sync` value stay in step across the page, and the reader's choice is remembered across pages with `localStorage` (guarded, because storage can be unavailable).
2. **Theming.** CSS custom properties for colors, spacing, borders, and type, with a documented list and sensible defaults. Note types map to colors through properties, and projects can add their own types.
3. **Packaging.** An ES module entry that registers the elements, a CSS file, and a way to import the CSS without the script.
4. **Tests.** Browser tests (for example Playwright or Web Test Runner) covering rendering without the script, tab keyboard navigation, cross-group syncing, remembered choices, and an accessibility audit (for example `axe-core`).

## Acceptance criteria

- [ ] Every element and attribute in `CONTRACT.md` is implemented, and none beyond it.
- [ ] Every element renders its content without the script loaded.
- [ ] Tabs pass keyboard and accessibility tests, and sync across groups with the same `sync` value.
- [ ] `pnpm --filter @ascribed/elements test` passes in CI.

## Out of scope

- Emitting these elements from Tessera source (phase 20).
- Changing the contract. Raise needed changes through the contract process.

## Handoff notes

**Built.** `packages/elements` implements `CONTRACT.md` with no additions.

- `src/tabs.ts`: `<ascribe-tabs>` and `<ascribe-tab>`, the only behavior (ARIA tabs pattern, arrow/Home/End keys, `sync` across groups, `localStorage` key `ascribe-tabs:<sync>`, guarded, with a per-page fallback). `src/group.ts`: `<ascribe-group>`, an empty class (CSS gives it `display: block`). `src/index.ts` registers those three; notes, steps, and availability are CSS only and are deliberately not registered.
- `css/style.css`: all styling, themed by `--ascribe-*` properties (documented in `README.md`). Generated text comes from attributes (`attr()`), so a note shows `label: heading`, or `label` alone with no heading (Q12, approved), a tab shows its `label`, and the availability lead-in is `ascribe-availability::before`.
- Browser tests in `test/`: Playwright (`playwright-core`) drives Chromium, Firefox, and WebKit (parameterized) against the compiled `dist` and the CSS; axe-core audits pages with and without the script. `test/global-setup.ts` compiles the library first.

**Public interface for phases 21 and 25.**

- `import "@ascribed/elements"` registers the elements; `@ascribed/elements/style.css` is the stylesheet, importable without the script. The package exports `dist/index.js` (run `pnpm --filter @ascribed/elements build` first; `dist/` is git-ignored) and `css/style.css`.
- A project styles its own note types or lifecycle states by selecting on `[type="…"]` or `[states$="…"]` and setting `--_color` and `--_background` (see README).

**Decisions.**

- CI installs Chromium, Firefox, and WebKit (`.github/workflows/js.yml`). Locally Chromium uses `/opt/pw-browsers/chromium` or `ASCRIBE_CHROMIUM`; `ASCRIBE_ENGINES` narrows the engines. This session's container couldn't download Firefox or WebKit, so those two engines were verified in CI only.
- A remembered choice is read from this page's choices first, then `localStorage`, so a failed write can't leave a stale value (tested by making `setItem` throw).
- Availability badges match the last state as a whole token (`[states="ga"], [states$=" ga"]`), so a project state such as `pre-ga` isn't styled as `ga`. **Contract wording:** §4's example selector `[states$="deprecated"]` has the same suffix problem and should read `[states="deprecated"], [states$=" deprecated"]`; it's an example, not behavior, so I left the contract untouched.
- Step numbering is compared against rendered reference markers in every engine, since generated counters can't be read back.
- No global `details` styling: the contract makes it optional, and a global rule would restyle every `<details>` on a site.
- The tab list is rebuilt when a group is connected and removed when it's disconnected, so moving the element (for example, view transitions) is safe. A group whose children arrive after it connects is not re-scanned.
- Step numbers use the `list-item` counter, so an `<ol start>` is honored.

**Decided.** Q12 (a titled note showed its type only by color) was approved: the heading line is `label: heading`. `CONTRACT.md` §1's rendering paragraph and the CSS were updated to match; the emitter's markup is unchanged, so no other phase is affected. The elements' contrast passes axe with the default colors; a site that overrides colors owns its own contrast. Nothing checks the emitted markup against these elements until phase 21.
