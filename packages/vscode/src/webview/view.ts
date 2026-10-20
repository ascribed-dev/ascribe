// The webview side of a sidebar webview view: the handshake, the answers'
// numbers, the state VS Code keeps for the view, and Escape. A view's script
// gives it a function that draws an answer and says what it drew.

import type { FromView, ToView } from "./viewProtocol.js";

declare function acquireVsCodeApi(): {
  postMessage(message: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
};

/** What a view's script gets to talk to the extension with. */
export interface ViewChannel<Report> {
  post(message: FromView<Report>): void;
  /** The answer drawn last, or 0 before the first. */
  readonly seq: number;
  /** What VS Code kept for the view: the same for every document it makes of it. */
  state(): Record<string, unknown>;
  setState(state: Record<string, unknown>): void;
}

/** Content security policy violations seen so far, for the reports. */
const violations: string[] = [];
document.addEventListener("securitypolicyviolation", (event) => {
  violations.push(`${event.violatedDirective} ${event.blockedURI}`);
});

/** The violations seen so far. */
export function seenViolations(): string[] {
  return [...violations];
}

/**
 * Starts a view: draws each answer newer than the last one drawn, says so,
 * and then says it's ready, which the extension answers.
 */
export function startView<Answer, Report>(
  draw: (answer: Answer, channel: ViewChannel<Report>) => Report,
): ViewChannel<Report> {
  const vscode = acquireVsCodeApi();
  let drawn = 0;
  const channel: ViewChannel<Report> = {
    post: (message) => vscode.postMessage(message),
    get seq() {
      return drawn;
    },
    state: () => {
      const state = vscode.getState();
      return typeof state === "object" && state !== null ? (state as Record<string, unknown>) : {};
    },
    setState: (state) => vscode.setState(state),
  };
  window.addEventListener("message", (event: MessageEvent<unknown>) => {
    const message = event.data as Partial<ToView<Answer>> | null;
    if (message?.type !== "answer" || typeof message.seq !== "number") return;
    // An answer older than the one drawn is late: the newer one stands.
    if (message.seq <= drawn) return;
    drawn = message.seq;
    const report = draw(message.answer as Answer, channel);
    channel.post({ type: "drawn", seq: drawn, report });
  });
  // The view doesn't keep the focus: Escape gives it back to the page's editor.
  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape" && !event.defaultPrevented) channel.post({ type: "focusEditor" });
  });
  channel.post({ type: "ready" });
  return channel;
}
