// Takes the docs' scenes of the site and the report, each in light and dark,
// written as `<name>-light.png` and `<name>-dark.png` into the folder given:
//
//   node scripts/screenshot.ts <folder> <elements | starter-sidebar | versions | html-report>
//
// `node scripts/docs-site/screenshots.ts <name>`, at the repository's root,
// runs this and records the release beside the pictures. It needs the
// example installed, ASCRIBE_BIN, and Chromium, as the end-to-end tests find
// it.
//
// Each scene is a copy of this example, as the end-to-end tests make one,
// built with `astro build` and served with `astro preview`, or, for the
// report, `ascribe diff --format html` in a repository of its own. The
// site's scenes show the starter's layout with its own stylesheet, as `init`
// writes them.
import { execFileSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import type { Browser, Locator, Page } from "playwright-core";
import {
  BASE,
  buildSite,
  copySite,
  launchChromium,
  servePreview,
  siteDir,
} from "../test/e2e/harness.ts";
import { versioned, write } from "../test/e2e/sites.ts";

const [WIDTH, HEIGHT] = [1280, 760];
const SCHEMES = ["light", "dark"] as const;

/** Pages for the sidebar's Guides: the Teams section's own page, beside its folder, and one after it. */
const TEAMS_PAGE = "---\ntype: page\ntitle: Work in teams\n---\n\nLooms a team shares.\n";
const TROUBLESHOOT_PAGE = "---\ntype: page\ntitle: Troubleshoot Loom\n---\n\nWhen a loom won't weave.\n";

/** The element library's every element: the five note types, steps, tabs, and badges. */
const ELEMENTS_PAGE = `---
type: page
title: Install Loom
available: cloud, self-managed preview 3.4
---

@note: Loom needs a server for each team.

@note {type=tip}: Install it where your builds run.

@note {type=important}: Loom 3.4 reads \`loom.yaml\`, not \`loom.json\`.

@note {type=warning}: Installing over a running Loom stops it.

@note {type=caution}: Back up your looms first.

@steps
1. Install the command.
2. Sign in.
3. Run \`loom weave\`.

@variant {pm=npm}:
\`\`\`shell
npm install -g @loom/cli
\`\`\`
@variant {pm=pnpm}:
\`\`\`shell
pnpm add -g @loom/cli
\`\`\`
@variant {pm=yarn}:
\`\`\`shell
yarn global add @loom/cli
\`\`\`
@end

## Streaming

@available: self-managed preview 3.4
Streaming pushes changes as you save.
`;

/** Builds and serves the copy at `root`, opens `route`, and pictures `region` of it in each scheme. */
async function picture(
  root: string,
  route: string,
  region: Region,
  file: (scheme: string) => string,
): Promise<void> {
  await buildSite(root);
  const server = await servePreview(root);
  const browser = await launchChromium();
  try {
    await shoot(browser, `${server.origin}${route}`, region, file);
  } finally {
    await browser.close();
    await server.stop();
  }
}

async function shoot(
  browser: Browser,
  url: string,
  region: Region,
  file: (scheme: string) => string,
): Promise<void> {
  // Twice as many pixels each way, for a screen that has them.
  const page = await browser.newPage({
    viewport: { width: WIDTH, height: HEIGHT },
    deviceScaleFactor: 2,
  });
  await page.goto(url);
  for (const colorScheme of SCHEMES) {
    await page.emulateMedia({ colorScheme });
    await page.evaluate(() => document.fonts.ready);
    const clip = await region(page);
    await page.screenshot({ path: file(colorScheme), clip, fullPage: true, animations: "disabled" });
    console.log(`Wrote ${file(colorScheme)}`);
  }
  await page.close();
}

/** A part of the page, as a rectangle in its coordinates. */
type Region = (page: Page) => Promise<{ x: number; y: number; width: number; height: number }>;

/** The box around `locators`, the first's top to the last's bottom, with room around it. */
function around(...locators: ((page: Page) => Locator)[]): Region {
  return async (page) => {
    const boxes = [];
    for (const locator of locators) {
      const box = await locator(page).boundingBox();
      if (box === null) throw new Error(`nothing to picture: ${String(locator(page))}`);
      boxes.push(box);
    }
    const pad = 16;
    const left = Math.max(0, Math.min(...boxes.map((b) => b.x)) - pad);
    const top = Math.max(0, Math.min(...boxes.map((b) => b.y)) - pad);
    const right = Math.max(...boxes.map((b) => b.x + b.width)) + pad;
    const bottom = Math.max(...boxes.map((b) => b.y + b.height)) + pad;
    return { x: left, y: top, width: right - left, height: bottom - top };
  };
}

const SCENES: Record<string, (out: string) => Promise<void>> = {
  /** The specimen page's article, in the starter's layout. */
  async elements(out) {
    const root = await copySite("scene-elements");
    await write(root, { "content/Guides/Install.md": ELEMENTS_PAGE });
    await picture(
      root,
      `${BASE}/guides/install`,
      around((page) => page.locator("main article")),
      (scheme) => path.join(out, `elements-${scheme}.png`),
    );
  },

  /** The starter's sidebar, with Guides' nested Teams section. */
  async "starter-sidebar"(out) {
    const root = await copySite("scene-sidebar");
    await write(root, {
      "content/Guides/Teams.md": TEAMS_PAGE,
      "content/Guides/Troubleshoot.md": TROUBLESHOOT_PAGE,
    });
    await picture(
      root,
      `${BASE}/guides/teams/share-a-loom`,
      around((page) => page.getByRole("navigation", { name: "Docs" })),
      (scheme) => path.join(out, `starter-sidebar-${scheme}.png`),
    );
  },

  /** An older line's page: its banner, its title and badge, and the lines it's in. */
  async versions(out) {
    const root = await versioned("scene-versions");
    await picture(
      root,
      `${BASE}/3.4/guides/my-setup`,
      around(
        (page) => page.locator("main > :first-child"),
        (page) => page.getByRole("navigation", { name: "Versions" }),
      ),
      (scheme) => path.join(out, `versions-${scheme}.png`),
    );
  },

  /** The report of a change with each kind of mark, in Chromium. */
  async "html-report"(out) {
    const file = report();
    const browser = await launchChromium();
    try {
      await shoot(
        browser,
        `file://${file}`,
        async () => ({ x: 0, y: 0, width: WIDTH, height: HEIGHT }),
        (scheme) => path.join(out, `html-report-${scheme}.png`),
      );
    } finally {
      await browser.close();
      rmSync(path.dirname(path.dirname(path.dirname(file))), { recursive: true, force: true });
    }
  },
};

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
    {
      cwd: root,
      encoding: "utf8",
      // A fixed date, so the commits, and the report's base, are the same on every run.
      env: {
        ...process.env,
        GIT_AUTHOR_DATE: "2026-10-01T12:00:00Z",
        GIT_COMMITTER_DATE: "2026-10-01T12:00:00Z",
      },
    },
  ).trim();
}

