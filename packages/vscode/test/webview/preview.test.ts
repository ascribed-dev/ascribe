// The preview's webview in a real browser: the shell HTML under its content
// security policy, the bundled element library, and the preview script,
// driven with the messages the extension sends. VS Code serves the files
// from its own origin; here a fixed origin stands in for it.
import { readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type BrowserContext, type Page } from "playwright-core";
import { existsSync } from "node:fs";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { shellHtml } from "../../src/preview/html.js";
import type { FromWebview, ReviewView, ToWebview } from "../../src/preview/protocol.js";

const dist = fileURLToPath(new URL("../../dist/webview/", import.meta.url));
const ORIGIN = "https://preview.test";

/** A 1×1 transparent PNG. */
const PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGNgYGD4DwABBAEAX+XDSwAAAABJRU5ErkJggg==",
  "base64",
);

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

interface Preview {
  page: Page;
  /** What the webview has posted to the extension. */
  posted: FromWebview[];
  send(message: ToWebview): Promise<void>;
  /** The next posted message of a type, after `from`. */
  next<T extends FromWebview["type"]>(
    type: T,
    from?: number,
  ): Promise<Extract<FromWebview, { type: T }>>;
}

/** The dev server the site preview frames, in the tests that show it. */
const SITE = "https://site.test";

