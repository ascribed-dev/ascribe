import { describe, expect, it } from "vitest";
import { nonCodeEntrypoints } from "./consumers.ts";

describe("nonCodeEntrypoints", () => {
  it("names the stylesheets and components, not the JavaScript", () => {
    expect(
      nonCodeEntrypoints({
        ".": { types: "./dist/index.d.ts", default: "./dist/index.js" },
        "./style.css": "./css/style.css",
        "./Elements.astro": "./src/Elements.astro",
        "./plain": "./dist/plain.js",
        "./module": "./dist/module.mjs",
      }),
    ).toEqual(["./style.css", "./Elements.astro"]);
  });
});
