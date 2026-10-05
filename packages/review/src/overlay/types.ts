// What the overlay gets from its host, and what it gives back. The types are
// shared with `./github` and `./place`, but only as types: nothing of them
// runs in the browser.
import type { CommentTarget, PendingReview, ReviewEvent } from "../github/session.js";
import type { Anchor } from "../place/anchor.js";
import type { LocatedThread, PlacedThreads } from "../place/place.js";
import type { Thread, ThreadComment } from "../shared/types.js";

export type { Anchor, CommentTarget, LocatedThread, PendingReview, PlacedThreads, ReviewEvent };

/** What the overlay draws for the page it's on. */
export interface OverlayData {
  /** The pull request: its number names it ("#128"). */
  pullRequest: { number: number; url: string };
  /** The page's threads, placed on its blocks (`ReviewSession.threads`). */
  threads: PlacedThreads;
  /** The viewer's pending review: what's unsent. */
  pending: PendingReview;
  /** The signed-in viewer's login. */
  viewer: string;
}

/** A thread on the pull request, and the pages it's on. */
export interface ThreadSummary {
  thread: LocatedThread;
  /** The pages that show it, by content path; empty when no page is known to. */
  pages: { path: string; title: string | null }[];
}

/** A failure the host reports: `message` is a sentence to show; `code` is a `ReviewError` code. */
export interface HostError {
  message: string;
  code?: string;
}

/**
 * Where the overlay gets its data and sends what the reviewer does. A host
 * keeps GitHub to itself: the overlay never sees a token or makes a request.
 */
export interface OverlayHost {
  load(): Promise<OverlayData>;
  /** How a comment on the block would go: as a thread, in the review's summary, or not yet. */
  commentTarget(anchor: Anchor): Promise<CommentTarget>;
  /** Comments on a block. `quote` is the block's text as the page shows it. */
  comment(anchor: Anchor, body: string, quote?: string): Promise<Thread>;
  reply(threadId: string, body: string, when: "now" | "withReview"): Promise<ThreadComment>;
  /** Every thread on the pull request's pages, with the pages each is on. */
  allThreads(): Promise<ThreadSummary[]>;
  resolve(threadId: string, resolved: boolean): Promise<void>;
  submit(event: ReviewEvent, body?: string): Promise<void>;
  /** Deletes the pending review and everything in it. */
  discard(): Promise<void>;
  openSource(anchor: Anchor): void;
  /** Shows a thread that's on another page: the host opens that page, and its overlay goes to the thread. */
  openThread(threadId: string, page: string): void;
  /** Opens a link from a comment. Without it, the link opens as the page would open it. */
  openLink?(url: string): void;
  /** Tells the reviewer something that happened, as the host shows notices. */
  notify?(message: string): void;
  /** Calls `listener` when the page re-rendered or the threads changed. */
  onDidChange(listener: () => void): void;
}
