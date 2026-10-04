// Review in the site preview: the Ascribe review app in `astro dev`'s
// toolbar, on copies of the site that are git repositories of their own, so
// `ascribe diff` has a base to compare with. Run by `pnpm test:e2e`, with
// what `e2e.test.ts` needs and `pnpm --filter @ascribed/review build`.
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Browser, Locator, Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { fakeGitHub, type FakeGitHub } from "./fake-github.js";
import { BASE, buildSite, copySite, launchChromium, serveDev, siteDir } from "./harness.js";

let browser: Browser;

beforeAll(async () => {
  browser = await launchChromium();
});

afterAll(async () => {
  await browser?.close();
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

const GUIDE = path.join("content", "Guides", "My Setup.md");

function git(root: string, ...args: string[]): string {
  return execFileSync(
    "git",
    ["-c", "user.name=Test", "-c", "user.email=test@example.com", ...args],
    { cwd: root, encoding: "utf8" },
  ).trim();
}

/** A copy of the site committed on `main` in a repository of its own, checked out on `change`. */
async function gitSite(name: string): Promise<string> {
  const root = await copySite(name);
  git(root, "init", "-q", "-b", "main");
  await writeFile(path.join(root, ".gitignore"), ".ascribe/\n.astro/\ndist/\n");
  git(root, "add", ".");
  git(root, "commit", "-qm", "The site");
  git(root, "checkout", "-qb", "change");
  return root;
}

async function edit(file: string, change: (text: string) => string): Promise<void> {
  await writeFile(file, change(await readFile(file, "utf8")));
}

/** Adds a paragraph before the guide's "Weave configuration" section. */
function addParagraph(text: string): string {
  return text.replace("## Weave configuration", "A paragraph about weaving.\n\n## Weave configuration");
}

/** The toolbar's button for the app. */
function appButton(page: Page): Locator {
  return page.locator('astro-dev-toolbar button.item[data-app-id="ascribe:review"]');
}

/** The app's panel, in its canvas. */
function panel(page: Page): Locator {
  return page.locator('astro-dev-toolbar-app-canvas[data-app-id="ascribe:review"] .panel');
}

/** Opens a page of the dev server and turns review on from the toolbar. */
async function startReview(origin: string, route: string): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
  const response = await page.goto(`${origin}${route}`);
  expect(response?.status(), route).toBe(200);
  await appButton(page).click();
  await expect
    .poll(async () => panel(page).textContent(), { timeout: 60_000 })
    .not.toContain("Starting review");
  return page;
}

describe("review in the site preview", () => {
  it("marks a working-tree change on the real page", async () => {
    const root = await gitSite("review-marks");
    await edit(path.join(root, GUIDE), addParagraph);
    const server = await serveDev(root);
    try {
      const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
      const added = page.locator('[data-ascribe-change="added"]');
      await expect.poll(() => added.count(), { timeout: 30_000 }).toBe(1);
      await expect(added.textContent()).resolves.toContain("A paragraph about weaving.");
      const text = await panel(page).textContent();
      expect(text).toContain("Against main");
      expect(text).toContain("1 change on this page");
      // No pull request: the app says how to get comments.
      expect(text).toContain("Showing changes only.");
      // Stepping goes to the change.
      await panel(page).getByRole("button", { name: "Next change" }).click();
      await expect.poll(async () => panel(page).textContent()).toContain("1 of 1 on this page");
      // dev.json says where the dev server is, for the editor.
      const devFile = JSON.parse(
        readFileSync(path.join(root, ".ascribe", "dev.json"), "utf8"),
      ) as { url: string; build: string };
      expect(devFile.build).toBe("site");
      expect(new URL(devFile.url).port).toBe(new URL(server.origin).port);
      expect(new URL(devFile.url).pathname).toBe(`${BASE}/`);
      await page.close();
    } finally {
      await server.stop();
    }
    if (process.platform === "win32") {
      // Windows has no SIGTERM: stopping the server ends it at once, before
      // any exit handler runs, as a crash would. Its dev.json may stay, and
      // its address no longer answers, which is what the editor checks.
      await expect(fetch(server.origin)).rejects.toThrow();
    } else {
      expect(existsSync(path.join(root, ".ascribe", "dev.json"))).toBe(false);
    }
  });

  it("marks nothing until review is turned on", async () => {
    const root = await gitSite("review-off");
    await edit(path.join(root, GUIDE), addParagraph);
    const server = await serveDev(root);
    try {
      const page = await browser.newPage();
      await page.goto(`${server.origin}${BASE}/guides/my-setup`);
      await expect.poll(() => appButton(page).count(), { timeout: 30_000 }).toBe(1);
      await page.waitForTimeout(1000);
      await expect(page.locator("[data-ascribe-change]").count()).resolves.toBe(0);
      await page.close();
    } finally {
      await server.stop();
    }
  });

  it("says when a route isn't an Ascribe page, and lists the changed pages", async () => {
    const root = await gitSite("review-not-page");
    await edit(path.join(root, GUIDE), addParagraph);
    await writeFile(
      path.join(root, "src", "pages", "changelog.astro"),
      "<html><head><title>Changelog</title></head><body><h1>Changelog</h1></body></html>\n",
    );
    const server = await serveDev(root);
    try {
      const page = await startReview(server.origin, `${BASE}/changelog`);
      const text = await panel(page).textContent();
      expect(text).toContain("Nothing to review here");
      const link = panel(page).getByRole("link", { name: `${BASE}/guides/my-setup` });
      // On macOS the watcher can report the edits made before the server
      // started, and the rebuild's full reload cancels a navigation that's
      // under way: click again until the page goes.
      const target = `${BASE}/guides/my-setup`;
      await expect
        .poll(
          async () => {
            if (new URL(page.url()).pathname !== target) {
              await link.click({ timeout: 5_000 }).catch(() => undefined);
              await page.waitForURL(`**${target}`, { timeout: 5_000 }).catch(() => undefined);
            }
            return new URL(page.url()).pathname;
          },
          { timeout: 30_000 },
        )
        .toBe(target);
      await page.waitForLoadState("load");
      // Review stays on, and the panel as it was.
      await expect
        .poll(() => page.locator('[data-ascribe-change="added"]').count(), { timeout: 30_000 })
        .toBe(1);
      await expect.poll(async () => panel(page).isVisible()).toBe(true);
      await page.close();
    } finally {
      await server.stop();
    }
  });

  it("lists the changes when a layout drops the anchors", async () => {
    const root = await gitSite("review-no-anchors");
    await edit(path.join(root, GUIDE), addParagraph);
    // A layout that rebuilds the content's elements without their attributes.
    await edit(path.join(root, "src", "layouts", "Docs.astro"), (text) =>
      text.replace(
        "</body>",
        `<script is:inline>for (const el of document.querySelectorAll("[data-ascribe-source]")) el.removeAttribute("data-ascribe-source");</script></body>`,
      ),
    );
    const server = await serveDev(root);
    try {
      const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
      const text = await panel(page).textContent();
      expect(text).toContain("This page has 1 change, but its blocks carry no source anchors.");
      expect(text).toContain("dropping the data-ascribe-source attributes");
      expect(text).toContain("added Guides/My Setup.md:");
      await page.close();
    } finally {
      await server.stop();
    }
  });

  describe.skipIf(process.platform === "win32")("with a pull request", () => {
    let github: FakeGitHub | undefined;
    afterAll(() => github?.remove());

    it("shows a thread beside its block, and sends a comment made there on submit", async () => {
      const root = await gitSite("review-threads");
      git(root, "remote", "add", "origin", "https://github.com/acme/docs.git");
      git(root, "update-ref", "refs/remotes/origin/main", "main");
      await edit(path.join(root, GUIDE), addParagraph);
      git(root, "commit", "-qam", "A paragraph");
      git(root, "config", "branch.change.remote", "origin");
      git(root, "config", "branch.change.merge", "refs/heads/change");
      git(root, "update-ref", "refs/remotes/origin/change", "change");
      const guide = "content/Guides/My Setup.md";
      const lines = (await readFile(path.join(root, GUIDE), "utf8")).split("\n");
      const intro = lines.findIndex((l) => l.startsWith("This guide sets up")) + 1;
      const added = lines.findIndex((l) => l.startsWith("A paragraph about weaving")) + 1;
      github = fakeGitHub({
        pullRequest: {
          id: "PR_1",
          number: 7,
          owner: "acme",
          name: "docs",
          headRefName: "change",
          headRefOid: git(root, "rev-parse", "change"),
          baseRefName: "main",
          baseRefOid: git(root, "rev-parse", "main"),
          files: [guide],
        },
        threads: [
          {
            id: "PRRT_0",
            path: guide,
            line: intro,
            comments: [
              { id: "PRRC_0", body: "Say what Loom is first.", state: "SUBMITTED", login: "maya" },
            ],
          },
        ],
        review: null,
        submitted: [],
      });
      const server = await serveDev(root, { PATH: github.path });
      try {
        const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
        await expect
          .poll(async () => panel(page).textContent(), { timeout: 30_000 })
          .toContain("#7 against origin/main");
        // The thread, beside the intro paragraph.
        const card = page.locator("[data-ascribe-overlay] .thread", {
          hasText: "Say what Loom is first.",
        });
        await expect.poll(() => card.count(), { timeout: 30_000 }).toBe(1);
        // A comment on the new paragraph.
        const block = page.locator('[data-ascribe-change="added"]');
        await block.hover();
        await page.getByRole("button", { name: "Comment on this block" }).click();
        await page.getByRole("textbox", { name: "Comment" }).fill("Is this still true?");
        await page
          .getByRole("group", { name: /^Comment on/ })
          .getByRole("button", { name: "Add to review" })
          .click();
        await expect
          .poll(async () => panel(page).textContent(), { timeout: 30_000 })
          .toContain("1 unsent comment");
        expect(github.state().submitted).toEqual([]);
        await panel(page).getByRole("button", { name: "Submit review…" }).click();
        await page.getByRole("dialog", { name: "Submit review" }).getByRole("button", { name: "Submit review" }).click();
        await expect.poll(() => github?.state().submitted.length, { timeout: 30_000 }).toBe(1);
        const made = github.state().threads.find((t) => t.comments[0]?.body === "Is this still true?");
        expect(made).toMatchObject({ path: guide, line: added });
        expect(made?.comments[0]?.state).toBe("SUBMITTED");
        await page.close();
      } finally {
        await server.stop();
      }
    });
  });
});

describe("astro build", () => {
  it("has no anchors, no overlay, and no review code by default", async () => {
    const root = await copySite("review-build");
    await buildSite(root);
    const dist = path.join(root, "dist");
    const files = readdirSync(dist, { recursive: true, encoding: "utf8" })
      .filter((f) => /\.(html|js|css)$/.test(f))
      .map((f) => path.join(dist, f));
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) {
      const text = await readFile(file, "utf8");
      for (const needle of ["data-ascribe-source", "data-ascribe-via", "ascribe:review", "data-ascribe-overlay", "ascribe-marks"]) {
        expect(text.includes(needle), `${needle} in ${file}`).toBe(false);
      }
    }
  });
});
