// Compares HTML as parsed trees, as tests/render/README.md says: parse both as
// a fragment in a <body> context, drop whitespace-only text outside <pre>, and
// compare node by node (element names, attributes as unordered sets, text and
// comments exactly).
import type { Nodes, Root } from "hast";
import { fromHtml } from "hast-util-from-html";

/** A comparable form of a node. */
type Shape = string | { tag: string; attributes: [string, string][]; children: Shape[] };

/** The path to the first difference between two HTML fragments, or `undefined` when they're equal. */
export function firstDifference(expected: string, actual: string): string | undefined {
  return compare(
    shapes(fromHtml(expected, { fragment: true })),
    shapes(fromHtml(actual, { fragment: true })),
    "body",
  );
}

function shapes(parent: Root | Extract<Nodes, { children: unknown[] }>, inPre = false): Shape[] {
  const out: Shape[] = [];
  for (const node of parent.children) {
    if (node.type === "text") {
      if (!inPre && node.value.trim() === "") continue;
      out.push(`text:${node.value}`);
    } else if (node.type === "comment") {
      out.push(`comment:${node.value}`);
    } else if (node.type === "element") {
      const attributes = Object.entries(node.properties)
        .map(
          ([name, value]) =>
            [name, Array.isArray(value) ? value.join(" ") : String(value)] as [string, string],
        )
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
      out.push({
        tag: node.tagName,
        attributes,
        children: shapes(node, inPre || node.tagName === "pre"),
      });
    }
  }
  return out;
}

function compare(expected: Shape[], actual: Shape[], path: string): string | undefined {
  const count = Math.max(expected.length, actual.length);
  for (let i = 0; i < count; i++) {
    const e = expected[i];
    const a = actual[i];
    const here = `${path} > [${i}]`;
    if (e === undefined || a === undefined) {
      return `${here}: expected ${describe(e)}, got ${describe(a)}`;
    }
    if (typeof e === "string" || typeof a === "string") {
      if (e !== a) return `${here}: expected ${describe(e)}, got ${describe(a)}`;
      continue;
    }
    if (e.tag !== a.tag) return `${here}: expected <${e.tag}>, got <${a.tag}>`;
    const attributes = JSON.stringify(e.attributes);
    if (attributes !== JSON.stringify(a.attributes)) {
      return `${here} <${e.tag}>: expected attributes ${attributes}, got ${JSON.stringify(a.attributes)}`;
    }
    const inner = compare(e.children, a.children, `${here} <${e.tag}>`);
    if (inner !== undefined) return inner;
  }
  return undefined;
}

function describe(shape: Shape | undefined): string {
  if (shape === undefined) return "nothing";
  return typeof shape === "string" ? JSON.stringify(shape) : `<${shape.tag}>`;
}
