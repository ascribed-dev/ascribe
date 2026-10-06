// Cases beyond tests/render/ for the marker rules, on hand-built trees.
import { describe, expect, it } from "vitest";
import { findAnchors, findEdits, toProperty, type HastNode } from "../src/attributes.js";
import rehypeAscribeAttributes from "../src/rehype.js";

const raw = (value: string): HastNode => ({ type: "raw", value });
const text = (value: string): HastNode => ({ type: "text", value });
const element = (
  tagName: string,
  children: HastNode[] = [],
  properties: Record<string, unknown> = {},
): HastNode => ({
  type: "element",
  tagName,
  properties,
  children,
});
const open = (attributes: string): HastNode => raw(`<ascribe-attributes${attributes}>`);
const close = raw("</ascribe-attributes>");

describe("findEdits", () => {
  it("accepts a marker written as one node", () => {
    const heading = element("h2", [
      text("Title "),
      raw('<ascribe-attributes id="t"></ascribe-attributes>'),
    ]);
    const edits = findEdits({ type: "root", children: [heading] });
    expect(edits).toHaveLength(1);
    expect(edits[0]?.attributes).toEqual([["id", "t"]]);
  });

  it("decodes the four escapes and only those", () => {
    const image = element("img");
    const marker = [open(' caption="a &quot;b&quot; &amp; &lt;c&gt; &copy; &amp;amp;"'), close];
    const edits = findEdits({ type: "root", children: [element("p", [image, ...marker])] });
    expect(edits[0]?.attributes).toEqual([["caption", 'a "b" & <c> &copy; &amp;']]);
  });

  it("ignores near-markers", () => {
    for (const marker of [
      [raw("<ascribe-attributes id='t'>"), close],
      [raw('<ascribe-attributes  id="t">'), close],
      [raw('<ascribe-attributes id="t" >'), close],
      [raw('<ascribe-attributes ID="t">'), close],
      [raw('<ascribe-attributes id="t">'), text("x"), close],
      [raw('<ascribe-attributes id="t">')],
    ]) {
      const heading = element("h2", [text("Title "), ...marker]);
      expect(findEdits({ type: "root", children: [heading] })).toEqual([]);
    }
  });

  it("applies only the marker that ends a heading", () => {
    const heading = element("h2", [
      text("A "),
      open(' id="one"'),
      close,
      text(" B "),
      open(' id="two"'),
      close,
    ]);
    const edits = findEdits({ type: "root", children: [heading] });
    expect(edits.map((edit) => edit.attributes)).toEqual([[["id", "two"]]]);
    expect(edits[0]?.trim).toEqual([{ node: heading.children?.[3], value: " B" }]);
  });

  it("gives a marker after an image to the image, not the heading that image ends", () => {
    const image = element("img");
    const heading = element("h2", [text("A "), image, open(' width="3"'), close]);
    const edits = findEdits({ type: "root", children: [heading] });
    expect(edits).toHaveLength(1);
    expect(edits[0]?.target).toBe(image);
  });

  it("gives a marker after a link to the link", () => {
    const link = element("a", [text("API key")], { href: "/g#api-key", title: "A token." });
    const paragraph = element("p", [link, open(' data-ascribe-term="api-key"'), close, text(" x")]);
    const edits = findEdits({ type: "root", children: [paragraph] });
    expect(edits).toHaveLength(1);
    expect(edits[0]?.target).toBe(link);
    expect(edits[0]?.attributes.map(toProperty)).toEqual([["dataAscribeTerm", "api-key"]]);
  });

  it("removes all the whitespace before a heading's marker, across nodes", () => {
    const heading = element("h2", [text("A"), text(" \t"), text("\n"), open(' id="a"'), close]);
    const edits = findEdits({ type: "root", children: [heading] });
    expect(edits[0]?.trim.map((t) => t.value)).toEqual(["", ""]);
  });
});

describe("rehypeAscribeAttributes", () => {
  it("sets properties, removes markers and the space before a heading's", () => {
    const image = element("img", [], { src: "a.png" });
    const heading = element("h2", [text("Title "), open(' id="title"'), close]);
    const tree = {
      type: "root" as const,
      children: [heading, element("p", [image, open(' width="600"'), close])],
    };
    rehypeAscribeAttributes()(tree as never);
    expect(heading.properties).toEqual({ id: "title" });
    expect(heading.children).toEqual([text("Title")]);
    expect(image.properties).toEqual({ src: "a.png", width: "600" });
    expect(tree.children[1]?.children).toEqual([image]);
  });
});

