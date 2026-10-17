// What the This page, Pages, and Content model views show, worked out from
// plain values so the unit tests can check it without VS Code.

import { pageDetail } from "../preview/reviewText.js";
import type {
  ChangedPage,
  InventoryEntry,
  InventoryFragment,
  InventoryPage,
  InventoryResult,
  LspRange,
  ModelKind,
  PageAbout,
  PageExample,
  PageHealth,
  PageInclude,
  PageMemo,
  PagePlace,
  PageResult,
} from "../shapes.js";

/** How an item of a view looks. */
export interface ItemLook {
  label: string;
  description: string;
  tooltip: string;
  /** A codicon. */
  icon: string;
  /** Whether it's dimmed: an entry nothing uses. */
  dimmed: boolean;
  /** Whether it has children. */
  expandable: boolean;
  /** Whether it starts expanded, when it has children. */
  expanded?: boolean;
}

// Pages.

/** An item of the Pages view. */
export type PagesNode =
  | { kind: "type"; type: string; pages: readonly InventoryPage[] }
  | { kind: "fragments"; fragments: readonly InventoryFragment[] }
  | { kind: "orphans"; pages: readonly InventoryPage[] }
  | { kind: "page"; page: InventoryPage }
  | { kind: "fragment"; fragment: InventoryFragment }
  | { kind: "includer"; path: string };

/** The name of the group of pages with no one content type. */
const NO_TYPE = "no type";

/** The Pages view's top level: a group per content type, then fragments, then orphans. */
export function pagesRoots(inventory: InventoryResult): PagesNode[] {
  const byType = new Map<string, InventoryPage[]>();
  for (const page of inventory.pages) {
    const type = page.type ?? NO_TYPE;
    const list = byType.get(type) ?? [];
    list.push(page);
    byType.set(type, list);
  }
  const types = [...byType.keys()].sort((a, b) =>
    a === NO_TYPE ? 1 : b === NO_TYPE ? -1 : a.localeCompare(b),
  );
  const nodes: PagesNode[] = types.map((type) => ({
    kind: "type",
    type,
    pages: byType.get(type) ?? [],
  }));
  if (inventory.fragments.length > 0) {
    nodes.push({ kind: "fragments", fragments: inventory.fragments });
  }
  const orphaned = new Set(inventory.orphans);
  const orphans = inventory.pages.filter((p) => orphaned.has(p.path));
  if (orphans.length > 0) nodes.push({ kind: "orphans", pages: orphans });
  return nodes;
}

/** An item's children in the Pages view. */
export function pagesChildren(node: PagesNode): PagesNode[] {
  switch (node.kind) {
    case "type":
    case "orphans":
      return node.pages.map((page) => ({ kind: "page", page }));
    case "fragments":
      return node.fragments.map((fragment) => ({ kind: "fragment", fragment }));
    case "fragment":
      return node.fragment.includedBy.map((path) => ({ kind: "includer", path }));
    case "page":
    case "includer":
      return [];
  }
}

/** The content path of the file an item opens, if it opens one. */
export function pagesPath(node: PagesNode): string | undefined {
  switch (node.kind) {
    case "page":
      return node.page.path;
    case "fragment":
      return node.fragment.path;
    case "includer":
      return node.path;
    default:
      return undefined;
  }
}

/** How an item of the Pages view looks: a page by its title, with its path. */
export function pagesLook(node: PagesNode): ItemLook {
  switch (node.kind) {
    case "type":
      return group(node.type, node.pages.length, "file", `Pages of type ${node.type}`);
    case "fragments":
      return group(
        "Fragments",
        node.fragments.length,
        "file-symlink-file",
        "Files that exist to be included",
      );
    case "orphans":
      return group(
        "Orphans",
        node.pages.length,
        "warning",
        "Pages no other page links to or includes. Without a navigation file a reader may still reach them, so this is a hint, not an error.",
      );
    case "page":
      return {
        label: node.page.title ?? basename(node.page.path),
        description: node.page.path,
        tooltip: `${node.page.path}\n${count(node.page.incoming, "link or include", "links and includes")} from other files`,
        icon: "file",
        dimmed: false,
        expandable: false,
      };
    case "fragment":
      return {
        label: node.fragment.path,
        description:
          node.fragment.includedBy.length === 0
            ? "not included"
            : count(node.fragment.includedBy.length, "file", "files"),
        tooltip: `${node.fragment.path}\nIncluded by ${count(node.fragment.includedBy.length, "file", "files")}`,
        icon: "file-symlink-file",
        dimmed: node.fragment.includedBy.length === 0,
        expandable: node.fragment.includedBy.length > 0,
      };
    case "includer":
      return {
        label: node.path,
        description: "includes it",
        tooltip: node.path,
        icon: "references",
        dimmed: false,
        expandable: false,
      };
  }
}

