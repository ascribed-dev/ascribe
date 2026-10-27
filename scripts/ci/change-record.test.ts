// The change-record check's pure parts. The script itself runs in CI on every
// pull request (ci.yml's `record` job).
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  type Changed,
  NOT_USER_VISIBLE,
  parseNameStatus,
  review,
  saysNotUserVisible,
} from "./change-record.ts";

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");

const added = (path: string): Changed => ({ status: "A", path });
const modified = (path: string): Changed => ({ status: "M", path });
const deleted = (path: string): Changed => ({ status: "D", path });

it("reads git's name-status lines", () => {
  expect(parseNameStatus("M\tSPEC.md\nA\tchanges/fixes/x.md\nT100\tlink\n\n")).toEqual([
    modified("SPEC.md"),
    added("changes/fixes/x.md"),
    { status: "T", path: "link" },
  ]);
});

describe("a contract", () => {
  it.each(["SPEC.md", "docs/content/contracts/json.md", "packages/elements/CONTRACT.md"])(
    "%s changed with no decision fails",
    (path) => {
      const { errors } = review([modified(path)], "");
      expect(errors).toHaveLength(1);
      expect(errors[0]).toContain(path);
    },
  );

  it("changed with a decision added or changed passes", () => {
    expect(
      review([modified("SPEC.md"), added("project-docs/decisions/991-x.md")], "").errors,
    ).toEqual([]);
    expect(
      review([modified("SPEC.md"), modified("project-docs/decisions/381-x.md")], "").errors,
    ).toEqual([]);
  });

  it("isn't decided by the folder's README or a deleted decision", () => {
    expect(
      review([modified("SPEC.md"), modified("project-docs/decisions/README.md")], "").errors,
    ).toHaveLength(1);
    expect(
      review([modified("SPEC.md"), deleted("project-docs/decisions/381-x.md")], "").errors,
    ).toHaveLength(1);
  });

  it("is only those files", () => {
    expect(
      review([modified("docs/content/SPEC.md"), modified("packages/review/CONTRACT.md")], "")
        .errors,
    ).toEqual([]);
  });
});

describe("a change to the source", () => {
  const source = [
    modified("crates/ascribe-core/src/lib.rs"),
    modified("packages/astro/src/index.ts"),
  ];

  it("with no changelog entry warns, and doesn't fail", () => {
    const { errors, warnings } = review(source, "");
    expect(errors).toEqual([]);
    expect(warnings).toHaveLength(1);
    expect(warnings[0]).toContain("2 files");
  });

  it("with an entry added doesn't", () => {
    expect(review([...source, added("changes/command/check-fix.md")], "").warnings).toEqual([]);
  });

  it("with an entry only changed, or the README added, still warns", () => {
    expect(review([...source, modified("changes/command/check-fix.md")], "").warnings).toHaveLength(
      1,
    );
    expect(review([...source, added("changes/README.md")], "").warnings).toHaveLength(1);
  });

  it("with the description's line doesn't", () => {
    const description = `## What this changes and why\r\n\r\n${NOT_USER_VISIBLE} a rename inside the crate.\r\n`;
    expect(review(source, description).warnings).toEqual([]);
  });

  it("outside a crate's or package's src/ doesn't", () => {
    expect(
      review([modified("crates/ascribe-core/tests/all/x.rs"), modified("scripts/check.ts")], "")
        .warnings,
    ).toEqual([]);
  });
});

describe("the description's line", () => {
  it("starts a line and says why", () => {
    expect(saysNotUserVisible(`${NOT_USER_VISIBLE} only the tests`)).toBe(true);
    expect(saysNotUserVisible(`${NOT_USER_VISIBLE}`)).toBe(false);
    expect(saysNotUserVisible(`${NOT_USER_VISIBLE}   `)).toBe(false);
    expect(saysNotUserVisible(`It's ${NOT_USER_VISIBLE} really`)).toBe(false);
    expect(saysNotUserVisible("not user-visible: lowercase")).toBe(false);
  });

  it("is the wording CONTRIBUTING.md gives", () => {
    expect(read("CONTRIBUTING.md")).toContain(`\`${NOT_USER_VISIBLE}\``);
  });
});

it("runs on every pull request, and counts toward `all checks`", () => {
  const ci = read(".github/workflows/ci.yml");
  expect(ci).toContain("node scripts/ci/change-record.ts");
  expect(ci).toMatch(/name: all checks\n {4}needs: \[[^\]]*\brecord\b/);
});
