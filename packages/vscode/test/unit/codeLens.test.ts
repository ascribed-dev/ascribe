// Which files a project's server is asked for lenses in, and how a lens in
// a code file two projects show is named (`src/codeLens.ts`).
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { asksForLenses, isCodeFile, lensTitle } from "../../src/codeLens.js";
import type { Project } from "../../src/projects.js";

const root = path.resolve("/work");
const project = (folder: string): Project => ({
  folder: path.join(root, folder),
  config: path.join(root, folder, "ascribe.toml"),
});
const docs = project("docs");
const api = project("api");
const nested = project("docs/nested");
const projects = [docs, api, nested];

describe("asksForLenses", () => {
  it("asks for a code file in no project", () => {
    expect(asksForLenses(docs, path.join(root, "crates/lib.rs"), "rust", projects)).toBe(true);
  });

  it("asks for a code file in another project's folder", () => {
    expect(asksForLenses(docs, path.join(root, "api/src/main.ts"), "typescript", projects)).toBe(
      true,
    );
    expect(
      asksForLenses(docs, path.join(root, "docs/nested/code/x.ts"), "typescript", projects),
    ).toBe(true);
  });

  it("asks for its own pages, and not for another project's", () => {
    expect(asksForLenses(docs, path.join(root, "docs/content/a.md"), "markdown", projects)).toBe(
      true,
    );
    expect(asksForLenses(docs, path.join(root, "api/content/a.md"), "markdown", projects)).toBe(
      false,
    );
    expect(
      asksForLenses(docs, path.join(root, "docs/nested/content/a.md"), "markdown", projects),
    ).toBe(false);
  });

  it("keeps Ascribe's own files to their project, whatever their language", () => {
    for (const name of ["ascribe.toml", "ascribe.permalinks", "code/ascribe.examples.toml"]) {
      expect(asksForLenses(docs, path.join(root, "api", name), "toml", projects)).toBe(false);
      expect(asksForLenses(docs, path.join(root, "docs", name), "toml", projects)).toBe(true);
    }
  });
});

describe("isCodeFile", () => {
  it("is any file but a page or one of Ascribe's own", () => {
    expect(isCodeFile("/w/a.ts", "typescript")).toBe(true);
    expect(isCodeFile("/w/notes.txt", "plaintext")).toBe(true);
    expect(isCodeFile("/w/a.md", "markdown")).toBe(false);
    expect(isCodeFile("/w/ascribe.toml", "toml")).toBe(false);
  });
});

describe("lensTitle", () => {
  it("leaves a lens one project answers for as it is", () => {
    expect(lensTitle("Shown in 3 pages", "docs", 1)).toBe("Shown in 3 pages");
  });

  it("names the project when a file is in two projects' sources", () => {
    expect(lensTitle("Shown in 3 pages", "docs", 2)).toBe("docs: shown in 3 pages");
    expect(lensTitle("Shown in 1 page", "api", 2)).toBe("api: shown in 1 page");
  });
});
