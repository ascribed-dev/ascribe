# The website, and the brand on everything else

The new brand applied: ascribed-dev.com rebuilt around [the story](../brand/story.md), and every surface Ascribe ships moved to the colors and type of [the identity](../brand/identity.md). The mark and its images are done (decision [#801](../decisions/801-pixel-mark.md)); the pages' own colors, the landing page, and the words on the other surfaces are what's left, and this is the plan for them.

## Who does what

- **The maintainer decides and looks:** the palette in the specimen before phase 1 merges, the landing page's mockup before phase 2 builds it, and the Marketplace publisher in phase 3, which only the maintainer can create.
- **An agent does the rest,** a phase at a time, each phase its own pull request.

## Decisions

These are settled. Don't reopen them in a phase; if one can't be met, stop and report.

1. **Everything is re-themed, not just the site.** The shared `color.*` tokens move to Deep River and Shallows, so the element library, review, the HTML report, the archive, the Astro toolbar, the site, and the VS Code preview's fallbacks change together. The site is built as a user's is, so it doesn't override what the product ships. _From:_ the maintainer, 21 October 2026.
2. **In VS Code, the user's theme still wins** (decision 47). The re-theme reaches the editor only where a theme leaves a `--vscode-*` variable out, and in the preview's frame.
3. **The landing page is rebuilt, and stays an Ascribe project:** written in `site/home/` with widgets, as it is now, and new widgets added where its layout needs them. _From:_ the maintainer, 21 October 2026.
4. **The copy comes from the story.** The headline, the sentence that says what Ascribe is, the pillars, the voice, and the words. No other tool is named, and the copy doesn't say "help authoring tool", "platform", or a superlative. Bryo Studio is the maker.
5. **The colors' roles are the identity's.** River is the accent and what changed. Moss is a tip, what was added, and what was reviewed. Amber is focus. Each has a text value and a graphic value in light (`#0b6f8e` and `#1689ab` for river, `#2f6b2b` and `#3f8a3a` for moss) and one value in dark.
6. **Headings are set in the system's serif:** `ui-serif`, then Charter, Cambria, and Georgia. Body and UI text stay in the system font, and code in `ui-monospace`. No web font ships (decision 48).
7. **The site follows the system's scheme.** There's no theme picker.
8. **Nothing has to be kept.** Ascribe is before 1.0 and has no users, so the extension's publisher and id can change without a migration.
9. **The gutter's split bar on notes is the site's own look,** a rule in `site.css`. The element library's default stays one bar, so no `--ascribe-*` property is added. _From:_ the maintainer, 21 October 2026.
10. **The mark moves.** Phase 4 builds the motion and isn't dropped. _From:_ the maintainer, 21 October 2026.

## The thread

**The use case:** someone opens ascribed-dev.com, any page of the docs, or a review report, and sees the new brand. The pages are in Deep River or Shallows, following their system, the headings are in the serif, and the landing page leads with "Docs as code, with the tools docs need."

**The layers it crosses:** `design/tokens.toml` and the generator, every generated stylesheet (the element library, review's marks and overlay, the report, the archive, the toolbar, the site, and the preview's fallbacks), the report embedded in the binary, the site's build and deploy, the screenshots, and the landing page's hero.

**The seam it tests:** whether Deep River's few hues make a whole UI palette. That means every semantic color, the notes, the states, and review's added, changed, removed, and moved, each passing 4.5:1 for text and 3:1 for a mark in both schemes, and still told apart from each other. River is both the accent and "changed", amber is focus and must not read as a warning, and moss must stay distinct from river for a reader with red-green color blindness. It also tests whether serif headings hold up on Windows and Linux, which have no `ui-serif`. If either fails, the hues or the stack change before a layout is built on them.

**Why the thread isn't thinner.** The shared tokens move every surface at once. Re-theming the neutrals now and the notes later would ship two palettes on one page, with two **Behavior change** entries. The palette is one change. The headline rides with it because it's one line and makes the change something a visitor can see.

**Left out, and where it comes back:**

| Left out | Phase |
|---|---|
| The landing page's layout: the pillars, the audiences, the call to action above the fold | 2 |
| Why Ascribe and Credits, read against the story | 2 |
| The one sentence on every other surface: the README, npm, the Marketplace, the command's help, the plugin | 3 |
| Bryo Studio as the publisher and the plugin's author | 3 |
| The gutter's split bar on notes, the three states, the mark in motion | 4 |

## Phases

| Phase | Result |
|---|---|
| [1: The new colors and type, everywhere](phase-1-thread.md) | Every page and report Ascribe draws is in Deep River or Shallows with serif headings, and the landing page leads with the story's headline. |
| [2: The landing page](phase-2-landing.md) | The landing page rebuilt around the story's pillars, from a mockup the maintainer approved, and Why Ascribe and Credits read against the story. |
| [3: The words everywhere, and the maker](phase-3-words.md) | One file holds the sentence and the headline, every other surface takes it or is tested against it, and Bryo Studio publishes the extension and the plugin. |
| [4: The gutter, and the mark in motion](phase-4-motif.md) | Notes carry the gutter's split bar, the three states show where the site shows writing, changes, and review, and the mark moves for anyone who hasn't asked it not to. |

Phase 1 first. Phases 2 and 3 can run in either order after it. Phase 4 needs phase 2's layout.

## Rules for every phase

- Branch before committing; never commit to `main`.
- Follow [the checklist for a change to how something looks](../checklists.md#a-change-to-how-something-looks). Every color is in `design/tokens.toml`, and nothing else writes one.
- Copy comes from [the story](../brand/story.md). If a page needs a sentence the story doesn't support, the story changes first.
- Every example on a page outside the docs is real: a file read with `@snippet`, a command's tested output, or a screenshot taken by the script.
- Each pull request has screenshots in light and dark of every surface it reaches.
- Before finishing a phase, the checks in `AGENTS.md` pass, and `cd site && npm test` passes after a build.

## Not in this plan

- The docs' structure and content, apart from getting started's first sentence in phase 3.
- VS Code's own UI, which follows the user's theme.
- A theme picker, a web font, or a hosted anything.
- Naming other tools, on the site or elsewhere.
