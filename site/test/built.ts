// The built site, for the tests: they run after `npm run build`, on dist/ and
// on the Ascribe build the site was made from.
import { existsSync } from "node:fs";
import { readdir, readFile, stat } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, type Browser } from "playwright-core";

export const siteDir = fileURLToPath(new URL("..", import.meta.url));
export const distDir = path.join(siteDir, "dist");
/** ../docs/ascribe.toml's site output for the `site` build: the published pages. */
export const outputDir = path.join(siteDir, "..", "docs", ".ascribe", "build", "site", "site");

/**
 * `[consumer] site` in ../docs/ascribe.toml: the address the site was built
 * for, which its absolute links start with. A preview's build changes it.
 */
export async function siteAddress(): Promise<string> {
  const config = await readFile(path.join(siteDir, "..", "docs", "ascribe.toml"), "utf8");
  const table = /^\[consumer\]$([\s\S]*?)(?=^\[|(?![\s\S]))/m.exec(config)?.[1] ?? "";
  const site = /^site\s*=\s*"([^"]+)"/m.exec(table)?.[1];
  if (site === undefined) throw new Error("docs/ascribe.toml's [consumer] has no site");
  return new URL(site).origin;
}

/** Fails with what to run when the site hasn't been built. */
export function requireBuild(): void {
  for (const dir of [distDir, outputDir]) {
    if (!existsSync(dir)) throw new Error(`${dir} doesn't exist: run \`npm run build\` first`);
  }
}

/** Every file under `dir`, relative to it, with `/` separators. */
export async function filesUnder(dir: string): Promise<string[]> {
  const entries = await readdir(dir, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => path.relative(dir, path.join(entry.parentPath, entry.name)))
    .map((file) => file.split(path.sep).join("/"))
    .toSorted();
}

/** The published pages: the site output's Markdown files, by path from the content root. */
export async function publishedPages(): Promise<string[]> {
  return (await filesUnder(outputDir)).filter(
    (file) => file.endsWith(".md") && !file.split("/").some((segment) => segment.startsWith("_")),
  );
}

/**
 * The file dist/ serves for a URL path, as a static host does: `/a/b/` is
 * `a/b/index.html`, and a path to a file is that file. Undefined if none.
 */
export async function fileFor(urlPath: string): Promise<string | undefined> {
  const decoded = decodeURIComponent(urlPath);
  const candidate = path.join(distDir, ...decoded.split("/"));
  if (!candidate.startsWith(distDir)) return undefined;
  for (const file of [candidate, path.join(candidate, "index.html")]) {
    try {
      if ((await stat(file)).isFile()) return file;
    } catch {
      // Not there; try the next.
    }
  }
  return undefined;
}

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css",
  ".js": "text/javascript",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".webp": "image/webp",
  ".wasm": "application/wasm",
};

/** Serves dist/ on a free port, with its 404 page for anything missing. */
export async function serve(): Promise<{ url: string; close: () => Promise<void> }> {
  const server: Server = createServer((request, response) => {
    void (async () => {
      const urlPath = new URL(request.url ?? "/", "http://localhost").pathname;
      const file = await fileFor(urlPath);
      if (file === undefined) {
        response.writeHead(404, { "content-type": TYPES[".html"] });
        response.end(await readFile(path.join(distDir, "404.html")));
        return;
      }
      const type = TYPES[path.extname(file)] ?? "application/octet-stream";
      response.writeHead(200, { "content-type": type });
      response.end(await readFile(file));
    })();
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  if (address === null || typeof address === "string") throw new Error("no port");
  return {
    url: `http://127.0.0.1:${address.port}`,
    close: () => new Promise((resolve) => server.close(() => resolve())),
  };
}

/** Chromium: `ASCRIBE_CHROMIUM`, or the bundled one in this environment, or Playwright's own. */
export function launchChromium(): Promise<Browser> {
  const configured = process.env["ASCRIBE_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  const executablePath = configured ?? (existsSync(bundled) ? bundled : undefined);
  return chromium.launch(executablePath === undefined ? {} : { executablePath });
}
