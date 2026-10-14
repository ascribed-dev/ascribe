import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { crawlers, gone, stale } from "./sources.ts";

const data = readFileSync(
  new URL("../../crates/ascribe-model/data/crawlers.toml", import.meta.url),
  "utf8",
);

describe("the crawlers' sources", () => {
  it("reads every entry of the data file", () => {
    const entries = crawlers(data);
    expect(entries.length).toBeGreaterThan(0);
    for (const entry of entries) expect(entry.source).toMatch(/^https:\/\//);
  });

  it("names a source read over a year ago, or with no date", () => {
    const entries = [
      { token: "A", source: "https://a.example/", read: "2025-01-01" },
      { token: "B", source: "https://b.example/", read: "2025-12-01" },
      { token: "C", source: "https://c.example/", read: "" },
    ];
    const problems = stale(entries, new Date("2026-10-12T00:00:00Z"));
    expect(problems).toHaveLength(2);
    expect(problems[0]).toContain("`A`");
    expect(problems[1]).toContain("`C`");
  });

  it("names a source that doesn't answer, asking each address once", async () => {
    const asked: string[] = [];
    const entries = [
      { token: "A", source: "https://a.example/", read: "2026-10-12" },
      { token: "B", source: "https://a.example/", read: "2026-10-12" },
      { token: "C", source: "https://c.example/", read: "2026-10-12" },
    ];
    const problems = await gone(entries, (url) => {
      asked.push(url);
      return Promise.resolve(url.startsWith("https://c") ? 404 : 200);
    });
    expect(asked).toEqual(["https://a.example/", "https://c.example/"]);
    expect(problems).toEqual([
      "`C`: https://c.example/ answered 404. Find where its operator documents it now, or remove the token.",
    ]);
  });

  it("asks again after a failed request or a server error, and not after a 404", async () => {
    const asked: string[] = [];
    const answers = new Map<string, (number | string)[]>([
      ["https://a.example/", ["fetch failed (ECONNRESET)", 200]],
      ["https://b.example/", [503, 503]],
      ["https://c.example/", [404]],
    ]);
    let pauses = 0;
    const entries = [...answers.keys()].map((source, i) => ({
      token: "ABC"[i] ?? "",
      source,
      read: "2026-10-12",
    }));
    const problems = await gone(
      entries,
      (url) => {
        asked.push(url);
        return Promise.resolve(answers.get(url)?.shift() ?? "no answer");
      },
      () => {
        pauses += 1;
        return Promise.resolve();
      },
    );
    expect(asked).toEqual([
      "https://a.example/",
      "https://a.example/",
      "https://b.example/",
      "https://b.example/",
      "https://c.example/",
    ]);
    expect(pauses).toBe(2);
    expect(problems.map((p) => p.slice(0, 4))).toEqual(["`B`:", "`C`:"]);
  });
});
