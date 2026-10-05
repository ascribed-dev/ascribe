// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createOverlay, type Overlay } from "../src/overlay/index.js";
import type {
  CommentTarget,
  OverlayData,
  OverlayHost,
  ThreadSummary,
} from "../src/overlay/types.js";
import type { Anchor } from "../src/place/anchor.js";
import type { LocatedThread } from "../src/place/place.js";
import type { ThreadComment } from "../src/shared/types.js";
import { blockText } from "../src/overlay/text.js";

const PAGE = `
<p data-ascribe-source="guide.md:3-3">Run the installer.</p>
<ul data-ascribe-source="guide.md:5-6">
<li data-ascribe-source="guide.md:5-5">Linux</li>
<li data-ascribe-source="guide.md:6-6">macOS</li>
</ul>
<ascribe-tabs data-ascribe-source="guide.md:8-12">
<div role="tablist"><button role="tab" aria-selected="true">Cloud</button><button role="tab" aria-selected="false">Self-hosted</button></div>
<ascribe-tab label="Cloud"><p data-ascribe-source="guide.md:9-9">Open the console.</p></ascribe-tab>
<ascribe-tab label="Self-hosted" hidden><p data-ascribe-source="guide.md:11-11">Edit lantern.yaml.</p></ascribe-tab>
</ascribe-tabs>
<details data-ascribe-source="guide.md:14-16"><summary>Why?</summary><p data-ascribe-source="guide.md:15-15">Because.</p></details>
<div class="ascribe-removed" data-ascribe-ui="" data-ascribe-change="removed" data-ascribe-was-source="guide.md:18-18">Gone text.</div>
`;

let n = 0;
function comment(body: string, init: Partial<ThreadComment> = {}): ThreadComment {
  return {
    id: `C${++n}`,
    author: { login: "maya", avatarUrl: undefined },
    body,
    createdAt: new Date(Date.now() - 2 * 3600_000).toISOString(),
    url: "https://github.com/acme/docs/pull/7",
    pending: false,
    ...init,
  };
}

function thread(id: string, line: number, init: Partial<LocatedThread> = {}): LocatedThread {
  return {
    id,
    kind: "review",
    repositoryPath: "docs/guide.md",
    subject: "line",
    side: "RIGHT",
    line,
    startLine: null,
    originalLine: line,
    originalStartLine: null,
    commit: "head",
    originalCommit: "abcdef1234",
    resolved: false,
    outdated: false,
    canResolve: true,
    canUnresolve: true,
    canReply: true,
    comments: [comment(`Thread ${id}`)],
    diffHunk: undefined,
    marker: undefined,
    quote: undefined,
    path: "guide.md",
    lines: { first: line, last: line },
    detached: undefined,
    ...init,
  };
}

const anchor = (source: string): Anchor => ({ source, via: [] });

/** A host over a fixed set of threads, recording what the overlay asks of it. */
class FakeHost implements OverlayHost {
  blocks: { anchor: Anchor; threads: LocatedThread[] }[] = [];
  removed: { anchor: Anchor; threads: LocatedThread[] }[] = [];
  detached: LocatedThread[] = [];
  others: ThreadSummary[] = [];
  unsent = 0;
  target: CommentTarget = { kind: "thread" };
  calls: string[] = [];
  quotes: (string | undefined)[] = [];
  failReply: { message: string; code: string } | undefined;
  private listeners: (() => void)[] = [];

