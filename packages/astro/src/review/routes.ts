// Which page of the build is at a route: read from the JSON output, which the
// dev server writes while review is on, since the site output doesn't say.
import { open, readFile } from "node:fs/promises";
import path from "node:path";

/** A route as compared: decoded, without a trailing `/` (except the root's). */
export function normalizeRoute(route: string): string {
  let decoded: string;
  try {
    decoded = decodeURI(route);
  } catch {
    decoded = route;
  }
  const trimmed = (decoded.split(/[?#]/)[0] ?? "").replace(/\/+$/, "");
  return trimmed === "" ? "/" : trimmed;
}

/** How much of a page's JSON to read: its `path` and `route` come first. */
const HEAD_BYTES = 4096;

/**
 * Every page of the build's JSON output (`<output-dir>/<build>/json/`), as
 * normalized route → content path. Empty when there's no JSON output.
 */
export async function readRoutes(jsonRoot: string): Promise<Map<string, string>> {
  const routes = new Map<string, string>();
  let files: { path?: unknown; kind?: unknown }[];
  try {
    const manifest = JSON.parse(
      await readFile(path.join(path.dirname(jsonRoot), "json.manifest.json"), "utf8"),
    ) as { files?: unknown };
    files = Array.isArray(manifest.files) ? (manifest.files as typeof files) : [];
  } catch {
    return routes;
  }
  await Promise.all(
    files.map(async (entry) => {
      if (entry.kind !== "page" || typeof entry.path !== "string") return;
      const page = await readHead(path.join(jsonRoot, ...entry.path.split("/")));
      if (page) routes.set(normalizeRoute(page.route), page.path);
    }),
  );
  return routes;
}

/** A page's `path` and `route`, from the start of its JSON, or the whole of it if need be. */
async function readHead(file: string): Promise<{ path: string; route: string } | undefined> {
  let head = "";
  try {
    const handle = await open(file, "r");
    try {
      const buffer = Buffer.alloc(HEAD_BYTES);
      const { bytesRead } = await handle.read(buffer, 0, HEAD_BYTES, 0);
      head = buffer.subarray(0, bytesRead).toString("utf8");
    } finally {
      await handle.close();
    }
  } catch {
    return undefined;
  }
  const field = (name: string): string | undefined => {
    const match = new RegExp(`"${name}"\\s*:\\s*("(?:[^"\\\\]|\\\\.)*")`).exec(head);
    if (!match?.[1]) return undefined;
    try {
      return JSON.parse(match[1]) as string;
    } catch {
      return undefined;
    }
  };
  const pagePath = field("path");
  const route = field("route");
  if (pagePath !== undefined && route !== undefined) return { path: pagePath, route };
  try {
    const page = JSON.parse(await readFile(file, "utf8")) as { path?: unknown; route?: unknown };
    return typeof page.path === "string" && typeof page.route === "string"
      ? { path: page.path, route: page.route }
      : undefined;
  } catch {
    return undefined;
  }
}
