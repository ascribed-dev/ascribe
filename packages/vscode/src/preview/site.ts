// The site preview: the page in the real site, from the site generator's dev
// server. `@ascribed/astro` writes where its dev server is to
// `<project>/.ascribe/dev.json` when it starts and removes it when it stops;
// a server that crashed leaves it behind, so the URL is checked first.
import { readFile } from "node:fs/promises";
import * as path from "node:path";
import type { PreviewSection } from "./protocol.js";

/** What `dev.json` holds (`packages/astro/src/review/devfile.ts`). */
export interface DevServer {
  /** The site's address: origin and base path, `http://localhost:4321/docs/`. */
  url: string;
  /** The build the dev server shows. */
  build: string;
}

/** `<project>/.ascribe/dev.json`. */
export function devFilePath(projectFolder: string): string {
  return path.join(projectFolder, ".ascribe", "dev.json");
}

/** Whether a URL's host is this machine: `localhost`, `127.x.x.x` or `[::1]`. */
function isLoopback(hostname: string): boolean {
  return hostname === "localhost" || hostname === "[::1]" || /^127(\.\d{1,3}){3}$/.test(hostname);
}

/** The dev server a `dev.json` names, or `undefined` when it isn't one or isn't on this machine. */
export function parseDevFile(text: string): DevServer | undefined {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return undefined;
  }
  if (typeof value !== "object" || value === null) return undefined;
  const { url, build } = value as Record<string, unknown>;
  if (typeof url !== "string" || typeof build !== "string") return undefined;
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return undefined;
    // A committed dev.json could name any site; a dev server is on this machine.
    if (!isLoopback(parsed.hostname)) return undefined;
  } catch {
    return undefined;
  }
  return { url, build };
}

/** Whether something answers at `url`: any HTTP response does, a refused connection or a timeout doesn't. */
export async function answers(url: string, timeoutMs = 2000): Promise<boolean> {
  try {
    const response = await fetch(url, {
      method: "HEAD",
      redirect: "manual",
      signal: AbortSignal.timeout(timeoutMs),
    });
    await response.body?.cancel();
    return true;
  } catch {
    return false;
  }
}

export type DevServerLookup =
  | { state: "running"; server: DevServer }
  /** No `dev.json`, or one whose server doesn't answer. */
  | { state: "none" };

/** The project's dev server, if one is running. */
export async function findDevServer(
  projectFolder: string,
  check: (url: string) => Promise<boolean> = answers,
): Promise<DevServerLookup> {
  let text: string;
  try {
    text = await readFile(devFilePath(projectFolder), "utf8");
  } catch {
    return { state: "none" };
  }
  const server = parseDevFile(text);
  if (!server || !(await check(server.url))) return { state: "none" };
  return { state: "running", server };
}

/**
 * The address of a page on the dev server: the server's origin and the page's
 * route (which has the site's base path in it), at the heading `section`
 * when there is one.
 */
export function pageUrl(server: DevServer, route: string, section?: string): string {
  const url = new URL(route, server.url);
  if (section) url.hash = encodeURIComponent(section);
  return url.toString();
}

/**
 * The heading to open the site at for an editor showing `line` (from 0) at
 * its top: the last heading written in the file at or before it.
 */
export function sectionAt(sections: readonly PreviewSection[], line: number): string | undefined {
  let found: PreviewSection | undefined;
  for (const section of sections) {
    if (section.line <= line && (!found || section.line >= found.line)) found = section;
  }
  return found?.id;
}

/** What Open Site Preview says when there's no dev server for the project. */
export function noDevServerMessage(project: string): string {
  return `Ascribe: there's no site preview for ${project}. Start its dev server (astro dev) with @ascribed/astro, then try again.`;
}
