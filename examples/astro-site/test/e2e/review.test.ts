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
import { fakeGitHub, type FakeGitHub, type FakeThread } from "./fake-github.js";
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
async function startReview(
  origin: string,
  route: string,
  colorScheme: "light" | "dark" = "light",
): Promise<Page> {
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, colorScheme });
  const response = await page.goto(`${origin}${route}`);
  expect(response?.status(), route).toBe(200);
  await appButton(page).click();
  // The panel has drawn, and review has started. (On macOS a delayed watcher
  // event can reload the page once more, so a panel seen empty isn't done.)
  await expect
    .poll(
      async () => {
        const text = (await panel(page).textContent().catch(() => null)) ?? "";
        return text !== "" && !text.includes("Starting review");
      },
      { timeout: 60_000 },
    )
    .toBe(true);
  return page;
}

describe("review in the site preview", () => {
  it("marks a working-tree change on the real page", async () => {
    const root = await gitSite("review-marks");
    await edit(path.join(root, GUIDE), addParagraph);
    const server = await serveDev(root);
    try {
      // The system is dark, but the site is light only: review's colors follow the site.
      const page = await startReview(server.origin, `${BASE}/guides/my-setup`, "dark");
      const added = page.locator('[data-ascribe-change="added"]');
      await expect.poll(() => added.count(), { timeout: 30_000 }).toBe(1);
      await expect(added.textContent()).resolves.toContain("A paragraph about weaving.");
      await expect(page.locator("html").getAttribute("data-ascribe-scheme")).resolves.toBe("light");
      await expect(panel(page).getAttribute("data-scheme")).resolves.toBe("light");
      const background = await added.evaluate((el) => getComputedStyle(el).backgroundColor);
      expect(background).toBe("rgb(199, 250, 229)");
      // A theme switch on the page, as a site's toggle makes: review follows it.
      await page.evaluate(() => {
        document.body.style.background = "#16181d";
        document.body.style.color = "#d9dde5";
      });
      await expect
        .poll(() => page.locator("html").getAttribute("data-ascribe-scheme"))
        .toBe("dark");
      await expect(panel(page).getAttribute("data-scheme")).resolves.toBe("dark");
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

  it("marks a changed titled code block, its caption and all", async () => {
    const root = await gitSite("review-code-title");
    await edit(path.join(root, "content", "Reference", "Options.md"), (text) =>
      text.replace("threads: 4", "threads: 8"),
    );
    const server = await serveDev(root);
    try {
      const page = await startReview(server.origin, `${BASE}/reference/options`);
      // The code-titles transformer put the block in a figure; the anchor marks the figure.
      const changed = page.locator('figure.code-title[data-ascribe-change="changed"]');
      await expect.poll(() => changed.count(), { timeout: 30_000 }).toBe(1);
      await expect(changed.locator("> figcaption").textContent()).resolves.toBe("loom.yaml");
      await expect(panel(page).textContent()).resolves.toContain("1 change on this page");
      await page.close();
    } finally {
      await server.stop();
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

  it("keeps comments off when the dev server listens on the network", async () => {
    const root = await gitSite("review-host");
    await edit(path.join(root, GUIDE), addParagraph);
    const server = await serveDev(root, { E2E_HOST: "0.0.0.0" });
    try {
      const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
      await expect
        .poll(() => page.locator('[data-ascribe-change="added"]').count(), { timeout: 30_000 })
        .toBe(1);
      await expect
        .poll(async () => panel(page).textContent(), { timeout: 30_000 })
        .toContain("listening on the network");
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
      await expect
        .poll(async () => panel(page).textContent(), { timeout: 30_000 })
        .toContain("This page has 1 change, but its blocks carry no source anchors.");
      const text = await panel(page).textContent();
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

    /** A copy of the site on a branch with pull request #7: `change` edits it, and `threads` are on it. */
    async function pullRequest(
      name: string,
      files: string[],
      change: (root: string) => Promise<void>,
      threads: (root: string) => Promise<FakeThread[]>,
    ): Promise<{ root: string; github: FakeGitHub }> {
      const root = await gitSite(name);
      git(root, "remote", "add", "origin", "https://github.com/acme/docs.git");
      git(root, "update-ref", "refs/remotes/origin/main", "main");
      await change(root);
      git(root, "commit", "-qam", "A change");
      git(root, "config", "branch.change.remote", "origin");
      git(root, "config", "branch.change.merge", "refs/heads/change");
      git(root, "update-ref", "refs/remotes/origin/change", "change");
      github?.remove();
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
          files,
        },
        threads: await threads(root),
        review: null,
        submitted: [],
      });
      return { root, github };
    }

    /** The line of `file` (from the site's root) that starts with `start`. */
    async function lineOf(root: string, file: string, start: string): Promise<number> {
      const lines = (await readFile(path.join(root, file), "utf8")).split("\n");
      return lines.findIndex((l) => l.startsWith(start)) + 1;
    }

    it("shows a thread beside its block, and sends a comment made there on submit", async () => {
      const guide = "content/Guides/My Setup.md";
      const { root, github } = await pullRequest(
        "review-threads",
        [guide],
        (root) => edit(path.join(root, GUIDE), addParagraph),
        async (root) => [
          {
            id: "PRRT_0",
            path: guide,
            line: await lineOf(root, guide, "This guide sets up"),
            comments: [
              { id: "PRRC_0", body: "Say what Loom is first.", state: "SUBMITTED", login: "maya" },
            ],
          },
        ],
      );
      const added = await lineOf(root, guide, "A paragraph about weaving");
      const server = await serveDev(root, { PATH: github.path });
      try {
        const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
        await expect
          .poll(async () => panel(page).textContent(), { timeout: 30_000 })
          .toContain("#7 against main");
        // The thread, beside the intro paragraph.
        const card = page.locator("[data-ascribe-overlay] .thread", {
          hasText: "Say what Loom is first.",
        });
        await expect.poll(() => card.count(), { timeout: 30_000 }).toBe(1);
        // At 1280 pixels the threads are in a column, and the panel stays clear of it.
        const panelBox = await panel(page).boundingBox();
        const cardBox = await card.boundingBox();
        expect(panelBox && cardBox && panelBox.x + panelBox.width).toBeLessThanOrEqual(cardBox?.x ?? 0);
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

    it("lists the threads on a page's fragments when a layout drops the anchors", async () => {
      const guide = "content/Guides/My Setup.md";
      const fragment = "content/_fragments/requirements.md";
      const { root, github } = await pullRequest(
        "review-threads-no-anchors",
        [fragment],
        async (root) => {
          await edit(path.join(root, fragment), (text) => text.replace("Node.js 20", "Node.js 22"));
          await edit(path.join(root, "src", "layouts", "Docs.astro"), (text) =>
            text.replace(
              "</body>",
              `<script is:inline>for (const el of document.querySelectorAll("[data-ascribe-source]")) el.removeAttribute("data-ascribe-source");</script></body>`,
            ),
          );
        },
        async (root) => [
          {
            id: "PRRT_0",
            path: guide,
            line: await lineOf(root, guide, "This guide sets up"),
            comments: [{ id: "PRRC_0", body: "On the guide.", state: "SUBMITTED", login: "maya" }],
          },
          {
            id: "PRRT_1",
            path: fragment,
            line: await lineOf(root, fragment, "- Node.js"),
            comments: [{ id: "PRRC_1", body: "Why 22?", state: "SUBMITTED", login: "sam" }],
          },
        ],
      );
      const server = await serveDev(root, { PATH: github.path });
      try {
        const page = await startReview(server.origin, `${BASE}/guides/my-setup`);
        await expect
          .poll(async () => panel(page).textContent(), { timeout: 30_000 })
          .toContain("This page has 2 comments and 1 change, but its blocks carry no source anchors.");
        expect(await panel(page).textContent()).toContain("Why 22?");
        // Each card names its file as the overlay's cards do.
        const where = await panel(page).locator(".thread .where").allTextContents();
        expect(where.sort()).toEqual(["My Setup.md:7", "requirements.md:3"]);
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
