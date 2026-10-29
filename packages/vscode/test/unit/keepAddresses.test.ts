import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { keepQuestion, pageMoves } from "../../src/keepAddresses.js";

// What Keep the addresses you've published records after the editor moves
// pages of a project without ascribe.addresses. The offer itself, made once,
// and the commands it runs, are the integration suite's.
describe("pageMoves", () => {
  const content = path.resolve("/project/content");
  const at = (...parts: string[]) => path.join(content, ...parts);

  it("gives each page moved inside the content root, as content paths", () => {
    expect(
      pageMoves(
        [
          { from: at("guides", "keys.md"), to: at("security", "keys.md") },
          { from: at("Setup.md"), to: at("Guides", "My Setup.md") },
        ],
        content,
      ),
    ).toEqual([
      { from: "guides/keys.md", to: "security/keys.md" },
      { from: "Setup.md", to: "Guides/My Setup.md" },
    ]);
  });

  it("leaves out what has no address of its own, and what didn't move", () => {
    expect(
      pageMoves(
        [
          // A fragment, a hidden folder's file, a file that isn't a page, one
          // outside the content root, and one into it from outside.
          { from: at("_setup.md"), to: at("_install.md") },
          { from: at(".drafts", "a.md"), to: at(".drafts", "b.md") },
          { from: at("logo.png"), to: at("images", "logo.png") },
          { from: path.resolve("/project/README.md"), to: path.resolve("/project/NOTES.md") },
          { from: path.resolve("/project/notes.md"), to: at("notes.md") },
          { from: at("same.md"), to: at("same.md") },
        ],
        content,
      ),
    ).toEqual([]);
  });
});

describe("keepQuestion", () => {
  it("names the project and how many pages moved", () => {
    expect(keepQuestion("docs", 1)).toMatch(
      /^Ascribe: you moved a page of docs, so its old address stops working\./,
    );
    expect(keepQuestion("docs", 3)).toMatch(
      /you moved 3 pages of docs, so their old addresses stop working/,
    );
    expect(keepQuestion("docs", 3)).toContain("records these moves");
  });
});
