import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { Browser, BrowserContext, Page } from "playwright-core";
import { launch, open, TABS, type Engine } from "./harness.js";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const axeSource = readFileSync(
  createRequire(import.meta.url).resolve("axe-core/axe.min.js"),
  "utf8",
);

const NOTE = `
<ascribe-note type="tip" label="Tip" heading="Try it without installing">

<p>Run Quill in the browser.</p>

</ascribe-note>

<ascribe-note type="security" label="Security">

<p>A project's own type.</p>

</ascribe-note>
`;

const STEPS = `
<ascribe-steps>

<ol start="3">
<li>Install the agent package.</li>
<li>Verify the install.</li>
</ol>

</ascribe-steps>
`;

const AVAILABILITY = `
<ascribe-availability scope="section">
<ascribe-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</ascribe-availability-target>; <ascribe-availability-target target="self-managed" dimension="deployment" states="preview" versions="3.4">Self-managed (preview, 3.4+)</ascribe-availability-target>
</ascribe-availability>
`;

const ALL = `<h1>Page</h1>${NOTE}${STEPS}${AVAILABILITY}${TABS}
<ascribe-group widget="quill-thing"><p>Grouped.</p></ascribe-group>`;

/**
 * Every test, for one engine. The engine files (`chromium.test.ts`,
 * `firefox.test.ts`, `webkit.test.ts`) each call this, so vitest runs the
 * engines in parallel workers, one file each.
 */
