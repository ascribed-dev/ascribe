// Project Settings' webview in a real browser: the shell under its content
// security policy, the bundled script and stylesheet, driven with the
// answers the extension sends. VS Code serves the files from its own origin;
// here a fixed origin stands in for it.
import { existsSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type {
  CheckView,
  FromSettings,
  SettingsAnswer,
  SettingsReport,
  SettingView,
} from "../../src/webview/settingsProtocol.js";
import type { FromView, ToView } from "../../src/webview/viewProtocol.js";
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

type Posted = FromView<SettingsReport> | FromSettings;

interface Tab {
  page: Page;
  posted: Posted[];
  /** Sends an answer and waits for the webview to say it drew it. */
  draw(seq: number, answer: SettingsAnswer): Promise<SettingsReport>;
  /** The next posted message of a type, after `from`. */
  next<T extends Posted["type"]>(type: T, from?: number): Promise<Extract<Posted, { type: T }>>;
}

async function open(): Promise<Tab> {
  const page = await context.newPage();
  page.setDefaultTimeout(10_000);
  const posted: Posted[] = [];
  await page.exposeFunction("__post", (message: Posted) => posted.push(message));
  await page.addInitScript(() => {
    let state: unknown = null;
    (window as unknown as Record<string, unknown>)["acquireVsCodeApi"] = () => ({
      postMessage: (message: unknown) =>
        (window as unknown as { __post(m: unknown): void }).__post(message),
      getState: () => state,
      setState: (value: unknown) => {
        state = value;
      },
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
          script: `${ORIGIN}/dist/settings.js`,
          style: `${ORIGIN}/dist/settings.css`,
          title: "quill: Project Settings",
        }),
      });
    } else if (pathname.startsWith("/dist/")) {
      const types: Record<string, string> = { ".css": "text/css", ".js": "text/javascript" };
      await route.fulfill({
        contentType: types[extname(pathname)] ?? "application/octet-stream",
        body: readFileSync(join(dist, pathname.slice("/dist/".length))),
      });
    } else {
      await route.fulfill({ status: 404, body: "" });
    }
  });
  await page.goto(`${ORIGIN}/`);
  const tab: Tab = {
    page,
    posted,
    next: async (type, from = 0) => {
      const deadline = Date.now() + 10_000;
      for (;;) {
        const found = posted.slice(from).find((m) => m.type === type);
        if (found) return found as never;
        if (Date.now() > deadline) throw new Error(`the webview never posted ${type}`);
        await new Promise((resolve) => setTimeout(resolve, 20));
      }
    },
    draw: async (seq, answer) => {
      const from = posted.length;
      await page.evaluate((m) => window.postMessage(m, "*"), {
        type: "answer",
        seq,
        answer,
      } satisfies ToView<SettingsAnswer>);
      const drawn = await tab.next("drawn", from);
      expect(drawn.seq).toBe(seq);
      return drawn.report;
    },
  };
  await tab.next("ready");
  return tab;
}

function setting(more: Partial<SettingView> & Pick<SettingView, "id">): SettingView {
  return {
    section: "[consumer]",
    name: "Trailing slash",
    key: "[consumer] trailing-slash",
    kind: "choice",
    choices: ["always", "never"],
    value: "always",
    defaultText: '"always"',
    where: "ascribe.toml:47",
    set: true,
    editable: true,
    resettable: true,
    description: [
      { text: "Routing. Whether page URLs end in ", code: false },
      { text: "/", code: true },
      { text: ".", code: false },
    ],
    problems: [],
    ...more,
  };
}

const ANSWER: SettingsAnswer = {
  message: null,
  settings: [
    setting({ id: "consumer.trailing-slash" }),
    setting({
      id: "consumer.base-path",
      name: "Base path",
      key: "[consumer] base-path",
      kind: "text",
      choices: [],
      value: "/",
      defaultText: '"/"',
      where: "ascribe.toml:48",
    }),
  ],
  checks: [],
};

function pageSize(more: Partial<CheckView> = {}): CheckView {
  return {
    slug: "page-size",
    code: "ASC149",
    named: false,
    level: setting({
      id: "checks.page-size.level",
      section: "Pages",
      name: "Page size",
      key: "[checks] page-size",
      kind: "level",
      choices: ["off", "advice", "warning", "error"],
      value: "advice",
      defaultText: "advice",
      where: "Not set in ascribe.toml",
      set: false,
      resettable: false,
      description: [{ text: "A page's Markdown is as long as the limit or longer.", code: false }],
    }),
    limit: setting({
      id: "checks.page-size.limit",
      section: "Pages",
      name: "Page size limit",
      key: "[checks.page-size] limit",
      kind: "number",
      choices: [],
      value: 50_000,
      defaultText: "50,000 characters",
      where: "Not set in ascribe.toml",
      set: false,
      resettable: false,
      description: [],
    }),
    ...more,
  };
}

