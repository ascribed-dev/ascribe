// What the This page, Pages, and Content model views show, worked out from
// plain values so the unit tests can check it without VS Code.

import * as path from "node:path";
import { pageDetail } from "../preview/reviewText.js";
import { comparable, within } from "../projects.js";
import type {
  ChangedPage,
  InventoryEntry,
  InventoryFragment,
  InventoryPage,
  InventoryResult,
  InventorySection,
  InventoryValue,
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
  /** Its context value, which its menu's commands are shown by. */
  context?: string | undefined;
}

// Pages.

/** An item of the Pages view: the navigation's sections, then the fragments by folder. */
export type PagesNode =
  | {
      kind: "section";
      section: InventorySection;
      /** The section it's nested in; `undefined` for the root. */
      within: InventorySection | undefined;
      /** Whether it sits among its parent's pages, where its page beside the folder sits. */
      placed: boolean;
      parent: PagesNode | undefined;
      children: PagesNode[];
    }
  | {
      kind: "page";
      path: string;
      page: InventoryPage | undefined;
      /** The section it's a page of. */
      section: InventorySection;
      /** Whether it has its place in `ascribe.nav`: one that isn't is marked. */
      placed: boolean;
      /** Whether it's its section's own page, which is first and doesn't move. */
      own: boolean;
      parent: PagesNode;
      children: PagesNode[];
    }
  | { kind: "fragments"; parent: undefined; children: PagesNode[] }
  | { kind: "folder"; folder: string; parent: PagesNode; children: PagesNode[] }
  | { kind: "fragment"; fragment: InventoryFragment; parent: PagesNode; children: PagesNode[] }
  | { kind: "includer"; path: string; parent: PagesNode; children: PagesNode[] };

/**
 * The Pages view's top level: the root section, Start, with its own page
 * and its pages; each section in it, in the navigation's order; then the
 * fragments by folder. Every node knows its parent and its children. With
 * `orphansOnly`, only the pages nothing links to or includes, and the
 * fragments nothing includes, in the sections and folders they're in.
 */
export function pagesRoots(inventory: InventoryResult, orphansOnly = false): PagesNode[] {
  const roots = allPages(inventory);
  if (!orphansOnly) return roots;
  const orphans = new Set(inventory.orphans);
  return roots.flatMap((node) => {
    const kept = keptOf(node, undefined, orphans);
    return kept ? [kept] : [];
  });
}

/**
 * A copy of `node` with only its orphans under it, under `parent`; nothing
 * when none is. A section's own page is an index page, or a page beside its
 * folder, which an orphan can be.
 */
function keptOf(
  node: PagesNode,
  parent: PagesNode | undefined,
  orphans: ReadonlySet<string>,
): PagesNode | undefined {
  switch (node.kind) {
    case "page":
      return orphans.has(node.path) && parent ? { ...node, parent, children: [] } : undefined;
    case "fragment":
      return node.fragment.includedBy.length === 0 && parent
        ? { ...node, parent, children: [] }
        : undefined;
    case "includer":
      return undefined;
    default: {
      const copy = { ...node, parent, children: [] } as PagesNode;
      copy.children = node.children.flatMap((child) => {
        const kept = keptOf(child, copy, orphans);
        return kept ? [kept] : [];
      });
      return copy.children.length > 0 ? copy : undefined;
    }
  }
}

function allPages(inventory: InventoryResult): PagesNode[] {
  const pages = new Map(inventory.pages.map((page) => [page.path, page]));
  const roots: PagesNode[] = [];
  const root = inventory.navigation;
  if (root) {
    const start = sectionNode(root, undefined, false, undefined, pages);
    roots.push(start);
    // The root's sections that don't sit among its pages come after it.
    const among = new Set(root.pages.map((page) => page.path));
    for (const nested of root.sections) {
      if (nested.page === null || !among.has(nested.page)) {
        roots.push(sectionNode(nested, root, false, undefined, pages));
      }
    }
  }
  if (inventory.fragments.length > 0) {
    const fragments: PagesNode = { kind: "fragments", parent: undefined, children: [] };
    const byFolder = new Map<string, InventoryFragment[]>();
    for (const fragment of inventory.fragments) {
      const folder = fragment.path.slice(0, fragment.path.lastIndexOf("/") + 1);
      byFolder.set(folder, [...(byFolder.get(folder) ?? []), fragment]);
    }
    for (const [folder, list] of byFolder) {
      const node: PagesNode = { kind: "folder", folder, parent: fragments, children: [] };
      for (const fragment of list) {
        const item: PagesNode = { kind: "fragment", fragment, parent: node, children: [] };
        item.children = fragment.includedBy.map((path) => ({
          kind: "includer",
          path,
          parent: item,
          children: [],
        }));
        node.children.push(item);
      }
      fragments.children.push(node);
    }
    roots.push(fragments);
  }
  return roots;
}