  async load(): Promise<OverlayData> {
    return {
      pullRequest: { number: 7, url: "https://github.com/acme/docs/pull/7" },
      threads: { blocks: this.blocks, removed: this.removed, detached: this.detached },
      pending: { id: undefined, threads: [], replies: [], conversation: [], count: this.unsent },
      viewer: "me",
    };
  }
  async commentTarget(): Promise<CommentTarget> {
    return this.target;
  }
  async comment(a: Anchor, body: string, quote?: string) {
    this.calls.push(`comment ${a.source} ${body}`);
    this.quotes.push(quote);
    const line = Number(a.source.split(":")[1]?.split("-")[0]);
    const made = thread(`N${++n}`, line, {
      comments: [comment(body, { pending: true, author: { login: "me", avatarUrl: undefined } })],
    });
    const block = this.blocks.find((b) => b.anchor.source === a.source);
    if (block) block.threads.push(made);
    else this.blocks.push({ anchor: a, threads: [made] });
    this.unsent++;
    return made;
  }
  async reply(threadId: string, body: string, when: "now" | "withReview") {
    this.calls.push(`reply ${threadId} ${when} ${body}`);
    if (this.failReply) throw this.failReply;
    const made = comment(body, { pending: when === "withReview" });
    if (when === "withReview") this.unsent++;
    for (const b of this.blocks) b.threads.find((t) => t.id === threadId)?.comments.push(made);
    return made;
  }
  async allThreads(): Promise<ThreadSummary[]> {
    const page: ThreadSummary["pages"] = [{ path: "guide.md", title: "Guide" }];
    return [
      ...this.blocks.flatMap((b) => b.threads),
      ...this.removed.flatMap((b) => b.threads),
      ...this.detached,
    ]
      .map((t) => ({ thread: t, pages: page }))
      .concat(this.others);
  }
  async resolve(threadId: string, resolved: boolean) {
    this.calls.push(`resolve ${threadId} ${resolved}`);
    for (const b of this.blocks) {
      const found = b.threads.find((t) => t.id === threadId);
      if (found) found.resolved = resolved;
    }
  }
  async submit(event: string, body?: string) {
    this.calls.push(`submit ${event} ${body ?? ""}`.trim());
    this.unsent = 0;
  }
  async discard() {
    this.calls.push("discard");
    this.unsent = 0;
  }
  openSource(a: Anchor) {
    this.calls.push(`openSource ${a.source}`);
  }
  openThread(threadId: string, page: string) {
    this.calls.push(`openThread ${threadId} ${page}`);
  }
  openLink(url: string) {
    this.calls.push(`openLink ${url}`);
  }
  notify(message: string) {
    this.calls.push(`notify ${message}`);
  }
  onDidChange(listener: () => void) {
    this.listeners.push(listener);
  }
  changed() {
    for (const listener of this.listeners) listener();
  }
}

let root: HTMLElement;
let host: FakeHost;
let overlay: Overlay | undefined;

beforeEach(() => {
  document.body.innerHTML = "<main><article></article></main>";
  root = document.querySelector("article") as HTMLElement;
  root.innerHTML = PAGE;
  host = new FakeHost();
});

afterEach(() => {
  overlay?.dispose();
  overlay = undefined;
});

/** Waits for the overlay's promises to settle. */
async function settle(): Promise<void> {
  for (let i = 0; i < 5; i++) await new Promise((resolve) => setTimeout(resolve, 0));
}

async function open(wide = true): Promise<Overlay> {
  overlay = createOverlay({ root, host, columnAt: wide ? 0 : 10_000 });
  await settle();
  return overlay;
}

/** The overlay's shadow roots, in document order: before the page, after it, the layer. */
function shadows(): ShadowRoot[] {
  return Array.from(document.querySelectorAll("[data-ascribe-overlay]"))
    .map((el) => el.shadowRoot)
    .filter((s): s is ShadowRoot => s !== null);
}

function all(selector: string): HTMLElement[] {
  return shadows().flatMap((s) => Array.from(s.querySelectorAll<HTMLElement>(selector)));
}

function one(selector: string, text?: string): HTMLElement {
  const found = all(selector).find((el) => text === undefined || el.textContent?.includes(text));
  if (!found) throw new Error(`no ${selector}${text ? ` with "${text}"` : ""}`);
  return found;
}

function card(id: string): HTMLElement {
  const found = all(".thread").find((el) => el.dataset["thread"] === id);
  if (!found) throw new Error(`no card for ${id}`);
  return found;
}

