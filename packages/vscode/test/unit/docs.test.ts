// The editor guide, the pages in docs/content/editor/, include four fragments
// generated from package.json and the action registry: the settings, the
// commands in the command palette, the actions, and the tools for agents. This test renders them
// and fails when one is out of date; run it with ASCRIBE_BLESS=1 to rewrite
// them. To change what a setting's row says, change its description in
// package.json. VS Code's manifest has no field for what a command does, so
// each command's description is here, in `commands`. The actions' commands
// share one row of the commands table, since the actions table describes
// each from src/actions/registry.ts. A setting or command no release has yet
// is listed in `available`, which gives its row an availability.

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { ACTIONS, commandId } from "../../src/actions/registry.js";

const BLESS = "ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts";

/** What each fragment starts with: a memo, touching its table, saying what generates it. */
const HEADER =
  "@memo: Generated from packages/vscode/package.json by packages/vscode/test/unit/docs.test.ts; " +
  `edit the manifest, or a command's description in the test, then run \`${BLESS}\` to rewrite it.\n`;

const ACTIONS_HEADER =
  "@memo: Generated from packages/vscode/src/actions/registry.ts by packages/vscode/test/unit/docs.test.ts; " +
  `edit the registry, then run \`${BLESS}\` to rewrite it.\n`;

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
    "Opens the active page on its project's dev server, in the browser. See [Site preview](../editor/preview.md#site-preview).",
  "ascribe.selectPreviewBuild":
    "Picks the build the preview shows, for the previewed page's project, as the preview's **Build** picker does. It's the same choice as **Switch Build**.",
  "ascribe.switchBuild":
    "Picks the build you're looking at in the active file's project: the one the preview renders and the status bar names. Choosing the editor build follows `[editor] build`. See [The status bar](../editor/preview.md#the-status-bar).",
  "ascribe.checkEveryBuild":
    "Runs `ascribe check` over every build of the active file's project, and adds to the Problems panel the problems of builds other than the editor's, which the language server doesn't check. When no file of a project is active and the workspace has several projects, it asks which. See [The Problems panel](../editor/settings.md#the-problems-panel).",
  "ascribe.search":
    "Finds words as readers read the active file's project's pages, as you type, unsaved text included, and opens the line that wrote the place you choose. It searches the build the status bar names; its **Search Every Build** button searches every build. See [Search](../editor/writing.md#search).",
  "ascribe.projectMenu":
    "Shows the menu of the active file's project, as clicking its status bar item does: switch the build, check every build, show the server's output, switch project, restart its server, dim what the build leaves out, or open the preview.",
  "ascribe.switchProject":
    "Lists the workspace's projects, the sidebar's first, each with its folder and its server's state, and shows the one you choose in the sidebar, starting its server, until a file of another project is active. Shown while the workspace has several projects. See [Switch Project](../editor/sidebar.md#switch-project).",
  "ascribe.toggleBuildLens":
    "Turns the build lens on or off for the active file's project: the editor dims what the build you're looking at leaves out of each page, with a hover that says why. See [The build lens](../editor/preview.md#the-build-lens).",
  "ascribe.startReview":
    "Marks what changed in the preview, against a base it asks for, for the active page's project. Its server must be running: open one of its pages first. See [Review in the preview](../editor/review.md).",
  "ascribe.stopReview": "Turns review off for the active page's project, and frees its base.",
  "ascribe.changedPages":
    "Lists the pages the change touches in the preview's build; choosing one opens it and its preview.",
  "ascribe.refreshComments":
    "Reads the pull request's review threads from GitHub again, for the active page's project. See [Comments in the preview](../editor/review.md#comments-in-the-preview).",
  "ascribe.submitReview":
    "Opens the preview's dialog for submitting your review's comments, on the page it shows or the first changed page. Shown while you have comments to submit. It's in the Review view's title bar too.",
  "ascribe.openSourceChanges":
    "Opens one diff of the changed pages' source files, at the base against the working tree: each page's own file, and the fragments and pages it changed through. It's in the Review view's title bar too. See [The Review view](../editor/review.md#the-review-view).",
  "ascribe.promptAgentFile":
    "Builds a prompt for your agent about the active file's problems, and puts it where `ascribe.agents.promptTarget` says, without sending it. Shown when the file has problems. See [Prompt your agent](../agents/prompts.md).",
  "ascribe.promptAgentProject":
    "Builds a prompt for your agent about the problems of the active file's project, as Prompt Agent to Fix This File does. Shown when the project has problems.",
  "ascribe.newPage":
    "Starts a page from its type's template: asks for the type, each with what it's for, then the title, makes the page in the content root, and opens it with the first thing to fill in selected; the page names its type in its frontmatter. Run on a folder in the Explorer, it makes the page in that folder. It's in the Pages view's title bar too. See [Start a page](../editor/actions.md#start-a-page).",
  "ascribe.nextPlaceholder":
    "Selects the next thing the page's template left to fill in, after the cursor, so typing replaces it; after the last, it goes back to the first.",
  "ascribe.actions":
    "Opens the [actions bar](../editor/actions.md#the-actions-bar): the fixes for problems at the cursor, then the actions that apply to the cursor or selection.",
};

