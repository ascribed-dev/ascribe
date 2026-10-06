// The comparison script's pure parts. The script itself runs in CI on pull
// requests labeled `optimization` (ci.yml).
import { expect, test } from "vitest";
import { isBinary, normalize, outputDir, projects, ROOT, rootSpellings } from "./outputs.ts";

test("every example project and the docs are compared", () => {
  const found = projects();
  expect(found).toContain("docs");
  expect(found).toContain("examples/quill");
  // Nested projects count too.
  expect(found).toContain("examples/monorepo/handbook/pages/security");
});

test("the output folder comes from ascribe.toml", () => {
  expect(outputDir('[project]\noutput-dir = "out/site"  # here\n')).toBe("out/site");
  expect(outputDir("[project]\ncontent-root = 'docs'\n")).toBe(".ascribe/build");
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
