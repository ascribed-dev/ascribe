// The extension's side of a webview view's handshake (`../webview/viewProtocol.ts`),
// apart from VS Code so the unit tests can play it out. VS Code makes the
// view's webview late, throws its document away when the view is hidden, and
// makes it again when it's shown; a message sent before its script runs is
// lost. So nothing is sent until the webview says it's ready, and then the
// newest answer is, not the one there was when the webview was made: the
// cursor may have moved in between.

import type { ToView } from "../webview/viewProtocol.js";

export class Handshake<Answer> {
  private seq = 0;
  private newest: ToView<Answer> | undefined;
  private ready = false;

  /** The newest answer, which the next ready gets. */
  get latest(): ToView<Answer> | undefined {
    return this.newest;
  }

  /** A webview was made, or its document thrown away: nothing reaches it until it's ready. */
  reset(): void {
    this.ready = false;
  }

  /** Drops the newest answer, which is out of date: the next ready gets nothing until a new one. */
  forget(): void {
    this.newest = undefined;
  }

  /** A new answer, numbered after the last; returned when it can be sent now. */
  answer(answer: Answer): ToView<Answer> | undefined {
    this.newest = { type: "answer", seq: ++this.seq, answer };
    return this.ready ? this.newest : undefined;
  }

  /** The webview is ready: the newest answer, for every ready, so a webview that reloads is drawn again. */
  readied(): ToView<Answer> | undefined {
    this.ready = true;
    return this.newest;
  }
}
