// The changelog page: CHANGELOG.md at the repository's root, with the entries
// in changes/ that no release has gathered yet in its unreleased section,
// rendered by Astro's Markdown, with its links pointed at the site and the
// repository. The files stay where they are, and nothing is copied from them.
import { readFile, stat } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { Loader } from "astro/loaders";
import { z } from "astro/zod";
import {
  changesDir,
  endsSentence,
  readChanges,
  withChanges,
} from "../../scripts/release/changes.ts";
import { contentDir } from "./site.ts";

/** The file the page renders, relative to the repository's root. */
export const changelogFile = "CHANGELOG.md";

/** The collection's one entry. */
export const changelogId = "changelog";

/**
 * A collection with one entry: CHANGELOG.md, read from the repository the
 * site is in, with the entries in `changes/` gathered into it as a release
 * would gather them, rendered with the site's Markdown settings. The links are left
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
      const repository = path.dirname(file);
      const changes = path.join(repository, changesDir);
      const load = async () => {
        const body = withChanges(await readFile(file, "utf8"), readChanges(repository));
        store.clear();
        store.set({
          id: changelogId,
          data: { repository },
          body,
          rendered: await context.renderMarkdown(body),
        });
      };
      await load();
      watcher?.add([file, changes]);
      const reload = (changed: string) => {
        const at = path.resolve(changed);
        if (at !== file && !at.startsWith(`${changes}${path.sep}`)) return;
        logger.info(`${path.relative(repository, at)} changed`);
        void load();
      };
      watcher?.on("change", reload);
      watcher?.on("add", reload);
      watcher?.on("unlink", reload);
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

/** A heading in the changelog's contents: a release, or an area of one. */
export interface ContentsEntry {
  /** Its anchor. */
  id: string;
  /** Its heading, as HTML. */
  html: string;
  /** How many entries it has. */
  count: number;
}

/** A release in the changelog's contents, with its areas. */
export interface ContentsRelease extends ContentsEntry {
  areas: ContentsEntry[];
}

/**
 * The anchored changelog made to scan: each release and each of its areas a
 * `<section>`, each area's heading with its count of entries, and each entry
 * a `<details>` whose `<summary>` is what the entry says first, with the rest
 * under it (decision #382c):
 *
 * - its bold text, when that's a sentence of its own;
 * - otherwise its first sentence: an entry whose bold text is only `Fixed:`,
 *   whose bold text starts a sentence (`<strong>@snippet</strong> puts…`),
 *   or that has none.
 *
 * An entry that's all summary isn't collapsed. An entry that says
 * **Behavior change** is marked, with a badge in its summary: its own words,
 * when the summary starts with them. Also returns the releases and their
 * areas, for the page's contents.
 */
export function collapseEntries(html: string): { html: string; contents: ContentsRelease[] } {
  const contents: ContentsRelease[] = [];
  const out: string[] = [];
  let release:
    { open: string[]; body: string[]; entry: ContentsRelease; behavior: number } | undefined;
  let area: { open: string; body: string[]; entry: ContentsEntry; behavior: number } | undefined;

  const closeArea = (): void => {
    if (release === undefined || area === undefined) return;
    const heading = area.open.replace(
      /<\/h3>$/,
      ` <span class="count">${area.entry.count}</span></h3>`,
    );
    release.body.push(
      `<section class="area" data-behavior="${area.behavior}">\n${heading}\n${area.body.join("\n")}\n</section>`,
    );
    release.entry.areas.push(area.entry);
    release.behavior += area.behavior;
    area = undefined;
  };
  const closeRelease = (): void => {
    closeArea();
    if (release === undefined) return;
    const none = release.behavior === 0 ? ['<p class="no-behavior">No behavior changes.</p>'] : [];
    out.push(
      `<section class="release" data-behavior="${release.behavior}">\n${[...release.open, ...none, ...release.body].join("\n")}\n</section>`,
    );
    contents.push(release.entry);
    release = undefined;
  };

  for (const block of blocks(html)) {
    if (block.tag === "h2") {
      closeRelease();
      const entry = { id: idOf(block.open), html: block.inner, count: 0, areas: [] };
      release = { open: [block.html], body: [], entry, behavior: 0 };
      continue;
    }
    if (release === undefined) {
      out.push(block.html);
      continue;
    }
    if (block.tag === "h3") {
      closeArea();
      area = {
        open: block.html,
        body: [],
        entry: { id: idOf(block.open), html: block.inner, count: 0 },
        behavior: 0,
      };
      continue;
    }
    if (block.tag === "ul") {
      const items = blocks(block.inner).filter((item) => item.tag === "li");
      const entries = items.map((item) => entryOf(item.inner));
      const behavior = entries.filter((entry) => entry.behavior).length;
      const list = `<ul class="entries">\n${entries.map((entry) => entry.html).join("\n")}\n</ul>`;
      release.entry.count += entries.length;
      if (area === undefined) {
        release.body.push(list);
        release.behavior += behavior;
      } else {
        area.body.push(list);
        area.entry.count += entries.length;
        area.behavior += behavior;
      }
      continue;
    }
    if (area === undefined) {
      // What heads a release, such as what it adds, before its first area.
      (release.body.length === 0 ? release.open : release.body).push(block.html);
    } else {
      area.body.push(block.html);
    }
  }
  closeRelease();
  return { html: out.join("\n"), contents };
}

