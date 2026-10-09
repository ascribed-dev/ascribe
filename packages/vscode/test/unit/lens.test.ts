import { describe, expect, it } from "vitest";
import type { BuildViewResult } from "../../src/shapes.js";
import { BuildLenses, lensView } from "../../src/ui/lens.js";

const range = (a: number, b: number, c: number, d: number) => ({
  start: { line: a, character: b },
  end: { line: c, character: d },
});

const answer = (extra: Partial<BuildViewResult> = {}): BuildViewResult => ({
  build: "self-hosted",
  documentVersion: 3,
  pageIncluded: true,
  pageDetail: null,
  excluded: [],
  ...extra,
});

describe("what the build lens shows", () => {
  it("dims each range the build leaves out, with a hover that names the build and says why", () => {
    const view = lensView(
      answer({
        excluded: [
          {
            range: range(42, 0, 49, 3),
            reason: "variant",
            detail: "Shows only edition=self-hosted",
          },
          {
            range: range(31, 0, 39, 120),
            reason: "availability",
            detail: "Scheduled rollouts: available on Lantern Cloud (preview), not Self-hosted 2.5",
          },
        ],
      }),
      60,
    );
    expect(view.banner).toBeUndefined();
    expect(view.dimmed).toEqual([
      {
        range: range(42, 0, 49, 3),
        hover: "**Left out of self-hosted.** Shows only edition=self-hosted.",
      },
      {
        range: range(31, 0, 39, 120),
        hover:
          "**Left out of self-hosted.** Scheduled rollouts: available on Lantern Cloud (preview), not Self-hosted 2.5.",
      },
    ]);
  });

  it("writes the server's text as text, not Markdown", () => {
    const view = lensView(
      answer({
        build: "docs_v2",
        excluded: [
          { range: range(0, 0, 1, 0), reason: "availability", detail: "Available on *a* `b`" },
        ],
      }),
      2,
    );
    expect(view.dimmed[0]?.hover).toBe("**Left out of docs\\_v2.** Available on \\*a\\* \\`b\\`.");
  });

  it("says at the top when the build leaves out the whole page, and dims all of it", () => {
    const detail =
      "The build self-hosted doesn't publish this page: its available frontmatter makes it unavailable for the build's target and version.";
    const view = lensView(answer({ pageIncluded: false, pageDetail: detail }), 12);
    expect(view.banner).toBe(detail);
    expect(view.dimmed.map((d) => d.range)).toEqual([range(0, 0, 12, 0)]);
  });

  it("shows nothing for a page the build leaves whole, or with no build", () => {
    expect(lensView(answer(), 10)).toEqual({ dimmed: [], banner: undefined });
    expect(
      lensView(
        answer({
          build: "",
          excluded: [{ range: range(0, 0, 1, 0), reason: "variant", detail: "x" }],
        }),
        10,
      ),
    ).toEqual({ dimmed: [], banner: undefined });
  });
});

describe("which projects have the lens on", () => {
  it("keeps it per project, however the folder is spelled, and says when it changes", () => {
    const lenses = new BuildLenses();
    const changed: string[] = [];
    lenses.onDidChange((folder) => changed.push(folder));
    expect(lenses.toggle("/repo/docs/")).toBe(true);
    expect(lenses.isOn("/repo/docs")).toBe(true);
    expect(lenses.isOn("/repo/handbook")).toBe(false);
    lenses.set("/repo/docs", true);
    expect(changed).toEqual(["/repo/docs/"]);
    expect(lenses.toggle("/repo/docs")).toBe(false);
    expect(lenses.isOn("/repo/docs/")).toBe(false);
    expect(changed).toEqual(["/repo/docs/", "/repo/docs"]);
    lenses.set("C:\\Repo\\Docs", true);
    expect(lenses.isOn("c:/repo/docs")).toBe(true);
  });
});
