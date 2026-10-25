// A guide's links to operations of Loom's API (`[apis.loom]`): each lands on
// its operation in the built site's reference. Redoc's, at /docs/api, is
// addressed `#operation/<operationId>`, which Redoc 3 rewrites to its own
// address and scrolls to, for an operation under a tag with a space too; a
// template's, here Stoplight Elements' hash router, is addressed
// `#/operations/<operationId>`.
import { rm, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  BASE,
  buildSite,
  copySite,
  edit,
  launchChromium,
  servePreview,
  siteDir,
} from "./harness.js";

const GUIDE = `${BASE}/guides/teams/share-a-loom`;

afterAll(async () => {
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

/**
 * Opens the guide, follows the link named `name`, and gives the address it
 * was written as, the address it lands on, and how far below the top of the
 * viewport the operation's heading is once the renderer has scrolled to it.
 */
async function follow(
  page: Page,
  origin: string,
  name: string,
): Promise<{ href: string | null; url: string; top: number }> {
  await page.goto(`${origin}${GUIDE}`);
  const link = page.getByRole("link", { name, exact: true });
  const href = await link.getAttribute("href");
  await link.click();
  // Redoc's heading for an operation is an h2 of its summary.
  const heading = page.locator("h2").filter({ hasText: name });
  await heading.waitFor({ state: "visible", timeout: 30_000 });
  // Redoc scrolls once it has drawn the whole spec.
  await expect
    .poll(() => heading.evaluate((element) => element.getBoundingClientRect().top), {
      timeout: 30_000,
    })
    .toBeLessThan(80);
  const top = await heading.evaluate((element) => element.getBoundingClientRect().top);
  return { href, url: page.url(), top };
}

describe("a link to an operation", () => {
  let server: Awaited<ReturnType<typeof servePreview>> | undefined;

  beforeAll(async () => {
    await buildSite();
    server = await servePreview();
  });

  afterAll(async () => {
    await server?.stop();
  });

  it("lands on the operation in Redoc, under a tag with a space too", async () => {
    if (server === undefined) throw new Error("the preview server didn't start");
    const { origin } = server;
    const browser = await launchChromium();
    try {
      const page = await browser.newPage();
      const traced: string[] = [];
      page.on("request", (request) => {
        if (request.url().includes("otel.")) traced.push(request.url());
      });
      for (const [name, id, own] of [
        ["Get a loom", "getLoom", "#/looms/getloom"],
        ["List invoices", "listInvoices", "#/billing-and-invoices/listinvoices"],
      ] as const) {
        const landed = await follow(page, origin, name);
        expect(landed.href).toBe(`${BASE}/api#operation/${id}`);
        expect(landed.url).toBe(`${origin}${BASE}/api${own}`);
        expect(landed.top).toBeGreaterThan(-2);
        expect(landed.top).toBeLessThan(80);
      }
      // The page sets disable-telemetry.
      expect(traced).toEqual([]);
    } finally {
      await browser.close();
    }
  });
});

describe("a link to an operation through a template", () => {
  it("lands on the operation in Stoplight Elements", async () => {
    const root = await copySite("elements");
    await edit(path.join(root, "ascribe.toml"), (text) =>
      text.replace(
        'reference = { renderer = "redoc", at = "/api" }',
        'reference = "/elements#/operations/{operationId}"',
      ),
    );
    await writeFile(
      path.join(root, "src", "pages", "elements.astro"),
      `---
import elements from "@stoplight/elements/web-components.min.js?url";
import styles from "@stoplight/elements/styles.min.css?url";
import spec from "../../api/openapi.yaml?url";
---

<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Loom API</title>
    <link rel="stylesheet" href={styles} />
  </head>
  <body>
    <elements-api apiDescriptionUrl={spec} router="hash" layout="sidebar"></elements-api>
    <script is:inline src={elements}></script>
  </body>
</html>
`,
    );
    await buildSite(root);
    const server = await servePreview(root);
    const { origin } = server;
    const browser = await launchChromium();
    try {
      const page = await browser.newPage();
      await page.goto(`${origin}${GUIDE}`);
      await page.getByRole("link", { name: "Get a loom", exact: true }).click();
      await page.waitForURL(`${origin}${BASE}/elements#/operations/getLoom`);
      // Elements draws only the operation its route names.
      const title = page.locator("h1", { hasText: "Get a loom" });
      await title.waitFor({ state: "visible", timeout: 30_000 });
      expect(await page.locator("h1").count()).toBe(1);
    } finally {
      await browser.close();
      await server.stop();
    }
  });
});
