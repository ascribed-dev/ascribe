// The editor guide, docs/content/guides/editor.md, includes two fragments
// generated from package.json: the settings, and the commands in the command
// palette. This test renders them and fails when one is out of date; run it
// with ASCRIBE_BLESS=1 to rewrite them. To change what a setting's row says,
// change its description in package.json. VS Code's manifest has no field for
// what a command does, so each command's description is here, in `commands`.

import { readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const BLESS = "ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts";

const HEADER =
  "<!-- Generated from packages/vscode/package.json by packages/vscode/test/unit/docs.test.ts. " +
  `Edit the manifest, or a command's description in the test, then run \`${BLESS}\`. -->\n`;

/** What each command in the palette does, by id. */
const commands: Record<string, string> = {
  "ascribe.restartServer":
    "Stops and starts every project's server that has started, including one that stopped after crashing, and forgets earlier crashes. A server that hasn't started stays off until it's needed; when none has, the command says so. It also picks up `ascribe.toml` files added or deleted.",
  "ascribe.showOutput":
    "Opens the log of the active file's project. When no file of a project is active and the workspace has several projects, it asks which, showing whether each one's server is running.",
  "ascribe.openPagePreview":
    "Opens the page preview of the active page in place of the editor, starting its project's server if it hasn't started.",
  "ascribe.openPreview":
    "Opens the page preview of the active page beside the editor, starting its project's server if it hasn't started.",
  "ascribe.openSitePreview":
    "Opens the active page on its project's dev server, in the browser. See [Site preview](../guides/editor.md#site-preview).",
  "ascribe.selectPreviewBuild":
    "Picks the build the preview shows, for the previewed page's project.",
  "ascribe.startReview":
    "Marks what changed in the preview, against a base it asks for, for the active page's project. Its server must be running: open one of its pages first. See [Review in the preview](../guides/editor.md#review-in-the-preview).",
  "ascribe.stopReview": "Turns review off for the active page's project, and frees its base.",
  "ascribe.changedPages":
    "Lists the pages the change touches in the preview's build; choosing one opens it and its preview.",
  "ascribe.refreshComments":
    "Reads the pull request's review threads from GitHub again, for the active page's project. See [Comments in the preview](../guides/editor.md#comments-in-the-preview).",
};

interface Setting {
  default: unknown;
  enum?: string[];
  enumDescriptions?: string[];
  description?: string;
  markdownDescription?: string;
}

interface Manifest {
  contributes: {
    commands: { command: string; title: string; category: string }[];
    menus: { commandPalette: { command: string; when: string }[] };
    configuration: { properties: Record<string, Setting> };
  };
}

const root = new URL("../../../../", import.meta.url);
const read = (path: string) => readFileSync(new URL(path, root), "utf8");
const manifest = JSON.parse(read("packages/vscode/package.json")) as Manifest;

/** Text for a table cell: phrases and HTML escaped outside code, and pipes. */
function cell(text: string): string {
  let code = false;
  let out = "";
  for (const c of text) {
    if (c === "`") code = !code;
    else if ((c === "{" || c === "<") && !code) out += "\\";
    if (c === "|") out += "\\";
    out += c;
  }
  return out;
}

/** The settings table, in the manifest's order. */
function settings(): string {
  let out = `${HEADER}\n| Setting | Default | What it does |\n|---|---|---|\n`;
  for (const [key, setting] of Object.entries(manifest.contributes.configuration.properties)) {
    const fallback = setting.default === "" ? "empty" : `\`${String(setting.default)}\``;
    let what = setting.markdownDescription ?? setting.description ?? "";
    (setting.enum ?? []).forEach((value, i) => {
      const description = setting.enumDescriptions?.[i];
      if (description) what += ` \`${value}\`: ${description}`;
    });
    out += `| \`${key}\` | ${fallback} | ${cell(what)} |\n`;
  }
  return out;
}

/** The commands table: those the palette shows, in the manifest's order. */
function commandTable(): string {
  const hidden = new Set(
    manifest.contributes.menus.commandPalette
      .filter((entry) => entry.when === "false")
      .map((entry) => entry.command),
  );
  let out = `${HEADER}\n| Command | What it does |\n|---|---|\n`;
  for (const command of manifest.contributes.commands) {
    if (hidden.has(command.command)) continue;
    const what = commands[command.command] ?? "";
    out += `| **${command.category}: ${command.title}** | ${cell(what)} |\n`;
  }
  return out;
}

const fragments: [string, string][] = [
  ["editor-settings.md", settings()],
  ["editor-commands.md", commandTable()],
];

describe("the editor guide", () => {
  it("describes every command in the palette, and nothing else", () => {
    const shown = manifest.contributes.commands
      .map((command) => command.command)
      .filter((id) =>
        manifest.contributes.menus.commandPalette.some(
          (entry) => entry.command === id && entry.when !== "false",
        ),
      );
    expect(Object.keys(commands).sort()).toEqual(shown.sort());
  });

  it.each(fragments)("has a current _generated/%s", (name, text) => {
    const path = new URL(`docs/content/_generated/${name}`, root);
    if (process.env.ASCRIBE_BLESS) writeFileSync(path, text);
    let current = "";
    try {
      current = readFileSync(path, "utf8");
    } catch {
      // Missing: it isn't current.
    }
    expect(current, `docs/content/_generated/${name} is out of date: run \`${BLESS}\``).toBe(text);
  });

  it.each(fragments)("includes _generated/%s", (name) => {
    expect(read("docs/content/guides/editor.md")).toContain(`@include: ../_generated/${name}\n`);
  });
});
