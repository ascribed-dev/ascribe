// A sidebar view drawn by a webview: This page, and the views that follow it.
// The extension makes the answers; the webview draws them (`../webview/view.ts`).
// The handshake is `handshake.ts`'s, the messages `../webview/viewProtocol.ts`'s,
// and the document `../webview/viewShell.ts`'s. The webview is never kept
// alive while it's hidden (`retainContextWhenHidden`), which would cost memory
// in every window: the newest answer is kept here and sent when it's shown again.

import { randomBytes } from "node:crypto";
import * as vscode from "vscode";
import { isObject, shape, text, whole, type Check } from "../preview/messages.js";
import type { FromView } from "../webview/viewProtocol.js";
import { viewHtml } from "../webview/viewShell.js";
import { Handshake } from "./handshake.js";

/** What a view's webview drew, as the tests see it. */
export interface Drawn<Report> {
  seq: number;
  report: Report;
  /** When the webview said it had drawn it (`Date.now()`). */
  at: number;
}

/** What a view does with what its webview asks. */
export interface ViewHandlers {
  /** Choose the item `key` of the answer `seq`. */
  open(seq: number, key: string): void;
  /** Escape in the view: give the focus back to the page's editor. */
  focusEditor(): void;
}

/** A message from a view's webview, when it is one the extension takes; `undefined` otherwise. */
function checkViewMessage<Report>(
  value: unknown,
  report: Check<Report>,
): FromView<Report> | undefined {
  if (!isObject(value)) return undefined;
  switch (value["type"]) {
    case "ready":
    case "focusEditor":
      return value as FromView<Report>;
    case "drawn":
      return shape<{ seq: number; report: Report }>({ seq: whole, report })(value)
        ? (value as FromView<Report>)
        : undefined;
    case "open":
      return shape<{ seq: number; key: string }>({ seq: whole, key: text })(value)
        ? (value as FromView<Report>)
        : undefined;
    default:
      return undefined;
  }
}

export class WebviewView<Answer, Report> implements vscode.WebviewViewProvider, vscode.Disposable {
  private view: vscode.WebviewView | undefined;
  private readonly handshake = new Handshake<Answer>();
  private drawnLast: Drawn<Report> | undefined;
  private waiters: (() => void)[] = [];
  private readonly viewDisposables: vscode.Disposable[] = [];
  private readonly shown = new vscode.EventEmitter<void>();
  /** The view was shown: made, or shown again after it was hidden. */
  readonly onDidShow = this.shown.event;

  constructor(
    private readonly id: string,
    private readonly extensionUri: vscode.Uri,
    private readonly files: { script: string; title: string },
    private readonly report: Check<Report>,
    private readonly handlers: ViewHandlers,
  ) {}

  register(): vscode.Disposable {
    return vscode.window.registerWebviewViewProvider(this.id, this);
  }

  /** Whether the view is showing: made, and neither collapsed nor in a hidden container. */
  get visible(): boolean {
    return this.view?.visible ?? false;
  }

  /** The newest answer: what the view draws next, or has drawn. */
  get latest(): Answer | undefined {
    return this.handshake.latest?.answer;
  }

  /** What the webview drew last. */
  get drawn(): Drawn<Report> | undefined {
    return this.drawnLast;
  }

  /** Sends an answer, now if the webview is ready, and otherwise when it is. */
  show(answer: Answer): void {
    const message = this.handshake.answer(answer);
    if (message) void this.view?.webview.postMessage(message);
  }

  /** Drops the newest answer, which is out of date: the view draws nothing until it gets another. */
  forget(): void {
    this.handshake.forget();
  }

  /** The answer `seq`, if it's the newest: what a choice in the webview names. */
  isLatest(seq: number): boolean {
    return this.handshake.latest?.seq === seq;
  }

  resolveWebviewView(view: vscode.WebviewView): void {
    for (const disposable of this.viewDisposables.splice(0)) disposable.dispose();
    this.view = view;
    this.handshake.reset();
    const root = vscode.Uri.joinPath(this.extensionUri, "dist", "webview");
    view.webview.options = { enableScripts: true, localResourceRoots: [root] };
    const file = (name: string) => view.webview.asWebviewUri(vscode.Uri.joinPath(root, name));
    view.webview.html = viewHtml({
      cspSource: view.webview.cspSource,
      nonce: randomBytes(16).toString("base64"),
      script: file(this.files.script).toString(),
      style: file("views.css").toString(),
      title: this.files.title,
    });
    this.viewDisposables.push(
      view.webview.onDidReceiveMessage((message: unknown) => this.receive(message)),
      // A hidden view's document is thrown away and made again when it's
      // shown, and says it's ready again: what's sent between is lost, and
      // the ready gets the newest answer.
      view.onDidChangeVisibility(() => {
        if (view.visible) this.shown.fire();
      }),
      view.onDidDispose(() => {
        if (this.view === view) this.view = undefined;
        this.handshake.reset();
      }),
    );
    this.shown.fire();
  }

  private receive(value: unknown): void {
    const message = checkViewMessage(value, this.report);
    if (!message) return;
    switch (message.type) {
      case "ready": {
        const answer = this.handshake.readied();
        if (answer) void this.view?.webview.postMessage(answer);
        return;
      }
      case "drawn":
        this.drawnLast = { seq: message.seq, report: message.report, at: Date.now() };
        for (const waiter of this.waiters) waiter();
        return;
      case "open":
        this.handlers.open(message.seq, message.key);
        return;
      case "focusEditor":
        this.handlers.focusEditor();
        return;
    }
  }

  /** Waits until what the webview drew satisfies `predicate`. */
  whenDrawn(
    description: string,
    predicate: (drawn: Drawn<Report>) => boolean,
    timeout = 10_000,
  ): Promise<Drawn<Report>> {
    return new Promise((resolve, reject) => {
      const deadline = setTimeout(() => {
        this.waiters = this.waiters.filter((w) => w !== check);
        reject(new Error(`Timed out waiting for the view to draw ${description}`));
      }, timeout);
      const check = (): void => {
        const drawn = this.drawnLast;
        if (drawn && predicate(drawn)) {
          clearTimeout(deadline);
          this.waiters = this.waiters.filter((w) => w !== check);
          resolve(drawn);
        }
      };
      this.waiters.push(check);
      check();
    });
  }

  dispose(): void {
    for (const disposable of this.viewDisposables.splice(0)) disposable.dispose();
    this.shown.dispose();
  }
}
