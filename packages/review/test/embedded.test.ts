// The Rust crate embeds the report's script and stylesheet; its copies must
// be what `pnpm --filter @ascribed/review embed` writes from the sources.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { bundle, target } from "../embed.js";

describe("the report files the crate embeds", () => {
  it("are current", async () => {
    const { js, css, hash } = await bundle();
    const stale = "is out of date: run `pnpm --filter @ascribed/review embed`";
    expect(readFileSync(target.js, "utf8") === js, `${target.js} ${stale}`).toBe(true);
    expect(readFileSync(target.css, "utf8") === css, `${target.css} ${stale}`).toBe(true);
    expect(readFileSync(target.hash, "utf8") === hash, `${target.hash} ${stale}`).toBe(true);
  });

  it("make no network requests", async () => {
    const { js, css } = await bundle();
    expect(css).not.toMatch(/url\(\s*["']?(https?:)?\/\//);
    expect(css).not.toMatch(/@import/);
    expect(js).not.toMatch(/\bfetch\(|XMLHttpRequest|WebSocket|sendBeacon/);
  });
});