// Content model.

/** An item of the Content model view. */
export type ModelNode =
  | { kind: "kind"; modelKind: ModelKind; entries: readonly InventoryEntry[] }
  | { kind: "entry"; entry: InventoryEntry };

/** Each kind's name, codicon, and order. */
const KINDS: readonly { kind: ModelKind; name: string; icon: string }[] = [
  { kind: "phrase", name: "Phrases", icon: "symbol-string" },
  { kind: "feature", name: "Features", icon: "tag" },
  { kind: "term", name: "Glossary", icon: "symbol-key" },
  { kind: "dimension", name: "Dimensions", icon: "symbol-enum" },
  { kind: "note", name: "Note types", icon: "note" },
  { kind: "widget", name: "Widgets", icon: "symbol-class" },
  { kind: "build", name: "Builds", icon: "package" },
];

/** The Content model view's top level: a node per kind the model has, in a fixed order. */
export function modelRoots(inventory: InventoryResult): ModelNode[] {
  return KINDS.flatMap(({ kind }) => {
    const entries = inventory.model.filter((e) => e.kind === kind);
    return entries.length > 0 ? [{ kind: "kind" as const, modelKind: kind, entries }] : [];
  });
}

/** An item's children in the Content model view. */
export function modelChildren(node: ModelNode): ModelNode[] {
  return node.kind === "kind" ? node.entries.map((entry) => ({ kind: "entry", entry })) : [];
}

/** How an item of the Content model view looks. An entry nothing uses is marked and dimmed. */
export function modelLook(node: ModelNode): ItemLook {
  if (node.kind === "kind") {
    const kind = KINDS.find((k) => k.kind === node.modelKind);
    const unused = node.entries.filter((e) => e.uses === 0).length;
    return {
      ...group(kind?.name ?? node.modelKind, node.entries.length, kind?.icon ?? "symbol-misc", ""),
      tooltip: `${count(node.entries.length, "entry", "entries")}${unused > 0 ? `, ${unused} unused` : ""}`,
    };
  }
  const { entry } = node;
  const icon = KINDS.find((k) => k.kind === entry.kind)?.icon ?? "symbol-misc";
  const unused = entry.uses === 0;
  const uses = entry.uses === null ? undefined : count(entry.uses, "use", "uses");
  const tooltip = [entry.key];
  if (entry.label !== null) tooltip.push(entry.label);
  if (uses !== undefined) tooltip.push(unused ? "Unused: nothing uses it" : `${uses} in pages`);
  if (entry.uses === null && entry.kind === "term") {
    tooltip.push("Marked: the links to its page are its uses");
  }
  if (entry.declaration === null) {
    tooltip.push(entry.kind === "note" ? "Built in" : "Not found in ascribe.toml");
  }
  return {
    label: entry.key,
    description: unused ? "unused" : (uses ?? ""),
    tooltip: tooltip.join("\n"),
    icon,
    dimmed: unused,
    expandable: false,
  };
}

// This page.

/** A link to or an include of the active page, or of the heading at the cursor. */
export type Place = PagePlace;

/** An item of the This page view. */
export type ThisPageNode =
  | { kind: "about"; rows: readonly AboutRow[] }
  | { kind: "health"; rows: readonly AboutRow[] }
  | { kind: "review"; review: ThisPageReview }
  | { kind: "threads"; threads: readonly PageThread[] }
  | { kind: "thread"; thread: PageThread }
  | {
      kind: "connections";
      heading: string | undefined;
      linked: number;
      included: number;
      groups: readonly ThisPageNode[];
    }
  | { kind: "memos"; heading: string | undefined; memos: readonly PageMemo[] }
  | { kind: "memo"; memo: PageMemo }
  | { kind: "group"; group: "links" | "includes"; files: readonly PlacesInFile[] }
  | { kind: "file"; file: PlacesInFile }
  | { kind: "place"; place: Place }
  | { kind: "included"; includes: readonly PageInclude[] }
  | { kind: "include"; include: PageInclude }
  | { kind: "row"; row: AboutRow };

