import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  gonePaths,
  gonePatterns,
  goneRoutes,
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
    // A retired address's rule keeps its 404.
    expect(readHostRedirects(siteRoot("/docs/beta /_ascribe/gone/docs/beta 404\n"))).toEqual([
      { from: "/docs/beta", to: "/_ascribe/gone/docs/beta", status: 404 },
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

/** A site output whose `_ascribe/redirects.json` lists `gone`. */
function withGone(gone: { path: string; reason: string; entry: string; page: string }[]): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-redirects-"));
  mkdirSync(path.join(root, "_ascribe"), { recursive: true });
  writeFileSync(
    path.join(root, "_ascribe", "redirects.json"),
    JSON.stringify({ version: 1, build: "site", redirects: [], gone }),
  );
  return root;
}

describe("gonePaths", () => {
  const beta = {
    path: "/docs/beta",
    reason: "The beta ended.",
    entry: "beta.md",
    page: "/docs/_ascribe/gone/beta",
  };

  it("gives one page for each retired address's page, with the reason and its project's root", () => {
    const first = withGone([beta, { ...beta, path: "/docs/beta.md" }]);
    const second = withGone([
      { ...beta, reason: "Another reason." },
      { path: "/old/", reason: "Gone.", entry: "/old/", page: "/_ascribe/gone/old/" },
    ]);
    const sources = [
      { siteRoot: first, home: "/docs" },
      { siteRoot: second, home: "/" },
    ];
    // A route for each build's base path, under Astro's `base`.
    expect(gonePatterns([first, second], "/")).toEqual([
      "/docs/_ascribe/gone/[...address]",
      "/_ascribe/gone/[...address]",
    ]);
    expect(gonePaths(sources, "/", "/docs/_ascribe/gone/[...address]")).toEqual([
      { params: { address: "beta" }, props: { reason: "The beta ended.", home: "/docs" } },
    ]);
    expect(gonePaths(sources, "/", "/_ascribe/gone/[...address]")).toEqual([
      { params: { address: "old" }, props: { reason: "Gone.", home: "/" } },
    ]);
    // Their routes from the host's root, which the sitemap leaves out.
    expect(goneRoutes([first, second])).toEqual(["/docs/_ascribe/gone/beta", "/_ascribe/gone/old"]);
  });

  it("puts the route under Astro's base, which is the base path", () => {
    const docs = withGone([beta]);
    expect(gonePatterns([docs], "/docs")).toEqual(["/_ascribe/gone/[...address]"]);
    expect(
      gonePaths([{ siteRoot: docs, home: "/docs" }], "/docs", "/_ascribe/gone/[...address]"),
    ).toEqual([
      { params: { address: "beta" }, props: { reason: "The beta ended.", home: "/docs" } },
    ]);
    expect(goneRoutes([docs])).toEqual(["/docs/_ascribe/gone/beta"]);
    // A folder outside Astro's base can't be served.
    expect(() => gonePatterns([docs], "/other")).toThrow(/isn't under Astro's `base`/);
  });

  it("gives none without the list, and refuses a version it doesn't know", () => {
    const none = mkdtempSync(path.join(tmpdir(), "ascribe-redirects-"));
    expect(gonePatterns([none], "/")).toEqual([]);
    const later = withGone([]);
    writeFileSync(
      path.join(later, "_ascribe", "redirects.json"),
      JSON.stringify({ version: 2, gone: [] }),
    );
    expect(() => gonePatterns([later], "/")).toThrow(/version 2/);
  });
});
