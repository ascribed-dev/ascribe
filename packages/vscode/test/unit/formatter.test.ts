// Which formatter VS Code runs on a project's pages (`src/formatterChoice.ts`):
// the extension offers to be the formatter only when another one is set.
import { describe, expect, it } from "vitest";
import { formatterOffer, otherFormatter } from "../../src/formatterChoice.js";

const OWN = "Ascribe.ascribe-vscode";

describe("otherFormatter", () => {
  it("names another extension's formatter", () => {
    expect(otherFormatter("esbenp.prettier-vscode", OWN)).toBe("esbenp.prettier-vscode");
  });

  it("is undefined for Ascribe's own, whatever its case", () => {
    expect(otherFormatter(OWN, OWN)).toBeUndefined();
    expect(otherFormatter("ascribe.ascribe-vscode", OWN)).toBeUndefined();
  });

  it("is undefined when no formatter is set", () => {
    for (const value of [undefined, null, "", "  ", 3]) {
      expect(otherFormatter(value, OWN)).toBeUndefined();
    }
  });
});

describe("formatterOffer", () => {
  it("names the formatter that's set", () => {
    expect(formatterOffer("Prettier - Code formatter")).toContain(
      "Prettier - Code formatter formats Markdown here",
    );
  });
});
