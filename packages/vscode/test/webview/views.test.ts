// The sidebar's webview views as VS Code embeds them: in a frame, on a host
// page that sets no `color-scheme`, over the sidebar's color. The view's body
// is transparent, so the sidebar shows through, unless the frame's scheme
// differs from its embedder's, when Chromium paints its own opaque backdrop
// behind the frame (#121212 in a dark one).
import { existsSync, readFileSync } from "node:fs";
import { extname, join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  chromium,
  type Browser,
  type BrowserContext,
  type Frame,
  type Page,
} from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { viewHtml } from "../../src/webview/viewShell.js";

const dist = fileURLToPath(new URL("../../dist/webview/", import.meta.url));
const VIEW = "https://view.test";
const HOST = "https://host.test";
/** Nu Disco Dark's `sideBar.background`, and a light theme's. */
const DARK_SIDEBAR = [0x18, 0x0d, 0x2a];
const LIGHT_SIDEBAR = [0xf3, 0xf3, 0xf3];

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

/** The stylesheet as 0.4.1 shipped it, with the root's scheme set. */
const OLD = ":root { color-scheme: light dark; }\n";

interface Embed {
  script: "thisPage" | "project";
  scheme: "dark" | "light";
  sidebar: number[];
  /** Serves the stylesheet as it was before the root's scheme was left `normal`. */
  old?: boolean;
}

/** A host page in the sidebar's color, with the view in a frame, as VS Code draws it. */
async function embed({
  script,
  scheme,
  sidebar,
  old = false,
}: Embed): Promise<{ page: Page; frame: Frame }> {
  const page = await context.newPage();
  page.setDefaultTimeout(10_000);
  await page.emulateMedia({ colorScheme: scheme });
  await page.addInitScript(() => {
    (window as unknown as Record<string, unknown>)["acquireVsCodeApi"] = () => ({
      postMessage: () => undefined,
      getState: () => null,
      setState: () => undefined,
    });
  });
  const rgb = `rgb(${sidebar.join(", ")})`;
  await page.route(`${HOST}/**`, (route) =>
    route.fulfill({
      contentType: "text/html",
      body: `<!doctype html><html><body style="margin: 0; background: ${rgb}">
<iframe src="${VIEW}/" style="display: block; width: 300px; height: 400px; border: 0"></iframe>
</body></html>`,
    }),
  );
  await page.route(`${VIEW}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/") {
      await route.fulfill({
        contentType: "text/html",
        body: viewHtml({
          cspSource: VIEW,
          nonce: "bm9uY2U=",
          script: `${VIEW}/dist/${script}.js`,
          style: `${VIEW}/dist/views.css`,
          title: script,
        }),
      });
    } else if (pathname.startsWith("/dist/")) {
      const types: Record<string, string> = {
        ".css": "text/css",
        ".js": "text/javascript",
        ".ttf": "font/ttf",
      };
      const file = readFileSync(join(dist, pathname.slice("/dist/".length)));
      await route.fulfill({
        contentType: types[extname(pathname)] ?? "application/octet-stream",
        body: old && pathname === "/dist/views.css" ? OLD + file.toString() : file,
      });
    } else {
      await route.fulfill({ status: 404, body: "" });
    }
  });
  await page.goto(`${HOST}/`);
  const frame = page.frames().find((f) => f.url() === `${VIEW}/`);
  if (!frame) throw new Error("the view's frame never loaded");
  await frame.waitForLoadState();
  // VS Code names the theme's kind on the body.
  await frame.evaluate((kind) => {
    document.body.classList.add(`vscode-${kind}`);
    document.documentElement.style.setProperty("--vscode-sideBar-background", "rgb(1, 2, 3)");
  }, scheme);
  return { page, frame };
}

/** The color drawn at a point of the page, read back from a screenshot. */
async function pixel(page: Page, x: number, y: number): Promise<number[]> {
  const shot = await page.screenshot({ clip: { x, y, width: 1, height: 1 } });
  const reader = await context.newPage();
  try {
    return await reader.evaluate(
      async (src) => {
        const image = new Image();
        image.src = src;
        await image.decode();
        const canvas = document.createElement("canvas");
        canvas.width = canvas.height = 1;
        const drawing = canvas.getContext("2d");
        if (!drawing) throw new Error("no 2d context");
        drawing.drawImage(image, 0, 0);
        return Array.from(drawing.getImageData(0, 0, 1, 1).data.slice(0, 3));
      },
      `data:image/png;base64,${shot.toString("base64")}`,
    );
  } finally {
    await reader.close();
  }
}

describe("the sidebar's webview views", () => {
  for (const script of ["thisPage", "project"] as const) {
    it(`show the sidebar's color through ${script}'s body in a dark theme`, async () => {
      const { page } = await embed({ script, scheme: "dark", sidebar: DARK_SIDEBAR });
      expect(await pixel(page, 250, 380)).toEqual(DARK_SIDEBAR);
      await page.close();
    });

    it(`show the sidebar's color through ${script}'s body in a light theme`, async () => {
      const { page } = await embed({ script, scheme: "light", sidebar: LIGHT_SIDEBAR });
      expect(await pixel(page, 250, 380)).toEqual(LIGHT_SIDEBAR);
      await page.close();
    });
  }

  it("would show Chromium's gray with the root's scheme set, as 0.4.1 drew it", async () => {
    const { page } = await embed({
      script: "project",
      scheme: "dark",
      sidebar: DARK_SIDEBAR,
      old: true,
    });
    expect(await pixel(page, 250, 380)).toEqual([0x12, 0x12, 0x12]);
    await page.close();
  });

  it("draw the build switch's native parts in the theme's scheme", async () => {
    const kinds: [string, string][] = [
      ["vscode-dark", "dark"],
      ["vscode-light", "light"],
      ["vscode-high-contrast", "dark"],
      ["vscode-high-contrast vscode-high-contrast-light", "light"],
    ];
    const { page, frame } = await embed({
      script: "project",
      scheme: "dark",
      sidebar: DARK_SIDEBAR,
    });
    const schemes = await frame.evaluate((kinds) => {
      const select = document.createElement("select");
      select.className = "value build-switch";
      document.querySelector("main")?.append(select);
      return kinds.map(([classes]) => {
        document.body.className = classes;
        return [classes, getComputedStyle(select).colorScheme];
      });
    }, kinds);
    expect(schemes).toEqual(kinds);
    await page.close();
  });
});
