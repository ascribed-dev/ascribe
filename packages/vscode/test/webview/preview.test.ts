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

async function open(): Promise<Preview> {
  const page = await context.newPage();
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

  it("refuses scripts and inline styles in a page's raw HTML, and says so", async () => {
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
