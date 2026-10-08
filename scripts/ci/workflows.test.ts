// What the workflows in .github/workflows/ must keep true, and nothing but a
// test would notice breaking.
//
// The JavaScript checks run in Playwright's container image, which has the
// browsers and their system packages installed already. A browser build
// belongs to one playwright-core version, so the image's tag has to be the
// version pnpm-lock.yaml installs: a mismatch starts no browser.
//
// A job with no timeout waits six hours for a download that stalled.
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, it } from "vitest";

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");
const workflows = readdirSync(fileURLToPath(new URL(".github/workflows/", root))).filter((name) =>
  name.endsWith(".yml"),
);

it("Playwright's image is the playwright-core version the lockfile installs", () => {
  const installed = new Set(
    Array.from(read("pnpm-lock.yaml").matchAll(/^ {2}playwright-core@([\d.]+):$/gm), ([, v]) => v),
  );
  expect(installed.size, "one playwright-core version in pnpm-lock.yaml").toBe(1);
  const images = workflows.flatMap((name) =>
    Array.from(
      read(`.github/workflows/${name}`).matchAll(/mcr\.microsoft\.com\/playwright:v([\d.]+)-/g),
      ([, v]) => v,
    ),
  );
  expect(images.length).toBeGreaterThan(0);
  // To fix: change the image's tag and its digest together, from
  // `docker buildx imagetools inspect mcr.microsoft.com/playwright:v<version>-noble`.
  expect(new Set(images)).toEqual(installed);
});

it("every job has a timeout", () => {
  const without: string[] = [];
  for (const name of workflows) {
    const lines = read(`.github/workflows/${name}`).split("\n");
    lines.forEach((line, i) => {
      const indent = /^( +)runs-on: /.exec(line)?.[1];
      if (indent === undefined) return;
      // The job's own keys are the lines at this indent, up to the next job.
      const rest = lines.slice(lines.findLastIndex((l, j) => j < i && /^ {2}\S/.test(l)) + 1);
      const end = rest.findIndex((l) => /^ {2}\S/.test(l));
      const job = end === -1 ? rest : rest.slice(0, end);
      if (!job.some((l) => l.startsWith(`${indent}timeout-minutes: `))) {
        without.push(`${name}:${i + 1}`);
      }
    });
  }
  expect(without).toEqual([]);
});