function type(field: HTMLElement, text: string): void {
  (field as HTMLTextAreaElement).value = text;
  field.dispatchEvent(new Event("input"));
}

describe("the overlay", () => {
  it("puts each block's threads in the column, in the page's order", async () => {
    host.blocks = [
      { anchor: anchor("guide.md:5-5"), threads: [thread("B", 5)] },
      { anchor: anchor("guide.md:3-3"), threads: [thread("A", 3), thread("A2", 3)] },
    ];
    await open();
    const slots = all(".column > .slot");
    expect(slots.map((s) => s.dataset["key"])).toEqual(["guide.md:3-3", "guide.md:5-5"]);
    expect(slots[0]?.querySelectorAll(".thread")).toHaveLength(2);
    expect(card("A").textContent).toContain("guide.md:3");
    expect(card("A").textContent).toContain("maya");
    expect(card("A").textContent).toContain("2 hours ago");
    expect(card("A").textContent).toContain("Thread A");
  });

  it("isolates its styles in shadow roots and changes nothing in the page but data-ascribe-ui elements", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    const text = root.textContent;
    await open(false);
    for (const shadow of shadows()) {
      const styled = shadow.adoptedStyleSheets?.length || shadow.querySelector("style");
      expect(styled).toBeTruthy();
    }
    // The marker is the only thing added inside the page, and it's marked as the overlay's.
    const added = Array.from(root.querySelectorAll("[data-ascribe-overlay]"));
    expect(added).toHaveLength(1);
    expect(added[0]?.hasAttribute("data-ascribe-ui")).toBe(true);
    for (const el of added) el.remove();
    expect(root.textContent).toBe(text);
    overlay?.dispose();
    overlay = undefined;
    expect(document.querySelectorAll("[data-ascribe-overlay]")).toHaveLength(0);
    expect(root.querySelector("[tabindex]")).toBeNull();
  });

  it("makes blocks focusable, and Enter on one opens the composer", async () => {
    await open();
    const block = root.querySelector<HTMLElement>('[data-ascribe-source="guide.md:3-3"]');
    expect(block?.tabIndex).toBe(0);
    block?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(one(".composer textarea").getAttribute("placeholder")).toBe("Comment on guide.md:3");
  });

  it("shows a Comment button on the block under the pointer", async () => {
    await open();
    const block = root.querySelector<HTMLElement>('[data-ascribe-source="guide.md:6-6"]');
    block?.dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    const tools = block?.querySelector<HTMLElement>("[data-ascribe-overlay]");
    expect(tools?.hidden).toBe(false);
    tools?.shadowRoot?.querySelector("button")?.click();
    await settle();
    expect(one(".composer").getAttribute("aria-label")).toBe("Comment on guide.md:6");
  });

  it("lists detached threads above the page, with the text they were on", async () => {
    host.detached = [
      thread("D", 20, {
        lines: undefined,
        detached: "line-gone",
        quote: "Schedules run in local time.",
      }),
    ];
    await open();
    const box = one(".detached");
    expect(box.textContent).toContain("1 comment has no block on this page any more:");
    const d = card("D");
    expect(d.textContent).toContain("Was on guide.md, line 20 at abcdef1, since removed");
    expect(d.querySelector(".badge.detached")?.textContent).toBe("Detached");
    expect(d.querySelector(".orig")?.textContent).toContain(
      "which is no longer on the page: Schedules run in local time.",
    );
    // A detached thread has no reply box or Resolve.
    expect(d.querySelector(".reply")).toBeNull();
    expect(d.textContent).not.toContain("Resolve");
  });

  it("labels an outdated thread beside its block, with a way to see the original text", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("O", 3, { outdated: true, quote: "Run the old installer." })],
      },
    ];
    await open();
    const o = card("O");
    expect(o.querySelector(".badge.outdated")?.textContent).toBe("Outdated");
    expect(o.querySelector(".orig")).toBeNull();
    one("button", "Original text").click();
    expect(card("O").querySelector(".orig")?.textContent).toContain(
      "The text when the comment was made: Run the old installer.",
    );
    one("button", "Hide original text").click();
    expect(card("O").querySelector(".orig")).toBeNull();
  });

  it("collapses resolved threads, and resolving acts at once", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("R", 3, { resolved: true }), thread("U", 3)],
      },
    ];
    await open();
    expect(card("R").classList.contains("collapsed")).toBe(true);
    expect(card("R").querySelector(".comment")).toBeNull();
    one("button", "Expand").click();
    expect(card("R").querySelector(".comment")).not.toBeNull();
    const resolve = card("U").querySelector<HTMLButtonElement>('[data-focus="resolve:U"]');
    expect(resolve?.title).toBe("Sent to GitHub right away, not held with your review");
    resolve?.click();
    await settle();
    expect(host.calls).toContain("resolve U true");
    expect(host.calls).toContain(
      "notify Resolved on GitHub. Resolving is sent right away, not held with your review.",
    );
    expect(card("U").querySelector(".badge.resolved")).not.toBeNull();
  });

  it("says where a thread's block is hidden, and shows it", async () => {
    host.blocks = [
      { anchor: anchor("guide.md:11-11"), threads: [thread("T", 11)] },
      { anchor: anchor("guide.md:15-15"), threads: [thread("S", 15)] },
    ];
    await open();
    expect(card("T").querySelector(".inside")?.textContent).toContain(
      "In the Self-hosted tab, which isn't showing.",
    );
    expect(card("S").querySelector(".inside")?.textContent).toContain(
      "Inside “Why?”, which is closed.",
    );
    const tabs = root.querySelectorAll<HTMLButtonElement>('[role="tab"]');
    const clicked = vi.fn();
    tabs[1]?.addEventListener("click", clicked);
    card("T").querySelector<HTMLButtonElement>(".inside button")?.click();
    expect(clicked).toHaveBeenCalled();
    card("S").querySelector<HTMLButtonElement>(".inside button")?.click();
    expect(root.querySelector("details")?.open).toBe(true);
  });

  it("puts threads on removed text on the removed block", async () => {
    host.removed = [
      { anchor: anchor("guide.md:18-18"), threads: [thread("X", 18, { side: "LEFT" })] },
    ];
    await open();
    const x = card("X");
    expect(x.querySelector(".badge.removed")?.textContent).toBe("On removed text");
    expect(x.textContent).not.toContain("Open source");
  });

  it("comments through the composer, and says where the comment goes", async () => {
    host.target = { kind: "summary", reason: "lines" };
    await open();
    const block = root.querySelector<HTMLElement>('[data-ascribe-source="guide.md:3-3"]');
    block?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(one(".composer").textContent).toContain(
      "GitHub only takes comments on the lines a pull request changes and the few around them.",
    );
    expect(one(".composer").textContent).toContain("Nothing is sent until you submit the review.");
    type(one(".composer textarea"), "Is this still true?");
    one(".composer button", "Add to review").click();
    await settle();
    expect(host.calls).toContain("comment guide.md:3-3 Is this still true?");
    expect(host.quotes).toEqual(["Run the installer."]);
    expect(all(".composer")).toHaveLength(0);
    const made = all(".thread").find((el) => el.textContent?.includes("Is this still true?"));
    expect(made?.querySelector(".badge.unsent")?.textContent).toBe("Unsent");
    expect(made?.textContent).toContain("you");
    expect(made?.textContent).toContain("not sent yet");
  });

  it("refuses a comment on lines that aren't pushed, and says why", async () => {
    host.target = { kind: "push-first", message: "Push them, then comment." };
    await open();
    const block = root.querySelector<HTMLElement>('[data-ascribe-source="guide.md:3-3"]');
    block?.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await settle();
    expect(one(".composer .refuse").textContent).toBe("Push them, then comment.");
    expect((one(".composer textarea") as HTMLTextAreaElement).disabled).toBe(true);
    expect((one(".composer button", "Add to review") as HTMLButtonElement).disabled).toBe(true);
    one(".composer button", "Cancel").click();
    expect(all(".composer")).toHaveLength(0);
  });

  it("replies now when nothing is unsent", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    await open();
    type(card("A").querySelector("textarea") as HTMLElement, "Done.");
    card("A").querySelector<HTMLButtonElement>('[data-focus="reply-now:A"]')?.click();
    await settle();
    expect(host.calls).toContain("reply A now Done.");
    expect(host.calls).toContain("notify Reply sent to GitHub.");
    expect(card("A").textContent).toContain("Done.");
  });

  it("adds a reply to the review, and turns Reply now off while there are unsent comments", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    await open();
    type(card("A").querySelector("textarea") as HTMLElement, "Later.");
    card("A").querySelector<HTMLButtonElement>('[data-focus="reply-add:A"]')?.click();
    await settle();
    expect(host.calls).toContain("reply A withReview Later.");
    const now = card("A").querySelector<HTMLButtonElement>('[data-focus="reply-now:A"]');
    expect(now?.getAttribute("aria-disabled")).toBe("true");
    expect(now?.title).toContain("GitHub adds replies to your review");
    const reason = card("A").querySelector(`#${now?.getAttribute("aria-describedby")}`);
    expect(reason?.textContent).toContain("Submit or discard it to reply at once.");
    type(card("A").querySelector("textarea") as HTMLElement, "Now?");
    now?.click();
    await settle();
    expect(host.calls.filter((c) => c.startsWith("reply"))).toHaveLength(1);
  });

  it("shows why a reply sent now was refused", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    host.failReply = { message: "GitHub adds replies to your unsent review.", code: "reply-held" };
    await open();
    type(card("A").querySelector("textarea") as HTMLElement, "Now.");
    card("A").querySelector<HTMLButtonElement>('[data-focus="reply-now:A"]')?.click();
    await settle();
    expect(card("A").querySelector(".reply .error")?.textContent).toBe(
      "GitHub adds replies to your unsent review.",
    );
    // The draft is kept.
    expect((card("A").querySelector("textarea") as HTMLTextAreaElement).value).toBe("Now.");
  });

  it("has no reply or resolve on a comment held in the review's summary", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("H", 3, { kind: "conversation", canReply: false, canResolve: false })],
      },
    ];
    await open();
    expect(card("H").querySelector(".badge.summary")?.textContent).toBe("In the review summary");
    expect(card("H").querySelector(".reply")).toBeNull();
    expect(card("H").textContent).not.toContain("Resolve");
  });

  it("shows the unsent bar and submits from its dialog", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("A", 3, { comments: [comment("Mine", { pending: true })] })],
      },
    ];
    host.unsent = 1;
    await open();
    expect(one(".unsent-bar").textContent).toContain("1 unsent comment");
    expect(one(".unsent-bar").textContent).toContain("Only you can see it until you submit.");
    one(".unsent-bar button", "Submit review…").click();
    await settle();
    const dialog = one(".dialog");
    expect(dialog.getAttribute("aria-label")).toBe("Submit review");
    expect(dialog.textContent).toContain("Submit review to #7");
    expect(dialog.textContent).toContain("guide.md:3: Mine");
    // Discard lives here only.
    expect(one(".unsent-bar").textContent).not.toContain("Discard");
    (dialog.querySelector('input[value="APPROVE"]') as HTMLInputElement).click();
    type(dialog.querySelector("textarea") as HTMLElement, "Looks good.");
    one(".dialog button", "Submit review").click();
    await settle();
    expect(host.calls).toContain("submit APPROVE Looks good.");
    expect(host.calls).toContain("notify Approved #7. Your comments are now on GitHub.");
    expect(all(".dialog")).toHaveLength(0);
    expect(all(".unsent-bar")).toHaveLength(0);
  });

  it("leaves the bar to a host that shows the count itself", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("A", 3, { comments: [comment("Mine", { pending: true })] })],
      },
    ];
    host.unsent = 1;
    overlay = createOverlay({ root, host, columnAt: 0, unsentBar: false });
    await settle();
    expect(all(".unsent-bar")).toHaveLength(0);
    expect(overlay.unsentCount()).toBe(1);
    overlay.submitReview();
    await settle();
    expect(one(".dialog").getAttribute("aria-label")).toBe("Submit review");
  });

  it("opens no submit dialog with nothing unsent", async () => {
    const shown = await open();
    expect(shown.unsentCount()).toBe(0);
    shown.submitReview();
    await settle();
    expect(all(".dialog")).toHaveLength(0);
  });

  it("confirms Discard with the count and outcome-named buttons", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [
          thread("A", 3, {
            comments: [comment("One", { pending: true }), comment("Two", { pending: true })],
          }),
        ],
      },
    ];
    host.unsent = 2;
    await open();
    one(".unsent-bar button").click();
    await settle();
    one(".dialog button", "Discard…").click();
    expect(one(".confirm").textContent).toContain("Discard 2 unsent comments?");
    one(".confirm button", "Keep review").click();
    expect(all(".confirm")).toHaveLength(0);
    expect(host.calls).not.toContain("discard");
    one(".dialog button", "Discard…").click();
    one(".confirm button", "Discard comments").click();
    await settle();
    expect(host.calls).toContain("discard");
    expect(all(".dialog")).toHaveLength(0);
  });

  it("lists every thread, filtered, and jumps to one on another page", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("A", 3), thread("R", 3, { resolved: true })],
      },
    ];
    host.detached = [thread("D", 20, { lines: undefined, detached: "line-gone" })];
    host.others = [
      {
        thread: thread("E", 4, { path: "other.md", lines: { first: 4, last: 4 } }),
        pages: [{ path: "other.md", title: "Other" }],
      },
    ];
    const o = await open();
    o.showAllComments();
    await settle();
    const filters = all(".segmented button").map((b) => b.textContent);
    expect(filters).toEqual(["Open 2", "Resolved 1", "Detached 1", "Unsent 0"]);
    const entries = () => all(".thread-list li").map((li) => li.textContent);
    expect(entries()).toHaveLength(2);
    expect(entries()[0]).toContain("guide.md:3 · Guide");
    one(".segmented button", "Detached").click();
    expect(entries()[0]).toContain("no block, was guide.md");
    one(".segmented button", "Unsent").click();
    expect(entries()).toEqual(["None."]);
    one(".segmented button", "Open").click();
    one(".thread-list button", "other.md:4").click();
    expect(host.calls).toContain("openThread E other.md");
    expect(all(".dialog")).toHaveLength(0);
  });

  it("goes to a thread on this page from the list", async () => {
    host.blocks = [{ anchor: anchor("guide.md:15-15"), threads: [thread("S", 15)] }];
    const o = await open();
    o.showAllComments();
    await settle();
    one(".thread-list button", "guide.md:15").click();
    // Its block was in a closed details: it's open now.
    expect(root.querySelector("details")?.open).toBe(true);
    expect((card("S").getRootNode() as ShadowRoot).activeElement).toBe(card("S"));
  });

  it("goes to a thread once the threads are read, when asked before", async () => {
    host.blocks = [{ anchor: anchor("guide.md:11-11"), threads: [thread("T", 11)] }];
    overlay = createOverlay({ root, host, columnAt: 0 });
    const clicked = vi.fn();
    root.querySelectorAll('[role="tab"]')[1]?.addEventListener("click", clicked);
    expect(overlay.goToThread("T")).toBe(true);
    await settle();
    expect(clicked).toHaveBeenCalled();
  });

  it("goes to a thread on the page it's re-reading, once it has read it", async () => {
    const o = await open();
    // The page changed under the overlay: the thread is on the new one.
    host.blocks = [{ anchor: anchor("guide.md:15-15"), threads: [thread("S", 15)] }];
    host.changed();
    expect(o.goToThread("S")).toBe(true);
    await settle();
    expect(root.querySelector("details")?.open).toBe(true);
  });

  it("opens a thread's own line, inside its block", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-5"),
        threads: [thread("A", 4, { lines: { first: 4, last: 4 } })],
      },
    ];
    await open();
    one("button", "Open source").click();
    expect(host.calls).toContain("openSource guide.md:4-4");
  });

  it("at narrow widths, shows count markers that open a panel", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3), thread("B", 3)] }];
    await open(false);
    expect(all(".column")).toHaveLength(0);
    const block = root.querySelector<HTMLElement>('[data-ascribe-source="guide.md:3-3"]');
    const marker = block
      ?.querySelector("[data-ascribe-overlay]")
      ?.shadowRoot?.querySelector("button");
    expect(marker?.textContent).toBe("2");
    expect(marker?.getAttribute("aria-label")).toBe("2 comments on this block. Show them.");
    expect(block?.style.paddingInlineEnd).toBe("44px");
    marker?.click();
    const sheet = one(".sheet");
    expect(sheet.getAttribute("role")).toBe("dialog");
    expect(sheet.querySelectorAll(".thread")).toHaveLength(2);
    one(".sheet button", "Comment").click();
    await settle();
    expect(one(".sheet .composer")).toBeTruthy();
    one(".sheet button", "Close").click();
    expect(all(".sheet")).toHaveLength(0);
  });

  it("opens a comment's links through the host", async () => {
    host.blocks = [
      {
        anchor: anchor("guide.md:3-3"),
        threads: [thread("A", 3, { comments: [comment("See [this](https://example.com/x).")] })],
      },
    ];
    await open();
    const link = card("A").querySelector("a");
    expect(link?.getAttribute("rel")).toBe("noopener noreferrer");
    link?.click();
    expect(host.calls).toContain("openLink https://example.com/x");
  });

  it("keeps a draft reply across a re-read", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    await open();
    type(card("A").querySelector("textarea") as HTMLElement, "Half a thought");
    host.changed();
    await settle();
    expect((card("A").querySelector("textarea") as HTMLTextAreaElement).value).toBe(
      "Half a thought",
    );
  });

  it("announces what happened", async () => {
    host.blocks = [{ anchor: anchor("guide.md:3-3"), threads: [thread("A", 3)] }];
    await open();
    type(card("A").querySelector("textarea") as HTMLElement, "Ok");
    card("A").querySelector<HTMLButtonElement>('[data-focus="reply-add:A"]')?.click();
    await settle();
    await new Promise((resolve) => setTimeout(resolve, 30));
    expect(one('[role="status"]').textContent).toBe("Reply added to your review.");
  });
});

describe("a block's text, for quoting", () => {
  it("reads as the page shows it: links as text, a line per item, code as written", () => {
    const el = document.createElement("div");
    el.innerHTML = `<p>Welcome to Quill. Start with
      <a href="Guides/My%20Setup.md">setting up Quill</a>.</p>
      <ul><li>Linux</li><li>macOS</li></ul>
      <pre><code>quill init
  --force</code></pre>
      <span data-ascribe-overlay><button>2</button></span>
      <svg aria-hidden="true"><text>icon</text></svg>`;
    expect(blockText(el)).toBe(
      "Welcome to Quill. Start with setting up Quill.\nLinux\nmacOS\nquill init\n  --force",
    );
  });

  it("numbers ordered items, keeps empty cells, quotes images' alt text, and skips hidden text", () => {
    const el = document.createElement("div");
    el.innerHTML = `<h2>Install<span class="sr-only">Section titled “Install”</span></h2>
      <ol start="3"><li>One</li><li>Two</li></ol>
      <table><tr><td></td><td>x</td></tr></table>
      <p hidden>Hidden</p><p style="display: none">None</p>
      <p><img alt="A diagram"></p>`;
    expect(blockText(el)).toBe("Install\n3. One\n4. Two\n| x\nA diagram");
  });
});