/** The actions' commands, which share one row. */
const actionCommands = new Set(ACTIONS.map(commandId));

/** The row that stands for every action's command. */
const ACTIONS_ROW = "ascribe.action.*";

/** What the actions' row says. */
const actionsRow = `Runs the action, such as **Ascribe: ${ACTIONS[0]?.title ?? ""}**, in a Markdown file of a project. Each of the [actions](../editor/actions.md) is a command.`;

/**
 * The availability of each setting or command newer than the extension's
 * first release, by id: `next` until a release ships it, then that release,
 * as `ascribe 0.2.0`. Remove an entry once every live line has it.
 */
const available: Record<string, string> = {
  "ascribe.submitReview": "ascribe 0.4.0",
  "ascribe.openSourceChanges": "ascribe 0.4.0",
  "ascribe.switchProject": "ascribe 0.4.0",
  "ascribe.search": "next",
};

/** The attribute block that ends a row's first cell, for an id in `available`. */
function availability(id: string): string {
  const spec = available[id];
  if (spec === undefined) return "";
  return /^[\w.-]+$/.test(spec) ? ` {available=${spec}}` : ` {available="${spec}"}`;
}

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
    languageModelTools: { name: string; toolReferenceName: string; userDescription: string }[];
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
  let out = `${HEADER}| Setting | Default | What it does |\n|---|---|---|\n`;
  for (const [key, setting] of Object.entries(manifest.contributes.configuration.properties)) {
    const value: unknown = setting.default;
    const empty = value === "" || (Array.isArray(value) && value.length === 0);
    const shown = typeof value === "string" ? value : JSON.stringify(value);
    const fallback = empty ? "empty" : `\`${shown}\``;
    let what = setting.markdownDescription ?? setting.description ?? "";
    (setting.enum ?? []).forEach((value, i) => {
      const description = setting.enumDescriptions?.[i];
      if (description) what += ` \`${value}\`: ${description}`;
    });
    out += `| \`${key}\`${availability(key)} | ${fallback} | ${cell(what)} |\n`;
  }
  return out;
}

/** The commands table: those the palette shows, in the manifest's order, the actions as one row. */
function commandTable(): string {
  const hidden = new Set(
    manifest.contributes.menus.commandPalette
      .filter((entry) => entry.when === "false")
      .map((entry) => entry.command),
  );
  let out = `${HEADER}| Command | What it does |\n|---|---|\n`;
  let actionsListed = false;
  for (const command of manifest.contributes.commands) {
    if (hidden.has(command.command)) continue;
    if (actionCommands.has(command.command)) {
      if (!actionsListed) {
        out += `| **${command.category}:** *an action*${availability(ACTIONS_ROW)} | ${cell(actionsRow)} |\n`;
      }
      actionsListed = true;
      continue;
    }
    const what = commands[command.command] ?? "";
    out += `| **${command.category}: ${command.title}**${availability(command.command)} | ${cell(what)} |\n`;
  }
  return out;
}

/** The actions table, in the registry's order. */
function actionTable(): string {
  let out = `${ACTIONS_HEADER}| Action | What it does | Where it applies |\n|---|---|---|\n`;
  for (const action of ACTIONS) {
    const lightbulb = action.lightbulb ? " Also in the lightbulb." : "";
    out += `| **${action.title}** | ${cell(action.description)}.${lightbulb} | ${cell(action.where)} |\n`;
  }
  return out;
}

/** The tools for agents, in the manifest's order, by the name chat references them with. */
function toolTable(): string {
  let out = `${HEADER}| Tool | What it gives |\n|---|---|\n`;
  for (const tool of manifest.contributes.languageModelTools) {
    out += `| \`#${tool.toolReferenceName}\`${availability(tool.name)} | ${cell(tool.userDescription)} |\n`;
  }
  return out;
}

const fragments: [string, string][] = [
  ["editor-settings.md", settings()],
  ["editor-commands.md", commandTable()],
  ["editor-actions.md", actionTable()],
  ["editor-tools.md", toolTable()],
];

describe("the editor guide", () => {
  it("describes every command in the palette but the actions', and nothing else", () => {
    const shown = manifest.contributes.commands
      .map((command) => command.command)
      .filter((id) => !actionCommands.has(id))
      .filter((id) =>
        manifest.contributes.menus.commandPalette.some(
          (entry) => entry.command === id && entry.when !== "false",
        ),
      );
    expect(Object.keys(commands).sort()).toEqual(shown.sort());
  });

  it("gives an availability only to settings and commands it shows", () => {
    const ids = [
      ...Object.keys(manifest.contributes.configuration.properties),
      ...Object.keys(commands),
      ...manifest.contributes.languageModelTools.map((tool) => tool.name),
      ACTIONS_ROW,
    ];
    expect(Object.keys(available).filter((id) => !ids.includes(id))).toEqual([]);
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
    const pages = readdirSync(new URL("docs/content/editor/", root)).filter((page) =>
      page.endsWith(".md"),
    );
    expect(
      pages.some((page) =>
        read(`docs/content/editor/${page}`).includes(`@include: ../_generated/${name}\n`),
      ),
      `no page in docs/content/editor/ includes _generated/${name}`,
    ).toBe(true);
  });
});
