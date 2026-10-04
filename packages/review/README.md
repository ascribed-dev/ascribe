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

## Marks

```ts
import { markChanges, setShow, showSources } from "@ascribed/review/marks";
import "@ascribed/review/marks.css";

// `page` holds the page rendered with anchors; `was`, optionally, the page as
// it was. `changes` is one page's `changes` from `ascribe diff --format json`.
const marks = markChanges(page, changes, { was });
setShow(page, "changes"); // or "will" (the page as it will be), or "was"
showSources(page); // each block's source file and lines, on hover
```

Each changed block gets a label (Added, Changed, Removed, Moved) and a bar in the margin whose color and style differ by kind. Inside a changed block, the words added are in `<ins>` and the words removed in `<del>`. A removed block is shown where it was, collapsed to one line with **Show**, copied from the page as it was (or from the change's text without it). A moved block links to a stub at its old place, and the stub back to it. `markChanges` returns the marks in page order, for stepping through them with `goTo`. `clearMarks` removes them all.

The blocks are found by their anchors: the element with exactly a change's anchor, or, when there's none (a paragraph in a tight list item has no element of its own), the smallest anchored element with the same file and includes whose lines contain it.

The colors are the `--ascribe-review-*` custom properties at the top of `marks.css`, with light and dark values.

## The static report

`src/report/` is the script of `ascribe diff --format html`'s report, which draws the list of changed pages and each page with its marks. It isn't an entry point: `pnpm --filter @ascribed/review embed` bundles it with the marks and `@ascribed/elements` into `crates/tessera-diff/src/html/`, where the binary embeds it, along with one stylesheet made of the element library's, `marks.css`, and `src/report/report.css`. It also writes the script's SHA-256 beside it, which the report's content security policy names as the one script allowed to run. Run it after changing any of them; `test/embedded.test.ts` fails until you do.
