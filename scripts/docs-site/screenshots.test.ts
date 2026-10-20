// The screenshots, the site's and the docs' scenes, are there, in light and dark, and none is more than
// one minor release behind the current line of `[releases.ascribe]` in
// docs/ascribe.toml, the release the docs describe. A picture's release is in the record beside it,
// which `node scripts/docs-site/screenshots.ts <name>` writes when it retakes
// the picture; the failure names that command. One minor release is where
// the plan started: a picture of the previous release still shows what the
// current one does, two releases back may not.
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import {
  codeSources,
  currentRelease,
  FOLDER,
  PICTURES,
  SCENES,
  SCHEMES,
  stale,
  type Taken,
} from "./screenshots.ts";

const root = fileURLToPath(new URL("../..", import.meta.url));
const current = currentRelease();

describe.each(PICTURES)("the $name picture", ({ name, folder }) => {
  test("is there in each scheme", () => {
    for (const scheme of SCHEMES) {
      const file = `${folder}/${name}-${scheme}.png`;
      expect(existsSync(path.join(root, file)), `${file} is missing`).toBe(true);
    }
  });

  test("is no more than one minor release behind the current one", () => {
    const file = path.join(root, folder, `${name}.json`);
    const record = JSON.parse(readFileSync(file, "utf8")) as Taken;
    expect(stale(name, record.release, current, folder)).toBeUndefined();
  });
});

test("the site's folder holds its pictures and their records, and nothing else", () => {
  const expected = PICTURES.filter(({ folder }) => folder === FOLDER).flatMap(({ name }) => [
    `${name}.json`,
    ...SCHEMES.map((scheme) => `${name}-${scheme}.png`),
  ]);
  expect(readdirSync(path.join(root, FOLDER)).sort()).toEqual(expected.sort());
});

test("each record among the docs' images is a scene's", () => {
  const records = readdirSync(path.join(root, SCENES)).filter((file) => file.endsWith(".json"));
  const expected = PICTURES.filter(({ folder }) => folder === SCENES).map(
    ({ name }) => `${name}.json`,
  );
  expect(records.sort()).toEqual(expected.sort());
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
  expect(stale("build-lens", "0.1.0", "0.3.0", SCENES)).toBe(
    "docs/content/images/build-lens-*.png show 0.1.0, more than 1 minor release behind 0.3.0: run `node scripts/docs-site/screenshots.ts build-lens`",
  );
});

test("the current release is read from docs/ascribe.toml", () => {
  expect(
    currentRelease('spec = "0.1"\n[releases.ascribe]\nlive = ["1.4.2", "1.10.0", "1.9.3"]\n'),
  ).toBe("1.10.0");
  expect(() => currentRelease('spec = "0.1"\n')).toThrow("[releases.ascribe]");
});

test("the code sources are read from docs/ascribe.toml, each up to its first wildcard", () => {
  expect(
    codeSources(
      'spec = "0.1"\n[sources.code]\npath = ".."\ninclude = ["examples/**", "packages/astro/starter/**", "README.md", "**/*.ts"]\n',
    ),
  ).toEqual(["examples", "packages/astro/starter", "README.md"]);
  expect(codeSources()).toContain("packages/astro/starter");
});
