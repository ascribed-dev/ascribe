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
const ORIGIN = "http://tessera.test";
const TYPES: Record<string, string> = {
  ".js": "text/javascript",
  ".css": "text/css",
  ".html": "text/html",
};

/** The bundled Chromium in this environment, if there is one; otherwise Playwright's own. */
function executablePath(): string | undefined {
  if (process.env["TESSERA_CHROMIUM"]) return process.env["TESSERA_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  return existsSync(bundled) ? bundled : undefined;
}

const ALL_ENGINES = ["chromium", "firefox", "webkit"] as const;
export type Engine = (typeof ALL_ENGINES)[number];
const wanted = process.env["TESSERA_ENGINES"]?.split(",").map((name) => name.trim());
export const ENGINES: readonly Engine[] = ALL_ENGINES.filter(
  (name) => wanted === undefined || wanted.includes(name),
);

/**
 * Launch one engine. The suite runs all three; TESSERA_ENGINES (a
 * comma-separated subset) narrows it on machines that can't install them all.
 */
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
  if (script) await page.waitForFunction(() => customElements.get("tessera-tabs") !== undefined);
  return page;
}

/** Two `pm` groups, one `os` group, and one labeled group. */
export const TABS = `
<tessera-tabs sync="pm" id="a">

<tessera-tab value="npm" label="npm">

Install with npm.

</tessera-tab>

<tessera-tab value="pnpm yarn" label="pnpm / yarn">

Install with pnpm or yarn.

</tessera-tab>

<tessera-tab value="bun" label="bun">

Install with bun.

</tessera-tab>

</tessera-tabs>

<tessera-tabs sync="pm" id="b">

<tessera-tab value="pnpm" label="pnpm">

Configure with pnpm.

</tessera-tab>

<tessera-tab value="npm" label="npm">

Configure with npm.

</tessera-tab>

</tessera-tabs>

<tessera-tabs sync="os" id="c">

<tessera-tab value="linux" label="Linux">

Linux steps.

</tessera-tab>

<tessera-tab value="windows" label="Windows">

Windows steps.

</tessera-tab>

</tessera-tabs>

<tessera-tabs id="d">

<tessera-tab label="First look">

First look content.

</tessera-tab>

<tessera-tab label="Second look">

Second look content.

</tessera-tab>

</tessera-tabs>
`;
