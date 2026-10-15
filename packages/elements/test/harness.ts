import { existsSync, readFileSync } from "node:fs";
import { extname, join, normalize } from "node:path";
import { fileURLToPath } from "node:url";
import {
  chromium,
  firefox,
  webkit,
  type Browser,
  type BrowserContext,
  type Page,
} from "playwright-core";

const root = fileURLToPath(new URL("..", import.meta.url));
const ORIGIN = "http://ascribe.test";
const TYPES: Record<string, string> = {
  ".js": "text/javascript",
  ".css": "text/css",
  ".html": "text/html",
};

/** The bundled Chromium in this environment, if there is one; otherwise Playwright's own. */
function executablePath(): string | undefined {
  if (process.env["ASCRIBE_CHROMIUM"]) return process.env["ASCRIBE_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  return existsSync(bundled) ? bundled : undefined;
}

const ALL_ENGINES = ["chromium", "firefox", "webkit"] as const;
export type Engine = (typeof ALL_ENGINES)[number];

/**
 * The engines to run. ASCRIBE_ENGINES (a comma-separated subset) chooses them;
 * without it, CI (`CI` set, as Actions does) runs all three, and a local run
 * Chromium alone, since Firefox and WebKit triple the time and an engine
 * difference is rare. One test file per engine (`test/<engine>.test.ts`)
 * skips itself when its engine isn't here.
 */
const wanted: readonly string[] =
  process.env["ASCRIBE_ENGINES"]?.split(",").map((name) => name.trim()) ??
  (process.env["CI"] ? ALL_ENGINES : ["chromium"]);
export const ENGINES: readonly Engine[] = ALL_ENGINES.filter((name) => wanted.includes(name));

/** Launch one engine. */
export async function launch(engine: Engine): Promise<Browser> {
  if (engine === "firefox") return firefox.launch();
  if (engine === "webkit") return webkit.launch();
  const path = executablePath();
  return chromium.launch(path === undefined ? {} : { executablePath: path });
}

export interface PageOptions {
  /** Load `dist/index.js` (the elements). Default true. */
  script?: boolean;
  /** Load `css/style.css`. Default true. */
  css?: boolean;
  /** Make `localStorage` throw, as in a private window with storage blocked. */
  blockStorage?: boolean;
}

/** Serve `body` from a fixed origin, so `localStorage` works, with the library files from disk. */
export async function open(
  context: BrowserContext,
  body: string,
  options: PageOptions = {},
): Promise<Page> {
  const { script = true, css = true, blockStorage = false } = options;
  const page = await context.newPage();
  if (blockStorage) {
    await page.addInitScript(() => {
      Object.defineProperty(window, "localStorage", {
        get() {
          throw new DOMException("blocked", "SecurityError");
        },
      });
    });
  }
  await page.route(`${ORIGIN}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/") {
      const head = [
        '<meta charset="utf-8"><title>Test page</title>',
        css ? '<link rel="stylesheet" href="/css/style.css">' : "",
        script ? '<script type="module" src="/dist/index.js"></script>' : "",
      ].join("");
      await route.fulfill({
        contentType: TYPES[".html"] ?? "text/html",
        body: `<!doctype html><html lang="en"><head>${head}</head><body><main>${body}</main></body></html>`,
      });
      return;
    }
    const file = normalize(join(root, pathname));
    if (!file.startsWith(root) || !existsSync(file)) {
      await route.fulfill({ status: 404, body: "not found" });
      return;
    }
    await route.fulfill({
      contentType: TYPES[extname(file)] ?? "application/octet-stream",
      body: readFileSync(file),
    });
  });
  await page.goto(`${ORIGIN}/`);
  if (script) await page.waitForFunction(() => customElements.get("ascribe-tabs") !== undefined);
  return page;
}

/** Two `pm` groups, one `os` group, and one labeled group. */
export const TABS = `
<ascribe-tabs sync="pm" id="a">

<ascribe-tab value="npm" label="npm">

Install with npm.

</ascribe-tab>

<ascribe-tab value="pnpm yarn" label="pnpm / yarn">

Install with pnpm or yarn.

</ascribe-tab>

<ascribe-tab value="bun" label="bun">

Install with bun.

</ascribe-tab>

</ascribe-tabs>

<ascribe-tabs sync="pm" id="b">

<ascribe-tab value="pnpm" label="pnpm">

Configure with pnpm.

</ascribe-tab>

<ascribe-tab value="npm" label="npm">

Configure with npm.

</ascribe-tab>

</ascribe-tabs>

<ascribe-tabs sync="os" id="c">

<ascribe-tab value="linux" label="Linux">

Linux steps.

</ascribe-tab>

<ascribe-tab value="windows" label="Windows">

Windows steps.

</ascribe-tab>

</ascribe-tabs>

<ascribe-tabs id="d">

<ascribe-tab label="First look">

First look content.

</ascribe-tab>

<ascribe-tab label="Second look">

Second look content.

</ascribe-tab>

</ascribe-tabs>
`;