/** A row of About or Health: a label and what it says, and where it goes. */
interface AboutRow {
  label: string;
  value: string;
  tooltip: string;
  /** Whether it says something needs doing: overdue, too long, stale. */
  warn?: boolean | undefined;
  /** What choosing it does, if anything. */
  opens?: RowTarget | undefined;
}

/** Where a row goes: a place in a file, or a command. */
type RowTarget =
  | { kind: "place"; uri: string; range: LspRange }
  | { kind: "command"; command: string; title: string };

/** VS Code's diagnostics for the page, by how loudly they speak. */
export interface ProblemCounts {
  errors: number;
  warnings: number;
  advice: number;
}

/** What This page knows besides the server's answer. */
export interface ThisPageContext {
  /** The page's `file:` URI, which rows in its frontmatter open. */
  uri?: string | undefined;
  /** Its problems, as the Problems panel counts them. */
  problems?: ProblemCounts | undefined;
}

/** The command a Builds row runs: the build picker. */
export const SWITCH_BUILD = "ascribe.switchBuild";

/** The command a Template row runs. */
export const NEXT_PLACEHOLDER = "ascribe.nextPlaceholder";

/** The command a Problems row runs. */
export const SHOW_PROBLEMS = "workbench.action.problems.focus";

/** What review says about the page, while review is on for its project. */
export interface ThisPageReview {
  /** The base's name, as the Review view says it. */
  base: string;
  /** The page as the branch changes it; `null` when the branch doesn't change it. */
  page: Omit<ChangedPage, "changes"> | null;
  /** Its open threads; `undefined` when the pull request's threads aren't known. */
  threads: readonly PageThread[] | undefined;
}

/** An open thread that shows on the page: on its own file, or on a file it includes. */
export interface PageThread {
  id: string;
  /** The content path of the file it's on. */
  path: string;
  /** Its first line, from 1; `undefined` for a thread on the whole file, or one that's detached. */
  line: number | undefined;
  /** Its first comment's author's login, when GitHub reports one. */
  author: string | undefined;
  /** Its first comment's first line. */
  excerpt: string;
}

/** The places in one file. */
interface PlacesInFile {
  uri: string;
  path: string;
  places: readonly Place[];
}

/**
 * The This page view's top level: About, then Review while review is on,
 * then Health, Connections, and Memos; nothing for a file that isn't a page.
 */
export function thisPageRoots(
  page: PageResult,
  review?: ThisPageReview,
  context: ThisPageContext = {},
): ThisPageNode[] {
  const { about } = page;
  if (!about) return [];
  const rows: AboutRow[] = [
    { label: "Title", value: about.title ?? basename(about.path), tooltip: about.path },
    {
      label: "Type",
      value: about.fragment ? "fragment" : (about.type ?? "none"),
      tooltip: about.fragment
        ? "A fragment exists to be included, and is published only where it is"
        : about.type === null
          ? "No one content type applies to it"
          : `Content type ${about.type}`,
    },
    about.fragment
      ? {
          label: "Address",
          value: "only where it's included",
          tooltip: "A fragment has no address of its own",
        }
      : { label: "Address", value: about.address ?? "", tooltip: "Its address on the site" },
  ];
  if (!about.fragment) {
    rows.push(
      about.markdownAddress === null
        ? {
            label: "Markdown",
            value: "not published",
            tooltip: "The project publishes no Markdown versions ([consumer] agents)",
          }
        : {
            label: "Markdown",
            value: about.markdownAddress,
            tooltip: "Its Markdown version's address, which agents read",
          },
    );
  }
  rows.push(...aboutRows(about, context.uri));
  const links = byFile(page.linkedFrom);
  const includes = byFile(page.includedBy);
  return [
    { kind: "about", rows },
    ...(review ? [{ kind: "review" as const, review }] : []),
    { kind: "health", rows: healthRows(page.health, context) },
    {
      kind: "connections",
      heading: page.heading?.text,
      linked: page.linkedFrom.length,
      included: page.includedBy.length,
      groups: [
        { kind: "group", group: "links", files: links },
        { kind: "group", group: "includes", files: includes },
        { kind: "included", includes: page.includes },
      ],
    },
    { kind: "memos", heading: page.heading?.text, memos: page.memos },
  ];
}

