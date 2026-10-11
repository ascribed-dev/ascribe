// The site's screenshots are there, in light and dark, and none is more than
// one minor release behind `[versions] current` in docs/ascribe.toml, the
// release the docs describe. A picture's release is in the record beside it,
// which `node scripts/docs-site/screenshots.ts <name>` writes when it retakes
// the picture; the failure names that command. One minor release is where
// the plan started: a picture of the previous release still shows what the
// current one does, two releases back may not.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { currentRelease, FOLDER, PICTURES, SCHEMES, stale, type Taken } from "./screenshots.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const current = currentRelease();

describe.each(PICTURES)("the $name picture", ({ name }) => {
  test("is there in each scheme", () => {
    for (const scheme of SCHEMES) {
      const file = `${FOLDER}/${name}-${scheme}.png`;
      expect(existsSync(path.join(root, file)), `${file} is missing`).toBe(true);
    }
  });

  test("is no more than one minor release behind the current one", () => {
    const file = path.join(root, FOLDER, `${name}.json`);
    const record = JSON.parse(readFileSync(file, "utf8")) as Taken;
    expect(stale(name, record.release, current)).toBeUndefined();
  });
});

test("the folder holds the pictures and their records, and nothing else", () => {
  const expected = PICTURES.flatMap(({ name }) => [
    `${name}.json`,
    ...SCHEMES.map((scheme) => `${name}-${scheme}.png`),
  ]);
  expect(readdirSync(path.join(root, FOLDER)).sort()).toEqual(expected.sort());
});

test("a picture two minor releases back is stale, and names the command that retakes it", () => {
  expect(stale("review", "0.2.0", "0.2.0")).toBeUndefined();
  expect(stale("review", "0.1.4", "0.2.0")).toBeUndefined();
  expect(stale("review", "0.0.9", "0.2.0")).toBe(
    "site/home/content/screenshots/review-*.png show 0.0.9, more than 1 minor release behind 0.2.0: run `node scripts/docs-site/screenshots.ts review`",
  );
  expect(stale("editor", "0.9.0", "1.0.0")).toContain(
    "node scripts/docs-site/screenshots.ts editor",
  );
});

test("the current release is read from docs/ascribe.toml", () => {
  expect(currentRelease('spec = "0.1"\n[versions]\ncurrent = "1.4.2"\n')).toBe("1.4.2");
  expect(() => currentRelease('spec = "0.1"\n')).toThrow("[versions] current");
});
