// This page's webview in a real browser: the view's shell under its content
// security policy, the bundled script and stylesheet with the codicon font,
// driven with the answers the extension sends. VS Code serves the files from
// its own origin; here a fixed origin stands in for it.
import { existsSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type {
  FromView,
  ThisPageAnswer,
  ThisPageProbe,
  ThisPageReport,
  ToView,
  ViewItem,
} from "../../src/webview/viewProtocol.js";
import { viewHtml } from "../../src/webview/viewShell.js";
import { NOTICE_ACTIONS, thisPageNotice } from "../../src/ui/sidebar.js";

const dist = fileURLToPath(new URL("../../dist/webview/", import.meta.url));
const ORIGIN = "https://view.test";

let browser: Browser;
let context: BrowserContext;

beforeAll(async () => {
  const configured = process.env["ASCRIBE_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  const executablePath = configured ?? (existsSync(bundled) ? bundled : undefined);
  browser = await chromium.launch(executablePath === undefined ? {} : { executablePath });
  context = await browser.newContext();
});

afterAll(async () => {
  await context?.close();
  await browser?.close();
});

type Posted = FromView<ThisPageReport>;

interface View {
  page: Page;
  posted: Posted[];
  /** Fonts the page loaded, by URL. */
  fonts: string[];
  send(seq: number, answer: ThisPageAnswer): Promise<void>;
  /** The next posted message of a type, after `from`. */
  next<T extends Posted["type"]>(type: T, from?: number): Promise<Extract<Posted, { type: T }>>;
  /** Loads the document again, as VS Code does when the view is shown again. */
  reload(): Promise<void>;
}

/** Opens the view, with `state` as what VS Code kept for it. */
async function open(state?: unknown): Promise<View> {
  const page = await context.newPage();
  page.setDefaultTimeout(10_000);
  const posted: Posted[] = [];
  const fonts: string[] = [];
  await page.exposeFunction("__post", (message: Posted) => posted.push(message));
  // The API VS Code gives a webview script, with its state kept across reloads.
  await page.addInitScript((kept) => {
    const store = window.sessionStorage;
    if (kept !== undefined && store.getItem("state") === null) {
      store.setItem("state", JSON.stringify(kept));
    }
    (window as unknown as Record<string, unknown>)["acquireVsCodeApi"] = () => ({
      postMessage: (message: unknown) =>
        (window as unknown as { __post(m: unknown): void }).__post(message),
      getState: () => JSON.parse(store.getItem("state") ?? "null") as unknown,
      setState: (value: unknown) => store.setItem("state", JSON.stringify(value)),
    });
  }, state);
  await page.route(`${ORIGIN}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/") {
      await route.fulfill({
        contentType: "text/html",
        body: viewHtml({
          cspSource: ORIGIN,
          nonce: "bm9uY2U=",
          script: `${ORIGIN}/dist/thisPage.js`,
          style: `${ORIGIN}/dist/views.css`,
          title: "This page",
        }),
      });
    } else if (pathname.startsWith("/dist/")) {
      const types: Record<string, string> = {
        ".css": "text/css",
        ".js": "text/javascript",
        ".ttf": "font/ttf",
      };
      if (extname(pathname) === ".ttf") fonts.push(pathname);
      await route.fulfill({
        contentType: types[extname(pathname)] ?? "application/octet-stream",
        body: readFileSync(join(dist, pathname.slice("/dist/".length))),
      });
    } else {
      await route.fulfill({ status: 404, body: "" });
    }
  });
  await page.goto(`${ORIGIN}/`);
  const view: View = {
    page,
    posted,
    fonts,
    send: (seq, answer) =>
      page.evaluate((m) => window.postMessage(m, "*"), {
        type: "answer",
        seq,
        answer,
      } satisfies ToView<ThisPageAnswer>),
    next: async (type, from = 0) => {
      const deadline = Date.now() + 10_000;
      for (;;) {
        const found = posted.slice(from).find((m) => m.type === type);
        if (found) return found as never;
        if (Date.now() > deadline) throw new Error(`the webview never posted ${type}`);
        await new Promise((resolve) => setTimeout(resolve, 20));
      }
    },
    reload: async () => {
      const from = posted.length;
      await page.reload();
      await view.next("ready", from);
    },
  };
  await view.next("ready");
  return view;
}

/** Sends an answer and waits for the webview to say it drew it. */
async function draw(view: View, seq: number, answer: ThisPageAnswer): Promise<ThisPageReport> {
  const from = view.posted.length;
  await view.send(seq, answer);
  const drawn = await view.next("drawn", from);
  expect(drawn.seq).toBe(seq);
  return drawn.report;
}

function item(key: string, kind: ViewItem["kind"], label: string, more: Partial<ViewItem> = {}) {
  return {
    key,
    kind,
    label,
    state: "",
    description: "",
    tooltip: "",
    icon: "blank",
    dimmed: false,
    warn: false,
    acts: false,
    open: false,
    children: [],
    ...more,
  } satisfies ViewItem;
}

const EXAMPLE = "code:crates/ascribe-cli/tests/output/check.json";
const STALE = "changed since it was tested";
const PLACE =
  "Generate a new key, put it in `quill.yaml`, and restart the agent. To install the agent first, see [](install-agent.md#install-agent).";
const MEMO = "The agent goes first because its installer writes the config file that";
const MEMO_FILE = "memos/agent-goes-first-because-its-installer-writes.md";

/** About, Health, Connections, and Memos, as the model builds them for a page, under its head. */
function sections(title = "API keys"): ThisPageAnswer {
  return {
    message: null,
    actions: [],
    head: { title: [{ type: "text", value: title }], type: "page", path: "keys.md" },
    sections: [
      item("about", "section", "About", {
        icon: "info",
        open: true,
        children: [
          item("about/0", "row", "Title", { description: title, tooltip: "keys.md" }),
          item("about/1", "row", "Description", {
            description: "52 characters",
            acts: true,
          }),
        ],
      }),
      item("health", "section", "Health", {
        icon: "pulse",
        description: "1 thing to do",
        children: [
          item("health/0", "row", "Example", {
            state: STALE,
            description: EXAMPLE,
            warn: true,
            acts: true,
          }),
        ],
      }),
      item("connections", "section", "Connections", {
        icon: "references",
        description: "to the page",
        children: [
          item("connections/0", "group", "Linked from", {
            description: "1",
            children: [
              item("connections/0/0", "group", "a.md", {
                description: "a.md",
                children: [
                  item("connections/0/0/0", "item", "See [](keys.md).", {
                    description: "line 4",
                    acts: true,
                  }),
                ],
              }),
            ],
          }),
          item("connections/1", "group", "Included by", { description: "0", dimmed: true }),
        ],
      }),
      item("memos", "section", "Memos", { icon: "note", description: "none", dimmed: true }),
    ],
  };
}

describe("This page's webview", () => {
  it("draws every section with only About open, its rows wrapped, from its own files", async () => {
    const view = await open();
    const report = await draw(view, 1, sections());
    expect(report.sections.map(({ label, open }) => [label, open])).toEqual([
      ["About", true],
      ["Health", false],
      ["Connections", false],
      ["Memos", false],
    ]);
    expect(report.sections[0]?.items).toEqual(["Title (API keys)", "Description (52 characters)"]);
    expect(report.wraps).toBe(true);
    expect(report.violations).toEqual([]);
    // The codicons are the extension's own font, read from its files.
    await view.page.evaluate(() => document.fonts.ready);
    expect(view.fonts).toEqual(["/dist/codicon.ttf"]);
    // A closed section's header still says whether to open it.
    await expect(view.page.locator("summary").nth(1).textContent()).resolves.toBe(
      "Health1 thing to do",
    );
    // Each header's twistie is the codicon chevron for open or closed.
    const twisties = await view.page
      .locator("summary > .twistie")
      .evaluateAll((spans) =>
        spans
          .filter((span) => getComputedStyle(span).display !== "none")
          .map((span) => [...span.classList].find((name) => name.startsWith("codicon-chevron"))),
      );
    expect(twisties).toEqual([
      "codicon-chevron-down",
      "codicon-chevron-right",
      "codicon-chevron-right",
      "codicon-chevron-right",
    ]);
  });

  it("wraps a long row at a narrow width instead of cutting it off", async () => {
    const view = await open();
    await view.page.setViewportSize({ width: 220, height: 600 });
    await draw(view, 1, sections());
    await view.page.locator("details[data-key=health] > summary").click();
    const row = view.page.locator(".row", { hasText: "Example" });
    const box = await row.boundingBox();
    const width = await view.page.evaluate(() => document.documentElement.clientWidth);
    expect(box?.width).toBeLessThanOrEqual(width);
    await expect(view.page.locator(".row dd", { hasText: EXAMPLE }).isVisible()).resolves.toBe(
      true,
    );
    // The row leads with its state.
    await expect(view.page.locator(".row dd > button > .state").textContent()).resolves.toBe(STALE);
    const [scroll, client] = await view.page.evaluate(() => [
      document.documentElement.scrollWidth,
      document.documentElement.clientWidth,
    ]);
    expect(scroll).toBe(client);
  });

  it("keeps a line's description whole beside a long label that wraps", async () => {
    const view = await open();
    await view.page.setViewportSize({ width: 320, height: 600 });
    await draw(view, 1, {
      ...sections(),
      sections: [
        item("connections", "section", "Connections", {
          icon: "references",
          open: true,
          children: [item("connections/0", "item", PLACE, { description: "line 21", acts: true })],
        }),
        item("memos", "section", "Memos", {
          icon: "note",
          open: true,
          children: [item("memos/0", "item", MEMO, { description: MEMO_FILE, acts: true })],
        }),
      ],
    });
    // A part of a line, and where it breaks: the two characters on either side of each break.
    const lines = (key: string, part: string) =>
      view.page.locator(`[data-key="${key}"] > .${part}`).evaluate((span) => {
        const text = span.textContent ?? "";
        const range = document.createRange();
        const top = (i: number) => {
          range.setStart(span.firstChild as Node, i);
          range.setEnd(span.firstChild as Node, i + 1);
          return Math.round(range.getBoundingClientRect().top);
        };
        const breaks: string[] = [];
        for (let i = 1; i < text.length; i++) {
          if (top(i) !== top(i - 1)) breaks.push(text.slice(i - 1, i + 1));
        }
        return breaks;
      });
    // A short description stays on one line beside the label's lines.
    await expect(lines("connections/0", "label")).resolves.not.toEqual([]);
    await expect(lines("connections/0", "description")).resolves.toEqual([]);
    // A long one wraps at its own breaks, after a hyphen or a slash, never inside a word.
    const breaks = await lines("memos/0", "description");
    expect(breaks).not.toEqual([]);
    for (const pair of breaks) expect(pair).toMatch(/^[-/]/);
    const [scroll, client] = await view.page.evaluate(() => [
      document.documentElement.scrollWidth,
      document.documentElement.clientWidth,
    ]);
    expect(scroll).toBe(client);
  });

  it("is a details element for each section, a button for each row that opens, and a list of About's rows", async () => {
    const view = await open();
    await draw(view, 1, sections());
    await expect(view.page.locator("details.section > summary").count()).resolves.toBe(4);
    await expect(view.page.locator("details[data-key=about] dl > .row > dt").count()).resolves.toBe(
      2,
    );
    await expect(
      view.page.getByRole("button", { name: "Description: 52 characters" }).count(),
    ).resolves.toBe(1);
    // No ARIA tree roles: they'd promise a tree's whole keyboard.
    await expect(view.page.locator("[role=tree], [role=treeitem]").count()).resolves.toBe(0);
    // Hover text is on the row's label.
    await expect(
      view.page.locator(".row .label", { hasText: "Title" }).getAttribute("title"),
    ).resolves.toBe("keys.md");
  });

  it("chooses a row with the keyboard, and gives the focus back with Escape", async () => {
    const view = await open();
    await draw(view, 3, sections());
    // Tab goes through the headers and the rows' buttons, in order.
    await view.page.keyboard.press("Tab");
    await expect(
      view.page.evaluate(() => document.activeElement?.closest("summary") !== null),
    ).resolves.toBe(true);
    await view.page.keyboard.press("Tab");
    await expect(view.page.evaluate(() => document.activeElement?.textContent)).resolves.toBe(
      "52 characters",
    );
    const from = view.posted.length;
    await view.page.keyboard.press("Enter");
    await expect(view.next("open", from)).resolves.toEqual({
      type: "open",
      seq: 3,
      key: "about/1",
    });
    // Space on a section's header opens it.
    await view.page.keyboard.press("Tab");
    await view.page.keyboard.press("Space");
    await expect(
      view.page.locator("details[data-key=health]").evaluate((d) => (d as HTMLDetailsElement).open),
    ).resolves.toBe(true);
    await view.page.keyboard.press("Escape");
    await expect(view.next("focusEditor", from)).resolves.toEqual({ type: "focusEditor" });
  });

  it("keeps its scroll and the keyboard's focus when a new answer comes", async () => {
    const view = await open({ sections: { health: true, connections: true } });
    await view.page.setViewportSize({ width: 300, height: 90 });
    await draw(view, 1, sections());
    const button = view.page.locator("button[data-key='connections/0/0/0']");
    await button.focus();
    await view.page.evaluate(() => {
      document.documentElement.scrollTop = 60;
      (window as unknown as Record<string, unknown>)["kept"] = document.activeElement;
    });
    // The cursor moved: the same rows, one of them saying something else.
    const moved = sections("Rotate keys");
    const report = await draw(view, 2, moved);
    expect(report.focused).toBe("connections/0/0/0");
    expect(report.scroll).toBe(60);
    await expect(view.page.locator(".row dd").first().textContent()).resolves.toBe("Rotate keys");
    // The row is the element it was, not one drawn again.
    await expect(
      view.page.evaluate(
        () => (window as unknown as Record<string, unknown>)["kept"] === document.activeElement,
      ),
    ).resolves.toBe(true);
  });

  it("gives a row the focus back without scrolling when the window gets it back", async () => {
    const view = await open({ sections: { health: true, connections: true } });
    await view.page.setViewportSize({ width: 300, height: 90 });
    await draw(view, 1, sections());
    await view.page.locator("button[data-key='health/0']").focus();
    // The row is scrolled out of view, and the window loses the focus.
    const back = await view.page.evaluate(() => {
      document.documentElement.scrollTop = 200;
      const scrolled = document.documentElement.scrollTop;
      (document.activeElement as HTMLElement).blur();
      window.dispatchEvent(new FocusEvent("focus"));
      return {
        scrolled,
        scroll: document.documentElement.scrollTop,
        focused: document.activeElement?.getAttribute("data-key"),
      };
    });
    expect(back.scrolled).toBeGreaterThan(0);
    expect(back).toEqual({ scrolled: back.scrolled, scroll: back.scrolled, focused: "health/0" });
  });

  it("gives the focus to the section's header when what had it is gone", async () => {
    const view = await open({ sections: { connections: true } });
    await draw(view, 1, sections());
    await view.page.locator("button[data-key='connections/0/0/0']").focus();
    const fewer = sections();
    const connections = fewer.sections[2];
    if (connections) connections.children = connections.children.slice(1);
    const report = await draw(view, 2, fewer);
    expect(report.focused).toBe("connections");
    await expect(view.page.evaluate(() => document.activeElement?.tagName)).resolves.toBe(
      "SUMMARY",
    );
  });

  it("moves the focus between the headers and rows with Up and Down", async () => {
    const view = await open();
    await draw(view, 1, sections());
    const focused = () =>
      view.page.evaluate(() => {
        const at = document.activeElement;
        return (at?.tagName === "SUMMARY" ? at.parentElement : at)?.getAttribute("data-key");
      });
    await view.page.locator("details[data-key=about] > summary").focus();
    await view.page.keyboard.press("ArrowDown");
    // About's Title has nothing to open, so its Description is next.
    await expect(focused()).resolves.toBe("about/1");
    await view.page.keyboard.press("ArrowDown");
    await expect(focused()).resolves.toBe("health");
    // A closed section's rows are skipped.
    await view.page.keyboard.press("ArrowDown");
    await expect(focused()).resolves.toBe("connections");
    await view.page.keyboard.press("ArrowUp");
    await view.page.keyboard.press("ArrowUp");
    await expect(focused()).resolves.toBe("about/1");
    // Down at the last header stays there, and the view doesn't scroll.
    await view.page.locator("details[data-key=memos] > summary").focus();
    await view.page.keyboard.press("ArrowDown");
    await expect(focused()).resolves.toBe("memos");
  });

  it("answers a probe with what has the focus and how far it's scrolled", async () => {
    const view = await open();
    await view.page.setViewportSize({ width: 300, height: 90 });
    await draw(view, 4, sections());
    const from = view.posted.length;
    await view.page.evaluate((m) => window.postMessage(m, "*"), {
      type: "probe",
      probe: 1,
      open: true,
      focus: "health/0",
      scroll: 40,
      press: true,
    } satisfies ThisPageProbe);
    await expect(view.next("open", from)).resolves.toEqual({
      type: "open",
      seq: 4,
      key: "health/0",
    });
    const drawn = await view.next("drawn", from);
    expect(drawn.seq).toBe(4);
    expect(drawn.report).toMatchObject({ probed: 1, focused: "health/0", scroll: 40 });
    expect(drawn.report.sections.every(({ open }) => open)).toBe(true);
  });

  it("ignores an answer older than the one it drew", async () => {
    const view = await open();
    await draw(view, 2, sections("Newer"));
    const from = view.posted.length;
    await view.send(1, sections("Older"));
    await draw(view, 3, sections("Newest"));
    expect(view.posted.slice(from).filter((m) => m.type === "drawn")).toHaveLength(1);
    await expect(view.page.locator(".row dd").first().textContent()).resolves.toBe("Newest");
  });

  it("keeps which sections the reader opened, for every page, and draws again when made again", async () => {
    const view = await open();
    await draw(view, 1, sections());
    await view.page.locator("details[data-key=health] > summary").click();
    await view.page.locator("details[data-key=about] > summary").click();
    await expect
      .poll(() => view.page.evaluate(() => sessionStorage.getItem("state")))
      .toBe(JSON.stringify({ sections: { health: true, about: false } }));
    // Another page, then the view hidden and shown: its document is made
    // again, says it's ready, and the extension sends the newest answer.
    const other = await draw(view, 2, sections("Quickstart"));
    expect(other.sections.map(({ open }) => open)).toEqual([false, true, false, false]);
    await view.reload();
    const again = await draw(view, 2, sections("Quickstart"));
    expect(again.sections.map(({ open }) => open)).toEqual([false, true, false, false]);
  });

  it("shows a message in place of the page", async () => {
    const view = await open();
    await draw(view, 1, sections());
    const report = await draw(view, 2, thisPageNotice({ kind: "noPage" }));
    expect(report).toMatchObject({
      message: "Open a page to see what's known about it.",
      actions: [],
      head: [],
      sections: [],
    });
    await expect(view.page.locator("p.message").textContent()).resolves.toBe(
      "Open a page to see what's known about it.",
    );
    await expect(view.page.locator(".head, details").count()).resolves.toBe(0);
  });

  it("says the server is starting, in place of the page", async () => {
    const view = await open();
    const report = await draw(view, 1, thisPageNotice({ kind: "starting" }));
    expect(report).toMatchObject({
      message: "Waiting for the project's language server.",
      actions: [],
      sections: [],
    });
    expect(report.violations).toEqual([]);
  });

  it("says the server failed, with Show Output and Restart, which the extension runs", async () => {
    const view = await open();
    const report = await draw(view, 4, thisPageNotice({ kind: "failed", project: "quill" }));
    expect(report).toMatchObject({
      message: "quill: the language server failed to start.",
      actions: ["Show Output", "Restart"],
      sections: [],
    });
    const from = view.posted.length;
    await view.page.getByRole("button", { name: "Restart" }).click();
    await expect(view.next("open", from)).resolves.toEqual({
      type: "open",
      seq: 4,
      key: NOTICE_ACTIONS.restart,
    });
    // The buttons are on the keyboard's way too.
    await view.page.keyboard.press("ArrowUp");
    await expect(view.page.evaluate(() => document.activeElement?.textContent)).resolves.toBe(
      "Show Output",
    );
  });

  it("says the folder is in Restricted Mode, with Manage Workspace Trust", async () => {
    const view = await open();
    const report = await draw(view, 1, thisPageNotice({ kind: "restricted" }));
    expect(report).toMatchObject({
      message: "This folder is in Restricted Mode, so Ascribe doesn't run or check its projects.",
      actions: ["Manage Workspace Trust"],
      sections: [],
    });
    const from = view.posted.length;
    await view.page.getByRole("button", { name: "Manage Workspace Trust" }).click();
    await expect(view.next("open", from)).resolves.toMatchObject({
      key: NOTICE_ACTIONS.manageTrust,
    });
  });

  it("heads the page with its title, its code spans drawn as code, then its type and path", async () => {
    const view = await open();
    const answer = sections();
    answer.head = {
      title: [
        { type: "code", value: "ascribe.toml" },
        { type: "text", value: " reference" },
      ],
      type: "reference",
      path: "reference/config.md",
    };
    const report = await draw(view, 1, answer);
    expect(report.head).toEqual(["ascribe.toml reference", "reference · reference/config.md"]);
    expect(report.code).toEqual(["ascribe.toml"]);
    await expect(view.page.locator(".head .page-title > code").count()).resolves.toBe(1);
    // The head comes before the sections, and isn't one.
    await expect(
      view.page.locator("main > *").evaluateAll((all) => all.map((each) => each.tagName)),
    ).resolves.toEqual(["HEADER", "DETAILS", "DETAILS", "DETAILS", "DETAILS"]);
  });

  it("names a fragment in its head", async () => {
    const view = await open();
    const answer = sections("_setup.md");
    answer.head = {
      title: [{ type: "text", value: "_setup.md" }],
      type: "fragment",
      path: "_setup.md",
    };
    const report = await draw(view, 1, answer);
    expect(report.head).toEqual(["_setup.md", "fragment · _setup.md"]);
    expect(report.code).toEqual([]);
  });

  it("keeps an open section's header at the top while its rows scroll", async () => {
    const view = await open({ sections: { health: true, connections: true } });
    await view.page.setViewportSize({ width: 300, height: 90 });
    await draw(view, 1, sections());
    const sticky = await view.page
      .locator("details.section > summary")
      .evaluateAll((all) => all.map((summary) => getComputedStyle(summary).position));
    // Memos is closed: there's nothing under it to scroll.
    expect(sticky).toEqual(["sticky", "sticky", "sticky", "static"]);
    // Scrolled into Connections, its header is at the top of the view.
    await view.page.evaluate(() => {
      document.querySelector("[data-key='connections/0/0/0']")?.scrollIntoView();
    });
    const top = await view.page
      .locator("details[data-key=connections] > summary")
      .evaluate((summary) => Math.round(summary.getBoundingClientRect().top));
    expect(top).toBe(0);
  });

  it("sets what people wrote as text, never as markup", async () => {
    const view = await open();
    const written = sections('<img src=x onerror="__post(1)"> `code`');
    written.head = {
      title: [
        { type: "text", value: '<img src=x onerror="__post(1)">' },
        { type: "code", value: "<b>bold</b>" },
      ],
      type: "page",
      path: "keys.md",
    };
    const report = await draw(view, 1, written);
    expect(report.code).toEqual(["<b>bold</b>"]);
    await expect(view.page.locator("main b").count()).resolves.toBe(0);
    await expect(view.page.locator("main img").count()).resolves.toBe(0);
    await expect(view.page.locator(".row dd").first().textContent()).resolves.toBe(
      '<img src=x onerror="__post(1)"> `code`',
    );
    expect(report.violations).toEqual([]);
  });
});
