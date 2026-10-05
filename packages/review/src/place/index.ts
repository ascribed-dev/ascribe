// Placing review threads on rendered blocks: source anchors, moving lines
// between a commit and the working tree, and matching threads to a page.
export {
  anchorKey,
  decodePath,
  encodePath,
  formatSource,
  parseSource,
  type Anchor,
  type SourceRange,
} from "./anchor.js";
export {
  lineHunks,
  lineMap,
  linesAt,
  parseHunks,
  shiftLine,
  type Hunk,
  type LineMap,
  type Shifted,
} from "./lines.js";
export {
  locateThreads,
  placeOnPage,
  type DetachReason,
  type LocateContext,
  type LocatedThread,
  type PageRef,
  type PlacedThreads,
} from "./place.js";
export type { Author, Side, Thread, ThreadComment } from "../shared/types.js";
