// Which page of the build is at a route, and its title: read from the JSON
// output, which the dev server writes while review is on, since the site
// output doesn't say.
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

/** A page of the build, as its JSON output names it. */
export interface RoutedPage {
  /** The content path. */
  path: string;
  title: string | null;
}

/** How much of a page's JSON to read: its `path`, `route`, and `title` come first. */
const HEAD_BYTES = 4096;

/**
 * Every page of the build's JSON output (`<output-dir>/<build>/json/`), by
 * normalized route. Empty when there's no JSON output.
 */
export async function readRoutes(jsonRoot: string): Promise<Map<string, RoutedPage>> {
  const routes = new Map<string, RoutedPage>();
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
      if (page) routes.set(normalizeRoute(page.route), { path: page.path, title: page.title });
    }),
  );
  return routes;
}

/** A page's `path`, `route`, and `title`, from the start of its JSON, or the whole of it if need be. */
async function readHead(
  file: string,
): Promise<{ path: string; route: string; title: string | null } | undefined> {
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
  // The page's own keys come before `frontmatter`, so the first of each is the page's.
  const pagePath = field("path");
  const route = field("route");
  const title = field("title");
  if (pagePath !== undefined && route !== undefined && title !== undefined) {
    return { path: pagePath, route, title };
  }
  try {
    const page = JSON.parse(await readFile(file, "utf8")) as {
      path?: unknown;
      route?: unknown;
      title?: unknown;
    };
    return typeof page.path === "string" && typeof page.route === "string"
      ? {
          path: page.path,
          route: page.route,
          title: typeof page.title === "string" ? page.title : null,
        }
      : undefined;
  } catch {
    return undefined;
  }
}