/** The page's opening, and the two paragraphs after it at the base, which the change removes and moves. */
const OPENING = "This guide sets up {product}. Back to the [documentation home](../index.md).\n";
const READS = "Loom reads `loom.yaml` when it starts.";
const BESIDE = "Keep `loom.yaml` beside your project's root.";
const REQUIREMENTS = "## Requirements\n@id: requirements\n";

/**
 * The example's project in a repository of its own, with a change to its
 * setup page's opening that changes, adds, removes, and moves a block, and the
 * report of it: the file's path.
 */
function report(): string {
  const scratch = mkdtempSync(path.join(tmpdir(), "report-shot-"));
  const root = path.join(scratch, "repository");
  mkdirSync(root);
  cpSync(path.join(siteDir, "ascribe.toml"), path.join(root, "ascribe.toml"));
  cpSync(path.join(siteDir, "content"), path.join(root, "content"), { recursive: true });
  writeFileSync(path.join(root, ".gitignore"), ".ascribe/\n");
  const page = path.join(root, "content", "Guides", "My Setup.md");
  const text = readFileSync(page, "utf8");
  for (const part of [OPENING, REQUIREMENTS])
    if (!text.includes(part)) throw new Error(`My Setup.md no longer has: ${part}`);
  writeFileSync(page, text.replace(OPENING, `${OPENING}\n${READS}\n\n${BESIDE}\n`));
  git(root, "init", "-q", "-b", "main");
  git(root, "add", ".");
  git(root, "commit", "-qm", "The docs");
  git(root, "checkout", "-qb", "change");
  const base = readFileSync(page, "utf8");
  const now = base
    .replace(
      `${OPENING}\n${READS}\n\n${BESIDE}\n`,
      `${OPENING.replace("sets up {product}", "installs {product} and sets it up")}\nRestart Loom after you change \`loom.yaml\`.\n`,
    )
    .replace(REQUIREMENTS, `${REQUIREMENTS}\n${BESIDE}\n`);
  writeFileSync(page, now);
  git(root, "commit", "-qam", "Say where loom.yaml goes");
  const binary = process.env["ASCRIBE_BIN"] ?? "ascribe";
  const html = execFileSync(binary, ["diff", "--base", "main", "--format", "html"], {
    cwd: root,
    maxBuffer: 1 << 30,
  });
  const file = path.join(root, "report", "review.html");
  mkdirSync(path.dirname(file));
  writeFileSync(file, html);
  return file;
}

const [out, name] = process.argv.slice(2);
const scene = name === undefined ? undefined : SCENES[name];
if (out === undefined || scene === undefined) {
  console.error(`Usage: node scripts/screenshot.ts <folder> <${Object.keys(SCENES).join(" | ")}>`);
  process.exit(2);
}
mkdirSync(out, { recursive: true });
try {
  await scene(path.resolve(out));
} finally {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
}
