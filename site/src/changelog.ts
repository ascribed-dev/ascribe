// The changelog page: CHANGELOG.md at the repository's root, rendered by
// Astro's Markdown, with its links pointed at the site and the repository.
// The file stays where it is, and nothing is copied from it.
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Loader } from "astro/loaders";
import { z } from "astro/zod";
import { contentDir } from "./site.ts";

/** The file the page renders, relative to the repository's root. */
export const changelogFile = "CHANGELOG.md";

/** The collection's one entry. */
export const changelogId = "changelog";

/**
 * A collection with one entry: CHANGELOG.md, read from the repository the
 * site is in, rendered with the site's Markdown settings. The links are left
 * as the file writes them; the page points them, since only it can see which
 * docs pages were published.
 */
export function changelogLoader(): Loader {
  return {
    name: "changelog",
    // Where the repository is, which the changelog's links are relative to.
    schema: z.object({ repository: z.string() }),
    load: async (context) => {
      const { config, store, watcher, logger } = context;
      const file = fileURLToPath(new URL(`../${changelogFile}`, config.root));
      const load = async () => {
        const body = await readFile(file, "utf8");
        store.clear();
        store.set({
          id: changelogId,
          data: { repository: path.dirname(file) },
          body,
          rendered: await context.renderMarkdown(body),
        });
      };
      await load();
      watcher?.add(file);
      watcher?.on("change", (changed) => {
        if (path.resolve(changed) !== file) return;
        logger.info(`${changelogFile} changed`);
        void load();
      });
    },
  };
}

/** What a link in the changelog can lead to. */
export interface Targets {
  /** Each published docs page's route, by its source file relative to the repository's root. */
  pages: ReadonlyMap<string, string>;
  /** The repository's address on GitHub. */
  repo: string;
  /** Whether a path, relative to the repository's root, is a file, a folder, or nothing. */
  kind: (file: string) => Promise<"file" | "folder" | undefined>;
}

/** A link in the changelog that leads nowhere. */
export class ChangelogLinkError extends Error {
  constructor(readonly problems: string[]) {
    super(
      `${changelogFile} has ${problems.length === 1 ? "a link" : "links"} the site can't publish:\n` +
        problems.map((problem) => `  ${problem}`).join("\n"),
    );
    this.name = "ChangelogLinkError";
  }
}

/**
 * The rendered changelog with each link pointed somewhere that exists:
 *
 * - a docs page's source, a Markdown file under `docs/content/`, becomes that
 *   page's route, with its fragment kept;
 * - any other file or folder in the repository becomes its address on GitHub,
 *   on `main`;
 * - an address, or a fragment of this page, stays as it is.
 *
 * Anything else, such as a docs page that isn't published or a file that
 * doesn't exist, fails with each line of `source` that holds it.
 */
export async function pointLinks(html: string, source: string, targets: Targets): Promise<string> {
  const problems: string[] = [];
  const pointed = new Map<string, string>();
  for (const [, raw = ""] of html.matchAll(/<a\s[^>]*?href="([^"]*)"/g)) {
    const href = decodeEntities(raw);
    if (pointed.has(href)) continue;
    const target = await pointLink(href, targets);
    if ("problem" in target) {
      const lines = linesOf(source, href);
      const where = lines.length === 0 ? changelogFile : `${changelogFile}:${lines.join(", ")}`;
      problems.push(`${where}: ${href}: ${target.problem}`);
    } else {
      pointed.set(href, target.href);
    }
  }
  if (problems.length > 0) throw new ChangelogLinkError(problems);
  return html.replace(
    /(<a\s[^>]*?href=")([^"]*)(")/g,
    (_, before: string, raw: string, after: string) => {
      const href = pointed.get(decodeEntities(raw));
      return `${before}${href === undefined ? raw : encodeAttribute(href)}${after}`;
    },
  );
}

