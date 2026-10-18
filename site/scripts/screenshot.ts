// Takes the site's picture of review: a pull request's changes marked on a
// docs page in `astro dev`, with its review comment beside the block it's
// about and the toolbar's panel open. One picture per scheme, written as
// `review-light.png` and `review-dark.png` into the folder given.
//
//   node scripts/screenshot.ts <folder>
//
// `node scripts/docs-site/screenshots.ts review`, at the repository's root,
// runs this and records the release beside the pictures. It needs the site
// installed with the Ascribe to show (`npm run build:checkout`, or `npm ci`),
// Chromium, as the site's tests find it, and a POSIX system for the fake `gh`.
//
// The pull request is the docs and the site as committed at HEAD, copied
// into a repository of their own with one change on a branch, and its comment
// comes from the end-to-end suite's fake GitHub. The clock is fixed, so the
// comment's age reads the same on every run.
import { execFileSync, spawn } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { fakeGitHub, type FakeGitHub } from "../../examples/astro-site/test/e2e/fake-github.ts";
import { codeSources } from "../../scripts/docs-site/screenshots.ts";
import { launchChromium } from "../test/built.ts";

const [WIDTH, HEIGHT] = [1280, 760];
const siteDir = fileURLToPath(new URL("..", import.meta.url));
const repositoryRoot = path.dirname(siteDir);

/** The page, what the pull request changes in it, and the comment on the change. */
const PAGE = "docs/content/getting-started.md";
const ROUTE = "/docs/getting-started/";
const FROM = "and previews pages as the site shows them.";
const TO = "and previews each page as your site will show it.";
const COMMENTS = [
  { body: "Better. Should the editor guide say it the same way?", login: "maya" },
  { body: "Yes, I'll change it there too.", login: "sam" },
];
/** A day after the fake's comments, which are all at noon on 1 October. */
const NOW = new Date("2026-10-02T15:00:00Z");

/**
 * What the site builds from at HEAD: the docs and the site; the changelog its
 * page draws, with the entries in `changes/` and the script that gathers them
 * (`src/changelog.ts`); and the files the docs and home/ take code examples
 * from, read from `[sources.code] include` in docs/ascribe.toml.
 */
const TREE = [
  "docs",
  "site",
  "CHANGELOG.md",
  "changes",
  "scripts/release/changes.ts",
  ...codeSources(),
];

