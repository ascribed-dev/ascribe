// The site output's attribute marker (`<ascribe-attributes>`) and source
// anchors (`<!--ascribe-anchor …-->`), found and applied on a hast tree.
//
// Both markdown processors Astro 7.3 can run hand a user plugin the same tree
// at the same point: after markdown became hast, before Astro's own image and
// heading-id passes. A marker is then two adjacent `raw` nodes (an open tag and
// a close tag), and the elements it applies to are `<h1>`–`<h6>`, `<img>`,
// and `<a>`.
// An anchor comment is a `raw` node beside the block it names. So the rules
// live here once, as pure functions from a tree to a list of edits, and each
// processor's adapter applies the edits with its own means (`rehype.ts`
// mutates the tree, `satteri.ts` queues commands).

import { find, html } from "property-information";

/** The parts of a hast node the rules read. Both processors' trees fit it. */
export interface HastNode {
  type: string;
  tagName?: string;
  value?: string;
  properties?: Record<string, unknown> | undefined;
  children?: HastNode[];
}

/** A marker's attribute: a name and its decoded value. */
export type Attribute = readonly [name: string, value: string];

/** What applying one marker changes. */
export interface Edit {
  /** The heading, image, or link that gets the attributes. */
  target: HastNode;
  /** The marker's attributes, in the order written. */
  attributes: Attribute[];
  /** The marker's nodes, which are removed. */
  remove: HastNode[];
  /**
   * For a heading: the text nodes directly before the marker, whose trailing
   * whitespace is removed. A node whose text becomes empty is removed
   * (`value` is `""`).
   */
  trim: { node: HastNode; value: string }[];
}

const OPEN = /^<ascribe-attributes((?: [a-z][a-z0-9-]*="[^"\r\n]*")*)>$/;
const CLOSE = "</ascribe-attributes>";
const ATTRIBUTE = / ([a-z][a-z0-9-]*)="([^"]*)"/g;
const HEADING = /^h[1-6]$/;

/** Finds every marker that applies to a heading, an image, or a link in `root`. */
export function findEdits(root: HastNode): Edit[] {
  const edits: Edit[] = [];
  visitParents(root, (parent) => {
    const children = parent.children ?? [];
    const isHeading = parent.type === "element" && HEADING.test(parent.tagName ?? "");
    for (let i = 0; i < children.length; i++) {
      const found = markerAt(children, i);
      if (found === undefined) continue;
      const { attributes, length } = found;
      const last = i + length - 1;
      const marker = children.slice(i, last + 1);
      const before = children[i - 1];
      if (
        before !== undefined &&
        before.type === "element" &&
        (before.tagName === "img" || before.tagName === "a")
      ) {
        // Directly after an image or a link, whatever else the marker ends.
        edits.push({ target: before, attributes, remove: marker, trim: [] });
      } else if (isHeading && last === children.length - 1) {
        edits.push({
          target: parent,
          attributes,
          remove: marker,
          trim: trailingWhitespace(children, i),
        });
      }
      i = last;
    }
  });
  return edits;
}

/** The marker starting at `children[i]`: its attributes and how many nodes it spans. */
function markerAt(
  children: HastNode[],
  i: number,
): { attributes: Attribute[]; length: number } | undefined {
  const first = children[i];
  if (first?.type !== "raw" || first.value === undefined) return undefined;
  if (first.value.endsWith(CLOSE)) {
    // One node holding both tags.
    const open = first.value.slice(0, -CLOSE.length);
    const attributes = parseOpen(open);
    return attributes && { attributes, length: 1 };
  }
  const second = children[i + 1];
  if (second?.type !== "raw" || second.value !== CLOSE) return undefined;
  const attributes = parseOpen(first.value);
  return attributes && { attributes, length: 2 };
}

function parseOpen(tag: string): Attribute[] | undefined {
  const match = OPEN.exec(tag);
  if (match === null) return undefined;
  const attributes: Attribute[] = [];
  for (const [, name, value] of (match[1] ?? "").matchAll(ATTRIBUTE)) {
    if (name !== undefined && value !== undefined) attributes.push([name, decode(value)]);
  }
  return attributes;
}

/** The marker's four escapes; any other `&` is literal. */
function decode(value: string): string {
  return value.replace(/&(quot|amp|lt|gt);/g, (_, name: string) =>
    name === "quot" ? '"' : name === "amp" ? "&" : name === "lt" ? "<" : ">",
  );
}

/** The spaces, tabs, and line breaks directly before `children[i]`, which a heading's marker removes. */
function trailingWhitespace(children: HastNode[], i: number): Edit["trim"] {
  const trim: Edit["trim"] = [];
  for (let j = i - 1; j >= 0; j--) {
    const node = children[j];
    if (node?.type !== "text" || node.value === undefined) break;
    const value = node.value.replace(/[ \t\r\n]+$/, "");
    if (value === node.value) break;
    trim.push({ node, value });
    if (value !== "") break;
  }
  return trim;
}

function visitParents(node: HastNode, visit: (parent: HastNode) => void): void {
  if (node.children === undefined) return;
  visit(node);
  for (const child of node.children) visitParents(child, visit);
}

/**
 * A marker attribute as a hast property: the property name hast and Astro use
 * (`class` is `className`) and a value of the shape that property has.
 */
