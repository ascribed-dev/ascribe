// Which files a project's server is asked for lenses in, and how a lens in
// a file two projects answer in is named (`src/codeLens.ts`).
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
const none: string[] = [];
const code = path.join(root, "code");

describe("asksForLenses", () => {
  it("asks for a code file in no project", () => {
    expect(asksForLenses(docs, path.join(root, "crates/lib.rs"), "rust", none, projects)).toBe(
      true,
    );
  });

  it("asks for a code file in another project's folder", () => {
    expect(
      asksForLenses(docs, path.join(root, "api/src/main.ts"), "typescript", none, projects),
    ).toBe(true);
    expect(
      asksForLenses(docs, path.join(root, "docs/nested/code/x.ts"), "typescript", none, projects),
    ).toBe(true);
  });

  it("asks for its own pages, and not for another project's", () => {
    expect(
      asksForLenses(docs, path.join(root, "docs/content/a.md"), "markdown", none, projects),
    ).toBe(true);
    expect(
      asksForLenses(docs, path.join(root, "api/content/a.md"), "markdown", none, projects),
    ).toBe(false);
    expect(
      asksForLenses(docs, path.join(root, "docs/nested/content/a.md"), "markdown", none, projects),
    ).toBe(false);
  });

  it("asks for a Markdown file in its sources' folders, whichever project's it is", () => {
    expect(asksForLenses(docs, path.join(code, "notes.md"), "markdown", [code], projects)).toBe(
      true,
    );
    expect(asksForLenses(docs, path.join(code, "notes.md"), "markdown", none, projects)).toBe(
      false,
    );
    expect(
      asksForLenses(docs, path.join(root, "api/content/a.md"), "markdown", [api.folder], projects),
    ).toBe(true);
  });

  it("doesn't ask for a nested project's page its registration covers but its sources don't", () => {
    const page = path.join(root, "docs/nested/content/a.md");
    expect(asksForLenses(docs, page, "markdown", [code], projects)).toBe(false);
  });

  it("keeps Ascribe's own files to their project, whatever their language", () => {
    for (const name of ["ascribe.toml", "ascribe.permalinks", "code/ascribe.examples.toml"]) {
      expect(
        asksForLenses(docs, path.join(root, "api", name), "toml", [api.folder], projects),
      ).toBe(false);
      expect(asksForLenses(docs, path.join(root, "docs", name), "toml", none, projects)).toBe(true);
    }
  });
});

describe("isCodeFile", () => {
  it("is any file but a page or one of Ascribe's own", () => {
    expect(isCodeFile("/w/a.ts", "typescript")).toBe(true);
    expect(isCodeFile("/w/notes.txt", "plaintext")).toBe(true);
    expect(isCodeFile("/w/a.md", "markdown")).toBe(false);
    expect(isCodeFile("/w/ascribe.toml", "toml")).toBe(false);
    expect(isCodeFile("/w/ascribe.nav", "toml")).toBe(false);
  });
});

describe("lensTitle", () => {
  it("leaves a lens only one project answers with as it is", () => {
    expect(lensTitle("Shown in 3 pages", "docs", 1)).toBe("Shown in 3 pages");
  });

  it("names the project when another project's server answers with a lens too", () => {
    expect(lensTitle("Shown in 3 pages", "docs", 2)).toBe("docs: shown in 3 pages");
    expect(lensTitle("Shown in 1 page", "api", 2)).toBe("api: shown in 1 page");
  });
});
