// Each redirect in public/_redirects goes from an address the site no longer
// serves to a page it does.
import { readFile } from "node:fs/promises";
import path from "node:path";
import { beforeAll, expect, test } from "vitest";
import { distDir, fileFor, requireBuild, siteAddress } from "./built.ts";

beforeAll(requireBuild);

/** The rules in dist/_redirects, without comments and blank lines: `[from, to, status?]`. */
async function redirects(): Promise<string[][]> {
  const text = await readFile(path.join(distDir, "_redirects"), "utf8");
  return text
    .split("\n")
    .map((line) => line.replace(/#.*/, "").trim())
    .filter((line) => line !== "")
    .map((line) => line.split(/\s+/));
}

test("the redirects are published with the site", async () => {
  await expect(redirects()).resolves.toBeInstanceOf(Array);
});

test("each redirect leads from an address that isn't a page to one that is", async () => {
  const site = await siteAddress();
  const problems: string[] = [];
  for (const [from = "", to = "", status = "301"] of await redirects()) {
    const rule = `${from} ${to}`;
    if (!/^30[1278]$/.test(status)) problems.push(`${rule}: status ${status} isn't a redirect`);
    if (!from.startsWith("/")) problems.push(`${rule}: ${from} isn't a path on the site`);
    else if ((await fileFor(from)) !== undefined) {
      problems.push(`${rule}: ${from} is still a page, so Netlify serves it instead`);
    }
    const target = new URL(to, site);
    if (target.origin !== site) problems.push(`${rule}: ${to} isn't on the site`);
    else if (!target.pathname.endsWith("/") || (await fileFor(target.pathname)) === undefined) {
      problems.push(`${rule}: ${to} isn't a page of the site`);
    }
  }
  expect(problems).toEqual([]);
});
