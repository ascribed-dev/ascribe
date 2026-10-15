// Which document links the editor follows (`src/linkSchemes.ts`): the same
// schemes as the language server, and the files it resolves.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { followableScheme, LINK_SCHEMES } from "../../src/linkSchemes.js";

describe("followableScheme", () => {
  it("follows web and email addresses and files, whatever their case", () => {
    for (const scheme of ["http", "https", "HTTPS", "mailto", "file"]) {
      expect(followableScheme(scheme)).toBe(true);
    }
  });

  it("doesn't follow any other scheme", () => {
    for (const scheme of ["command", "COMMAND", "vscode", "vscode-insiders", "javascript", "tel"]) {
      expect(followableScheme(scheme)).toBe(false);
    }
  });

  it("has the language server's schemes", () => {
    const source = readFileSync(
      new URL("../../../../crates/ascribe-lsp/src/links.rs", import.meta.url),
      "utf8",
    );
    const list = /const LINK_SCHEMES: \[&str; \d+\] = \[([^\]]*)\];/.exec(source)?.[1];
    expect(list, "LINK_SCHEMES in links.rs").toBeDefined();
    const schemes = [...(list ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
    expect(schemes).toEqual([...LINK_SCHEMES]);
  });
});