async function pointLink(
  href: string,
  targets: Targets,
): Promise<{ href: string } | { problem: string }> {
  // An address (`https:`, `mailto:`, `//host`), or a heading on this page.
  if (/^[a-z][a-z\d+.-]*:/i.test(href) || href.startsWith("//") || href.startsWith("#")) {
    return { href };
  }
  const hash = href.indexOf("#");
  const destination = hash === -1 ? href : href.slice(0, hash);
  const fragment = hash === -1 ? "" : href.slice(hash);
  let file: string;
  try {
    file = path.posix.normalize(decodeURIComponent(destination.split("?")[0] ?? ""));
  } catch {
    return { problem: "isn't a path" };
  }
  if (file.startsWith("/") || file === ".." || file.startsWith("../")) {
    return { problem: "is outside the repository: write it relative to the repository's root" };
  }
  if (file.startsWith(`${contentDir}/`) && file.endsWith(".md")) {
    const route = targets.pages.get(file);
    return route === undefined
      ? { problem: `isn't a page the docs publish` }
      : { href: `${route}${fragment}` };
  }
  const kind = await targets.kind(file === "." ? "" : file.replace(/\/$/, ""));
  if (kind === undefined) return { problem: "isn't a file in the repository" };
  const view = kind === "folder" ? "tree" : "blob";
  return { href: `${targets.repo}/${view}/main/${file === "." ? "" : file}${fragment}` };
}

/** Whether a path under `root` is a file, a folder, or nothing. */
export function kindUnder(root: string): Targets["kind"] {
  return async (file) => {
    try {
      // Outside the site: the repository the site is in, which Netlify's clone has.
      const found = await stat(path.join(root, ...file.split("/")));
      return found.isDirectory() ? "folder" : found.isFile() ? "file" : undefined;
    } catch {
      return undefined;
    }
  };
}

/**
 * Each section's heading gets an anchor that holds from release to release: a
 * release's is `v` and its version, its tag on GitHub (`v0.2.0`), and the
 * unreleased section's is `unreleased`. A heading inside one is the release's
 * anchor and its own (`v0.2.0-the-language`), so the same heading in two
 * releases doesn't clash. The unreleased section says it's unreleased.
 */
export function anchorReleases(html: string): string {
  let release = "";
  return html.replace(
    /<h([23])((?:\s[^>]*)?)>([\s\S]*?)<\/h\1>/g,
    (_, level: string, attributes: string, inner: string) => {
      const text = decodeEntities(inner.replace(/<[^>]*>/g, "")).trim();
      const rest = attributes.replace(/\sid="[^"]*"/, "");
      if (level === "2") {
        const unreleased = /\bunreleased\b/i.test(text);
        release = unreleased ? "unreleased" : `v${/^\S+/.exec(text)?.[0] ?? slug(text)}`;
        const heading = `<h2 id="${encodeAttribute(release)}"${rest}>${inner}</h2>`;
        return unreleased
          ? `${heading}\n<p class="unreleased-note">In <code>main</code>, not yet released.</p>`
          : heading;
      }
      const id = release === "" ? slug(text) : `${release}-${slug(text)}`;
      return `<h3 id="${encodeAttribute(id)}"${rest}>${inner}</h3>`;
    },
  );
}

/** The text of the first paragraph: the page's description. */
export function firstParagraph(html: string): string | undefined {
  const inner = /<p(?:\s[^>]*)?>([\s\S]*?)<\/p>/.exec(html)?.[1];
  if (inner === undefined) return undefined;
  return decodeEntities(inner.replace(/<[^>]*>/g, ""))
    .replace(/\s+/g, " ")
    .trim();
}

/** A heading's text as an id: lowercase words joined by `-`. */
function slug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]/gu, "")
    .trim()
    .replace(/\s+/g, "-");
}

/** The lines of the source, from 1, that hold a link's destination. */
function linesOf(source: string, href: string): number[] {
  return source.split("\n").flatMap((line, index) => (line.includes(href) ? [index + 1] : []));
}

function decodeEntities(value: string): string {
  return value
    .replace(/&#(\d+);/g, (_, code: string) => String.fromCodePoint(Number(code)))
    .replace(/&#x([\da-f]+);/gi, (_, code: string) => String.fromCodePoint(parseInt(code, 16)))
    .replaceAll("&quot;", '"')
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

function encodeAttribute(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll('"', "&quot;");
}
