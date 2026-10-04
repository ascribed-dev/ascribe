// @ascribed/review/overlay: the review overlay, in the browser. A pull
// request's threads beside the blocks of a rendered page, with replying,
// resolving, commenting, and submitting, all through a host
// (`OverlayHost`) that keeps GitHub to itself.
export { createOverlay, type Overlay, type OverlayOptions } from "./overlay.js";
export { renderMarkdown, safeUrl } from "./markdown.js";
export type {
  Anchor,
  CommentTarget,
  HostError,
  LocatedThread,
  OverlayData,
  OverlayHost,
  PendingReview,
  PlacedThreads,
  ReviewEvent,
  ThreadSummary,
} from "./types.js";
export type { Author, Thread, ThreadComment } from "../shared/types.js";
