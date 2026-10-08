// The specimen page and its candidates. A stale design/specimen.html fails
// here; run with ASCRIBE_BLESS=1 (or `node scripts/design/specimen.ts`) to
// rewrite it, then look at it.
import { readFileSync, writeFileSync } from "node:fs";
import { expect, test } from "vitest";
import {
  checkPairs,
  COLORS,
  contrast,
  difference,
  readCandidate,
  readInputs,
  readMark,
  renderSpecimen,
  SPECIMEN,
} from "./specimen.ts";

const inputs = readInputs();

test("contrast is WCAG 2's", () => {
  expect(contrast("#000000", "#ffffff")).toBeCloseTo(21, 5);
  expect(contrast("#ffffff", "#ffffff")).toBe(1);
  expect(contrast("#767676", "#ffffff")).toBeCloseTo(4.54, 2);
  expect(contrast("#0969da", "#ffffff")).toBe(contrast("#ffffff", "#0969da"));
});

test("red and green move closer for a reader without red or green cones", () => {
  const [green, red] = ["#1a7f37", "#cf222e"];
  expect(difference(green, red, "protan")).toBeLessThan(difference(green, red, "normal"));
  expect(difference(green, red, "deutan")).toBeLessThan(difference(green, red, "normal"));
  expect(difference(green, green, "deutan")).toBe(0);
});

test("there's a baseline, and two or three palettes to choose from", () => {
  expect(inputs.candidates.filter((c) => c.baseline).map((c) => c.id)).toEqual(["baseline"]);
  expect(inputs.candidates.filter((c) => !c.baseline).length).toBeGreaterThanOrEqual(2);
  expect(inputs.marks.length).toBeGreaterThanOrEqual(3);
});

test("every candidate palette passes every pairing, in light and dark", () => {
  for (const candidate of inputs.candidates.filter((c) => !c.baseline)) {
    const failing = checkPairs(candidate, inputs.pairs)
      .filter((c) => !c.pass)
      .map((c) => `${c.scheme}: ${c.pair.fg} on ${c.pair.bg}, ${c.ratio.toFixed(2)}:1`);
    expect(failing, candidate.id).toEqual([]);
  }
});

test("the pairs use every semantic color but the two that carry no meaning", () => {
  const used = new Set(inputs.pairs.flatMap((p) => [p.fg, p.bg]));
  expect(COLORS.filter((c) => !used.has(c))).toEqual(["border", "changed-background"]);
});

test("a candidate names the key it got wrong", () => {
  const good = readFileSync(
    new URL("../../design/candidates/baseline.toml", import.meta.url),
    "utf8",
  );
  expect(() => readCandidate("x", good.replace('"blue.600"', '"blue.601"'))).toThrow(
    'x.toml color.accent.light: no palette entry "blue.601"',
  );
  expect(() => readCandidate("x", good.replace(/^focus = .*$/m, ""))).toThrow(
    "x.toml color.focus: expected a table",
  );
  expect(() => readCandidate("x", good.replace('"#ffffff"', '"#FFF"'))).toThrow(
    'x.toml palette.gray.0: "#FFF" isn\'t a lowercase #rrggbb',
  );
});

test("a mark is paths in a viewBox, with a title and a note", () => {
  const svg = (body: string) =>
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">${body}</svg>`;
  expect(readMark("x", svg("<title>X</title><desc>Why.</desc><path d='M0 0'/>")).title).toBe("X");
  expect(() => readMark("x", svg("<title>X</title><path d='M0 0'/>"))).toThrow("<desc>");
  expect(() => readMark("x", svg("<title>X</title><desc>.</desc><text>a</text>"))).toThrow(
    "paths only",
  );
  expect(() => readMark("x", "<svg><title>X</title><desc>.</desc></svg>")).toThrow("viewBox");
});

test("design/specimen.html is up to date", () => {
  const fresh = renderSpecimen(inputs);
  if (process.env.ASCRIBE_BLESS) writeFileSync(SPECIMEN, fresh);
  const committed = readFileSync(SPECIMEN, "utf8");
  expect(
    committed === fresh,
    "design/specimen.html is stale: run `node scripts/design/specimen.ts`",
  ).toBe(true);
});