/** About's rows past what the page is: each only when it has something to say. */
function aboutRows(about: PageAbout, uri: string | undefined): AboutRow[] {
  const rows: AboutRow[] = [];
  const at = (range: LspRange | null): RowTarget | undefined =>
    uri && range ? { kind: "place", uri, range } : undefined;
  const { description, review, listing, template } = about;
  if (description) {
    const { length, limit } = description;
    const long = description.reported;
    rows.push({
      label: "Description",
      value:
        length === null
          ? "missing"
          : limit === null
            ? count(length, "character", "characters")
            : `${length} of ${limit} characters${long ? ", too long" : ""}`,
      tooltip:
        length === null
          ? `No \`${description.field}\`: write a sentence saying what the page is for`
          : limit === null
            ? `Its \`${description.field}\`, which search results show with its title`
            : `Its \`${description.field}\`, its line in llms.txt, reads best under ${limit} characters`,
      warn: length === null || long,
      opens: at(description.range),
    });
  }
  if (review) {
    rows.push({
      label: "Review",
      value: reviewText(review.date, review.days, review.overdue),
      tooltip: `\`${review.field}\`: ${review.date}`,
      warn: review.reported,
      opens: at(review.range),
    });
  }
  if (about.availability !== null) {
    rows.push({
      label: "Availability",
      value: about.availability,
      tooltip: "Its own `available`: where and since when it applies",
    });
  }
  if (about.builds.length > 0) {
    const published = about.builds.filter((b) => b.leftOut === null).map((b) => b.name);
    const left = about.builds.filter((b) => b.leftOut !== null);
    const parts = [
      published.length > 0 ? `published by ${published.join(", ")}` : "",
      left.length > 0 ? `left out by ${left.map((b) => b.name).join(", ")}` : "",
    ].filter((part) => part !== "");
    rows.push({
      label: "Builds",
      value: capitalized(parts.join("; ")),
      tooltip: [
        ...left.map(
          (b) =>
            `${b.name} leaves it out: ${b.leftOut === "variant" ? "its `variant` names none of the build's values" : "its `available` makes it unavailable there"}`,
        ),
        "Choose a build to see the page as it publishes it",
      ].join("\n"),
      opens: { kind: "command", command: SWITCH_BUILD, title: "Switch Build" },
    });
  }
  if (listing) {
    const rule = listing.rule === null ? "" : `\n${listing.rule}`;
    rows.push({
      label: "Listing",
      value: `${capitalized(listing.listing)}${listingFrom(listing.from)}`,
      tooltip: `In build ${listing.build}${rule}`,
      opens:
        listing.uri && listing.range
          ? { kind: "place", uri: listing.uri, range: listing.range }
          : undefined,
    });
  }
  for (const permalink of about.permalinks) {
    rows.push({
      label: "Permalink",
      value: permalink.name,
      tooltip:
        permalink.heading === null
          ? "The page's permalink: an address that keeps leading here"
          : `A permalink to “${permalink.heading}”: an address that keeps leading here`,
      opens: at(permalink.range),
    });
  }
  if (template) {
    rows.push({
      label: "Template",
      value: `${template.type} template, ${count(template.left, "placeholder", "placeholders")} left`,
      tooltip: "Goes to the next place the template left to fill in",
      warn: true,
      opens: { kind: "command", command: NEXT_PLACEHOLDER, title: "Go to Next Placeholder" },
    });
  }
  return rows;
}

/** "overdue since 3 October", "due in 12 days", "due today". */
function reviewText(date: string, days: number | null, overdue: boolean): string {
  const [year, month, day] = date.split("-").map(Number);
  const months = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  const thisYear =
    days === null
      ? undefined
      : new Date(Date.UTC(year ?? 0, (month ?? 1) - 1, (day ?? 1) - days)).getUTCFullYear();
  const named = `${day} ${months[(month ?? 1) - 1] ?? ""}${thisYear === year ? "" : ` ${year}`}`;
  if (overdue) return `overdue since ${named}`;
  if (days === null) return `due ${named}`;
  if (days === 0) return "due today";
  return `due in ${count(days, "day", "days")}`;
}

/** Where a listing comes from, as the end of its row. */
function listingFrom(from: string): string {
  switch (from) {
    case "default":
      return " (the default)";
    case "page":
      return ", by its own `listing`";
    case "site":
      return ", by `[listing] default`";
    case "older-lines":
      return ", by `[listing] older-lines`";
    case "folder":
      return ", by `[listing.folders]`";
    case "type":
      return ", by `[listing.types]`";
    default:
      return "";
  }
}

