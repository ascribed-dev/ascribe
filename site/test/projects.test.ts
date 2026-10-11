// The site's two projects: the docs, under their base path, and the site's own
// pages, from home/, at the root. What the pages outside the docs take from
// the docs is checked here: the docs' route, the permalinks they link to,
// and the install commands they repeat from getting started.
import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { beforeAll, expect, test } from "vitest";
import { docsPath } from "../src/site.ts";
import { filesUnder, requireBuild, siteAddress, siteDir } from "./built.ts";

const docsDir = path.join(siteDir, "..", "docs");
const homeDir = path.join(siteDir, "home");
/** home/ascribe.toml's site output for the `site` build: the pages outside the docs. */
const homeOutput = path.join(homeDir, ".ascribe", "build", "site", "site");

beforeAll(requireBuild);

/** A string in a project's `[consumer]`. */
async function consumer(dir: string, key: string): Promise<string> {
  const config = await readFile(path.join(dir, "ascribe.toml"), "utf8");
  const table = /^\[consumer\]$([\s\S]*?)(?=^\[|(?![\s\S]))/m.exec(config)?.[1] ?? "";
  const found = new RegExp(`^${key}\\s*=\\s*"([^"]+)"`, "m").exec(table)?.[1];
  if (found === undefined) throw new Error(`${dir}/ascribe.toml's [consumer] has no ${key}`);
  return found;
}

/** The pages outside the docs, as their site output writes them. */
async function homePages(): Promise<{ file: string; text: string }[]> {
  const files = (await filesUnder(homeOutput)).filter(
    (file) => file.endsWith(".md") && !file.startsWith("_"),
  );
  return Promise.all(
    files.map(async (file) => ({
      file,
      text: await readFile(path.join(homeOutput, file), "utf8"),
    })),
  );
}

test("the docs' route is their base path, and the site's own pages are at the root", async () => {
  // Astro's `base` is its default, `/`.
  expect(await consumer(docsDir, "base-path")).toBe(`/${docsPath}`);
  expect(await consumer(homeDir, "base-path")).toBe("/");
  expect(existsSync(path.join(siteDir, "src", "pages", docsPath, "[...slug].astro"))).toBe(true);
});

test("each permalink the pages outside the docs link to is one the docs publish", async () => {
  const site = await siteAddress();
  const permalinks = `${site}${await consumer(docsDir, "permalink-path")}`;
  const names = new Set<string>();
  const elsewhere: string[] = [];
  for (const { file, text } of await homePages()) {
    for (const [, link = ""] of text.matchAll(/\]\(([^)\s]+)/g)) {
      if (link.startsWith(permalinks)) names.add(link.slice(permalinks.length).replace(/\/$/, ""));
      // A link into the docs is a permalink, which doesn't move when a page does.
      else if (link.startsWith(`${site}/`)) elsewhere.push(`${file}: ${link}`);
    }
  }
  expect(elsewhere, "link to the docs' permalink instead").toEqual([]);
  expect([...names]).toContain("getting-started");

  // `ascribe permalinks verify` fails on a name the docs don't have, and on
  // one they retired, with the reason. The link checker sees only the first.
  const binary =
    process.env["ASCRIBE_BIN"] ??
    path.join(
      siteDir,
      "node_modules",
      ".bin",
      process.platform === "win32" ? "ascribe.cmd" : "ascribe",
    );
  const run = (args: string[]) =>
    execFileSync(
      binary,
      ["permalinks", "verify", "--config", docsDir, "--color", "never", ...args],
      {
        encoding: "utf8",
        shell: process.platform === "win32",
      },
    );
  expect(() => run([...names])).not.toThrow();
  expect(() => run(["no-such-permalink"])).toThrow();
});

test("the landing page's install commands are getting started's", async () => {
  const gettingStarted = await readFile(
    path.join(docsDir, "content", "getting-started.md"),
    "utf8",
  );
  const index = await readFile(path.join(homeDir, "content", "index.md"), "utf8");
  const commands = Array.from(index.matchAll(/^```sh\n([\s\S]*?)^```$/gm), ([, block = ""]) =>
    block.trim(),
  );
  expect(commands.length).toBe(2);
  for (const command of commands) {
    expect(gettingStarted, command).toMatch(new RegExp(`^${RegExp.escape(command)}$`, "m"));
  }
});
