# @ascribed/review

Reviewing [Ascribe](https://github.com/ascribed-dev/ascribe) pages as readers see them: what a change does to each rendered page, and a pull request's review threads beside the blocks they're about.

Each entry point is separate, so a host bundles only what it uses.

| Entry point | Runs in | What it's for |
|---|---|---|
| `@ascribed/review/marks` | the browser | Marking the changed blocks of a rendered page, from `ascribe diff --format json`'s changes |
| `@ascribed/review/overlay` | the browser | The review overlay: changed blocks and review threads on a rendered page |
| `@ascribed/review/github` | Node | Reading and writing a pull request's review threads on GitHub |
| `@ascribed/review/place` | Node | Placing review threads on the blocks of a page |

The rendered page is Ascribe's site output with source anchors (`ascribe build --emit site --anchors`, and always in the editor's page preview), which say where each block is written. The [site-render contract](https://github.com/ascribed-dev/ascribe/blob/main/docs/contracts/site-render.md#7-source-anchors) describes them.
