// The Project view's webview in a real browser, as This page's is in
// thisPage.test.ts: the view's shell under its content security policy, the
// bundled script and stylesheet, driven with the answers the extension sends.
import { existsSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { projectAnswer, type ProjectFacts } from "../../src/ui/projectHealth.js";
import type { EveryBuildResult } from "../../src/ui/checkEveryBuild.js";
import type {
  FromView,
  ProjectAnswer,
  ProjectReport,
  ToView,
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

type Posted = FromView<ProjectReport>;

interface View {
  page: Page;
  posted: Posted[];
  /** Sends an answer and waits for the webview to say it drew it. */
  draw(seq: number, answer: ProjectAnswer): Promise<ProjectReport>;
  /** The next posted message of a type, after `from`. */
  next<T extends Posted["type"]>(type: T, from?: number): Promise<Extract<Posted, { type: T }>>;
}

async function open(): Promise<View> {
  const page = await context.newPage();
  page.setDefaultTimeout(10_000);
  const posted: Posted[] = [];
  await page.exposeFunction("__post", (message: Posted) => posted.push(message));
  await page.addInitScript(() => {
    const store = window.sessionStorage;
    (window as unknown as Record<string, unknown>)["acquireVsCodeApi"] = () => ({
      postMessage: (message: unknown) =>
        (window as unknown as { __post(m: unknown): void }).__post(message),
      getState: () => JSON.parse(store.getItem("state") ?? "null") as unknown,
      setState: (value: unknown) => store.setItem("state", JSON.stringify(value)),
    });
  });
  await page.route(`${ORIGIN}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/") {
      await route.fulfill({
        contentType: "text/html",
        body: viewHtml({
          cspSource: ORIGIN,
          nonce: "bm9uY2U=",
          script: `${ORIGIN}/dist/project.js`,
          style: `${ORIGIN}/dist/views.css`,
          title: "Project",
        }),
      });
    } else if (pathname.startsWith("/dist/")) {
      const types: Record<string, string> = {
        ".css": "text/css",
        ".js": "text/javascript",
        ".ttf": "font/ttf",
      };
      await route.fulfill({
        contentType: types[extname(pathname)] ?? "application/octet-stream",
        body: readFileSync(join(dist, pathname.slice("/dist/".length))),
      });
    } else {
      await route.fulfill({ status: 404, body: "" });
    }
  });
  await page.goto(`${ORIGIN}/`);
  // VS Code sets the theme's colors on the document; two are enough to tell dimmed from not.
  await page.evaluate(() => {
    const style = document.documentElement.style;
    style.setProperty("--vscode-foreground", "rgb(30, 30, 30)");
    style.setProperty("--vscode-disabledForeground", "rgb(160, 160, 160)");
  });
  const view: View = {
    page,
    posted,
    draw: async (seq, answer) => {
      const from = posted.length;
      await page.evaluate((m) => window.postMessage(m, "*"), {
        type: "answer",
        seq,
        answer,
      } satisfies ToView<ProjectAnswer>);
      const drawn = await view.next("drawn", from);
      expect(drawn.seq).toBe(seq);
      return drawn.report;
    },
    next: async (type, from = 0) => {
      const deadline = Date.now() + 10_000;
      for (;;) {
        const found = posted.slice(from).find((m) => m.type === type);
        if (found) return found as never;
        if (Date.now() > deadline) throw new Error(`the webview never posted ${type}`);
        await new Promise((resolve) => setTimeout(resolve, 20));
      }
    },
  };
  await view.next("ready");
  return view;
}

const facts = (
  everyBuild: EveryBuildResult,
  name = "docs",
  more: Partial<ProjectFacts> = {},
): ProjectFacts => ({
  name,
  folder: "docs",
  state: "running",
  version: "0.4.0",
  everyBuild,
  inventory: { orphans: 0, unused: 0 },
  agentFiles: { kind: "made", at: 0, stale: [], error: null, current: true },
  now: 0,
  time: () => "14:02",
  day: () => "21 October",
  ...more,
});

const none = { count: 0, first: null };

/** The tasks every project's Actions has after Check Every Build. */
const TASKS = ["Build", "Sync Agent Files", "Observe Examples"];

const made = (current: boolean, errors = 2, warnings = 1, examples = 0): EveryBuildResult => ({
  kind: "made",
  at: 0,
  summary: { errors, warnings, advice: 0 },
  error: null,
  current,
  unsaved: 0,
  counted: {
    examples: {
      count: examples,
      first: null,
    },
    reviews: none,
    permalinks: none,
  },
});

describe("the Project view's webview", () => {
  it("draws the head, Health, and Actions, open, from its own files", async () => {
    const view = await open();
    const report = await view.draw(1, projectAnswer(facts({ kind: "none" })));
    expect(report.head).toEqual(["docs", "Folder: docs", "Server: running", "Version: 0.4.0"]);
    expect(report.sections).toEqual([
      { key: "health", label: "Health4 things to do", open: true },
      { key: "actions", label: "Actions", open: true },
    ]);
    expect(report.health[0]).toEqual({
      key: "problems",
      value: "—",
      note: "Needs every build checked",
      state: "unknown",
      dimmed: false,
    });
    expect(report.health.map(({ key }) => key)).toEqual([
      "problems",
      "examples",
      "reviews",
      "permalinks",
      "orphans",
      "unused",
      "agentFiles",
    ]);
    // Each count from the check has the check beside it, then the actions.
    expect(report.buttons).toEqual([...Array(5).fill("Check Every Build"), ...TASKS]);
    expect(report.violations).toEqual([]);
  });

  it("tells none from not checked: a dimmed 0, and a dash that says why", async () => {
    const view = await open();
    const some = await view.draw(1, projectAnswer(facts(made(true, 0))));
    expect(some.health[0]).toMatchObject({ value: "1 warning", dimmed: false });
    const none = await view.draw(2, projectAnswer(facts(made(true, 0, 0))));
    expect(none.health[0]).toMatchObject({ value: "0", state: "none", dimmed: true });
    expect(none.health.every((count) => count.value === "0" && count.dimmed)).toBe(true);
    expect(none.buttons).toEqual(["Check Every Build", ...TASKS]);
    const stale = await view.draw(3, projectAnswer(facts(made(false))));
    expect(stale.health[0]).toMatchObject({
      value: "—",
      note: "Out of date: the project changed after the check at 14:02",
      state: "outOfDate",
      dimmed: false,
    });
    expect(stale.buttons).toEqual([...Array(5).fill("Check Every Build"), ...TASKS]);
  });

  it("opens a current count's list, and runs the check, from the keyboard", async () => {
    const view = await open();
    const report = await view.draw(4, projectAnswer(facts(made(true))));
    expect(report.buttons).toEqual(["2 errors, 1 warning", "Check Every Build", ...TASKS]);
    await expect(
      view.page
        .getByRole("button", {
          name: "Problems in every build: 2 errors, 1 warning. Checked at 14:02",
        })
        .count(),
    ).resolves.toBe(1);
    // Tab: Health's header, the count, Actions' header, then Check Every Build.
    await view.page.keyboard.press("Tab");
    await view.page.keyboard.press("Tab");
    const from = view.posted.length;
    await view.page.keyboard.press("Enter");
    await expect(view.next("open", from)).resolves.toEqual({
      type: "open",
      seq: 4,
      key: "problems",
    });
    await view.page.keyboard.press("Tab");
    await view.page.keyboard.press("Tab");
    const later = view.posted.length;
    await view.page.keyboard.press("Space");
    await expect(view.next("open", later)).resolves.toEqual({
      type: "open",
      seq: 4,
      key: "checkEveryBuild",
    });
    await view.page.keyboard.press("Escape");
    await expect(view.next("focusEditor", from)).resolves.toEqual({ type: "focusEditor" });
  });

  it("tells none from out of date for every count: a dimmed 0, or a dash with a warning that says why", async () => {
    const view = await open();
    const current = await view.draw(
      1,
      projectAnswer(
        facts(made(true, 0, 0, 2), "docs", {
          inventory: { orphans: 3, unused: 0 },
          agentFiles: { kind: "made", at: 0, stale: ["AGENTS.md"], error: null, current: true },
        }),
      ),
    );
    const byKey = (report: ProjectReport) =>
      Object.fromEntries(report.health.map((count) => [count.key, count]));
    expect(byKey(current)).toMatchObject({
      examples: { value: "2 examples", state: "current", dimmed: false },
      reviews: { value: "0", state: "none", dimmed: true },
      orphans: { value: "3 pages", state: "current", dimmed: false },
      unused: { value: "0", state: "none", dimmed: true },
      agentFiles: { value: "1 file", state: "current", dimmed: false },
    });
    // The counts above zero open their lists.
    expect(current.buttons).toEqual([
      "2 examples",
      "3 pages",
      "1 file",
      "Check Every Build",
      ...TASKS,
    ]);
    const stale = await view.draw(
      2,
      projectAnswer(
        facts(made(false, 0, 0, 2), "docs", {
          agentFiles: { kind: "made", at: 0, stale: [], error: null, current: false },
        }),
      ),
    );
    for (const key of ["examples", "reviews", "permalinks", "agentFiles"]) {
      expect(byKey(stale)[key]).toMatchObject({ value: "—", state: "outOfDate", dimmed: false });
      await expect(
        view.page.locator(`.count[data-key=${key}] .note .codicon-warning`).count(),
      ).resolves.toBe(1);
    }
    await expect(view.page.locator(".count.none .codicon-warning").count()).resolves.toBe(0);
  });

  it("runs a task by its key, saying in its tooltip that it's a task", async () => {
    const view = await open();
    await view.draw(5, projectAnswer(facts({ kind: "none" })));
    await expect(
      view.page.getByRole("button", { name: "Sync Agent Files" }).getAttribute("title"),
    ).resolves.toBe("A task: runs ascribe agents sync in the terminal.");
    const from = view.posted.length;
    await view.page.getByRole("button", { name: "Build", exact: true }).click();
    await expect(view.next("open", from)).resolves.toEqual({ type: "open", seq: 5, key: "build" });
  });

  it("keeps a section the reader closed closed when it draws again", async () => {
    const view = await open();
    await view.draw(1, projectAnswer(facts({ kind: "none" })));
    await view.page.locator("details[data-key=actions] > summary").click();
    const again = await view.draw(2, projectAnswer(facts({ kind: "running" })));
    expect(again.sections.map(({ open }) => open)).toEqual([true, false]);
    expect(again.health[0]).toMatchObject({ note: "Checking every build…", state: "running" });
  });

  it("shows a message in place of a project, and sets the project's name as text", async () => {
    const view = await open();
    const message = await view.draw(1, projectAnswer(undefined));
    expect(message.message).toBe("Open a page of a project to see the project.");
    await expect(view.page.locator("p.message").count()).resolves.toBe(1);
    const named = await view.draw(
      2,
      projectAnswer(facts({ kind: "none" }, '<img src=x onerror="__post(1)">')),
    );
    await expect(view.page.locator("main img").count()).resolves.toBe(0);
    expect(named.head[0]).toBe('<img src=x onerror="__post(1)">');
    expect(named.violations).toEqual([]);
  });
});