export function suiteFor(engine: Engine): void {
  let browser: Browser;
  let context: BrowserContext;

  beforeAll(async () => {
    browser = await launch(engine);
  });
  afterAll(async () => {
    await browser.close();
  });

  /** A fresh context per test, so remembered choices don't leak between tests. */
  async function fresh(body: string, options?: Parameters<typeof open>[2]): Promise<Page> {
    context = await browser.newContext();
    return open(context, body, options);
  }

  async function pseudo(page: Page, selector: string, pseudoElement: string, property: string) {
    return page
      .locator(selector)
      .first()
      .evaluate((el, [p, prop]) => getComputedStyle(el, p)[prop as never] as string, [
        pseudoElement,
        property,
      ] as const);
  }

  /**
   * The text of a computed `content` value. Engines serialize a `content` built
   * from several parts differently: Chromium joins them (`"Tip: Heading"`),
   * Firefox and WebKit keep the parts (`"Tip" ": " "Heading"`).
   */
  function contentText(content: string): string {
    const parts = content.match(/"(?:[^"\\]|\\.)*"/g) ?? [];
    return parts.map((part) => part.slice(1, -1)).join("");
  }

  describe("without the script", () => {
    it("shows every tab's content, each introduced by its label", async () => {
      const page = await fresh(TABS, { script: false });
      for (const text of ["Install with npm.", "Install with pnpm or yarn.", "Install with bun."]) {
        await expect(page.getByText(text).isVisible()).resolves.toBe(true);
      }
      const labels = await page
        .locator("#a > ascribe-tab")
        .evaluateAll((tabs) => tabs.map((tab) => getComputedStyle(tab, "::before").content));
      expect(labels).toEqual(['"npm"', '"pnpm / yarn"', '"bun"']);
      await expect(page.locator("[role=tablist]").count()).resolves.toBe(0);
      await context.close();
    });

    it("shows a note's heading, or its label when it has none", async () => {
      const page = await fresh(NOTE, { script: false });
      expect(contentText(await pseudo(page, "ascribe-note[type=tip]", "::before", "content"))).toBe(
        "Tip: Try it without installing",
      );
      expect(
        contentText(await pseudo(page, "ascribe-note[type=security]", "::before", "content")),
      ).toBe("Security");
      await expect(page.getByText("A project's own type.").isVisible()).resolves.toBe(true);
      await context.close();
    });

    it("themes note types through custom properties, and unknown types get the default", async () => {
      const page = await fresh(NOTE, { script: false });
      const colors = await page
        .locator("ascribe-note")
        .evaluateAll((notes) => notes.map((note) => getComputedStyle(note).borderInlineStartColor));
      expect(colors[0]).not.toBe(colors[1]);
      await page.addStyleTag({
        content: '[type="security"] { --_color: rgb(1, 2, 3); }',
      });
      const themed = await page
        .locator("ascribe-note[type=security]")
        .evaluate((note) => getComputedStyle(note).borderInlineStartColor);
      expect(themed).toBe("rgb(1, 2, 3)");
      await context.close();
    });

    it("numbers steps from the list's own start", async () => {
      const item = (list: string) => `<ascribe-steps>${list}</ascribe-steps>`;
      const page = await fresh(
        item('<ol id="real" start="3"><li>Step</li><li>Step</li></ol>') +
          item('<ol id="ref"><li>Step</li><li>Step</li></ol>') +
          item('<ol id="other"><li>Step</li><li>Step</li></ol>'),
        { script: false },
      );
      // Generated counters can't be read back, so compare the rendered markers with ones written as text.
      await page.addStyleTag({
        content:
          '#ref li:nth-child(1)::before { content: "3"; } #ref li:nth-child(2)::before { content: "4"; }',
      });
      const marker = async (list: string, n: number) =>
        (
          await page.locator(`${list} li:nth-child(${n})`).screenshot({ animations: "disabled" })
        ).toString("base64");
      expect(await marker("#real", 1)).toBe(await marker("#ref", 1));
      expect(await marker("#real", 2)).toBe(await marker("#ref", 2));
      // The reference isn't trivially equal: unnumbered-from-3 markers look different.
      expect(await marker("#other", 1)).not.toBe(await marker("#ref", 1));
      await context.close();
    });

    it("styles by the last state only, not by names that merely end the same way", async () => {
      const page = await fresh(
        [
          "ga",
          "pre-ga",
          "preview ga",
          "early-preview",
          "beta",
          "omega",
          "x deprecated",
          "pre-deprecated",
        ]
          .map(
            (states) =>
              `<ascribe-availability scope="block"><ascribe-availability-target target="t" dimension="d" states="${states}">T</ascribe-availability-target></ascribe-availability>`,
          )
          .join(""),
        { script: false },
      );
      const colors = await page
        .locator("ascribe-availability-target")
        .evaluateAll((all) => all.map((el) => getComputedStyle(el).backgroundColor));
      const [ga, preGa, previewGa, earlyPreview, beta, omega, xDeprecated, preDeprecated] = colors;
      const neutral = preGa;
      expect(ga).toBe(previewGa);
      expect(ga).not.toBe(neutral);
      expect(omega).toBe(neutral);
      expect(earlyPreview).toBe(neutral);
      expect(preDeprecated).toBe(neutral);
      expect(beta).not.toBe(neutral);
      expect(xDeprecated).not.toBe(neutral);
      await context.close();
    });

    it("reads as a sentence, with a lead-in from CSS", async () => {
      const page = await fresh(AVAILABILITY, { script: false });
      expect(await pseudo(page, "ascribe-availability", "::before", "content")).toBe(
        '"Available: "',
      );
      const text = await page.locator("ascribe-availability").innerText();
      expect(text.replace(/\s+/g, " ").trim()).toBe(
        "Quill Cloud (GA); Self-managed (preview, 3.4+)",
      );
      await context.close();
    });

    it("sits inline in a table row's first cell, with no lead-in", async () => {
      const page = await fresh(
        `<table><tr><td><code>stream</code> <ascribe-availability scope="row"><ascribe-availability-target target="cloud" dimension="deployment" states="beta">Quill Cloud (Beta)</ascribe-availability-target></ascribe-availability></td><td>Streams.</td></tr></table>`,
        { script: false },
      );
      const row = page.locator("ascribe-availability");
      expect(await row.evaluate((el) => getComputedStyle(el).display)).toBe("inline");
      expect(await pseudo(page, "ascribe-availability", "::before", "content")).toBe("none");
      const code = await page.locator("code").boundingBox();
      const badge = await page.locator("ascribe-availability-target").boundingBox();
      // On the code's line, after it.
      expect(badge && code && badge.x > code.x && Math.abs(badge.y - code.y) < code.height).toBe(
        true,
      );
      await context.close();
    });

    it("keeps all content in the DOM with neither script nor CSS", async () => {
      const page = await fresh(ALL, { script: false, css: false });
      const text = await page.locator("main").innerText();
      for (const expected of [
        "Run Quill in the browser.",
        "Install the agent package.",
        "Quill Cloud (GA)",
        "Install with bun.",
        "Windows steps.",
        "Second look content.",
        "Grouped.",
      ]) {
        expect(text).toContain(expected);
      }
      await context.close();
    });
  });

  describe("light and dark", () => {
    const LIGHT = "rgb(71, 77, 198)";
    const DARK = "rgb(145, 161, 254)";
    const optIn = "<style>:root { color-scheme: light dark; }</style>";

    async function noteColor(
      body: string,
      colorScheme: "light" | "dark",
      scheme?: string,
    ): Promise<string> {
      context = await browser.newContext({ colorScheme });
      const page = await open(context, body + NOTE, { script: false });
      if (scheme !== undefined) {
        await page.evaluate(
          (s) => document.documentElement.setAttribute("data-ascribe-scheme", s),
          scheme,
        );
      }
      const color = await page
        .locator("ascribe-note[type=security]")
        .evaluate((note) => getComputedStyle(note).borderInlineStartColor);
      await context.close();
      return color;
    }

    it("stays light on a page that declares no color-scheme", async () => {
      expect(await noteColor("", "dark")).toBe(LIGHT);
    });

    it("follows the system on a page that declares light dark", async () => {
      expect(await noteColor(optIn, "dark")).toBe(DARK);
      expect(await noteColor(optIn, "light")).toBe(LIGHT);
    });

    it("follows data-ascribe-scheme over the system and color-scheme", async () => {
      expect(await noteColor("", "light", "dark")).toBe(DARK);
      expect(await noteColor(optIn, "dark", "light")).toBe(LIGHT);
    });

    it("lets a site's own value win", async () => {
      const own = "<style>:root { --ascribe-note-color: rgb(1, 2, 3); }</style>";
      expect(await noteColor(optIn + own, "dark")).toBe("rgb(1, 2, 3)");
    });
  });

  describe("glossary terms", () => {
    it("underlines a term's link with dots, and no other link", async () => {
      const page = await fresh(
        '<p><a id="term" href="/g#t" title="A term." data-ascribe-term="t">term</a> <a id="other" href="/x" title="X">other</a></p>',
        { script: false },
      );
      const style = (id: string) =>
        page.locator(`#${id}`).evaluate((a) => getComputedStyle(a).textDecorationStyle);
      expect(await style("term")).toBe("dotted");
      expect(await style("other")).toBe("solid");
      await context.close();
    });
  });

  describe("with the script", () => {
    it("registers exactly tabs, tab, and group", async () => {
      const page = await fresh(ALL);
      const defined = await page.evaluate(() =>
        [
          "ascribe-tabs",
          "ascribe-tab",
          "ascribe-group",
          "ascribe-note",
          "ascribe-steps",
          "ascribe-availability",
          "ascribe-availability-target",
        ].filter((name) => customElements.get(name) !== undefined),
      );
      expect(defined).toEqual(["ascribe-tabs", "ascribe-tab", "ascribe-group"]);
      await context.close();
    });

    it("builds the ARIA tabs structure", async () => {
      const page = await fresh(TABS);
      const group = page.locator("#a");
      await expect(
        group.evaluate((el) => (el.firstElementChild as HTMLElement).getAttribute("role")),
      ).resolves.toBe("tablist");
      const buttons = group.locator("[role=tab]");
      await expect(buttons.allTextContents()).resolves.toEqual(["npm", "pnpm / yarn", "bun"]);
      await expect(
        buttons.evaluateAll((all) => all.map((b) => b.getAttribute("aria-selected"))),
      ).resolves.toEqual(["true", "false", "false"]);
      await expect(
        buttons.evaluateAll((all) => all.map((b) => (b as HTMLElement).tabIndex)),
      ).resolves.toEqual([0, -1, -1]);
      const wiring = await group.evaluate((el) =>
        Array.from(el.querySelectorAll(":scope > [role=tablist] > [role=tab]")).map((button) => {
          const panel = document.getElementById(button.getAttribute("aria-controls") ?? "");
          return {
            panelRole: panel?.getAttribute("role"),
            labelledBy: panel?.getAttribute("aria-labelledby") === button.id,
            hidden: panel?.hasAttribute("hidden"),
          };
        }),
      );
      expect(wiring).toEqual([
        { panelRole: "tabpanel", labelledBy: true, hidden: false },
        { panelRole: "tabpanel", labelledBy: true, hidden: true },
        { panelRole: "tabpanel", labelledBy: true, hidden: true },
      ]);
      await expect(page.getByText("Install with npm.").isVisible()).resolves.toBe(true);
      await expect(page.getByText("Install with bun.").isVisible()).resolves.toBe(false);
      await context.close();
    });

    it("moves with the arrow keys, wrapping, and with Home and End", async () => {
      const page = await fresh(TABS);
      const buttons = page.locator("#a [role=tab]");
      const selected = () =>
        buttons.evaluateAll((all) =>
          all.findIndex((b) => b.getAttribute("aria-selected") === "true"),
        );
      await buttons.nth(0).focus();
      await page.keyboard.press("ArrowRight");
      expect(await selected()).toBe(1);
      await expect(buttons.nth(1).evaluate((b) => document.activeElement === b)).resolves.toBe(
        true,
      );
      await page.keyboard.press("ArrowRight");
      await page.keyboard.press("ArrowRight");
      expect(await selected()).toBe(0);
      await page.keyboard.press("ArrowLeft");
      expect(await selected()).toBe(2);
      await page.keyboard.press("Home");
      expect(await selected()).toBe(0);
      await page.keyboard.press("End");
      expect(await selected()).toBe(2);
      await expect(page.getByText("Install with bun.").isVisible()).resolves.toBe(true);
      await expect(page.getByText("Install with npm.").isVisible()).resolves.toBe(false);
      await context.close();
    });

    it("selects on click", async () => {
      const page = await fresh(TABS);
      await page.locator("#d").getByRole("tab", { name: "Second look" }).click();
      await expect(page.getByText("Second look content.").isVisible()).resolves.toBe(true);
      await expect(page.getByText("First look content.").isVisible()).resolves.toBe(false);
      await context.close();
    });

    it("syncs groups with the same sync value, matching any of a tab's values", async () => {
      const page = await fresh(TABS);
      await page.locator("#b").getByRole("tab", { name: "pnpm" }).click();
      // Group a's second tab lists `pnpm yarn`.
      await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe(
        "pnpm / yarn",
      );
      await page.locator("#a").getByRole("tab", { name: "npm", exact: true }).click();
      await expect(page.locator("#b [aria-selected=true]").textContent()).resolves.toBe("npm");
      await context.close();
    });

    it("keeps a group's selection when it has no tab for the choice", async () => {
      const page = await fresh(TABS);
      await page.locator("#a").getByRole("tab", { name: "bun" }).click();
      await expect(page.locator("#b [aria-selected=true]").textContent()).resolves.toBe("pnpm");
      await context.close();
    });

    it("doesn't sync other dimensions or labeled groups", async () => {
      const page = await fresh(TABS);
      await page.locator("#a").getByRole("tab", { name: "bun" }).click();
      await expect(page.locator("#c [aria-selected=true]").textContent()).resolves.toBe("Linux");
      await page.locator("#d").getByRole("tab", { name: "Second look" }).click();
      await expect(page.evaluate(() => localStorage.length)).resolves.toBe(1);
      await page.locator("#c").getByRole("tab", { name: "Windows" }).click();
      await expect(page.locator("#d [aria-selected=true]").textContent()).resolves.toBe(
        "Second look",
      );
      await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe("bun");
      await context.close();
    });

    it("remembers the choice in localStorage across pages", async () => {
      const page = await fresh(TABS);
      await page.locator("#b").getByRole("tab", { name: "pnpm" }).click();
      expect(await page.evaluate(() => localStorage.getItem("ascribe-tabs:pm"))).toBe("pnpm");
      await page.reload();
      await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe(
        "pnpm / yarn",
      );
      await expect(page.locator("#b [aria-selected=true]").textContent()).resolves.toBe("pnpm");
      // A labeled group has nothing to remember.
      await expect(page.locator("#d [aria-selected=true]").textContent()).resolves.toBe(
        "First look",
      );
      await context.close();
    });

    it("prefers this page's choice when storage writes fail but reads work", async () => {
      const page = await fresh(TABS);
      await page.evaluate(() => localStorage.setItem("ascribe-tabs:pm", "npm"));
      await page.evaluate(() => {
        Storage.prototype.setItem = () => {
          throw new DOMException("full", "QuotaExceededError");
        };
      });
      await page.locator("#b").getByRole("tab", { name: "pnpm" }).click();
      // A group that starts later reads the choice just made, not the older stored one.
      await page.evaluate(() => {
        const late = document.createElement("ascribe-tabs");
        late.id = "late";
        late.setAttribute("sync", "pm");
        late.innerHTML =
          '<ascribe-tab value="npm" label="npm">n</ascribe-tab><ascribe-tab value="pnpm" label="pnpm">p</ascribe-tab>';
        document.querySelector("main")?.append(late);
      });
      await expect(page.locator("#late [aria-selected=true]").textContent()).resolves.toBe("pnpm");
      await context.close();
    });

    it("falls back to the first tab when the remembered value matches nothing", async () => {
      const page = await fresh(TABS);
      await page.evaluate(() => localStorage.setItem("ascribe-tabs:pm", "deno"));
      await page.reload();
      await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe("npm");
      await context.close();
    });

    it("works for the page when storage is unavailable", async () => {
      const page = await fresh(TABS, { blockStorage: true });
      await page.locator("#b").getByRole("tab", { name: "pnpm" }).click();
      await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe(
        "pnpm / yarn",
      );
      await context.close();
    });
  });

  describe("accessibility", () => {
    async function audit(page: Page) {
      await page.addScriptTag({ content: axeSource });
      return page.evaluate(async () => {
        const axe = (
          window as unknown as { axe: { run: () => Promise<{ violations: unknown[] }> } }
        ).axe;
        return (await axe.run()).violations;
      });
    }

    it("has no violations with the script", async () => {
      const page = await fresh(ALL);
      expect(await audit(page)).toEqual([]);
      await page.locator("#a").getByRole("tab", { name: "bun" }).click();
      expect(await audit(page)).toEqual([]);
      await context.close();
    });

    it("has no violations without the script", async () => {
      const page = await fresh(ALL, { script: false });
      expect(await audit(page)).toEqual([]);
      await context.close();
    });

    it("has no violations in dark mode", async () => {
      context = await browser.newContext({ colorScheme: "dark" });
      const page = await open(
        context,
        `<style>:root { color-scheme: light dark; background: #0c0f16; color: #f0f3f7; }</style>${ALL}`,
      );
      expect(await audit(page)).toEqual([]);
      await context.close();
    });
  });
}