function sectionNode(
  section: InventorySection,
  within: InventorySection | undefined,
  placed: boolean,
  parent: PagesNode | undefined,
  pages: ReadonlyMap<string, InventoryPage>,
): PagesNode {
  const node: PagesNode = { kind: "section", section, within, placed, parent, children: [] };
  const page = (path: string, isPlaced: boolean, own: boolean): PagesNode => ({
    kind: "page",
    path,
    page: pages.get(path),
    section,
    placed: isPlaced,
    own,
    parent: node,
    children: [],
  });
  if (section.page !== null) node.children.push(page(section.page, true, true));
  const sits = new Map<string, InventorySection>();
  for (const nested of section.sections) {
    if (nested.page !== null) sits.set(nested.page, nested);
  }
  const among = new Set<InventorySection>();
  for (const entry of section.pages) {
    const nested = sits.get(entry.path);
    if (nested) {
      among.add(nested);
      node.children.push(sectionNode(nested, section, true, node, pages));
    } else {
      node.children.push(page(entry.path, entry.placed, false));
    }
  }
  // The root's other sections are on the view's top level.
  if (within !== undefined || parent !== undefined) {
    for (const nested of section.sections) {
      if (!among.has(nested)) node.children.push(sectionNode(nested, section, false, node, pages));
    }
  }
  return node;
}

/** An item's children in the Pages view. */
export function pagesChildren(node: PagesNode): PagesNode[] {
  return node.children;
}

/** An item's parent in the Pages view, which revealing a page walks up. */
export function pagesParent(node: PagesNode): PagesNode | undefined {
  return node.parent;
}

/** The node of a page or a fragment, by its content path: where it's first in the view. */
export function pagesFind(roots: readonly PagesNode[], path: string): PagesNode | undefined {
  for (const node of roots) {
    if (
      (node.kind === "page" && node.path === path) ||
      (node.kind === "fragment" && node.fragment.path === path)
    ) {
      return node;
    }
    const found = pagesFind(node.children, path);
    if (found) return found;
  }
  return undefined;
}

/**
 * The content path of the page or fragment an item is, which VS Code
 * decorates as the Explorer does: its problems' color, for one. An item
 * that only opens a file, as an includer does, isn't that file.
 */
export function pagesResource(node: PagesNode): string | undefined {
  if (node.kind === "page") return node.path;
  if (node.kind === "fragment") return node.fragment.path;
  return undefined;
}

/** The content path of the file an item opens, if it opens one. */
export function pagesPath(node: PagesNode): string | undefined {
  switch (node.kind) {
    case "page":
      return node.path;
    case "fragment":
      return node.fragment.path;
    case "includer":
      return node.path;
    default:
      return undefined;
  }
}

/**
 * The content path of `file`, a file system path, in the content root
 * `root`, or `undefined` outside it. Windows paths compare without case, as
 * the server writes the drive in upper case and VS Code in lower; the path
 * keeps the file's own case.
 */
export function contentPathOf(file: string, root: string): string | undefined {
  if (!within(file, root) || comparable(file) === comparable(root)) return undefined;
  // `comparable` folds a Windows path's case, so the path is cut from the
  // file's own, normalised the same way but in its case.
  const normal = /^[a-zA-Z]:|^\\\\/.test(file)
    ? path.win32.normalize(file).replace(/\\/g, "/")
    : path.posix.normalize(file);
  return normal.slice(comparable(root).length + 1);
}

/** The context value of an item, which its menu's commands are shown by. */
export const PAGES_CONTEXT = {
  section: "ascribe.pages.section",
  page: "ascribe.pages.page",
  /** A section's own page, which doesn't move among its pages. */
  own: "ascribe.pages.own",
  unplaced: "ascribe.pages.unplaced",
  fragment: "ascribe.pages.fragment",
} as const;

/**
 * Whether a page is left out of the build the status bar shows: its
 * `builds`, which list a versioned build's lines (`site@3.4`) too, don't
 * have it. Nothing is left out while no build is known.
 */
