// The comparison script's pure parts. The script itself runs in CI on pull
// requests labeled `optimization` (ci.yml).
import { expect, test } from "vitest";
import {
  Acceptance,
  isBinary,
  normalize,
  outputDir,
  projects,
  reword,
  ROOT,
  rootSpellings,
} from "./outputs.ts";

test("each folder with an ascribe.toml is a project, nested ones too", () => {
  const files = [
    "docs/ascribe.toml",
    "docs/content/index.md",
    "examples/monorepo/handbook/ascribe.toml",
    "examples/monorepo/handbook/pages/security/ascribe.toml",
  ];
  expect(projects(files)).toEqual([
    "docs",
    "examples/monorepo/handbook",
    "examples/monorepo/handbook/pages/security",
  ]);
});

test("the output folder comes from ascribe.toml", () => {
  expect(outputDir('[project]\noutput-dir = "out/site"  # here\n')).toBe("out/site");
  expect(outputDir("[project]\noutput-dir = 'out/single'\n")).toBe("out/single");
  expect(outputDir("[project]\ncontent-root = 'docs'\n")).toBe(".ascribe/build");
});

test("the base changes whole words only", () => {
  expect(reword("the theme, then the_x; The other the.")).toBe("a theme, then the_x; The other a.");
});

test("a root is replaced however it's written", () => {
  const spellings = rootSpellings(["C:\\work\\ascribe\\", "/tmp/w"]);
  const text = [
    "C:\\work\\ascribe\\docs",
    "C:/work/ascribe/docs",
    '"C:\\\\work\\\\ascribe\\\\docs"',
    "/tmp/w/docs",
  ].join("\n");
  expect(normalize(Buffer.from(text), spellings).toString()).toBe(
    [`${ROOT}\\docs`, `${ROOT}/docs`, `"${ROOT}\\\\docs"`, `${ROOT}/docs`].join("\n"),
  );
});

test("binary files are left as they are", () => {
  const bytes = Buffer.from([0x89, 0x50, 0x00, 0x2f, 0x74, 0x6d, 0x70]);
  expect(isBinary(bytes)).toBe(true);
  expect(normalize(bytes, ["/tmp"])).toEqual(bytes);
});

test("an accepted difference matches its pattern whole, with `*` for anything", () => {
  const accept = new Acceptance([
    "*: ascribe diff --base HEAD --format html.txt",
    "examples/astro-site/dist: index.html",
    "docs: (unused)",
  ]);
  expect(accept.accepts("examples/monorepo/docs: ascribe diff --base HEAD --format html.txt")).toBe(
    true,
  );
  expect(accept.accepts("docs: ascribe diff --base HEAD --format json.txt")).toBe(false);
  expect(accept.accepts("examples/astro-site/dist: index.html")).toBe(true);
  expect(accept.accepts("examples/astro-site/dist: guides/index.html")).toBe(false);
  expect(accept.accepts("examples/astro-site/dist: index.htmlx")).toBe(false);
  expect(accept.unused()).toEqual(["docs: (unused)"]);
});
