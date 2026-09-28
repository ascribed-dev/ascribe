import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { Browser, BrowserContext, Page } from "playwright-core";
import { launch, open, TABS } from "./harness.js";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";

const axeSource = readFileSync(
  createRequire(import.meta.url).resolve("axe-core/axe.min.js"),
  "utf8",
);

const NOTE = `
<tessera-note type="tip" label="Tip" heading="Try it without installing">

<p>Run Quill in the browser.</p>

</tessera-note>

<tessera-note type="security" label="Security">

<p>A project's own type.</p>

</tessera-note>
`;

const STEPS = `
<tessera-steps>

<ol start="3">
<li>Install the agent package.</li>
<li>Verify the install.</li>
</ol>

</tessera-steps>
`;

const AVAILABILITY = `
<tessera-availability scope="section">
<tessera-availability-target target="cloud" dimension="deployment" states="ga">Quill Cloud (GA)</tessera-availability-target>; <tessera-availability-target target="self-managed" dimension="deployment" states="preview" versions="3.4">Self-managed (preview, 3.4+)</tessera-availability-target>
</tessera-availability>
`;

const ALL = `<h1>Page</h1>${NOTE}${STEPS}${AVAILABILITY}${TABS}
<tessera-group widget="quill-thing"><p>Grouped.</p></tessera-group>`;

let browser: Browser;
let context: BrowserContext;

beforeAll(async () => {
  browser = await launch();
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

describe("without the script", () => {
  it("shows every tab's content, each introduced by its label", async () => {
    const page = await fresh(TABS, { script: false });
    for (const text of ["Install with npm.", "Install with pnpm or yarn.", "Install with bun."]) {
      await expect(page.getByText(text).isVisible()).resolves.toBe(true);
    }
    const labels = await page
      .locator("#a > tessera-tab")
      .evaluateAll((tabs) => tabs.map((tab) => getComputedStyle(tab, "::before").content));
    expect(labels).toEqual(['"npm"', '"pnpm / yarn"', '"bun"']);
    await expect(page.locator("[role=tablist]").count()).resolves.toBe(0);
    await context.close();
  });

  it("shows a note's heading, or its label when it has none", async () => {
    const page = await fresh(NOTE, { script: false });
    expect(await pseudo(page, "tessera-note[type=tip]", "::before", "content")).toBe(
      '"Try it without installing"',
    );
    expect(await pseudo(page, "tessera-note[type=security]", "::before", "content")).toBe(
      '"Security"',
    );
    await expect(page.getByText("A project's own type.").isVisible()).resolves.toBe(true);
    await context.close();
  });

  it("themes note types through custom properties, and unknown types get the default", async () => {
    const page = await fresh(NOTE, { script: false });
    const colors = await page
      .locator("tessera-note")
      .evaluateAll((notes) => notes.map((note) => getComputedStyle(note).borderInlineStartColor));
    expect(colors[0]).not.toBe(colors[1]);
    await page.addStyleTag({
      content: '[type="security"] { --_color: rgb(1, 2, 3); }',
    });
    const themed = await page
      .locator("tessera-note[type=security]")
      .evaluate((note) => getComputedStyle(note).borderInlineStartColor);
    expect(themed).toBe("rgb(1, 2, 3)");
    await context.close();
  });

  it("numbers steps with the list's own counter, so `start` is honored", async () => {
    const page = await fresh(STEPS, { script: false });
    const markers = await page
      .locator("tessera-steps li")
      .evaluateAll((items) => items.map((li) => getComputedStyle(li, "::before").content));
    // Computed style doesn't resolve counters; `list-item` is the counter that follows `start`.
    expect(markers).toEqual(["counter(list-item)", "counter(list-item)"]);
    const display = await page
      .locator("tessera-steps li")
      .first()
      .evaluate((li) => getComputedStyle(li).display);
    expect(display).toBe("list-item");
    await context.close();
  });

  it("reads as a sentence, with a lead-in from CSS", async () => {
    const page = await fresh(AVAILABILITY, { script: false });
    expect(await pseudo(page, "tessera-availability", "::before", "content")).toBe('"Available: "');
    const text = await page.locator("tessera-availability").innerText();
    expect(text.replace(/\s+/g, " ").trim()).toBe("Quill Cloud (GA); Self-managed (preview, 3.4+)");
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

describe("with the script", () => {
  it("registers exactly tabs, tab, and group", async () => {
    const page = await fresh(ALL);
    const defined = await page.evaluate(() =>
      [
        "tessera-tabs",
        "tessera-tab",
        "tessera-group",
        "tessera-note",
        "tessera-steps",
        "tessera-availability",
        "tessera-availability-target",
      ].filter((name) => customElements.get(name) !== undefined),
    );
    expect(defined).toEqual(["tessera-tabs", "tessera-tab", "tessera-group"]);
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
    await expect(buttons.nth(1).evaluate((b) => document.activeElement === b)).resolves.toBe(true);
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
    expect(await page.evaluate(() => localStorage.getItem("tessera-tabs:pm"))).toBe("pnpm");
    await page.reload();
    await expect(page.locator("#a [aria-selected=true]").textContent()).resolves.toBe(
      "pnpm / yarn",
    );
    await expect(page.locator("#b [aria-selected=true]").textContent()).resolves.toBe("pnpm");
    // A labeled group has nothing to remember.
    await expect(page.locator("#d [aria-selected=true]").textContent()).resolves.toBe("First look");
    await context.close();
  });

  it("falls back to the first tab when the remembered value matches nothing", async () => {
    const page = await fresh(TABS);
    await page.evaluate(() => localStorage.setItem("tessera-tabs:pm", "deno"));
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
      const axe = (window as unknown as { axe: { run: () => Promise<{ violations: unknown[] }> } })
        .axe;
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
});
