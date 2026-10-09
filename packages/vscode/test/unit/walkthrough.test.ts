// The getting-started walkthrough in package.json names commands, views,
// context keys, files, and pages of the docs site, none of which VS Code
// checks: a step whose button names a command that doesn't exist does
// nothing, and one whose completion event never fires stays unchecked.

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { PAGE_STEPS } from "../../src/ui/walkthroughSteps.js";

interface Step {
  id: string;
  title: string;
  description: string;
  media: { markdown?: string; image?: string; altText?: string };
  completionEvents?: string[];
}

interface Manifest {
  contributes: {
    commands: { command: string }[];
    menus: { commandPalette: { command: string; when: string }[] };
    viewsContainers: { activitybar: { id: string }[] };
    views: Record<string, { id: string }[]>;
    walkthroughs: { id: string; title: string; description: string; steps: Step[] }[];
  };
}

const root = new URL("../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");
const manifest = JSON.parse(read("package.json")) as Manifest;
const contributes = manifest.contributes;
const walkthrough = contributes.walkthroughs[0];
const steps = walkthrough?.steps ?? [];

/** VS Code's own commands that a step may run, each checked by hand. */
const VSCODE_COMMANDS = new Set(["workbench.action.files.openFolder"]);

const SITE = "https://ascribed-dev.com/";

const commands = new Set(contributes.commands.map((command) => command.command));
const containers = contributes.viewsContainers.activitybar.map((container) => container.id);
const views = Object.values(contributes.views)
  .flat()
  .map((view) => view.id);

/** The `[label](target)` links in a step's description. */
function links(markdown: string): string[] {
  return [...markdown.matchAll(/\]\(([^)\s]+)\)/g)].map((match) => match[1] ?? "");
}

/** The command a `command:` link runs, without its arguments. */
function commandOf(link: string): string {
  return link
    .slice("command:".length)
    .replace(/^toSide:/, "")
    .replace(/\?.*$/, "");
}

/** Every context key the extension sets, from its source. */
function contextKeys(): Set<string> {
  const keys = new Set<string>();
  const walk = (dir: URL) => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.isDirectory()) walk(new URL(`${entry.name}/`, dir));
      else if (entry.name.endsWith(".ts")) {
        const text = readFileSync(new URL(entry.name, dir), "utf8");
        for (const match of text.matchAll(/"setContext",\s*"([\w.]+)"/g)) keys.add(match[1] ?? "");
      }
    }
  };
  walk(new URL("src/", root));
  return keys;
}

/** A heading's id on the docs site, as github-slugger makes it. */
function slug(heading: string): string {
  return heading
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, "")
    .replace(/\s/g, "-");
}

/** Whether a link to the docs site names a page, and a heading of it, that exist. */
function onSite(url: string): boolean {
  const [path = "", id] = url.slice(SITE.length).split("#");
  const file = new URL(`../../docs/content/${path.replace(/\/$/, "") || "index"}.md`, root);
  if (!existsSync(file)) return false;
  if (id === undefined) return true;
  const headings = readFileSync(file, "utf8")
    .split("\n")
    .filter((line) => /^#{2,6} /.test(line))
    .map((line) => slug(line.replace(/^#+ /, "")));
  return headings.includes(id);
}

describe("the getting-started walkthrough", () => {
  it("is the only one, with steps", () => {
    expect(contributes.walkthroughs.map((w) => w.id)).toEqual(["start"]);
    expect(steps.length).toBeGreaterThan(0);
  });

  it("gives every step a completion event", () => {
    for (const step of steps) expect(step.completionEvents ?? [], step.id).not.toEqual([]);
  });

  it("runs only commands that exist", () => {
    for (const step of steps) {
      for (const link of links(step.description).filter((l) => l.startsWith("command:"))) {
        const command = commandOf(link);
        if (command.startsWith("workbench.view.extension.")) {
          expect(containers, step.id).toContain(command.slice("workbench.view.extension.".length));
        } else if (!VSCODE_COMMANDS.has(command)) {
          expect(commands, `${step.id}: ${command}`).toContain(command);
        }
      }
    }
  });

  it("completes on commands, views, and context keys that exist, and links it shows", () => {
    const keys = contextKeys();
    for (const step of steps) {
      for (const event of step.completionEvents ?? []) {
        const [kind = "", value = ""] = event.split(/:(.*)/s);
        if (kind === "onCommand") expect(commands, `${step.id}: ${event}`).toContain(value);
        else if (kind === "onView") expect(views, `${step.id}: ${event}`).toContain(value);
        else if (kind === "onContext") {
          for (const key of value.match(/[\w.]+/g) ?? []) {
            expect(keys, `${step.id}: ${event}`).toContain(key);
          }
        } else if (kind === "onLink") {
          expect(links(step.description), `${step.id}: ${event}`).toContain(value);
        } else expect.fail(`${step.id}: ${event} isn't an event this test checks`);
      }
    }
  });

  it("completes a page step on its button's command and on the command it runs", () => {
    for (const step of steps) {
      for (const link of links(step.description).filter((l) => l.startsWith("command:"))) {
        const command = PAGE_STEPS[commandOf(link)];
        if (command === undefined) continue;
        expect(step.completionEvents, step.id).toEqual([
          `onCommand:${command}`,
          `onCommand:${commandOf(link)}`,
        ]);
      }
    }
  });

  it("declares its page commands, hidden from the palette", () => {
    for (const [id, command] of Object.entries(PAGE_STEPS)) {
      expect(commands).toContain(id);
      expect(commands).toContain(command);
      expect(contributes.menus.commandPalette.find((entry) => entry.command === id)?.when).toBe(
        "false",
      );
    }
  });

  it("has every step's media", () => {
    for (const step of steps) {
      const file = step.media.markdown ?? step.media.image;
      expect(file, step.id).toBeDefined();
      expect(existsSync(new URL(file ?? "", root)), `${step.id}: ${file}`).toBe(true);
      if (step.media.image) expect(step.media.altText, step.id).toBeTruthy();
    }
  });

  it("links to pages and headings of the docs site that exist", () => {
    for (const step of steps) {
      for (const link of links(step.description).filter((l) => l.startsWith(SITE))) {
        expect(onSite(link), `${step.id}: ${link}`).toBe(true);
      }
    }
  });
});