function git(root: string, ...args: string[]): string {
  return execFileSync(
    "git",
    [
      "-c",
      "user.name=Test",
      "-c",
      "user.email=test@example.com",
      "-c",
      "commit.gpgsign=false",
      ...args,
    ],
    { cwd: root, encoding: "utf8" },
  ).trim();
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address !== null ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

/** A copy of HEAD's docs and site on a branch with pull request #7, and its fake GitHub. */
function pullRequest(scratch: string): { root: string; github: FakeGitHub } {
  const root = path.join(scratch, "repository");
  mkdirSync(root);
  const archive = execFileSync("git", ["archive", "--format=tar", "HEAD", "--", ...TREE], {
    cwd: repositoryRoot,
    maxBuffer: 1 << 30,
  });
  execFileSync("tar", ["-x", "-C", root], { input: archive });
  // The site's installed packages, which the copy builds with: copied, since
  // Astro's dev server doesn't follow a link to them.
  cpSync(path.join(siteDir, "node_modules"), path.join(root, "site", "node_modules"), {
    recursive: true,
    verbatimSymlinks: true,
  });
  writeFileSync(path.join(root, ".gitignore"), "node_modules\n.ascribe/\n.astro/\ndist/\n");
  git(root, "init", "-q", "-b", "main");
  git(root, "add", ".");
  git(root, "commit", "-qm", "The docs");
  git(root, "checkout", "-qb", "change");
  git(root, "remote", "add", "origin", "https://github.com/acme/docs.git");
  git(root, "update-ref", "refs/remotes/origin/main", "main");
  const file = path.join(root, PAGE);
  const text = readFileSync(file, "utf8");
  if (!text.includes(FROM)) throw new Error(`${PAGE} no longer says: ${FROM}`);
  writeFileSync(file, text.replace(FROM, TO));
  git(root, "commit", "-qam", "Say what the preview shows");
  git(root, "config", "branch.change.remote", "origin");
  git(root, "config", "branch.change.merge", "refs/heads/change");
  git(root, "update-ref", "refs/remotes/origin/change", "change");
  const line = text.split("\n").findIndex((l) => l.includes(FROM)) + 1;
  const github = fakeGitHub({
    pullRequest: {
      id: "PR_1",
      number: 7,
      owner: "acme",
      name: "docs",
      headRefName: "change",
      headRefOid: git(root, "rev-parse", "change"),
      baseRefName: "main",
      baseRefOid: git(root, "rev-parse", "main"),
      files: [PAGE],
    },
    threads: [
      {
        id: "PRRT_0",
        path: PAGE,
        line,
        comments: COMMENTS.map((c, i) => ({ id: `PRRC_${i}`, state: "SUBMITTED", ...c })),
      },
    ],
    review: null,
    submitted: [],
  });
  return { root, github };
}

/** `astro dev` in the copy's site, with the fake `gh` first on its path. */
async function serve(
  root: string,
  github: FakeGitHub,
): Promise<{ origin: string; stop(): Promise<void> }> {
  const port = await freePort();
  // Astro's own API, as the end-to-end suite's dev server uses it.
  const script = [
    'import { dev } from "astro";',
    `await dev({ root: process.cwd(), logLevel: "warn", server: { host: "127.0.0.1", port: ${port} } });`,
    'process.on("SIGTERM", () => process.exit(0));',
  ].join("\n");
  const child = spawn(process.execPath, ["--input-type=module", "-e", script], {
    cwd: path.join(root, "site"),
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, PATH: github.path, NODE_ENV: "development" },
  });
  let output = "";
  child.stdout.on("data", (chunk: Buffer) => (output += chunk.toString()));
  child.stderr.on("data", (chunk: Buffer) => (output += chunk.toString()));
  const origin = `http://127.0.0.1:${port}`;
  const stop = (): Promise<void> =>
    new Promise((resolve) => {
      if (child.exitCode !== null) return resolve();
      child.once("exit", () => resolve());
      child.kill("SIGTERM");
    });
  const deadline = Date.now() + 120_000;
  for (;;) {
    if (child.exitCode !== null)
      throw new Error(`astro dev exited with ${child.exitCode}:\n${output}`);
    const up = await fetch(`${origin}${ROUTE}`).then(
      (response) => response.ok,
      () => false,
    );
    if (up) return { origin, stop };
    if (Date.now() > deadline) {
      await stop();
      throw new Error(`astro dev didn't answer in two minutes:\n${output}`);
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }
}

async function main(out: string): Promise<void> {
  const scratch = mkdtempSync(path.join(tmpdir(), "review-shot-"));
  const { root, github } = pullRequest(scratch);
  const server = await serve(root, github);
  const browser = await launchChromium();
  try {
    // Twice as many pixels each way, for a screen that has them.
    const page = await browser.newPage({
      viewport: { width: WIDTH, height: HEIGHT },
      deviceScaleFactor: 2,
    });
    await page.clock.setFixedTime(NOW);
    await page.emulateMedia({ colorScheme: "light" });
    await page.goto(`${server.origin}${ROUTE}`);
    await page.locator('astro-dev-toolbar button.item[data-app-id="ascribe:review"]').click();
    const panel = page.locator('astro-dev-toolbar-app-canvas[data-app-id="ascribe:review"] .panel');
    await panel.filter({ hasText: "#7 against main" }).waitFor({ timeout: 120_000 });
    const changed = page.locator('[data-ascribe-change="changed"]').first();
    await changed.waitFor({ timeout: 60_000 });
    await page
      .locator("[data-ascribe-overlay] .thread", { hasText: COMMENTS[0]?.body ?? "" })
      .first()
      .waitFor({ timeout: 60_000 });
    // The changed block near the top, with its section's heading above it.
    await changed.evaluate((block) => {
      window.scrollTo(0, block.getBoundingClientRect().top + window.scrollY - 250);
    });
    await page.mouse.move(WIDTH / 2, 10);
    // Review follows the page into the other scheme, on the same page.
    for (const colorScheme of ["light", "dark"] as const) {
      await page.emulateMedia({ colorScheme });
      await panel.and(page.locator(`[data-scheme="${colorScheme}"]`)).waitFor();
      // The overlay places its cards after a scroll or a change of style, and
      // the page's colors finish their transitions.
      await page.waitForTimeout(3000);
      const file = path.join(out, `review-${colorScheme}.png`);
      await page.screenshot({ path: file, animations: "disabled", caret: "hide" });
      console.log(`Wrote ${file}`);
    }
  } finally {
    await browser.close();
    await server.stop();
    github.remove();
    rmSync(scratch, { recursive: true, force: true });
  }
}

const out = process.argv[2];
if (out === undefined) {
  console.error("Usage: node scripts/screenshot.ts <folder>");
  process.exit(2);
}
mkdirSync(out, { recursive: true });
await main(path.resolve(out));
