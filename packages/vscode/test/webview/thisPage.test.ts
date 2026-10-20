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
  ThisPageReport,
  ToView,
  ViewItem,
} from "../../src/webview/viewProtocol.js";
import { viewHtml } from "../../src/webview/viewShell.js";

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

const EXAMPLE = "code:crates/ascribe-cli/tests/output/check.json, changed since it was tested";

/** About, Health, Connections, and Memos, as the model builds them for a page. */
function sections(title = "API keys"): ThisPageAnswer {
  return {
    message: null,
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
          item("health/0", "row", "Example", { description: EXAMPLE, warn: true, acts: true }),
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

  it("shows a message in place of the sections", async () => {
    const view = await open();
    await draw(view, 1, sections());
    const report = await draw(view, 2, {
      message: "Open a page to see what's known about it.",
      sections: [],
    });
    expect(report).toMatchObject({
      message: "Open a page to see what's known about it.",
      sections: [],
    });
    await expect(view.page.locator("p.message").textContent()).resolves.toBe(
      "Open a page to see what's known about it.",
    );
  });

  it("sets what people wrote as text, never as markup", async () => {
    const view = await open();
    const report = await draw(view, 1, sections('<img src=x onerror="__post(1)"> `code`'));
    await expect(view.page.locator("main img").count()).resolves.toBe(0);
    await expect(view.page.locator(".row dd").first().textContent()).resolves.toBe(
      '<img src=x onerror="__post(1)"> `code`',
    );
    expect(report.violations).toEqual([]);
  });
});