describe("Project Settings' webview", () => {
  it("draws each setting with its name, key, description, default, place, and value, under its policy", async () => {
    const tab = await open();
    const report = await tab.draw(1, ANSWER);
    expect(report).toEqual({
      message: null,
      settings: [
        {
          id: "consumer.trailing-slash",
          value: "always",
          where: "ascribe.toml:47",
          problems: [],
          editable: true,
          resettable: true,
        },
        {
          id: "consumer.base-path",
          value: "/",
          where: "ascribe.toml:48",
          problems: [],
          editable: true,
          resettable: true,
        },
      ],
      violations: [],
    });
    const first = tab.page.locator("section.setting").first();
    expect(await first.getAttribute("class")).toBe("setting set");
    expect(await first.locator("h3").textContent()).toBe("Trailing slash");
    // Under its table's heading.
    expect(await tab.page.locator("h2.section").allTextContents()).toEqual(["[consumer]"]);
    expect(await first.locator(".key").textContent()).toBe("[consumer] trailing-slash");
    expect(await first.locator(".description").textContent()).toBe(
      "Routing. Whether page URLs end in /.",
    );
    expect(await first.locator(".description code").textContent()).toBe("/");
    expect(await first.locator(".default").textContent()).toBe('Default: "always"');
    // The control is labelled by the setting's name.
    expect(await tab.page.getByRole("combobox", { name: "Trailing slash" }).count()).toBe(1);
    expect(await tab.page.locator("a").count()).toBe(0);
    await tab.page.close();
  });

  it("posts a choice, a typed value on Enter, and a link to the line, naming the answer it drew", async () => {
    const tab = await open();
    await tab.draw(3, ANSWER);
    let from = tab.posted.length;
    await tab.page.getByRole("combobox", { name: "Trailing slash" }).selectOption("never");
    expect(await tab.next("set", from)).toEqual({
      type: "set",
      seq: 3,
      id: "consumer.trailing-slash",
      value: "never",
    });
    from = tab.posted.length;
    const base = tab.page.getByRole("textbox", { name: "Base path" });
    await base.fill("docs");
    await base.press("Enter");
    expect(await tab.next("set", from)).toEqual({
      type: "set",
      seq: 3,
      id: "consumer.base-path",
      value: "docs",
    });
    from = tab.posted.length;
    await tab.page.getByRole("button", { name: "ascribe.toml:48" }).click();
    expect(await tab.next("openLine", from)).toEqual({
      type: "openLine",
      seq: 3,
      id: "consumer.base-path",
    });
    from = tab.posted.length;
    await tab.page.getByRole("button", { name: "Your editor's settings" }).click();
    expect(await tab.next("editorSettings", from)).toEqual({ type: "editorSettings" });
    await tab.page.close();
  });

  it("shows the server's problems under a setting, and the next answer's value in its control", async () => {
    const tab = await open();
    await tab.draw(1, ANSWER);
    const [slash, base] = ANSWER.settings as [SettingView, SettingView];
    const report = await tab.draw(2, {
      message: null,
      settings: [
        { ...slash, value: "never" },
        {
          ...base,
          value: "docs",
          problems: [
            { message: 'base-path must start with "/", such as "/docs/"', severity: "error" },
          ],
        },
      ],
      checks: [],
    });
    expect(report.settings.map((s) => [s.value, s.problems])).toEqual([
      ["never", []],
      ["docs", ['base-path must start with "/", such as "/docs/"']],
    ]);
    expect(
      await tab.page.getByRole("list", { name: "Problems with Base path" }).textContent(),
    ).toBe('base-path must start with "/", such as "/docs/"');
    // An older answer, late, is ignored.
    await tab.page.evaluate((m) => window.postMessage(m, "*"), {
      type: "answer",
      seq: 1,
      answer: ANSWER,
    } satisfies ToView<SettingsAnswer>);
    await tab.draw(3, { message: "The server is starting.", settings: [], checks: [] });
    expect(await tab.page.locator(".message").textContent()).toBe("The server is starting.");
    expect(tab.posted.filter((m) => m.type === "drawn").map((m) => m.seq)).toEqual([1, 2, 3]);
    await tab.page.close();
  });

  it("checks a number before sending it, and doesn't change a setting it can't", async () => {
    const tab = await open();
    await tab.draw(1, {
      message: null,
      settings: [
        setting({
          id: "editor.size",
          name: "Size",
          key: "[editor] size",
          kind: "number",
          choices: [],
          value: 5,
          defaultText: "5",
        }),
        setting({ id: "consumer.slugger", editable: false, set: false, where: "Not set" }),
      ],
      checks: [],
    });
    const size = tab.page.getByRole("textbox", { name: "Size" });
    const from = tab.posted.length;
    await size.fill("five");
    await size.press("Enter");
    await size.fill("7");
    await size.press("Enter");
    expect(await tab.next("set", from)).toEqual({
      type: "set",
      seq: 1,
      id: "editor.size",
      value: 7,
    });
    expect(tab.posted.slice(from).filter((m) => m.type === "set")).toHaveLength(1);
    expect(await tab.page.getByRole("combobox", { name: "Trailing slash" }).isDisabled()).toBe(
      true,
    );
    await tab.page.close();
  });

  it("posts Reset to default for a setting the file sets, and offers none for one it leaves out", async () => {
    const tab = await open();
    await tab.draw(4, {
      message: null,
      settings: [
        setting({ id: "consumer.trailing-slash" }),
        setting({
          id: "consumer.headers",
          name: "Headers",
          key: "[consumer] headers",
          choices: ["netlify"],
          value: null,
          defaultText: "",
          where: "Not set in ascribe.toml",
          set: false,
          resettable: false,
        }),
      ],
      checks: [],
    });
    expect(await tab.page.getByRole("button", { name: /^Reset/ }).count()).toBe(1);
    // A choice with no value shows none, rather than its first.
    expect(await tab.page.getByRole("combobox", { name: "Headers" }).inputValue()).toBe("");
    const from = tab.posted.length;
    await tab.page.getByRole("button", { name: "Reset Trailing slash to default" }).click();
    expect(await tab.next("reset", from)).toEqual({
      type: "reset",
      seq: 4,
      id: "consumer.trailing-slash",
    });
    await tab.page.close();
  });

  it("draws each check with what it reports, its level, and its limit, and posts each", async () => {
    const tab = await open();
    const report = await tab.draw(5, {
      message: null,
      settings: [],
      checks: [
        {
          slug: "page-orphan",
          code: "ASC155",
          named: true,
          level: setting({
            id: "checks.page-orphan.level",
            section: "Across the project",
            name: "Page orphan",
            key: "[checks] page-orphan",
            kind: "level",
            choices: ["off", "advice", "warning", "error"],
            value: "off",
            defaultText: "advice",
            where: "ascribe.toml:30",
            description: [{ text: "Nothing links to a page.", code: false }],
          }),
          limit: null,
        },
        pageSize(),
      ],
    });
    expect(report.settings.map((s) => [s.id, s.value, s.resettable])).toEqual([
      ["checks.page-orphan.level", "off", true],
      ["checks.page-size.level", "advice", false],
      ["checks.page-size.limit", "50000", false],
    ]);
    expect(await tab.page.locator("h2.section").allTextContents()).toEqual(["Checks"]);
    const checks = tab.page.locator("section.check");
    expect(await checks.first().getAttribute("class")).toBe("setting check set");
    expect(await checks.nth(1).locator(".description").textContent()).toBe(
      "A page's Markdown is as long as the limit or longer.",
    );
    expect(await checks.nth(1).locator(".area").textContent()).toBe("Pages · ASC149");
    let from = tab.posted.length;
    await tab.page.getByRole("combobox", { name: "Page size" }).selectOption("warning");
    expect(await tab.next("set", from)).toEqual({
      type: "set",
      seq: 5,
      id: "checks.page-size.level",
      value: "warning",
    });
    from = tab.posted.length;
    const limit = tab.page.getByRole("textbox", { name: "Page size Limit" });
    await limit.fill("40000");
    await limit.press("Enter");
    expect(await tab.next("set", from)).toEqual({
      type: "set",
      seq: 5,
      id: "checks.page-size.limit",
      value: 40_000,
    });
    await tab.page.close();
  });

  it("sends a size as a number of bytes, or as the text typed", async () => {
    const tab = await open();
    const size = pageSize();
    await tab.draw(1, {
      message: null,
      settings: [],
      checks: [
        {
          ...size,
          limit: size.limit && {
            ...size.limit,
            name: "Image large limit",
            kind: "size",
            value: "500 KB",
          },
        },
      ],
    });
    const limit = tab.page.getByRole("textbox", { name: "Page size Limit" });
    let from = tab.posted.length;
    await limit.fill("1.5 MB");
    await limit.press("Enter");
    expect(await tab.next("set", from)).toMatchObject({ value: "1.5 MB" });
    from = tab.posted.length;
    await limit.fill("300000");
    await limit.press("Enter");
    expect(await tab.next("set", from)).toMatchObject({ value: 300_000 });
    await tab.page.close();
  });
});
