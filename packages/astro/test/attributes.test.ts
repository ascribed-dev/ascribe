// Cases beyond tests/render/ for the marker rules, on hand-built trees.
import { describe, expect, it } from "vitest";
import { findEdits, toProperty, type HastNode } from "../src/attributes.js";
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

  it("gives a marker after an image to the image, not the heading that image ends (Q145)", () => {
    const image = element("img");
    const heading = element("h2", [text("A "), image, open(' width="3"'), close]);
    const edits = findEdits({ type: "root", children: [heading] });
    expect(edits).toHaveLength(1);
    expect(edits[0]?.target).toBe(image);
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