export function leftOut(page: InventoryPage | undefined, build: string | undefined): boolean {
  return page !== undefined && build !== undefined && !page.builds.includes(build);
}

/**
 * How an item of the Pages view looks: a section by its label and folder, a
 * page by its title. A page the build the status bar shows, `build`, leaves
 * out is dimmed, and says so.
 */
export function pagesLook(node: PagesNode, build?: string): ItemLook {
  switch (node.kind) {
    case "section": {
      const { section } = node;
      const root = section.folder === "";
      return {
        label: section.label,
        description: root ? "" : section.folder,
        tooltip: [
          root ? "The content root's section" : `The section of ${section.folder}`,
          section.listed
            ? "Listed in ascribe.nav"
            : "In its default place: ascribe.nav doesn't list it",
          ...(section.listsPages ? [] : ["Its pages are in the default order"]),
        ].join("\n"),
        icon: root ? "home" : "folder",
        dimmed: false,
        expandable: node.children.length > 0,
        expanded: !node.placed,
        context: PAGES_CONTEXT.section,
      };
    }
    case "fragments":
      return group(
        "Fragments",
        node.children.reduce((n, folder) => n + folder.children.length, 0),
        "file-symlink-file",
        "Files that exist to be included",
      );
    case "folder":
      return group(
        node.folder === "" ? "the content root" : node.folder,
        node.children.length,
        "folder",
        `The fragments in ${node.folder === "" ? "the content root" : node.folder}`,
      );
    case "page": {
      const { path } = node;
      const incoming = node.page?.incoming ?? 0;
      const out = leftOut(node.page, build);
      return {
        label: node.page?.title ?? basename(path),
        description: out ? `Not in ${build}` : node.placed ? path : "not placed",
        tooltip: [
          path,
          `${count(incoming, "link or include", "links and includes")} from other files`,
          ...(node.own ? ["The section's own page, which comes first"] : []),
          ...(node.placed
            ? []
            : [
                `ascribe.nav lists ${node.section.folder || "the root"}'s pages and leaves it out, so it comes after them. Place Here appends it.`,
              ]),
          ...(out ? [`The build ${build}, which the status bar shows, leaves it out`] : []),
        ].join("\n"),
        icon: node.placed ? "file" : "warning",
        dimmed: out,
        expandable: false,
        context: node.own
          ? PAGES_CONTEXT.own
          : node.placed
            ? PAGES_CONTEXT.page
            : PAGES_CONTEXT.unplaced,
      };
    }
    case "fragment":
      return {
        label: basename(node.fragment.path),
        description:
          node.fragment.includedBy.length === 0
            ? "not included"
            : count(node.fragment.includedBy.length, "file", "files"),
        tooltip: `${node.fragment.path}\nIncluded by ${count(node.fragment.includedBy.length, "file", "files")}`,
        icon: "file-symlink-file",
        dimmed: node.fragment.includedBy.length === 0,
        expandable: node.fragment.includedBy.length > 0,
        context: PAGES_CONTEXT.fragment,
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

/** Why `to` can't be the new path of the file at `from`, or `undefined` when it can. */
export function newPathProblem(from: string, to: string): string | undefined {
  const path = to.trim();
  if (path === from) return "That's its path now: enter a new one.";
  if (!path.endsWith(".md")) return "A page's or fragment's path ends in .md.";
  if (
    path.startsWith("/") ||
    path.split("/").some((part) => part === "" || part === ".." || part === ".")
  ) {
    return "Enter a path from the content root, such as guides/install.md.";
  }
  return undefined;
}

/** Whether an item can be dragged: any section but the root, and any page but a section's own. */
export function pagesDraggable(node: PagesNode): boolean {
  if (node.kind === "section") return node.section.folder !== "";
  return node.kind === "page" && !node.own;
}

/** What dropping a dragged item on another does. */
export type PagesDrop =
  /** An `ascribe/edit` operation on `ascribe.nav`. */
  | { kind: "edit"; action: "moveSection" | "movePage"; args: Record<string, string | null> }
  /** A page's file moving to another section's folder, which is asked first. */
  | { kind: "move"; page: string; folder: string; label: string }
  /** Nothing, and why when it's worth saying. */
  | { kind: "none"; why?: string };

/**
 * What dropping `dragged` on `target` does. A section moves among the
 * sections beside it, and a page among its section's pages: dropped on an
 * item above, it goes before it, and on one below, after it. A page dropped
 * on its own section goes first. A page dropped on another section, or on a
 * page of one, moves its file there.
 */
export function pagesDrop(dragged: PagesNode, target: PagesNode): PagesDrop {
  if (dragged === target) return { kind: "none" };
  // A section beside its page sits among its parent's pages, as its page.
  const asPage = (node: PagesNode): { path: string; section: InventorySection } | undefined => {
    if (node.kind === "page" && !node.own) return { path: node.path, section: node.section };
    if (node.kind === "section" && node.placed && node.within && node.section.page !== null) {
      return { path: node.section.page, section: node.within };
    }
    return undefined;
  };
  const page = asPage(dragged);
  if (page) {
    const there = asPage(target);
    if (there && there.section === page.section) {
      const order = page.section.pages.map((p) => p.path);
      return {
        kind: "edit",
        action: "movePage",
        args: { page: page.path, before: placeAt(order, page.path, there.path) },
      };
    }
    const into =
      target.kind === "section" && !target.placed
        ? target.section
        : target.kind === "page"
          ? target.section
          : target.kind === "section" && target.placed
            ? target.section
            : undefined;
    if (into === page.section) {
      const first = page.section.pages.find((p) => p.path !== page.path);
      return {
        kind: "edit",
        action: "movePage",
        args: { page: page.path, before: first?.path ?? null },
      };
    }
    if (into && dragged.kind === "page") {
      return { kind: "move", page: page.path, folder: into.folder, label: into.label };
    }
    if (into) return { kind: "none", why: "A section moves to another folder with its files." };
    return { kind: "none" };
  }
  if (dragged.kind === "section" && dragged.section.folder !== "") {
    const within = dragged.within;
    if (target.kind === "section" && !target.placed && target.within === within && within) {
      const order = within.sections.filter((s) => !isPlaced(within, s)).map((s) => s.folder);
      return {
        kind: "edit",
        action: "moveSection",
        args: {
          folder: dragged.section.folder,
          before: placeAt(order, dragged.section.folder, target.section.folder),
        },
      };
    }
    return { kind: "none", why: "A section moves among the sections beside it." };
  }
  return { kind: "none" };
}

/** Whether a section sits among its parent's pages, where its page beside the folder sits. */
function isPlaced(parent: InventorySection, section: InventorySection): boolean {
  return section.page !== null && parent.pages.some((p) => p.path === section.page);
}

/**
 * What `moved` goes before when it's dropped on `on`, in `order`: `on` when
 * it's above, and what's after `on` when it's below; `null` for the end.
 */
function placeAt(order: readonly string[], moved: string, on: string): string | null {
  const from = order.indexOf(moved);
  const at = order.indexOf(on);
  if (from > at) return on;
  const rest = order.filter((item) => item !== moved);
  return rest[rest.indexOf(on) + 1] ?? null;
}

// Content model.

/** An item of the Content model view. */
export type ModelNode =
  | { kind: "kind"; modelKind: ModelKind; entries: readonly InventoryEntry[] }
  | { kind: "entry"; entry: InventoryEntry }
  | { kind: "value"; dimension: InventoryEntry; value: InventoryValue };

/** Each kind's name, codicon, and order. */
const KINDS: readonly { kind: ModelKind; name: string; icon: string }[] = [
  { kind: "type", name: "Page types", icon: "symbol-file" },
  { kind: "phrase", name: "Phrases", icon: "symbol-string" },
  { kind: "feature", name: "Features", icon: "tag" },
  { kind: "term", name: "Glossary", icon: "symbol-key" },
  { kind: "dimension", name: "Dimensions", icon: "symbol-enum" },
  { kind: "note", name: "Note types", icon: "note" },
  { kind: "widget", name: "Widgets", icon: "symbol-class" },
  { kind: "build", name: "Builds", icon: "package" },
];

/**
 * The Content model view's top level: a node per kind the model has, in a
 * fixed order. With `unusedOnly`, only the entries nothing uses, and the
 * kinds that have one.
 */
export function modelRoots(inventory: InventoryResult, unusedOnly = false): ModelNode[] {
  return KINDS.flatMap(({ kind }) => {
    const entries = inventory.model.filter((e) => e.kind === kind && (!unusedOnly || e.uses === 0));
    return entries.length > 0 ? [{ kind: "kind" as const, modelKind: kind, entries }] : [];
  });
}

/** An item's children in the Content model view: a kind's entries, and a dimension's values. */
export function modelChildren(node: ModelNode): ModelNode[] {
  if (node.kind === "kind") return node.entries.map((entry) => ({ kind: "entry", entry }));
  if (node.kind === "entry") {
    return (node.entry.values ?? []).map((value) => ({
      kind: "value",
      dimension: node.entry,
      value,
    }));
  }
  return [];
}

/**
 * What an item of the Content model view offers in its menu, as words of
 * its context value: `references` (Find All References) on an entry pages
 * use, declared in `ascribe.toml`; `rename` (Rename…) on a phrase or a
 * dimension's value declared there; `insert` (Insert at the Cursor) on a
 * phrase or a widget.
 */
const MODEL_CONTEXT = {
  references: "references",
  rename: "rename",
  insert: "insert",
} as const;

/** The kinds pages use, whose uses Find All References lists. */
const USED: readonly ModelKind[] = ["phrase", "feature", "term", "dimension", "note", "widget"];

function modelContext(node: ModelNode): string | undefined {
  if (node.kind === "kind") return undefined;
  if (node.kind === "value") {
    return node.value.declaration === null
      ? "ascribe.model.value"
      : `ascribe.model.value ${MODEL_CONTEXT.rename}`;
  }
  const { entry } = node;
  const words = [`ascribe.model.${entry.kind}`];
  const declared = entry.declaration !== null;
  if (declared && USED.includes(entry.kind)) words.push(MODEL_CONTEXT.references);
  if (declared && entry.kind === "phrase") words.push(MODEL_CONTEXT.rename);
  if (entry.kind === "phrase" || entry.kind === "widget") words.push(MODEL_CONTEXT.insert);
  return words.join(" ");
}

/** How an item of the Content model view looks. An entry nothing uses is marked and dimmed. */
export function modelLook(node: ModelNode): ItemLook {
  if (node.kind === "kind") {
    const kind = KINDS.find((k) => k.kind === node.modelKind);
    const unused = node.entries.filter((e) => e.uses === 0).length;
    const noun =
      node.modelKind === "type"
        ? count(node.entries.length, "type", "types")
        : count(node.entries.length, "entry", "entries");
    const none = node.modelKind === "type" ? "without pages" : "unused";
    return {
      ...group(kind?.name ?? node.modelKind, node.entries.length, kind?.icon ?? "symbol-misc", ""),
      tooltip: `${noun}${unused > 0 ? `, ${unused} ${none}` : ""}`,
    };
  }
  if (node.kind === "value") {
    const { value, dimension } = node;
    return {
      label: value.value,
      description: value.label ?? "",
      tooltip: [
        `A value of ${dimension.key}`,
        ...(value.label === null ? [] : [value.label]),
        ...(value.declaration === null ? ["Not found in ascribe.toml"] : []),
      ].join("\n"),
      icon: "symbol-enum-member",
      dimmed: false,
      expandable: false,
      context: modelContext(node),
    };
  }
  const { entry } = node;
  const icon = KINDS.find((k) => k.kind === entry.kind)?.icon ?? "symbol-misc";
  const unused = entry.uses === 0;
  const pageType = entry.kind === "type";
  const uses =
    entry.uses === null
      ? undefined
      : pageType
        ? count(entry.uses, "page", "pages")
        : count(entry.uses, "use", "uses");
  const tooltip = [entry.key];
  if (entry.label !== null) tooltip.push(entry.label);
  if (uses !== undefined) {
    tooltip.push(
      pageType
        ? unused
          ? "No page has it"
          : `${uses} of this type`
        : unused
          ? "Unused: nothing uses it"
          : `${uses} in pages`,
    );
  }
  if (entry.uses === null && entry.kind === "term") {
    tooltip.push("Marked: the links to its page are its uses");
  }
  if (entry.declaration === null) {
    tooltip.push(
      entry.kind === "note"
        ? "Built in"
        : pageType
          ? "The type every page has while ascribe.toml declares none"
          : "Not found in ascribe.toml",
    );
  }
  return {
    label: entry.key,
    description: unused ? (pageType ? "no pages" : "unused") : (uses ?? ""),
    tooltip: tooltip.join("\n"),
    icon,
    dimmed: unused,
    expandable: (entry.values ?? []).length > 0,
    context: modelContext(node),
  };
}

/** The place in `ascribe.toml` an item of Content model opens: its declaration. */
export function modelDeclaration(node: ModelNode): LspRange | undefined {
  if (node.kind === "entry") return node.entry.declaration ?? undefined;
  if (node.kind === "value") return node.value.declaration ?? undefined;
  return undefined;
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
