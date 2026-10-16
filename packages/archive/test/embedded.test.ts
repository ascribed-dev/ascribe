// The Rust crate embeds the archive's search script; its copy must be what
// `pnpm --filter @ascribed/archive embed` writes from the sources.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { bundle, target } from "../embed.js";

describe("the search script the crate embeds", () => {
  it("is current", async () => {
    const js = await bundle();
    const stale = "is out of date: run `pnpm --filter @ascribed/archive embed`";
    expect(readFileSync(target, "utf8") === js, `${target} ${stale}`).toBe(true);
  });

  it("makes no network request and stores nothing", async () => {
    const js = await bundle();
    expect(js).not.toMatch(/\bfetch\(|XMLHttpRequest|WebSocket|sendBeacon|EventSource/);
    expect(js).not.toMatch(/localStorage|sessionStorage|indexedDB|document\.cookie/);
  });
});