describe("toProperty", () => {
  it("uses hast's property names and shapes", () => {
    expect(toProperty(["id", "x"])).toEqual(["id", "x"]);
    expect(toProperty(["class", "a  b"])).toEqual(["className", ["a", "b"]]);
    expect(toProperty(["data-caption", "hi"])).toEqual(["dataCaption", "hi"]);
    expect(toProperty(["caption", "hi"])).toEqual(["caption", "hi"]);
  });
});

describe("findAnchors", () => {
  const anchor = (attributes: string): HastNode => raw(`<!--ascribe-anchor${attributes}-->`);

  it("applies an anchor to the element it names, and removes it with the line ending after it", () => {
    const paragraph = element("p", [text("Hi")]);
    const comment = anchor(' tag="p" source="a.md:1-2" via="b.md:3"');
    const newline = text("\n");
    const edits = findAnchors({ type: "root", children: [comment, newline, paragraph] });
    expect(edits).toEqual([
      {
        remove: [comment, newline],
        targets: [
          {
            node: paragraph,
            attributes: [
              ["data-ascribe-source", "a.md:1-2"],
              ["data-ascribe-via", "b.md:3"],
            ],
          },
        ],
      },
    ]);
  });

  it("applies to nothing when the next node isn't the element it names", () => {
    for (const next of [element("ul"), text("tight text"), element("strong")]) {
      const comment = anchor(' tag="p" source="a.md:1-1"');
      const edits = findAnchors({ type: "root", children: [comment, next] });
      expect(edits).toEqual([{ remove: [comment], targets: [] }]);
    }
  });

  it("applies a code block's anchor to the figure a code title put it in", () => {
    const pre = element("pre", [text("code")]);
    const figure: HastNode = {
      type: "element",
      tagName: "figure",
      properties: { className: ["code-title"] },
      children: [element("figcaption", [text("a.ts")]), pre],
    };
    const [edit] = findAnchors({
      type: "root",
      children: [anchor(' tag="pre" source="a.md:3-6"'), text("\n"), figure],
    });
    expect(edit?.targets).toEqual([
      { node: figure, attributes: [["data-ascribe-source", "a.md:3-6"]] },
    ]);
    // Any other figure isn't the block.
    const other: HastNode = { ...figure, properties: {} };
    const [none] = findAnchors({
      type: "root",
      children: [anchor(' tag="pre" source="a.md:3-6"'), other],
    });
    expect(none?.targets).toEqual([]);
  });

  it("gives a list's items their lines only when they match", () => {
    const list = element("ul", [text("\n"), element("li"), text("\n"), element("li")]);
    const [edit] = findAnchors({
      type: "root",
      children: [anchor(' tag="ul" source="a%20b.md:3-6" items="3-4 5-6"'), list],
    });
    expect(edit?.targets.map((t) => t.attributes)).toEqual([
      [["data-ascribe-source", "a%20b.md:3-6"]],
      [["data-ascribe-source", "a%20b.md:3-4"]],
      [["data-ascribe-source", "a%20b.md:5-6"]],
    ]);
    const [short] = findAnchors({
      type: "root",
      children: [anchor(' tag="ul" source="a.md:3-6" items="3-4"'), list],
    });
    expect(short?.targets).toHaveLength(1);
  });

  it("writes the anchor into raw HTML's first tag", () => {
    const html = raw('<DIV class="x">\nhi\n</DIV>');
    const [edit] = findAnchors({
      type: "root",
      children: [anchor(' tag="div" source="a.md:1-3" via="x&quot;y.md:2"'), html],
    });
    expect(edit?.replace).toEqual({
      node: html,
      value:
        '<DIV data-ascribe-source="a.md:1-3" data-ascribe-via="x&quot;y.md:2" class="x">\nhi\n</DIV>',
    });
  });

  it("ignores near-anchors", () => {
    for (const value of [
      '<!--ascribe-anchor tag="p"-->',
      '<!--ascribe-anchor source="a.md:1-1"-->',
      "<!--ascribe-anchor tag='p' source='a.md:1-1'-->",
      '<!-- ascribe-anchor tag="p" source="a.md:1-1"-->',
      '<!--ascribe-anchor tag="p" source="a.md:1-1" -->',
    ]) {
      expect(findAnchors({ type: "root", children: [raw(value), element("p")] })).toEqual([]);
    }
  });

  it("is applied by the rehype plugin", () => {
    const tree = {
      type: "root",
      children: [anchor(' tag="h2" source="a.md:1-1"'), text("\n"), element("h2", [text("T")])],
    };
    rehypeAscribeAttributes()(tree as never);
    expect(tree.children).toEqual([element("h2", [text("T")], { dataAscribeSource: "a.md:1-1" })]);
  });
});