/** One entry of the changelog, from its `<li>`'s inner HTML. */
function entryOf(inner: string): { html: string; behavior: boolean } {
  const behavior = /<strong>Behavior changes?:?\s*/.test(inner);
  const parts = blocks(inner);
  // A loose list's item is paragraphs and blocks; a tight one's is inline
  // HTML, then any blocks.
  const loose = parts[0]?.tag === "p";
  const firstBlock = parts.findIndex((part) => BLOCK.has(part.tag));
  let lead: string;
  let after: string;
  if (loose && parts[0] !== undefined) {
    lead = parts[0].inner;
    after = inner.slice(parts[0].end);
  } else {
    const at = firstBlock === -1 ? inner.length : (parts[firstBlock]?.start ?? inner.length);
    lead = inner.slice(0, at);
    after = inner.slice(at);
  }
  lead = lead.trim();
  const [summary, rest] = splitSummary(lead);
  // The badge is the summary's own words where it starts with them.
  const label = /^<strong>(Behavior changes?:)/;
  const shown = !behavior
    ? summary
    : label.test(summary)
      ? summary.replace(label, '<strong><span class="badge">$1</span>')
      : `<span class="badge">Behavior change</span> ${summary}`;
  const body = [rest.trim() === "" ? "" : `<p>${rest.trim()}</p>`, after.trim()]
    .filter((part) => part !== "")
    .join("\n");
  const kind = behavior ? "entry behavior" : "entry";
  const html =
    body === ""
      ? `<li class="${kind}"><p class="summary">${shown}</p></li>`
      : `<li class="${kind}"><details><summary>${shown}</summary>\n<div class="entry-body">\n${body}\n</div></details></li>`;
  return { html, behavior };
}

/**
 * An entry's first paragraph split into its summary and the rest: its leading
 * `<strong>` when that's a sentence of its own, as `endsSentence` in
 * scripts/release/changes.ts says, or else its first sentence.
 */
function splitSummary(lead: string): [string, string] {
  const first = blocks(lead)[0];
  if (first?.tag === "strong" && first.start === 0 && endsSentence(textOf(first.inner))) {
    return [lead.slice(0, first.end), lead.slice(first.end)];
  }
  const end = sentenceEnd(lead);
  return [lead.slice(0, end), lead.slice(end)];
}

/** Where the first sentence of some inline HTML ends: after a `.`, `!`, or `?` outside any element, before a space. */
function sentenceEnd(html: string): number {
  let depth = 0;
  for (let i = 0; i < html.length; i++) {
    const char = html[i];
    if (char === "<") {
      const close = html.indexOf(">", i);
      if (close === -1) break;
      const tag = html.slice(i, close + 1);
      const name = /^<\/?([a-z][\w-]*)/i.exec(tag)?.[1]?.toLowerCase() ?? "";
      if (tag.startsWith("</")) depth--;
      else if (!VOID.has(name) && !tag.endsWith("/>")) depth++;
      i = close;
      continue;
    }
    if (
      depth === 0 &&
      (char === "." || char === "!" || char === "?") &&
      /^(\s|$)/.test(html.slice(i + 1, i + 2))
    ) {
      return i + 1;
    }
  }
  return html.length;
}

/** Elements that have no closing tag. */
const VOID = new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "source",
  "track",
  "wbr",
]);

/** Elements that end a tight list item's inline HTML. */
const BLOCK = new Set([
  "p",
  "ul",
  "ol",
  "details",
  "pre",
  "div",
  "table",
  "blockquote",
  "figure",
  "h4",
  "h5",
  "h6",
]);

/** An element at the top level of some HTML. */
interface Block {
  tag: string;
  /** Its opening tag. */
  open: string;
  inner: string;
  html: string;
  start: number;
  end: number;
}

/**
 * The elements at the top level of some HTML, as Astro's Markdown writes it:
 * well formed, with `<` escaped in text and code. Text between them is left
 * out.
 */
function blocks(html: string): Block[] {
  const found: Block[] = [];
  const tags = /<!--[\s\S]*?-->|<(\/?)([a-zA-Z][\w-]*)\b[^>]*>/g;
  let depth = 0;
  let start = 0;
  let open = "";
  let openEnd = 0;
  let tag = "";
  for (const match of html.matchAll(tags)) {
    const [whole, closing, rawName] = match;
    if (rawName === undefined) continue;
    const name = rawName.toLowerCase();
    const at = match.index;
    if (closing === "/") {
      depth--;
      if (depth === 0) {
        const end = at + whole.length;
        found.push({
          tag,
          open,
          inner: html.slice(openEnd, at),
          html: html.slice(start, end),
          start,
          end,
        });
      }
      continue;
    }
    if (VOID.has(name) || whole.endsWith("/>")) {
      if (depth === 0) {
        const end = at + whole.length;
        found.push({ tag: name, open: whole, inner: "", html: whole, start: at, end });
      }
      continue;
    }
    if (depth === 0) {
      start = at;
      open = whole;
      openEnd = at + whole.length;
      tag = name;
    }
    depth++;
  }
  return found;
}

/** An opening tag's `id`. */
function idOf(open: string): string {
  return decodeEntities(/\sid="([^"]*)"/.exec(open)?.[1] ?? "");
}

/** Some HTML's text. */
function textOf(html: string): string {
  return decodeEntities(html.replace(/<[^>]*>/g, ""));
}
