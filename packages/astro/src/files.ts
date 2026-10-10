// Serving `_ascribe/files/`: the files pages link to that aren't pages or
// images. Astro doesn't copy them, so the integration
// does: into the build's output directory, and in dev through a middleware.
// Both put them at `<base>_ascribe/files/`, the URL the site output links to.
//
// With `[consumer] agents = true`, the same for the plain output, which is
// laid out by URL: `llms.txt`, each page's Markdown, and their files, all at
// `<base>` and below.
import { createReadStream } from "node:fs";
import { cp, stat } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** The directory of published files inside a site output root. */
function publishedDir(siteRoot: string): string {
  return path.join(siteRoot, "_ascribe", "files");
}

/** Copies the published files to `<outDir>/_ascribe/files/`. Returns whether there were any. */
export async function copyPublishedFiles(siteRoot: string, outDir: URL): Promise<boolean> {
  return copyDirectory(
    publishedDir(siteRoot),
    path.join(fileURLToPath(outDir), "_ascribe", "files"),
  );
}

/** Copies the plain output for agents to the build output's root, the base path. Returns whether there was one. */
export async function copyAgentFiles(plainRoot: string, outDir: URL): Promise<boolean> {
  return copyDirectory(plainRoot, fileURLToPath(outDir));
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

/** A dev-server middleware serving the published files at `<base>_ascribe/files/`. */
export function filesMiddleware(siteRoot: string, base: string): Middleware {
  return directoryMiddleware(publishedDir(siteRoot), `${base}_ascribe/files/`);
}

/** A dev-server middleware serving the plain output for agents at `<base>`: `llms.txt`, each page's `.md`, and their files. */
export function agentsMiddleware(plainRoot: string, base: string): Middleware {
  return directoryMiddleware(plainRoot, base);
}

/** A middleware serving the files in `root` at the URL path `prefix`, which ends in `/`. */
function directoryMiddleware(root: string, prefix: string): Middleware {
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
        if (req.method === "HEAD") return void res.end();
        createReadStream(file).pipe(res);
      },
      () => next(),
    );
  };
}
