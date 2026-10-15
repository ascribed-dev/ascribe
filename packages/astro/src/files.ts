// Serving `_ascribe/files/`: the files pages link to that aren't pages or
// images. Astro doesn't copy them, so the integration
// does: into the build's output directory, and in dev through a middleware.
// Both put them at `<base-path>_ascribe/files/`, the URL the site output links
// to, where `<base-path>` is the project's `[consumer] base-path`: Astro's
// `base` or a path under it.
//
// With `[consumer] agents = true`, the same for the plain output, which is
// laid out by URL: `llms.txt`, each page's Markdown, and their files, all at
// `<base-path>` and below.
import { createReadStream } from "node:fs";
import { cp, stat } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { normalizeBase } from "./project.js";

/** The directory of published files inside a site output root. */
function publishedDir(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "files");
}

/**
 * The base path's folder in Astro's build output, which is at Astro's `base`:
 * `docs/` for a base path of `/docs/` under a `base` of `/`, `""` when they're the same.
 */
export function folderUnder(basePath: string, astroBase: string): string {
  const root = normalizeBase(astroBase);
  return basePath.startsWith(root) ? basePath.slice(root.length) : "";
}

/**
 * Copies the published files to `<outDir>/<folder>_ascribe/files/`, `folder` being the
 * base path's (`folderUnder`). Returns whether there were any.
 */
export async function copyPublishedFiles(
  siteRoot: string,
  outDir: URL,
  folder = "",
): Promise<boolean> {
  return copyDirectory(
    publishedDir(siteRoot),
    path.join(fileURLToPath(outDir), folder, "_ascribe", "files"),
  );
}

/** Copies the plain output for agents to the base path's folder of the build output. Returns whether there was one. */
export async function copyAgentFiles(
  plainRoot: string,
  outDir: URL,
  folder = "",
): Promise<boolean> {
  return copyDirectory(plainRoot, path.join(fileURLToPath(outDir), folder));
}

async function copyDirectory(from: string, to: string): Promise<boolean> {
  if (
    !(await stat(from).then(
      (s) => s.isDirectory(),
      () => false,
    ))
  )
    return false;
  await cp(from, to, { recursive: true });
  return true;
}

const TYPES: Record<string, string> = {
  ".pdf": "application/pdf",
  ".zip": "application/zip",
  ".json": "application/json",
  ".yaml": "text/yaml; charset=utf-8",
  ".yml": "text/yaml; charset=utf-8",
  ".txt": "text/plain; charset=utf-8",
  ".csv": "text/csv; charset=utf-8",
  ".md": "text/markdown; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".svg": "image/svg+xml",
};

type Middleware = (req: IncomingMessage, res: ServerResponse, next: () => void) => void;

/** A dev-server middleware serving the published files at `<base>_ascribe/files/`, `base` being the project's base path. */
export function filesMiddleware(siteRoot: string, base: string): Middleware {
  return directoryMiddleware(publishedDir(siteRoot), `${base}_ascribe/files/`);
}

/** Runs `middlewares` in order, each passing a request it doesn't serve to the next, and the last to `next`. */
export function inOrder(
  middlewares: readonly Middleware[],
  req: IncomingMessage,
  res: ServerResponse,
  next: () => void,
): void {
  const [first, ...rest] = middlewares;
  if (first === undefined) return next();
  first(req, res, () => inOrder(rest, req, res, next));
}

/**
 * A dev-server middleware serving the plain output for agents at `<base>`: `llms.txt`, each page's
 * `.md`, and their files, each with the headers `headers` gives its path from the host's root.
 */
export function agentsMiddleware(
  plainRoot: string,
  base: string,
  headers: (pathname: string) => Record<string, string> = () => ({}),
): Middleware {
  return directoryMiddleware(plainRoot, base, headers);
}

/**
 * A middleware serving the files in `root` at the URL path `prefix`, which ends in `/`, each with
 * the headers `headers` gives its path.
 */
function directoryMiddleware(
  root: string,
  prefix: string,
  headers: (pathname: string) => Record<string, string> = () => ({}),
): Middleware {
  // Only under the base path, as in the build. Vite strips the base from
  // `req.url` before a plugin's middleware runs, but connect keeps the original in `originalUrl`.
  return (req, res, next) => {
    const pathname =
      ((req as { originalUrl?: string }).originalUrl ?? req.url ?? "").split(/[?#]/, 1)[0] ?? "";
    if (!pathname.startsWith(prefix) || (req.method !== "GET" && req.method !== "HEAD"))
      return next();
    let relative: string;
    try {
      relative = decodeURIComponent(pathname.slice(prefix.length));
    } catch {
      return next();
    }
    const file = path.resolve(root, relative);
    if (file !== root && !file.startsWith(root + path.sep)) return next();
    stat(file).then(
      (info) => {
        if (!info.isFile()) return next();
        res.setHeader(
          "Content-Type",
          TYPES[path.extname(file).toLowerCase()] ?? "application/octet-stream",
        );
        res.setHeader("Content-Length", info.size);
        for (const [name, value] of Object.entries(headers(pathname))) res.setHeader(name, value);
        if (req.method === "HEAD") return void res.end();
        createReadStream(file).pipe(res);
      },
      () => next(),
    );
  };
}
