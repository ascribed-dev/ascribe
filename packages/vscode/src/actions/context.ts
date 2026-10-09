// The one cached `ascribe/context` answer, for the active editor's
// selection. The context menu's keys, the lightbulb, and the actions read it;
// only it sends the request. A selection change asks after a short delay, and
// a newer one cancels the older; the lightbulb only reads. No VS Code in this
// module: the sending and the timer are given to it.

import type { ContextResult, LspRange } from "../shapes.js";

/** A document's version and a range in it: what an answer is for. */
export interface Where {
  uri: string;
  version: number;
  range: LspRange;
}

/** Sends `ascribe/context` for `where`, giving up when `signal` aborts. */
export type Send = (where: Where, signal: AbortSignal) => Promise<ContextResult>;

/** How long a selection must stay put before its context is asked for. */
export const CONTEXT_DELAY_MS = 150;

const keyOf = ({ uri, version, range: { start, end } }: Where): string =>
  `${uri}@${version}:${start.line}:${start.character}-${end.line}:${end.character}`;

export class ContextCache {
  private latest: { key: string; where: Where; result: ContextResult } | undefined;
  private pending:
    { key: string; promise: Promise<ContextResult>; abort: AbortController } | undefined;
  private timer: ReturnType<typeof setTimeout> | undefined;

  /**
   * @param send sends the request.
   * @param onAnswer called with each new answer, and with `undefined` when
   *   the cache is cleared.
   * @param onError called when a scheduled request fails.
   */
  constructor(
    private readonly send: Send,
    private readonly onAnswer: (
      where: Where | undefined,
      result: ContextResult | undefined,
    ) => void,
    private readonly onError: (error: unknown) => void = () => undefined,
    private readonly delayMs = CONTEXT_DELAY_MS,
  ) {}

  /** The latest answer, when it's for exactly this document version and range; it never asks. */
  get(where: Where): ContextResult | undefined {
    return this.latest?.key === keyOf(where) ? this.latest.result : undefined;
  }

  /**
   * The answer for `where`: the cached one, the one on its way, or a new
   * request, which cancels a scheduled or pending one for anything else.
   */
  request(where: Where): Promise<ContextResult> {
    const key = keyOf(where);
    if (this.latest?.key === key) return Promise.resolve(this.latest.result);
    if (this.pending?.key === key) return this.pending.promise;
    this.cancel();
    const abort = new AbortController();
    const promise = this.send(where, abort.signal).then((result) => {
      if (this.pending?.key === key) this.pending = undefined;
      if (!abort.signal.aborted) {
        this.latest = { key, where, result };
        this.onAnswer(where, result);
      }
      return result;
    });
    promise.catch(() => {
      if (this.pending?.key === key) this.pending = undefined;
    });
    this.pending = { key, promise, abort };
    return promise;
  }

  /**
   * Asks for `where` once it has stayed the selection for the delay. A later
   * call, a request for something else, or `clear` cancels it, and the
   * request it sent if that hasn't been answered.
   */
  schedule(where: Where): void {
    const key = keyOf(where);
    if (this.latest?.key === key || this.pending?.key === key) {
      clearTimeout(this.timer);
      this.timer = undefined;
      return;
    }
    this.cancel();
    this.timer = setTimeout(() => {
      this.timer = undefined;
      this.request(where).catch((error: unknown) => {
        if (!isCancellation(error)) this.onError(error);
      });
    }, this.delayMs);
  }

  /** Forgets the answer and cancels what's scheduled or on its way: another editor is active. */
  clear(): void {
    this.cancel();
    if (this.latest) {
      this.latest = undefined;
      this.onAnswer(undefined, undefined);
    }
  }

  dispose(): void {
    this.cancel();
  }

  private cancel(): void {
    clearTimeout(this.timer);
    this.timer = undefined;
    this.pending?.abort.abort();
    this.pending = undefined;
  }
}

/** Whether a failed request was cancelled, which isn't worth reporting. */
export function isCancellation(error: unknown): boolean {
  return (
    (error instanceof Error && error.name === "AbortError") ||
    // The language client rejects a cancelled request with the protocol's
    // RequestCancelled code.
    (typeof error === "object" && error !== null && (error as { code?: unknown }).code === -32800)
  );
}
