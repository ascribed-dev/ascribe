// What the Used by, Pages, and Content model views show, worked out from
// plain values so the unit tests can check it without VS Code.

import type {
  InventoryEntry,
  InventoryFragment,
  InventoryPage,
  InventoryResult,
  ModelKind,
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
  const orphans = inventory.pages.filter((p) => inventory.orphans.includes(p.path));
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

// Used by.

/** A place that uses the active page, heading, or fragment. */
export interface Place {
  /** The file's URI. */
  uri: string;
  /** The file's path as people read it (relative to the content root). */
  path: string;
  /** Where, as the server's range. */
  range: { start: { line: number; character: number }; end: { line: number; character: number } };
  /** The text of the place's first line, trimmed. */
  text: string;
}

/** An item of the Used by view. */
export type UsedByNode =
  | { kind: "section"; section: "links" | "includes"; files: readonly UsedByFile[] }
  | { kind: "file"; file: UsedByFile }
  | { kind: "place"; place: Place };

/** The places in one file. */
interface UsedByFile {
  uri: string;
  path: string;
  places: readonly Place[];
}

/** Whether a place is an `@include`, rather than a link. */
function isInclude(place: Place): boolean {
  return /^(?:[>\s]|[-*+]\s|\d+[.)]\s)*@include\b/.test(place.text);
}

/** The Used by view's top level: the links, grouped by file, then the includes. */
export function usedByRoots(places: readonly Place[]): UsedByNode[] {
  const links = byFile(places.filter((p) => !isInclude(p)));
  const includes = byFile(places.filter(isInclude));
  const nodes: UsedByNode[] = [];
  if (links.length > 0) nodes.push({ kind: "section", section: "links", files: links });
  if (includes.length > 0) nodes.push({ kind: "section", section: "includes", files: includes });
  return nodes;
}

function byFile(places: readonly Place[]): UsedByFile[] {
  const files = new Map<string, UsedByFile & { places: Place[] }>();
  for (const place of places) {
    const file = files.get(place.uri) ?? { uri: place.uri, path: place.path, places: [] };
    file.places.push(place);
    files.set(place.uri, file);
  }
  return [...files.values()].sort((a, b) => a.path.localeCompare(b.path));
}

/** An item's children in the Used by view. */
export function usedByChildren(node: UsedByNode): UsedByNode[] {
  switch (node.kind) {
    case "section":
      return node.files.map((file) => ({ kind: "file", file }));
    case "file":
      return node.file.places.map((place) => ({ kind: "place", place }));
    case "place":
      return [];
  }
}

/** How an item of the Used by view looks. */
export function usedByLook(node: UsedByNode): ItemLook {
  switch (node.kind) {
    case "section": {
      const places = node.files.reduce((n, f) => n + f.places.length, 0);
      return node.section === "links"
        ? group("Linked from", places, "link", count(node.files.length, "file", "files"))
        : group(
            "Included by",
            places,
            "file-symlink-file",
            count(node.files.length, "file", "files"),
          );
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
  }
}

/** What the Used by view asks about: a heading's line asks about the heading, any other the file. */
export function isHeadingLine(text: string): boolean {
  return /^ {0,3}#{1,6}(?:\s|$)/.test(text);
}

// Shared.

function group(label: string, size: number, icon: string, tooltip: string): ItemLook {
  return { label, description: String(size), tooltip, icon, dimmed: false, expandable: true };
}

function count(n: number, one: string, many: string): string {
  return `${n} ${n === 1 ? one : many}`;
}

function basename(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}