/** Health's rows: the problems, the size, and each tested example. */
function healthRows(health: PageHealth, { uri, problems }: ThisPageContext): AboutRow[] {
  const rows: AboutRow[] = [];
  if (problems) {
    const parts = [
      problems.errors > 0 ? count(problems.errors, "error", "errors") : "",
      problems.warnings > 0 ? count(problems.warnings, "warning", "warnings") : "",
      problems.advice > 0 ? `${problems.advice} advice` : "",
    ].filter((part) => part !== "");
    rows.push({
      label: "Problems",
      value: parts.length > 0 ? parts.join(", ") : "none",
      tooltip: "Opens the Problems panel, where Show Active File Only keeps it to this page",
      warn: problems.errors > 0 || problems.warnings > 0,
      opens: { kind: "command", command: SHOW_PROBLEMS, title: "Show Problems" },
    });
  }
  const { size } = health;
  if (size) {
    const over = size.limit !== null && size.characters >= size.limit;
    rows.push({
      label: "Size",
      value: `${thousands(size.characters)}${size.limit === null ? "" : ` of ${thousands(size.limit)}`} characters, in ${size.build}`,
      tooltip: [
        `Its Markdown in build ${size.build}, the largest of the builds that publish it`,
        size.limit === null
          ? "page-size is off"
          : `page-size reports a page at ${thousands(size.limit)} characters`,
      ].join("\n"),
      warn: over,
    });
  }
  for (const example of health.examples) {
    rows.push({
      label: "Example",
      value: `${example.address}, ${exampleState(example)}`,
      tooltip: example.tested ?? exampleState(example),
      warn: example.state !== "tested",
      opens: uri ? { kind: "place", uri, range: example.range } : undefined,
    });
  }
  return rows;
}

function exampleState(example: PageExample): string {
  switch (example.state) {
    case "tested":
      return "tested";
    case "stale":
      return "changed since it was tested";
    case "untested":
      return "not tested";
  }
}

function byFile(places: readonly Place[]): PlacesInFile[] {
  const files = new Map<string, PlacesInFile & { places: Place[] }>();
  for (const place of places) {
    const file = files.get(place.uri) ?? { uri: place.uri, path: place.path, places: [] };
    file.places.push(place);
    files.set(place.uri, file);
  }
  // The server lists places file by file in content path order.
  return [...files.values()];
}

/** An item's children in the This page view. */
export function thisPageChildren(node: ThisPageNode): ThisPageNode[] {
  switch (node.kind) {
    case "about":
    case "health":
      return node.rows.map((row) => ({ kind: "row", row }));
    case "included":
      return node.includes.map((include) => ({ kind: "include", include }));
    case "review":
      return reviewRows(node.review);
    case "threads":
      return node.threads.map((thread) => ({ kind: "thread", thread }));
    case "connections":
      return [...node.groups];
    case "memos":
      return node.memos.map((memo) => ({ kind: "memo", memo }));
    case "group":
      return node.files.map((file) => ({ kind: "file", file }));
    case "file":
      return node.file.places.map((place) => ({ kind: "place", place }));
    case "place":
    case "include":
    case "row":
    case "thread":
    case "memo":
      return [];
  }
}

/** Review's rows: what the branch changed on the page, then its open threads when they're known. */
function reviewRows({ page, threads }: ThisPageReview): ThisPageNode[] {
  const rows: ThisPageNode[] = [
    {
      kind: "row",
      row: page
        ? {
            label: page.status === "added" ? "Added" : "Changed",
            value: pageDetail(page),
            tooltip: "What this branch changes on the page",
          }
        : {
            label: "Unchanged",
            value: "this branch doesn't change it",
            tooltip: "Neither the page nor a file it includes changed on this branch",
          },
    },
  ];
  if (threads) rows.push({ kind: "threads", threads });
  return rows;
}

