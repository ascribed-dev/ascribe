// The site output's attribute marker (`<ascribe-attributes>`), found and
// applied on a hast tree.
//
// Both markdown processors Astro 7.3 can run hand a user plugin the same tree
// at the same point: after markdown became hast, before Astro's own image and
// heading-id passes. A marker is then two adjacent `raw` nodes (an open tag and
// a close tag), and the elements it applies to are `<h1>`–`<h6>` and `<img>`.
// So the rules live here once, as a pure function from a tree to a list of
// edits, and each processor's adapter applies the edits with its own means
// (`rehype.ts` mutates the tree, `satteri.ts` queues commands).

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
  /** The heading or image that gets the attributes. */
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

/** Finds every marker that applies to a heading or an image in `root`. */
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
      if (before !== undefined && before.type === "element" && before.tagName === "img") {
        // Directly after an image, whatever else the marker ends.
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
