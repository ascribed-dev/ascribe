import { describe, expect, it } from "vitest";
import { changedElsewhere, otherModels } from "../../src/takenFiles.js";

describe("an edit of a key [from] takes", () => {
  it("names the other projects' ascribe.toml it changed", () => {
    const changed = ["site/docs/index.md", "docs/ascribe.toml", "site/ascribe.toml"];
    expect(otherModels(changed, "site/ascribe.toml")).toEqual(["docs/ascribe.toml"]);
    expect(changedElsewhere(["docs/ascribe.toml"])).toBe(
      "Changed docs/ascribe.toml, which [from] takes it from: the edit is made where the key is written.",
    );
  });

  it("says nothing when only the project's own files change", () => {
    expect(otherModels(["ascribe.toml", "docs/a.md"], "ascribe.toml")).toEqual([]);
    expect(changedElsewhere([])).toBeUndefined();
  });
});
