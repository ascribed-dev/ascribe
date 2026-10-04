// The pull request's threads, apart from VS Code: what the header says about
// them (`src/preview/threadsText.ts`).
import { describe, expect, it } from "vitest";
import type { ThreadsView } from "../../src/preview/protocol.js";
import {
  againstText,
  imagesAsLinks,
  showSourceComments,
  threadsNotice,
} from "../../src/preview/threadsText.js";

const ON: ThreadsView = {
  state: "on",
  pullRequest: { number: 128, url: "https://github.com/acme/quill/pull/128", baseRefName: "main" },
  local: { state: "same", behind: 0, ahead: 0 },
  gh: false,
  message: null,
  goTo: null,
};

describe("threadsNotice", () => {
  it("says nothing while the checkout is the pull request's head", () => {
    expect(threadsNotice(ON)).toBeUndefined();
  });

  it("offers a sign-in when signed out, and the GitHub CLI when it's signed in", () => {
    const view: ThreadsView = { ...ON, state: "signed-out", pullRequest: null, local: null };
    expect(threadsNotice(view)).toEqual({
      text: "Showing changes only. Comments need GitHub.",
      actions: [{ label: "Sign in to see comments", message: "signIn" }],
    });
    expect(threadsNotice({ ...view, gh: true })?.actions.map((a) => a.label)).toEqual([
      "Sign in to see comments",
      "Use GitHub CLI",
    ]);
  });

  it("says why the comments couldn't be read, and offers to try again", () => {
    expect(
      threadsNotice({ ...ON, state: "error", message: "Couldn't reach github.com: offline" }),
    ).toEqual({
      text: "Couldn't read the comments. Couldn't reach github.com: offline",
      actions: [{ label: "Try again", message: "refresh" }],
    });
  });

  it("says how the checkout differs from the pull request, with what helps", () => {
    const local = (state: ThreadsView["local"]) => threadsNotice({ ...ON, local: state });
    expect(local({ state: "behind", behind: 1, ahead: 0 })).toEqual({
      text: "Your checkout is 1 commit behind #128, so some comments may be on lines you don't have.",
      actions: [{ label: "Pull", message: "pull" }],
    });
    expect(local({ state: "ahead", behind: 0, ahead: 2 })).toEqual({
      text: "You have 2 commits that aren't pushed. You can comment only on lines that are on GitHub.",
      actions: [{ label: "Push", message: "push" }],
    });
    expect(local({ state: "ahead", behind: 0, ahead: 1 })?.text).toContain("1 commit that isn't");
    expect(local({ state: "missing", behind: 0, ahead: 0 })?.actions).toEqual([
      { label: "Fetch", message: "fetch" },
    ]);
    expect(local({ state: "diverged", behind: 3, ahead: 1 })?.text).toContain(
      "(3 behind, 1 ahead)",
    );
  });
});

describe("againstText", () => {
  it("names the pull request when there is one", () => {
    expect(againstText(ON)).toBe("#128 against ");
    expect(againstText(null)).toBe("Against ");
    expect(againstText({ ...ON, state: "signed-out", pullRequest: null })).toBe("Against ");
  });
});

describe("showSourceComments", () => {
  it("shows threads on source lines unless the GitHub Pull Requests extension does, with auto", () => {
    expect(showSourceComments("auto", false)).toBe(true);
    expect(showSourceComments("auto", true)).toBe(false);
    expect(showSourceComments(undefined, true)).toBe(false);
    expect(showSourceComments("on", true)).toBe(true);
    expect(showSourceComments("off", false)).toBe(false);
  });
});

describe("imagesAsLinks", () => {
  it("makes each image a link, as the overlay shows it", () => {
    expect(
      imagesAsLinks("See ![the chart](https://x.test/c.png) and ![](https://x.test/d.png)."),
    ).toBe("See [Image: the chart](https://x.test/c.png) and [Image](https://x.test/d.png).");
    expect(imagesAsLinks("![ref][1]\n\n[1]: https://x.test/e.png")).toBe(
      "[Image: ref][1]\n\n[1]: https://x.test/e.png",
    );
  });

  it("leaves code, escaped images, and links alone", () => {
    const body = "`![a](b)` and \\![c](d) and [e](f)\n```\n![g](h)\n```\n``x`![i](j)`` ![k](l)";
    expect(imagesAsLinks(body)).toBe(
      "`![a](b)` and \\![c](d) and [e](f)\n```\n![g](h)\n```\n``x`![i](j)`` [Image: k](l)",
    );
  });

  it("reads a long body of unclosed code spans in one pass", () => {
    const body = "``a`".repeat(16_000);
    const start = performance.now();
    expect(imagesAsLinks(body)).toBe(body);
    expect(performance.now() - start).toBeLessThan(1000);
  });
});
