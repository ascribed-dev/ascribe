import { describe, expect, it } from "vitest";
import { diagnosticSource, isAscribeSource, problemsText } from "../../src/problems.js";

describe("problems in words", () => {
  it("says nothing for none", () => {
    expect(problemsText({ errors: 0, warnings: 0, advice: 0 })).toBe("");
  });

  it("says one severity alone, leaving out the rest", () => {
    expect(problemsText({ errors: 1, warnings: 0, advice: 0 })).toBe("1 error");
    expect(problemsText({ errors: 0, warnings: 2, advice: 0 })).toBe("2 warnings");
    expect(problemsText({ errors: 0, warnings: 0, advice: 1 })).toBe("advice on 1 line");
    expect(problemsText({ errors: 0, warnings: 0, advice: 3 })).toBe("advice on 3 lines");
  });

  it("joins two with and, and all three as a list", () => {
    expect(problemsText({ errors: 0, warnings: 1, advice: 2 })).toBe(
      "1 warning and advice on 2 lines",
    );
    expect(problemsText({ errors: 2, warnings: 1, advice: 3 })).toBe(
      "2 errors, 1 warning, and advice on 3 lines",
    );
  });
});

describe("the source a diagnostic names", () => {
  it("is the server's alone with one project, and names the project with several", () => {
    expect(diagnosticSource(undefined)).toBe("ascribe");
    expect(diagnosticSource("docs")).toBe("ascribe · docs");
    expect(diagnosticSource("handbook/pages/nested")).toBe("ascribe · handbook/pages/nested");
  });

  it("is Ascribe's either way, and no other tool's", () => {
    expect(isAscribeSource("ascribe")).toBe(true);
    expect(isAscribeSource("ascribe · docs")).toBe(true);
    expect(isAscribeSource(undefined)).toBe(false);
    expect(isAscribeSource("ascribe-stub")).toBe(false);
    expect(isAscribeSource("markdownlint")).toBe(false);
  });
});