/** How an item of the This page view looks. */
export function thisPageLook(node: ThisPageNode): ItemLook {
  switch (node.kind) {
    case "about":
      return { ...section("About", "", "info", "What the page is"), expanded: true };
    case "health": {
      const warned = node.rows.filter((row) => row.warn).length;
      return {
        ...section(
          "Health",
          warned === 0 ? "" : count(warned, "thing to do", "things to do"),
          "pulse",
          "Its problems, its size, and its tested examples",
        ),
        expandable: node.rows.length > 0,
        dimmed: node.rows.length === 0,
        expanded: true,
      };
    }
    case "included": {
      const size = node.includes.length;
      return {
        ...group("Includes", size, "file-submodule", "The fragments and code files it shows"),
        dimmed: size === 0,
        expandable: size > 0,
      };
    }
    case "include": {
      const { include } = node;
      const snippet = include.kind === "snippet";
      return {
        label: include.written,
        description: include.path ?? "not found",
        tooltip: `${snippet ? "@snippet" : "@include"}: ${include.written}, line ${include.range.start.line + 1}`,
        icon: snippet ? "code" : "file-symlink-file",
        dimmed: include.uri === null,
        expandable: false,
      };
    }
    case "review":
      return {
        ...section(
          "Review",
          `against ${node.review.base}`,
          "git-pull-request",
          `This branch's changes to the page, against ${node.review.base}, and its open threads`,
        ),
        expanded: true,
      };
    case "threads": {
      const size = node.threads.length;
      return {
        ...group(
          "Open threads",
          size,
          "comment-discussion",
          `${count(size, "open thread", "open threads")} on the page, or on a file it includes`,
        ),
        dimmed: size === 0,
        expandable: size > 0,
        expanded: true,
      };
    }
    case "thread": {
      const { thread } = node;
      const where = thread.line === undefined ? thread.path : `${thread.path}:${thread.line}`;
      const by = thread.author === undefined ? "" : `${thread.author} · `;
      return {
        label: thread.excerpt,
        description: `${by}${where}`,
        tooltip: `${thread.excerpt}\n${by}${where}\nOpens the thread in the preview`,
        icon: "comment",
        dimmed: false,
        expandable: false,
      };
    }
    case "connections": {
      const to = node.heading === undefined ? "to the page" : `to “${node.heading}”`;
      return {
        ...section(
          "Connections",
          to,
          "references",
          `${count(node.linked, "link", "links")} and ${count(node.included, "include", "includes")} ${to}`,
        ),
        expanded: true,
      };
    }
    case "memos": {
      const about = node.heading === undefined ? "about the page" : `about “${node.heading}”`;
      const size = node.memos.length;
      return {
        ...section(
          "Memos",
          size === 0 ? "none" : about,
          "note",
          `${count(size, "memo", "memos")} ${about}: why it's written the way it is`,
        ),
        dimmed: size === 0,
        expandable: size > 0,
        expanded: true,
      };
    }
    case "memo": {
      const { memo } = node;
      const notes = [memo.personal ? "personal" : "", memo.state === "outdated" ? "outdated" : ""]
        .filter((note) => note !== "")
        .join(", ");
      return {
        label: memo.firstLine || basename(memo.path),
        description: notes === "" ? memo.path : `${memo.path} · ${notes}`,
        tooltip:
          memo.state === "outdated"
            ? `${memo.path}\nIts quote isn't on the page any more: read it against the page`
            : memo.path,
        icon: memo.personal ? "lock" : "note",
        dimmed: false,
        expandable: false,
      };
    }
    case "group": {
      const places = node.files.reduce((n, f) => n + f.places.length, 0);
      const look =
        node.group === "links"
          ? group("Linked from", places, "link", count(node.files.length, "file", "files"))
          : group(
              "Included by",
              places,
              "file-symlink-file",
              count(node.files.length, "file", "files"),
            );
      return { ...look, dimmed: places === 0, expandable: places > 0 };
    }
    case "file":
      return {
        label: basename(node.file.path),
        description: node.file.path,
        tooltip: node.file.path,
        icon: "file",
        dimmed: false,
        expandable: true,
      };
    case "place":
      return {
        label: node.place.text,
        description: `line ${node.place.range.start.line + 1}`,
        tooltip: `${node.place.path}:${node.place.range.start.line + 1}`,
        icon: "references",
        dimmed: false,
        expandable: false,
      };
    case "row":
      return {
        label: node.row.label,
        description: node.row.value,
        tooltip: node.row.tooltip,
        icon: node.row.warn ? "warning" : "blank",
        dimmed: false,
        expandable: false,
      };
  }
}

// Shared.

function section(label: string, description: string, icon: string, tooltip: string): ItemLook {
  return { label, description, tooltip, icon, dimmed: false, expandable: true };
}

function group(label: string, size: number, icon: string, tooltip: string): ItemLook {
  return { label, description: String(size), tooltip, icon, dimmed: false, expandable: true };
}

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

function capitalized(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** A number with commas between its thousands, as `page-size` writes it. */
function thousands(n: number): string {
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

function basename(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}
