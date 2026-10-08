// Which page of the build is at a route, and its title: read from the JSON
// output, which the dev server writes while review is on, since the site
// output doesn't say.
import { open, readFile } from "node:fs/promises";
import path from "node:path";
import type { FormattedPiece } from "@ascribed/review/overlay";

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
  /** The title formatted, when its field sets `inline = "code"`. */
  formatted_title: FormattedPiece[] | null;
}

/** How much of a page's JSON to read at a time. */
const CHUNK_BYTES = 4096;

/**
 * Where the page's own keys end in the JSON output, which writes it
 * pretty-printed: `availability` comes after `path`, `route`, `title`,
 * `frontmatter`, and `formatted`, and before the page's blocks.
 */
const AFTER_HEAD = Buffer.from('\n  "availability":');

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
      if (page) {
        const { route, ...routed } = page;
        routes.set(normalizeRoute(route), routed);
      }
    }),
  );
  return routes;
}

/**
 * A page's `path`, `route`, `title`, and formatted title, from the start of
 * its JSON up to `availability`, or the whole of it if need be.
 */
async function readHead(file: string): Promise<(RoutedPage & { route: string }) | undefined> {
  let text: string;
  try {
    const handle = await open(file, "r");
    try {
      const chunks: Buffer[] = [];
      let read = 0;
      let end = -1;
      for (;;) {
        const buffer = Buffer.alloc(CHUNK_BYTES);
        const { bytesRead } = await handle.read(buffer, 0, CHUNK_BYTES, read);
        if (bytesRead === 0) break;
        chunks.push(buffer.subarray(0, bytesRead));
        read += bytesRead;
        // The marker may straddle two chunks, so look from just before this one.
        end = Buffer.concat(chunks).indexOf(
          AFTER_HEAD,
          Math.max(0, read - bytesRead - AFTER_HEAD.length),
        );
        if (end >= 0) break;
      }
      const all = Buffer.concat(chunks);
      // The keys before `availability`, closed as an object of their own.
      text =
        end >= 0
          ? `${all.subarray(0, end).toString("utf8").replace(/,\s*$/, "")}}`
          : all.toString("utf8");
    } finally {
      await handle.close();
    }
  } catch {
    return undefined;
  }
  let page: { path?: unknown; route?: unknown; title?: unknown; formatted?: { title?: unknown } };
  try {
    page = JSON.parse(text) as typeof page;
  } catch {
    return undefined;
  }
  if (typeof page.path !== "string" || typeof page.route !== "string") return undefined;
  const formatted = page.formatted?.title;
  return {
    path: page.path,
    route: page.route,
    title: typeof page.title === "string" ? page.title : null,
    formatted_title: Array.isArray(formatted) ? (formatted as FormattedPiece[]) : null,
  };
}
