import { mkdirSync, mkdtempSync, utimesSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  headersFor,
  headersOutcome,
  mergeRules,
  missingRules,
  netlifyHeaders,
  readHeaders,
  readHostHeaders,
  type HeaderRule,
} from "../src/headers.js";

const unlisted: HeaderRule = {
  path: "/docs/internal/runbook.md",
  headers: {
    Link: '<https://docs.example.com/docs/internal/runbook/>; rel="canonical"',
    "X-Robots-Tag": "noindex",
  },
};
const listed: HeaderRule = {
  path: "/docs/guides/install.md",
  headers: { Link: '<https://docs.example.com/docs/guides/install/>; rel="canonical"' },
};

function siteRoot(list: unknown): string {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-headers-"));
  mkdirSync(path.join(root, "_ascribe"));
  writeFileSync(path.join(root, "_ascribe", "headers.json"), JSON.stringify(list));
  return root;
}

describe("readHeaders", () => {
  it("reads the list, and nothing when there's none", () => {
    expect(readHeaders(siteRoot({ version: 1, headers: [listed] }))).toEqual([listed]);
    expect(readHeaders(mkdtempSync(path.join(tmpdir(), "ascribe-headers-")))).toBeUndefined();
  });

  it("refuses a version it doesn't know", () => {
    expect(() => readHeaders(siteRoot({ version: 2, headers: [] }))).toThrow(/version 2/);
  });
});

describe("readHostHeaders", () => {
  it("reads back the host's file a build writes, and nothing when there's none", () => {
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-headers-"));
    expect(readHostHeaders(root)).toBeUndefined();
    mkdirSync(path.join(root, "_ascribe", "host"), { recursive: true });
    writeFileSync(
      path.join(root, "_ascribe", "host", "_headers"),
      netlifyHeaders(mergeRules([[listed, unlisted]])),
    );
    expect(readHostHeaders(root)).toEqual([listed, unlisted]);
  });
});

describe("headersFor", () => {
  it("gives a path its rule's headers, and none to a path without one", () => {
    const root = siteRoot({ version: 1, headers: [unlisted, listed] });
    expect(headersFor(root, unlisted.path)).toEqual(unlisted.headers);
    expect(headersFor(root, "/docs/internal/runbook/")).toEqual({});
  });

  it("reads the list again when a build rewrites it", () => {
    const root = siteRoot({ version: 1, headers: [unlisted] });
    expect(headersFor(root, unlisted.path)["X-Robots-Tag"]).toBe("noindex");
    const file = path.join(root, "_ascribe", "headers.json");
    writeFileSync(file, JSON.stringify({ version: 1, headers: [] }));
    utimesSync(file, new Date(), new Date(Date.now() + 5000));
    expect(headersFor(root, unlisted.path)).toEqual({});
  });
});

describe("netlifyHeaders", () => {
  it("writes each path with its headers indented below it", () => {
    expect(netlifyHeaders(mergeRules([[unlisted, listed]]))).toBe(
      "/docs/guides/install.md\n" +
        '  Link: <https://docs.example.com/docs/guides/install/>; rel="canonical"\n' +
        "/docs/internal/runbook.md\n" +
        '  Link: <https://docs.example.com/docs/internal/runbook/>; rel="canonical"\n' +
        "  X-Robots-Tag: noindex\n",
    );
  });

  it("keeps the first project's rule for a path two share", () => {
    const other = { path: listed.path, headers: { "X-Robots-Tag": "noindex" } };
    expect(mergeRules([[listed], [other, unlisted]])).toEqual([listed, unlisted]);
  });
});

describe("missingRules", () => {
  it("is empty when the site's file has every line, whatever the case of a name", () => {
    const own =
      "# the site's own\n/*\n  X-Frame-Options: DENY\n" +
      "/docs/internal/runbook.md\n" +
      '  link: <https://docs.example.com/docs/internal/runbook/>; rel="canonical"\n' +
      "  x-robots-tag:noindex\n" +
      "/docs/guides/install.md\n" +
      '\tLink: <https://docs.example.com/docs/guides/install/>; rel="canonical"\n';
    expect(missingRules(own, [unlisted, listed])).toEqual([]);
  });

  it("names the lines a path lacks, and a rule under another path doesn't count", () => {
    const own =
      "/docs/internal/runbook.md\n" +
      '  Link: <https://docs.example.com/docs/internal/runbook/>; rel="canonical"\n' +
      "/docs/*\n  X-Robots-Tag: noindex\n";
    expect(missingRules(own, [unlisted, listed])).toEqual([
      { path: unlisted.path, headers: { "X-Robots-Tag": "noindex" } },
      listed,
    ]);
  });
});

describe("headersOutcome", () => {
  const rules = [listed, unlisted];

  it("writes _headers for a site without one", () => {
    expect(headersOutcome({ rules, base: "/", own: undefined })).toEqual({
      kind: "write",
      text: netlifyHeaders(rules),
    });
  });

  it("writes nothing when there are no rules", () => {
    expect(headersOutcome({ rules: [], base: "/", own: undefined })).toEqual({ kind: "none" });
  });

  it("leaves the site's own alone when it has the lines", () => {
    const outcome = headersOutcome({ rules, base: "/", own: netlifyHeaders(rules) });
    expect(outcome.kind).toBe("note");
  });

  it("fails with the lines to add when the site's own lacks them", () => {
    const own = netlifyHeaders([listed]);
    const outcome = headersOutcome({ rules, base: "/", own });
    expect(outcome.kind).toBe("fail");
    expect(outcome.kind === "fail" && outcome.text).toBe(
      "public/_headers is the site's own, and lacks the headers Ascribe's pages need. Add these lines to it:\n" +
        netlifyHeaders([unlisted]),
    );
  });

  it("says the lines instead of writing them under a base that isn't the root", () => {
    const outcome = headersOutcome({ rules, base: "/docs", own: undefined });
    expect(outcome.kind).toBe("note");
    expect(outcome.kind === "note" && outcome.text).toContain(netlifyHeaders(rules));
  });
});
