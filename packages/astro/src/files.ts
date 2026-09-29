// Serving `_tessera/files/`: the files pages link to that aren't pages or
// images (asset contract §3.2). Astro doesn't copy them, so the integration
// does: into the build's output directory, and in dev through a middleware.
// Both put them at `<base>_tessera/files/`, the URL the site output links to.
import { createReadStream } from "node:fs";
import { cp, stat } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

/** The directory of published files inside a site output root. */
export function publishedDir(siteRoot: string): string {
  return path.join(siteRoot, "_tessera", "files");
}

/** Copies the published files to `<outDir>/_tessera/files/`. Returns how many top-level entries were there. */
export async function copyPublishedFiles(siteRoot: string, outDir: URL): Promise<boolean> {
  const from = publishedDir(siteRoot);
  if (
    !(await stat(from).then(
      (s) => s.isDirectory(),
      () => false,
    ))
  )
    return false;
  await cp(from, path.join(fileURLToPath(outDir), "_tessera", "files"), { recursive: true });
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

/** A dev-server middleware serving the published files at `<base>_tessera/files/`. */
export function filesMiddleware(
  siteRoot: string,
  base: string,
): (req: IncomingMessage, res: ServerResponse, next: () => void) => void {
  const root = publishedDir(siteRoot);
  // Only under the base path, as in the build (asset contract §3.2). Vite strips the base from
  // `req.url` before a plugin's middleware runs, but connect keeps the original in `originalUrl`.
  const prefix = `${base}_tessera/files/`;
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