export function toProperty([name, value]: Attribute): [string, string | string[]] {
  const info = find(html, name);
  if (info.spaceSeparated) return [info.property, value.split(/\s+/).filter((s) => s !== "")];
  if (info.commaSeparated) return [info.property, value.split(/\s*,\s*/)];
  return [info.property, value];
}

/** What applying one source anchor (site-render contract §7) changes. */
export interface AnchorEdit {
  /** The anchor comment, and the whitespace-only text directly after it, which are removed. */
  remove: HastNode[];
  /** Elements that get attributes: the block, and a list's items. Empty when the anchor applies to nothing. */
  targets: { node: HastNode; attributes: Attribute[] }[];
  /** Raw HTML whose first tag gets the anchor's attributes: the node and its new text. */
  replace?: { node: HastNode; value: string };
}

const ANCHOR = /^<!--ascribe-anchor((?: [a-z][a-z0-9-]*="[^"\r\n]*")+)-->\s*$/;

/** Finds every source anchor comment in `root`, and what it applies to. */
export function findAnchors(root: HastNode): AnchorEdit[] {
  const edits: AnchorEdit[] = [];
  visitParents(root, (parent) => {
    const children = parent.children ?? [];
    for (let i = 0; i < children.length; i++) {
      const node = children[i];
      if (node?.type !== "raw" || node.value === undefined) continue;
      const anchor = parseAnchor(node.value);
      if (anchor === undefined) continue;
      const edit: AnchorEdit = { remove: [node], targets: [] };
      let next = i + 1;
      const after = children[next];
      if (after?.type === "text" && after.value?.trim() === "") {
        edit.remove.push(after);
        next++;
      }
      const target = children[next];
      if (anchor.tag === "pre" && target !== undefined && isCodeTitle(target)) {
        // A titled code block, which the code-titles transformer put in a
        // figure: the anchor marks the figure, so the mark holds the caption.
        edit.targets.push({
          node: target,
          attributes: anchorAttributes(anchor.source, anchor.via),
        });
      } else if (target?.type === "element" && target.tagName?.toLowerCase() === anchor.tag) {
        edit.targets.push({
          node: target,
          attributes: anchorAttributes(anchor.source, anchor.via),
        });
        if ((anchor.tag === "ul" || anchor.tag === "ol") && anchor.items.length > 0) {
          const items = (target.children ?? []).filter(
            (child) => child.type === "element" && child.tagName === "li",
          );
          const path = anchor.source.slice(0, anchor.source.lastIndexOf(":"));
          if (items.length === anchor.items.length) {
            items.forEach((item, n) =>
              edit.targets.push({
                node: item,
                attributes: anchorAttributes(`${path}:${anchor.items[n]}`, anchor.via),
              }),
            );
          }
        }
      } else if (target?.type === "raw" && target.value !== undefined) {
        const value = intoFirstTag(target.value, anchor);
        if (value !== undefined) edit.replace = { node: target, value };
      }
      edits.push(edit);
    }
  });
  return edits;
}

/** Whether a node is a `<figure class="code-title">` holding a `<pre>` (`code-titles.ts`). */
function isCodeTitle(node: HastNode): boolean {
  if (node.type !== "element" || node.tagName?.toLowerCase() !== "figure") return false;
  const className = node.properties?.["className"] ?? node.properties?.["class"];
  const classes = Array.isArray(className)
    ? className.map(String)
    : typeof className === "string"
      ? className.split(/\s+/)
      : [];
  return (
    classes.includes("code-title") &&
    (node.children ?? []).some(
      (child) => child.type === "element" && child.tagName?.toLowerCase() === "pre",
    )
  );
}

interface Anchor {
  tag: string;
  source: string;
  via: string | undefined;
  items: string[];
}

function parseAnchor(text: string): Anchor | undefined {
  const match = ANCHOR.exec(text);
  if (match === null) return undefined;
  const attributes = new Map<string, string>();
  for (const [, name, value] of (match[1] ?? "").matchAll(ATTRIBUTE)) {
    if (name !== undefined && value !== undefined) attributes.set(name, decode(value));
  }
  const tag = attributes.get("tag");
  const source = attributes.get("source");
  if (tag === undefined || source === undefined) return undefined;
  const via = attributes.get("via");
  return {
    tag: tag.toLowerCase(),
    source,
    via: via === "" ? undefined : via,
    items: (attributes.get("items") ?? "").split(/\s+/).filter((s) => s !== ""),
  };
}

function anchorAttributes(source: string, via: string | undefined): Attribute[] {
  const attributes: Attribute[] = [["data-ascribe-source", source]];
  if (via !== undefined) attributes.push(["data-ascribe-via", via]);
  return attributes;
}

/** Raw HTML with the anchor's attributes after the name of its first tag, when that tag opens the element the anchor names. */
function intoFirstTag(html: string, anchor: Anchor): string | undefined {
  const open = /^<([A-Za-z][A-Za-z0-9-]*)(?=[\s>/])/.exec(html);
  if (open?.[1] === undefined || open[1].toLowerCase() !== anchor.tag) return undefined;
  const written = anchorAttributes(anchor.source, anchor.via)
    .map(([name, value]) => ` ${name}="${encode(value)}"`)
    .join("");
  return html.slice(0, open[0].length) + written + html.slice(open[0].length);
}

function encode(value: string): string {
  return value.replace(/[&<>"]/g, (ch) =>
    ch === "&" ? "&amp;" : ch === "<" ? "&lt;" : ch === ">" ? "&gt;" : "&quot;",
  );
}