async function open(options: { frameOrigin?: string } = {}): Promise<Preview> {
  const page = await context.newPage();
  page.setDefaultTimeout(10_000);
  const posted: FromWebview[] = [];
  await page.exposeFunction("__post", (message: FromWebview) => posted.push(message));
  // The API VS Code gives a webview script.
  await page.addInitScript(() => {
    (window as unknown as Record<string, unknown>)["acquireVsCodeApi"] = () => ({
      postMessage: (message: unknown) =>
        (window as unknown as { __post(m: unknown): void }).__post(message),
      getState: () => undefined,
      setState: () => undefined,
    });
  });
  await page.route(`${ORIGIN}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/") {
      await route.fulfill({
        contentType: "text/html",
        body: shellHtml({
          cspSource: ORIGIN,
          elementsScript: `${ORIGIN}/dist/elements.js`,
          elementsStyle: `${ORIGIN}/dist/elements.css`,
          marksStyle: `${ORIGIN}/dist/marks.css`,
          previewScript: `${ORIGIN}/dist/preview.js`,
          previewStyle: `${ORIGIN}/dist/preview.css`,
          ...options,
        }),
      });
    } else if (pathname.startsWith("/dist/")) {
      const type = extname(pathname) === ".css" ? "text/css" : "text/javascript";
      await route.fulfill({
        contentType: type,
        body: readFileSync(join(dist, pathname.slice("/dist/".length))),
      });
    } else if (pathname.endsWith(".png")) {
      await route.fulfill({ contentType: "image/png", body: PNG });
    } else {
      await route.fulfill({ status: 404, body: "" });
    }
  });
  await page.route(`${SITE}/**`, (route) =>
    route.fulfill({
      contentType: "text/html",
      body: `<!doctype html><title>Site</title><h1>${new URL(route.request().url()).pathname}</h1>`,
    }),
  );
  await page.route("https://elsewhere.test/**", (route) =>
    route.fulfill({ contentType: "text/html", body: "<h1>elsewhere</h1>" }),
  );
  await page.goto(`${ORIGIN}/`);
  const preview: Preview = {
    page,
    posted,
    send: (message) => page.evaluate((m) => window.postMessage(m, "*"), message),
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
  await preview.next("ready");
  return preview;
}

const BUILDS = [
  { name: "site", editor: true, description: "variants: switch; availability: badge" },
  { name: "cloud", editor: false, description: "variants: deployment=cloud; availability: badge" },
];

function render(seq: number, html: string | null, extra: Partial<ToWebview> = {}): ToWebview {
  return {
    type: "render",
    seq,
    build: "site",
    builds: BUILDS,
    title: "A page",
    formattedTitle: null,
    available: [],
    html,
    assets: [],
    problems: [],
    ...extra,
  } as ToWebview;
}

const TABS = `<ascribe-tabs sync="pm">
<ascribe-tab value="npm" label="npm">

<p>npm arm</p>

</ascribe-tab>
<ascribe-tab value="pnpm" label="pnpm">

<p>pnpm arm</p>

</ascribe-tab>
</ascribe-tabs>`;

describe("the preview webview", () => {
  it("draws the page with the element library, and the tabs work", async () => {
    const preview = await open();
    await preview.send(
      render(
        1,
        `<h2 id="install">Install</h2>\n${TABS}\n<ascribe-note type="tip"><p>Hi</p></ascribe-note>`,
      ),
    );
    const drawn = await preview.next("rendered");
    expect(drawn).toMatchObject({ seq: 1 });
    if (drawn.type !== "rendered") throw new Error();
    expect(drawn.report.elementsDefined).toBe(true);
    expect(drawn.report.headings).toEqual(["install"]);
    expect(drawn.report.elements["ascribe-tabs"]).toBe(1);
    expect(drawn.report.violations).toEqual([]);

    // The script built the tab list; choosing the second arm shows its panel.
    const tabs = preview.page.locator("ascribe-tabs [role=tab]");
    await expect(tabs.count()).resolves.toBe(2);
    await tabs.nth(1).click();
    await expect(tabs.nth(1).getAttribute("aria-selected")).resolves.toBe("true");
    await expect(preview.page.getByText("pnpm arm", { exact: true }).isVisible()).resolves.toBe(
      true,
    );
    await expect(preview.page.getByText("npm arm", { exact: true }).isVisible()).resolves.toBe(
      false,
    );
    await expect(preview.page.locator("h1").textContent()).resolves.toBe("A page");
    await preview.page.close();
  });

  it("heads the page with its formatted title, code spans as code", async () => {
    const preview = await open();
    await preview.send(
      render(1, "<p>Keys.</p>", {
        title: "ascribe.toml <keys>",
        formattedTitle: [
          { type: "code", value: "ascribe.toml" },
          { type: "text", value: " <keys>" },
        ],
      }),
    );
    await preview.next("rendered");
    await expect(preview.page.locator("h1").textContent()).resolves.toBe("ascribe.toml <keys>");
    await expect(preview.page.locator("h1 code").textContent()).resolves.toBe("ascribe.toml");
    await preview.page.close();
  });

  it("points an image at the webview URL of its source file, however the reference is spelled", async () => {
    const preview = await open();
    await preview.send(
      render(
        1,
        // comrak writes a space as %20; the server's reference has the raw space.
        `<p><img src="./My%20Diagrams/a.png" alt="A" width="300"><img src="../_fragments/b.png" alt="B"><img src="https://example.com/remote.png" alt="C"></p>`,
        {
          assets: [
            { reference: "./My Diagrams/a.png", uri: `${ORIGIN}/project/My%20Diagrams/a.png?v=1` },
            { reference: "../_fragments/b.png", uri: `${ORIGIN}/project/_fragments/b.png?v=1` },
          ],
        },
      ),
    );
    const images = await preview.next("images");
    if (images.type !== "images") throw new Error();
    expect(images.images.map((i) => i.src)).toEqual([
      `${ORIGIN}/project/My%20Diagrams/a.png?v=1`,
      `${ORIGIN}/project/_fragments/b.png?v=1`,
      "https://example.com/remote.png",
    ]);
    // The two project files load (the fake server has them); the remote image is
    // refused by the content security policy, and the reader sees it as broken.
    expect(images.images.map((i) => i.loaded)).toEqual([true, true, false]);
    // The other attributes of the image stay.
    await expect(preview.page.locator("img").first().getAttribute("width")).resolves.toBe("300");
    await preview.page.close();
  });

  it("takes scripts out of a page's raw HTML, refuses inline styles, and says so", async () => {
    const preview = await open();
    await preview.send(
      render(
        1,
        `<script>window.pwned = 1</script>
<img src="./x.png" onerror="window.pwned = 2" alt="">
<div id="styled" style="color: rgb(255, 0, 0)">styled</div>
<a href="javascript:window.pwned=3">js</a>`,
      ),
    );
    await preview.next("rendered");
    // The page is disarmed before it's shown, as the review report does it.
    await expect(preview.page.locator("main script").count()).resolves.toBe(0);
    await expect(preview.page.locator("main img").getAttribute("onerror")).resolves.toBeNull();
    await expect(preview.page.locator("main a").getAttribute("href")).resolves.toBeNull();
    // Let a failed image's error handler run, if it were going to.
    await preview.page.waitForTimeout(200);
    await expect(preview.page.evaluate(() => (window as { pwned?: number }).pwned)).resolves.toBe(
      undefined,
    );
    await expect(
      preview.page.locator("#styled").evaluate((e) => getComputedStyle(e).color),
    ).resolves.not.toBe("rgb(255, 0, 0)");
    // The page records violations; a second render carries them in its report.
    const seen = preview.posted.length;
    await preview.send(render(2, ""));
    const second = await preview.next("rendered", seen);
    if (second.type !== "rendered") throw new Error();
    const directives = second.report.violations.map((v) => v.split(" ")[0]);
    expect(directives).toContain("style-src-attr");
    await preview.page.close();
  });

  it("posts a click on a link to the extension, and follows a fragment itself", async () => {
    const preview = await open();
    await preview.send(
      render(
        1,
        `<h2 id="top">Top</h2>${"<p>filler</p>".repeat(300)}` +
          `<p><a id="other" href="/guides/setup/#install">setup</a> <a id="jump" href="#end">end</a></p>` +
          `<h2 id="end">End</h2>`,
      ),
    );
    await preview.next("rendered");
    await preview.page.locator("a", { hasText: "setup" }).click();
    await expect(preview.next("open")).resolves.toEqual({
      type: "open",
      href: "/guides/setup/#install",
    });
    const before = preview.posted.length;
    await preview.page.locator("a", { hasText: "end" }).click();
    await preview.page.waitForTimeout(100);
    expect(preview.posted.slice(before)).toEqual([]);
    await expect(preview.page.evaluate(() => window.scrollY)).resolves.toBeGreaterThan(0);
    await preview.page.close();
  });

  it("lists the builds, posts the one picked, and keeps the scroll position when the page redraws", async () => {
    const preview = await open();
    const tall = `<h2 id="a">A</h2>${"<p>filler</p>".repeat(200)}<h2 id="b">B</h2>${"<p>filler</p>".repeat(200)}`;
    await preview.send(render(1, tall));
    await preview.next("rendered");
    const options = await preview.page.locator("select option").allTextContents();
    expect(options).toEqual(["site (editor)", "cloud"]);
    await expect(preview.page.locator("select").inputValue()).resolves.toBe("site");

    await preview.page.selectOption("select", "cloud");
    await expect(preview.next("build")).resolves.toEqual({ type: "build", name: "cloud" });

    // Scroll, then draw again with the same content: the reader stays put.
    await preview.page.evaluate(() => window.scrollTo(0, 1500));
    await preview.send(render(2, tall.replace("B</h2>", "B!</h2>"), { build: "cloud" }));
    await preview.next("rendered", 1);
    await expect(preview.page.locator("select").inputValue()).resolves.toBe("cloud");
    await expect(preview.page.evaluate(() => Math.round(window.scrollY))).resolves.toBe(1500);

    // The cursor moved to section b in the editor: the preview scrolls to it.
    await preview.send({ type: "reveal", id: "b" });
    await preview.page.waitForFunction(() => window.scrollY > 2500);
    await preview.page.close();
  });

  it("shows why there is no page, without a page", async () => {
    const preview = await open();
    await preview.send(
      render(1, null, {
        builds: [],
        problems: [{ severity: "info", message: "x.md is a fragment." }],
      }),
    );
    await preview.next("rendered");
    await expect(preview.page.locator("[data-role=problems] li").textContent()).resolves.toBe(
      "x.md is a fragment.",
    );
    await expect(preview.page.locator("[data-role=page]").isHidden()).resolves.toBe(true);
    await expect(preview.page.locator("select").isDisabled()).resolves.toBe(true);
    await preview.page.close();
  });

  it("offers the server's output beside a problem that asks for it", async () => {
    const preview = await open();
    await preview.send(
      render(1, null, {
        builds: [],
        problems: [
          { severity: "info", message: "x.md is a fragment." },
          { severity: "error", message: "The server failed.", action: "showOutput" },
        ],
      }),
    );
    await preview.next("rendered");
    const items = preview.page.locator("[data-role=problems] li");
    await expect(items.nth(0).locator("button").count()).resolves.toBe(0);
    await items.nth(1).getByRole("button", { name: "Show Output" }).click();
    await expect(preview.next("showOutput")).resolves.toEqual({ type: "showOutput" });
    await preview.page.close();
  });

  it("shows the page-level availability the way the sample layout does", async () => {
    const preview = await open();
    await preview.send(
      render(1, "<p>x</p>", {
        available: [
          { target: "cloud", dimension: "deployment", states: ["ga"], text: "Loom Cloud (GA)" },
          {
            target: "self-managed",
            dimension: "deployment",
            states: ["preview"],
            versions: ["3.4"],
            text: "Self-managed (preview, 3.4+)",
          },
        ],
      } as Partial<ToWebview>),
    );
    await preview.next("rendered");
    const text = await preview.page.locator("[data-role=availability]").textContent();
    expect(text).toContain("Loom Cloud (GA)");
    expect(text).toContain("Self-managed (preview, 3.4+)");
    await expect(
      preview.page.locator("ascribe-availability-target[versions='3.4']").count(),
    ).resolves.toBe(1);
    await preview.page.close();
  });
});

const REVIEWED = `<p data-ascribe-source="page.md:1-1">First, unchanged.</p>
<p data-ascribe-source="page.md:3-3">Run the new installer.</p>
<p data-ascribe-source="_f/frag.md:1-1" data-ascribe-via="page.md:5">From a fragment.</p>`;
const WAS = `<p data-ascribe-source="page.md:1-1">First, unchanged.</p>
<p data-ascribe-source="page.md:3-3">Run the installer.</p>
<p data-ascribe-source="page.md:5-5">A paragraph that went away.</p>`;

function reviewed(seq: number, review: Partial<ReviewView> = {}): ToWebview {
  return render(seq, REVIEWED, {
    path: "page.md",
    review: {
      base: "main",
      commit: "1a2b3c4",
      page: {
        path: "page.md",
        route: "/page/",
        status: "changed",
        own_file_changed: true,
        because: [],
        page_changed: [],
        counts: { changed: 1, added: 1, removed: 1, moved: 0 },
        changes: [
          {
            kind: "changed",
            now: { source: "page.md:3-3", via: [] },
            was: { source: "page.md:3-3", via: [] },
            words: {
              now: [[8, 12]],
              was: [],
              now_text: "Run the new installer.",
              was_text: "Run the installer.",
            },
          },
          {
            kind: "removed",
            was: { source: "page.md:5-5", via: [] },
            after: { source: "page.md:3-3", via: [] },
            text: "A paragraph that went away.",
          },
          { kind: "added", now: { source: "_f/frag.md:1-1", via: ["page.md:5"] } },
        ],
      },
      wasHtml: WAS,
      causes: [],
      goToFirst: false,
      threads: null,
      errors: 0,
      ...review,
    },
  } as Partial<ToWebview>);
}

describe("review in the preview webview", () => {
  it("marks the page's changes and says how many there are", async () => {
    const preview = await open();
    await preview.send(reviewed(1));
    const drawn = await preview.next("rendered");
    if (drawn.type !== "rendered") throw new Error();
    expect(drawn.report.marks).toEqual({ changed: 1, removed: 1, added: 1 });
    expect(drawn.report.reviewHeader).toContain("Against main");
    expect(drawn.report.reviewHeader).toContain("3 changes on this page");
    await expect(preview.page.locator("[data-role=review] b").getAttribute("title")).resolves.toBe(
      "Compared with 1a2b3c4, where this branch left main",
    );
    // The changed words, and the removed block as the old page rendered it.
    await expect(preview.page.locator("ins.ascribe-ins").textContent()).resolves.toBe("new ");
    await expect(preview.page.locator(".ascribe-removed-body").textContent()).resolves.toBe(
      "A paragraph that went away.",
    );
    await preview.page.close();
  });

  it("steps through the changes, then offers the next changed page", async () => {
    const preview = await open();
    await preview.send(reviewed(1));
    await preview.next("rendered");
    const next = preview.page.getByRole("button", { name: "Next change" });
    await next.click();
    await expect(preview.page.locator(".review .position").textContent()).resolves.toBe(
      "1 of 3 on this page",
    );
    await next.click();
    await next.click();
    await expect(preview.page.locator(".review .position").textContent()).resolves.toBe(
      "3 of 3 on this page",
    );
    const from = preview.posted.length;
    await next.click();
    await preview.next("atEnd", from);
    await expect(preview.page.locator(".review .notice").textContent()).resolves.toContain(
      "That was the last change on this page.",
    );
    await preview.send({
      type: "nextPage",
      page: { path: "other.md", title: "Other page" },
      first: false,
    });
    await preview.page.getByRole("button", { name: "Next changed page: Other page" }).click();
    await expect(preview.next("openPage", from)).resolves.toEqual({
      type: "openPage",
      path: "other.md",
    });
    await preview.page.close();
  });

  it("shows the page as it will be and as it was", async () => {
    const preview = await open();
    await preview.send(reviewed(1));
    await preview.next("rendered");
    const content = preview.page.locator("[data-role=content]");
    await preview.page.getByRole("button", { name: "As it was" }).click();
    await expect(content.getAttribute("data-ascribe-show")).resolves.toBe("was");
    await expect(preview.page.getByText("A paragraph that went away.").isVisible()).resolves.toBe(
      true,
    );
    await expect(preview.page.getByText("From a fragment.").isVisible()).resolves.toBe(false);
    await preview.page.getByRole("button", { name: "As it will be" }).click();
    await expect(preview.page.getByText("From a fragment.").isVisible()).resolves.toBe(true);
    await expect(preview.page.getByText("A paragraph that went away.").isVisible()).resolves.toBe(
      false,
    );
    await preview.page.close();
  });

  it("opens a mark's source, and the file a page changed through", async () => {
    const preview = await open();
    await preview.send(
      reviewed(1, { causes: [{ label: "_f/frag.md", path: "/project/docs/_f/frag.md" }] }),
    );
    await preview.next("rendered");
    await preview.page.locator("[data-ascribe-change=added] .ascribe-label").click();
    await expect(preview.next("openSource")).resolves.toEqual({
      type: "openSource",
      source: "_f/frag.md:1-1",
    });
    await expect(preview.page.locator(".review .grow").textContent()).resolves.toBe(
      "Against main · changed only through _f/frag.md",
    );
    await preview.page.getByRole("button", { name: "_f/frag.md" }).click();
    await expect(preview.next("openFile")).resolves.toEqual({
      type: "openFile",
      path: "/project/docs/_f/frag.md",
    });
    await preview.page.close();
  });

  it("says when the project has errors, and opens the Problems panel", async () => {
    const preview = await open();
    await preview.send(reviewed(1));
    await preview.next("rendered");
    await expect(preview.page.locator(".review .notice").count()).resolves.toBe(0);
    const from = preview.posted.length;
    await preview.send(reviewed(2, { errors: 3 }));
    await preview.next("rendered", from);
    await expect(preview.page.locator(".review .notice").first().textContent()).resolves.toContain(
      "This project has 3 errors, so a page may not show as it will once they're fixed.",
    );
    await preview.page.getByRole("button", { name: "Show problems" }).click();
    await expect(preview.next("showProblems")).resolves.toEqual({ type: "showProblems" });
    await preview.page.close();
  });

  it("shows no header and leaves no marks when review is turned off", async () => {
    const preview = await open();
    await preview.send(reviewed(1));
    await preview.next("rendered");
    const from = preview.posted.length;
    await preview.send(
      render(2, REVIEWED, { path: "page.md", review: null } as Partial<ToWebview>),
    );
    const drawn = await preview.next("rendered", from);
    if (drawn.type !== "rendered") throw new Error();
    expect(drawn.report.marks).toEqual({});
    expect(drawn.report.reviewHeader).toBeNull();
    await expect(preview.page.locator("[data-ascribe-change]").count()).resolves.toBe(0);
    await expect(preview.page.locator("[data-ascribe-ui]").count()).resolves.toBe(0);
    await expect(preview.page.locator("[data-role=review]").isHidden()).resolves.toBe(true);
    // No page: no header at all.
    await preview.send(render(3, null, { review: null } as Partial<ToWebview>));
    const empty = await preview.next("rendered", from + 1);
    if (empty.type !== "rendered") throw new Error();
    expect(empty.report.reviewHeader).toBeNull();
    await preview.page.close();
  });
});

/** A located review thread, as the extension sends it in a `load` answer. */
function thread(id: string, over: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id,
    kind: "review",
    repositoryPath: "docs/page.md",
    path: "page.md",
    subject: "line",
    side: "RIGHT",
    line: 3,
    startLine: null,
    originalLine: 3,
    originalStartLine: null,
    commit: "abc1234",
    originalCommit: "abc1234",
    resolved: false,
    outdated: false,
    canResolve: true,
    canUnresolve: false,
    canReply: true,
    diffHunk: undefined,
    marker: undefined,
    quote: undefined,
    lines: { first: 3, last: 3 },
    detached: undefined,
    comments: [
      {
        id: `${id}-c1`,
        author: { login: "ana", avatarUrl: "https://avatars.example.com/ana" },
        body: "Say which installer. <img src=x onerror=alert(1)>",
        createdAt: "2026-10-04T10:00:00Z",
        url: "https://github.com/acme/quill/pull/128#c1",
        pending: false,
      },
    ],
    ...over,
  };
}

const THREADS_ON = {
  state: "on",
  pullRequest: { number: 128, url: "https://github.com/acme/quill/pull/128", baseRefName: "main" },
  local: { state: "same", behind: 0, ahead: 0 },
  gh: false,
  message: null,
  goTo: null,
} as const;

/**
 * Answers the overlay's requests as the extension would, from `answer`, until
 * the page closes. Returns the requests seen.
 */
function serveThreads(
  preview: Preview,
  answer: (method: string, params: Record<string, unknown>) => unknown,
): Extract<FromWebview, { type: "threads" }>[] {
  const seen: Extract<FromWebview, { type: "threads" }>[] = [];
  let at = 0;
  const timer = setInterval(() => {
    if (preview.page.isClosed()) {
      clearInterval(timer);
      return;
    }
    for (; at < preview.posted.length; at++) {
      const message = preview.posted[at];
      if (message?.type !== "threads") continue;
      seen.push(message);
      void preview.send({
        type: "threadsResult",
        id: message.id,
        result: answer(message.method, message.params),
      });
    }
  }, 10);
  return seen;
}

function pending(count: number, threads: unknown[] = []): Record<string, unknown> {
  return { id: count > 0 ? "R1" : undefined, threads, replies: [], conversation: [], count };
}

describe("review threads in the preview webview", () => {
  it("shows the pull request's threads beside their blocks, with no request and no policy violation", async () => {
    const preview = await open();
    const outside: string[] = [];
    preview.page.on("request", (request) => {
      if (!request.url().startsWith(ORIGIN)) outside.push(request.url());
    });
    const requests = serveThreads(preview, (method) => {
      if (method === "load") {
        return {
          pullRequest: { number: 128, url: THREADS_ON.pullRequest.url },
          threads: {
            blocks: [{ anchor: { source: "page.md:3-3", via: [] }, threads: [thread("T1")] }],
            removed: [],
            detached: [thread("T2", { lines: undefined, detached: "line-gone", line: null })],
          },
          pending: pending(0),
          viewer: "kyle",
        };
      }
      if (method === "allThreads") return [];
      return null;
    });
    await preview.send(reviewed(1, { threads: THREADS_ON }));
    const drawn = await preview.next("threadsDrawn");
    if (drawn.type !== "threadsDrawn") throw new Error();
    expect(drawn.report).toEqual({
      blocks: { "page.md:3-3": ["T1"] },
      detached: ["T2"],
      unsent: 0,
    });
    // The overlay asked for the page's blocks, and where its changed and removed ones were.
    const load = requests.find((r) => r.method === "load");
    expect(load?.params["anchors"]).toEqual([
      { source: "page.md:1-1", via: [] },
      { source: "page.md:3-3", via: [] },
      { source: "_f/frag.md:1-1", via: ["page.md:5"] },
    ]);
    expect(load?.params["removed"]).toEqual([
      { source: "page.md:3-3", via: [] },
      { source: "page.md:5-5", via: [] },
    ]);
    // The card, beside the page; the comment's raw HTML is text.
    await expect(
      preview.page.getByText("Say which installer.", { exact: false }).first().isVisible(),
    ).resolves.toBe(true);
    await expect(preview.page.locator("[data-ascribe-overlay] >> img").count()).resolves.toBe(0);
    await expect(preview.page.locator("[data-role=review]").textContent()).resolves.toMatch(
      /^#128 against main/,
    );
    // Avatars aren't fetched: nothing left the webview's origin.
    expect(outside).toEqual([]);
    const seen = preview.posted.length;
    await preview.send(reviewed(2, { threads: THREADS_ON }));
    const second = await preview.next("rendered", seen);
    if (second.type !== "rendered") throw new Error();
    expect(second.report.violations).toEqual([]);
    await preview.page.close();
  });

  it("comments on a block, then submits the review", async () => {
    const preview = await open();
    let unsent: unknown[] = [];
    const requests = serveThreads(preview, (method, params) => {
      if (method === "load") {
        return {
          pullRequest: { number: 128, url: THREADS_ON.pullRequest.url },
          threads: {
            blocks: unsent.length
              ? [{ anchor: { source: "page.md:1-1", via: [] }, threads: unsent }]
              : [],
            removed: [],
            detached: [],
          },
          pending: pending(unsent.length, unsent),
          viewer: "kyle",
        };
      }
      if (method === "allThreads") return [];
      if (method === "commentTarget") return { kind: "thread" };
      if (method === "comment") {
        const made = thread("T9", {
          lines: { first: 1, last: 1 },
          line: 1,
          comments: [
            {
              id: "T9-c1",
              author: { login: "kyle", avatarUrl: undefined },
              body: String(params["body"]),
              createdAt: "2026-10-04T10:00:00Z",
              url: "",
              pending: true,
            },
          ],
        });
        unsent = [made];
        return made;
      }
      if (method === "submit") {
        unsent = [];
        return null;
      }
      return null;
    });
    await preview.send(reviewed(1, { threads: THREADS_ON }));
    await preview.next("threadsDrawn");
    await preview.page.getByText("First, unchanged.").hover();
    await preview.page.getByRole("button", { name: "Comment on this block" }).click();
    await preview.page.getByRole("textbox", { name: "Comment" }).fill("Is this still true?");
    await preview.page.getByRole("button", { name: "Add to review" }).click();
    await expect
      .poll(() => requests.find((r) => r.method === "comment")?.params)
      .toEqual({
        anchor: { source: "page.md:1-1", via: [] },
        body: "Is this still true?",
        quote: "First, unchanged.",
      });
    // The unsent bar, then the dialog.
    await preview.page.getByRole("button", { name: "Submit review…" }).click();
    await preview.page.getByRole("radio", { name: "Approve" }).check();
    await preview.page.getByRole("button", { name: "Submit review", exact: true }).click();
    await expect
      .poll(() => requests.find((r) => r.method === "submit")?.params)
      .toEqual({ event: "APPROVE" });
    await expect.poll(() => preview.posted.some((m) => m.type === "notify")).toBe(true);
    await preview.page.close();
  });

  it("asks the extension for an agent prompt about a thread, the page, or a fragment's pages", async () => {
    const preview = await open();
    const requests = serveThreads(preview, (method) => {
      if (method === "load") {
        return {
          pullRequest: { number: 128, url: THREADS_ON.pullRequest.url },
          threads: {
            blocks: [{ anchor: { source: "page.md:3-3", via: [] }, threads: [thread("T1")] }],
            removed: [],
            detached: [],
          },
          pending: pending(0),
          viewer: "kyle",
        };
      }
      if (method === "allThreads") return [];
      return null;
    });
    const view = reviewed(1, { threads: THREADS_ON });
    if (view.type !== "render" || !view.review?.page) throw new Error();
    view.review.page.because = ["_f/frag.md", "ascribe.toml"];
    await preview.send(view);
    await preview.next("threadsDrawn");
    const prompts = () =>
      requests.filter((r) => r.method === "promptAgent").map((r) => r.params["request"]);

    await preview.page.getByRole("button", { name: "Prompt agent", exact: true }).click();
    await expect.poll(prompts).toEqual([{ kind: "thread", threadId: "T1" }]);

    const more = preview.page.getByRole("button", { name: "More review actions" });
    await more.click();
    await expect(more.getAttribute("aria-expanded")).resolves.toBe("true");
    // The model file isn't a fragment: it has no item.
    await expect(preview.page.getByRole("menuitem").allTextContents()).resolves.toEqual([
      "Prompt agent: review this page",
      "Prompt agent: check the pages that show _f/frag.md",
    ]);
    await preview.page.keyboard.press("ArrowDown");
    await preview.page.keyboard.press("Enter");
    await expect.poll(prompts).toEqual([
      { kind: "thread", threadId: "T1" },
      { kind: "fragment-reach", fragment: "_f/frag.md" },
    ]);
    await expect(preview.page.getByRole("menu").count()).resolves.toBe(0);
    await more.click();
    await preview.page.keyboard.press("Escape");
    await expect(preview.page.getByRole("menu").count()).resolves.toBe(0);
    await more.click();
    await preview.page.getByRole("menuitem", { name: "Prompt agent: review this page" }).click();
    await expect.poll(() => prompts().at(-1)).toEqual({ kind: "page-changes" });
    await preview.page.close();
  });

  it("says comments need GitHub when signed out, and offers both ways in", async () => {
    const preview = await open();
    await preview.send(
      reviewed(1, {
        threads: { ...THREADS_ON, state: "signed-out", pullRequest: null, local: null, gh: true },
      }),
    );
    const drawn = await preview.next("rendered");
    if (drawn.type !== "rendered") throw new Error();
    expect(drawn.report.reviewHeader).toContain("Showing changes only. Comments need GitHub.");
    expect(drawn.report.marks).toEqual({ changed: 1, removed: 1, added: 1 });
    await expect(preview.page.locator("[data-ascribe-overlay]").count()).resolves.toBe(0);
    await preview.page.getByRole("button", { name: "Sign in to see comments" }).click();
    await preview.page.getByRole("button", { name: "Use GitHub CLI" }).click();
    expect(preview.posted.filter((m) => m.type === "signIn" || m.type === "useGh")).toEqual([
      { type: "signIn" },
      { type: "useGh" },
    ]);
    await preview.page.close();
  });

  it("says when the checkout is behind the pull request, and offers to pull", async () => {
    const preview = await open();
    serveThreads(preview, (method) =>
      method === "load"
        ? {
            pullRequest: { number: 128, url: THREADS_ON.pullRequest.url },
            threads: { blocks: [], removed: [], detached: [] },
            pending: pending(0),
            viewer: "kyle",
          }
        : [],
    );
    await preview.send(
      reviewed(1, { threads: { ...THREADS_ON, local: { state: "behind", behind: 2, ahead: 0 } } }),
    );
    const drawn = await preview.next("rendered");
    if (drawn.type !== "rendered") throw new Error();
    expect(drawn.report.reviewHeader).toContain(
      "Your checkout is 2 commits behind #128, so some comments may be on lines you don't have.",
    );
    await preview.page.getByRole("button", { name: "Pull" }).click();
    expect(preview.posted.find((m) => m.type === "git")).toEqual({ type: "git", command: "pull" });
    await preview.page.close();
  });
});

describe("the Page | Site switch", () => {
  const frame = (preview: Preview) => preview.page.locator('iframe[title="Site preview"]');

  it("marks the pressed side with the active border in a high-contrast dark theme, which gives buttons no background", async () => {
    const preview = await open();
    await preview.send(render(1, "<p>Hello</p>"));
    await preview.next("rendered");
    // What VS Code sets for its high-contrast dark theme: no button background.
    await preview.page.evaluate(() => {
      document.body.classList.add("vscode-high-contrast");
      const root = document.documentElement.style;
      root.setProperty("--vscode-foreground", "rgb(255, 255, 255)");
      root.setProperty("--vscode-button-foreground", "rgb(255, 255, 255)");
      root.setProperty("--vscode-contrastActiveBorder", "rgb(243, 133, 24)");
    });
    const pressed = preview.page
      .getByRole("group", { name: "Preview" })
      .getByRole("button", { name: "Page" });
    const style = await pressed.evaluate((e) => {
      const { color, backgroundColor, boxShadow } = getComputedStyle(e);
      return { color, backgroundColor, boxShadow };
    });
    expect(style).toEqual({
      color: "rgb(255, 255, 255)",
      backgroundColor: "rgba(0, 0, 0, 0)",
      boxShadow: "rgb(243, 133, 24) 0px 0px 0px 1px inset",
    });
    await preview.page.close();
  });

  it("asks for the site, then shows the dev server's page in a frame", async () => {
    const preview = await open({ frameOrigin: SITE });
    await preview.send(render(1, "<p>Hello</p>"));
    await preview.next("rendered");
    const group = preview.page.getByRole("group", { name: "Preview" });
    await expect(
      group.getByRole("button", { name: "Page" }).getAttribute("aria-pressed"),
    ).resolves.toBe("true");
    const from = preview.posted.length;
    await group.getByRole("button", { name: "Site" }).click();
    await expect(preview.next("surface", from)).resolves.toEqual({
      type: "surface",
      surface: "site",
    });
    // The page preview hides at once; the frame waits for its address.
    await expect(preview.page.getByText("Hello").isVisible()).resolves.toBe(false);
    await expect(preview.page.getByLabel("Build").isVisible()).resolves.toBe(false);
    await preview.send({ type: "surface", surface: "site", url: `${SITE}/docs/install` });
    const shown = await preview.next("siteShown", from);
    expect(shown).toEqual({ type: "siteShown", url: `${SITE}/docs/install` });
    await expect(
      preview.page.frameLocator('iframe[title="Site preview"]').locator("h1").textContent(),
    ).resolves.toBe("/docs/install");
    await expect(frame(preview).boundingBox()).resolves.toMatchObject({ width: 1280 });

    // Back to the page, as it was.
    await group.getByRole("button", { name: "Page" }).click();
    await expect(preview.page.getByText("Hello").isVisible()).resolves.toBe(true);
    await expect(frame(preview).isVisible()).resolves.toBe(false);
    await preview.page.close();
  });

  it("doesn't reload the frame for the same page, so a link followed in it stays", async () => {
    const preview = await open({ frameOrigin: SITE });
    await preview.send({ type: "surface", surface: "site", url: `${SITE}/a` });
    await preview.next("siteShown");
    const count = () => preview.posted.filter((m) => m.type === "siteShown").length;
    await preview.send({ type: "surface", surface: "site", url: `${SITE}/a` });
    await new Promise((resolve) => setTimeout(resolve, 300));
    expect(count()).toBe(1);
    await preview.send({ type: "surface", surface: "site", url: `${SITE}/b` });
    await expect.poll(count).toBe(2);
    await preview.page.close();
  });

  it("frames only the dev server's origin", async () => {
    const preview = await open({ frameOrigin: SITE });
    await preview.page.evaluate(() => {
      const refused: string[] = [];
      (window as unknown as { refused: string[] }).refused = refused;
      document.addEventListener("securitypolicyviolation", (event) =>
        refused.push(`${event.effectiveDirective} ${event.blockedURI}`),
      );
    });
    await preview.send({ type: "surface", surface: "site", url: "https://elsewhere.test/" });
    // The webview's policy refuses the frame.
    await expect
      .poll(() => preview.page.evaluate(() => (window as unknown as { refused: string[] }).refused))
      .toContainEqual(expect.stringMatching(/^frame-src https:\/\/elsewhere\.test/));
    await preview.page.close();
  });

  it("says why the site can't show, and tries again", async () => {
    const preview = await open();
    await preview.send({
      type: "surface",
      surface: "site",
      problem: "There's no site preview for quill. Start its dev server.",
    });
    await expect(
      preview.page.getByText("There's no site preview for quill.").isVisible(),
    ).resolves.toBe(true);
    await expect(frame(preview).isVisible()).resolves.toBe(false);
    const from = preview.posted.length;
    await preview.page.getByRole("button", { name: "Try again" }).click();
    await expect(preview.next("surface", from)).resolves.toEqual({
      type: "surface",
      surface: "site",
    });
    await preview.page.close();
  });
});
