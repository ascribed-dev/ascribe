import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  mergeRedirects,
  netlifyRedirects,
  readHostRedirects,
  redirectsOutcome,
  type Redirect,
} from "../src/redirects.js";

const moved: Redirect = { from: "/docs/guides/my-setup", to: "/docs/guides/setup", status: 301 };
const markdown: Redirect = {
  from: "/docs/guides/my-setup.md",
  to: "/docs/guides/setup.md",
  status: 301,
};

function siteRoot(text: string): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-redirects-"));
  mkdirSync(path.join(root, "_ascribe", "host"), { recursive: true });
  writeFileSync(path.join(root, "_ascribe", "host", "_redirects"), text);
  return root;
}

describe("readHostRedirects", () => {
  it("reads the pairs in order, and none when there's no file", () => {
    expect(readHostRedirects(siteRoot(netlifyRedirects([moved, markdown])))).toEqual([
      moved,
      markdown,
    ]);
    expect(readHostRedirects(mkdtempSync(path.join(tmpdir(), "ascribe-redirects-")))).toEqual([]);
  });
});

describe("mergeRedirects", () => {
  it("keeps the first project's pair for an address two redirect", () => {
    const other = { ...moved, to: "/docs/elsewhere" };
    expect(mergeRedirects([[moved], [other, markdown]])).toEqual([moved, markdown]);
  });
});

describe("redirectsOutcome", () => {
  const sources = {
    redirects: [moved, markdown],
    base: "/",
    published: () => false,
    own: undefined,
  };

  it("writes nothing without redirects", () => {
    expect(redirectsOutcome({ ...sources, redirects: [] })).toEqual({ kind: "none" });
  });

  it("writes Netlify's lines at the root", () => {
    expect(redirectsOutcome(sources)).toEqual({
      kind: "write",
      text: "/docs/guides/my-setup /docs/guides/setup 301\n/docs/guides/my-setup.md /docs/guides/setup.md 301\n",
    });
  });

  it("keeps the site's own after Ascribe's", () => {
    const outcome = redirectsOutcome({ ...sources, own: "/old /new 302" });
    expect(outcome.kind).toBe("write");
    const text = outcome.kind === "write" ? outcome.text : "";
    expect(text.indexOf("/docs/guides/my-setup ")).toBeLessThan(text.indexOf("/old /new 302\n"));
  });

  it("gives the lines in a note under another base", () => {
    const outcome = redirectsOutcome({ ...sources, base: "/handbook/" });
    expect(outcome.kind).toBe("note");
    expect(outcome.kind === "note" && outcome.text).toContain(
      "/docs/guides/my-setup /docs/guides/setup 301",
    );
  });

  it("fails when a redirect would hide a page the site publishes", () => {
    const outcome = redirectsOutcome({
      ...sources,
      published: (route) => route === "/docs/guides/my-setup",
    });
    expect(outcome.kind).toBe("fail");
    expect(outcome.kind === "fail" && outcome.text).toContain("/docs/guides/my-setup ");
  });
});
