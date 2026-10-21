import { existsSync, readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { PROMPT_COMMANDS, PROMPT_TARGETS } from "../../src/actions/prompt.js";
import { MCP_LABEL } from "../../src/agents/mcpServer.js";
import { SYNCED_KEY } from "../../src/agents/synced.js";
import { PAGES_CONTEXT } from "../../src/ui/sidebar.js";

const read = (relative: string) => readFileSync(new URL(relative, import.meta.url), "utf8");

interface Manifest {
  private?: boolean;
  capabilities: { untrustedWorkspaces: { supported: boolean | "limited" } };
  main: string;
  activationEvents: string[];
  contributes: {
    commands: { command: string }[];
    menus: {
      commandPalette: { command: string; when: string }[];
      "editor/title": { command: string; when: string }[];
      "view/item/context": { command: string; when: string }[];
      [menu: string]: { command?: string; when?: string }[];
    };
    views: Record<string, { id: string; name: string; when?: string }[]>;
    viewsWelcome: { view: string; when: string; contents: string }[];
    configuration: {
      properties: Record<string, { default: unknown; enum?: string[]; scope?: string }>;
    };
    languages: { id: string; filenames?: string[] }[];
    grammars: { path: string; injectTo: string[]; scopeName: string }[];
    semanticTokenTypes: { id: string }[];
    semanticTokenModifiers: { id: string }[];
    semanticTokenScopes: { language: string; scopes: Record<string, string[]> }[];
    keybindings: { command: string; key: string; mac?: string; when: string }[];
    chatSkills: { path: string; when?: string }[];
    mcpServerDefinitionProviders: { id: string; label: string }[];
    languageModelTools: {
      name: string;
      toolReferenceName: string;
      canBeReferencedInPrompt: boolean;
      when: string;
      modelDescription: string;
      inputSchema: { type: string; properties: Record<string, unknown>; required?: string[] };
    }[];
  };
}

const manifest = JSON.parse(read("../../package.json")) as Manifest;

describe("package.json", () => {
  it("shows the Pages view's item commands by the context values the view gives", () => {
    const shownOn = (command: string) =>
      manifest.contributes.menus["view/item/context"].find((entry) => entry.command === command)
        ?.when;
    expect(shownOn("ascribe.pages.placeHere")).toBe(
      `view == ascribe.pages && viewItem == ${PAGES_CONTEXT.unplaced}`,
    );
    // A section's label is its `label` in ascribe.nav, and a page's its `nav-label`.
    const labelled = [
      PAGES_CONTEXT.section,
      PAGES_CONTEXT.page,
      PAGES_CONTEXT.own,
      PAGES_CONTEXT.unplaced,
    ].map((value) => value.replace("ascribe.pages.", ""));
    expect(shownOn("ascribe.pages.renameLabel")).toBe(
      `view == ascribe.pages && viewItem =~ /^ascribe\\.pages\\.(${labelled.join("|")})$/`,
    );
  });

  it("offers Switch Project only while the workspace has several projects", () => {
    const shown = Object.values(manifest.contributes.menus)
      .flat()
      .filter((entry) => entry.command === "ascribe.switchProject");
    // The Command Palette, and the title bars of Pages and Content model.
    expect(shown.map((entry) => entry.when)).toEqual([
      "ascribe.active && ascribe.multipleProjects",
      "view == ascribe.pages && ascribe.multipleProjects",
      "view == ascribe.model && ascribe.multipleProjects",
    ]);
  });

  it("has no Projects view, and keeps Pages, with the welcome messages, in every workspace", () => {
    const views = manifest.contributes.views.ascribe ?? [];
    // Decision #654a's order: Project first.
    expect(views.map((view) => view.id)).toEqual([
      "ascribe.project",
      "ascribe.review",
      "ascribe.thisPage",
      "ascribe.pages",
      "ascribe.model",
    ]);
    expect(views.find((view) => view.id === "ascribe.pages")?.when).toBeUndefined();
    const welcome = manifest.contributes.viewsWelcome;
    expect(welcome.map((entry) => [entry.view, entry.when])).toEqual([
      ["ascribe.pages", "!isWorkspaceTrusted"],
      ["ascribe.pages", "isWorkspaceTrusted && !ascribe.active"],
      ["ascribe.pages", "ascribe.active && ascribe.sidebar.failed"],
      ["ascribe.model", "ascribe.active && ascribe.sidebar.failed"],
    ]);
    const named = [
      ...manifest.contributes.commands.map((command) => command.command),
      ...views.map((view) => `${view.id} ${view.name}`),
    ];
    expect(named.filter((name) => /\bprojects\b/i.test(name))).toEqual([]);
  });

  it("is private, so nothing can be published by accident", () => {
    expect(manifest.private).toBe(true);
  });

  it("activates for workspaces that contain ascribe.toml, and to restore a preview", () => {
    expect(manifest.activationEvents).toEqual([
      "workspaceContains:**/ascribe.toml",
      "onWebviewPanel:ascribe.preview",
    ]);
  });

  it("runs nothing in an untrusted workspace, since it runs the project's binary, and says so", () => {
    // `activate` waits for trust before it starts anything.
    expect(manifest.capabilities.untrustedWorkspaces.supported).toBe("limited");
  });

  it("hides every command from the palette until a workspace has ascribe.toml", () => {
    const hidden = manifest.contributes.menus.commandPalette;
    expect(hidden.map((entry) => entry.command).sort()).toEqual(
      manifest.contributes.commands.map((command) => command.command).sort(),
    );
    for (const entry of hidden) {
      // A review thread's buttons in the source editor act on that thread, so
      // the palette never shows them.
      if (entry.command.startsWith("ascribe.review.")) expect(entry.when).toBe("false");
      // So do the links of the sidebar's welcome, which act on the project it shows.
      else if (entry.command.startsWith("ascribe.sidebar.")) expect(entry.when).toBe("false");
      // And the Pages and Content model views', which act on their item or view.
      else if (
        entry.command.startsWith("ascribe.pages.") ||
        entry.command.startsWith("ascribe.model.")
      )
        expect(entry.when).toBe("false");
      // So do the walkthrough's buttons, which bring back a page first.
      else if (entry.command.startsWith("ascribe.walkthrough.")) expect(entry.when).toBe("false");
      // Prompt agent on one problem is a code action on that problem.
      else if (entry.command === "ascribe.promptAgent") expect(entry.when).toBe("false");
      // An action, the actions bar, and Go to Next Placeholder are for a page,
      // so they're shown in a Markdown file of a project.
      else if (
        entry.command === "ascribe.actions" ||
        entry.command === "ascribe.nextPlaceholder" ||
        entry.command.startsWith("ascribe.action.")
      ) {
        expect(entry.when).toBe("ascribe.inProject && editorLangId == markdown");
      } else expect(entry.when).toMatch(/^ascribe\.active\b/);
    }
  });

  it("binds the actions bar's key only in a page of a project, with the editor focused", () => {
    // Ctrl+K A is unbound in VS Code's default keymaps on every platform;
    // Ctrl+Alt+A would be AltGr+A, which types a letter in some layouts.
    expect(manifest.contributes.keybindings).toEqual([
      {
        command: "ascribe.actions",
        key: "ctrl+k a",
        mac: "cmd+k a",
        when: "editorTextFocus && ascribe.inProject && editorLangId == markdown",
      },
    ]);
  });

  it("shows Start Review on the preview's title bar while review is off, then Changed Pages and Refresh Comments", () => {
    const title = manifest.contributes.menus["editor/title"];
    expect(title.find((entry) => entry.command === "ascribe.startReview")?.when).toBe(
      "activeWebviewPanelId == 'ascribe.preview' && !ascribe.reviewOn",
    );
    expect(title.find((entry) => entry.command === "ascribe.changedPages")?.when).toBe(
      "activeWebviewPanelId == 'ascribe.preview' && ascribe.reviewOn",
    );
    expect(title.find((entry) => entry.command === "ascribe.refreshComments")?.when).toBe(
      "activeWebviewPanelId == 'ascribe.preview' && ascribe.reviewOn",
    );
  });

  it("names the previews as the README does, keeping the preview command's id", () => {
    const titles = new Map(
      (manifest.contributes.commands as { command: string; title: string }[]).map((c) => [
        c.command,
        c.title,
      ]),
    );
    expect(titles.get("ascribe.openPagePreview")).toBe("Open Page Preview");
    expect(titles.get("ascribe.openPreview")).toBe("Open Page Preview to the Side");
    expect(titles.get("ascribe.openSitePreview")).toBe("Open Site Preview");
    // Open Site Preview is on a page's title bar only while a dev server has written dev.json.
    const title = manifest.contributes.menus["editor/title"];
    expect(title.find((entry) => entry.command === "ascribe.openSitePreview")?.when).toBe(
      "ascribe.active && ascribe.devServer && resourceLangId == markdown",
    );
  });

  it("declares the commands and settings the extension reads", () => {
    // The actions' commands are the registry's, held to it by actions.test.ts.
    const commands = manifest.contributes.commands
      .map((command) => command.command)
      .filter((id) => !id.startsWith("ascribe.action."));
    expect(commands).toEqual([
      "ascribe.openPreview",
      "ascribe.openPagePreview",
      "ascribe.openSitePreview",
      "ascribe.selectPreviewBuild",
      ...Object.values(PROMPT_COMMANDS),
      "ascribe.newPage",
      "ascribe.actions",
      "ascribe.nextPlaceholder",
      "ascribe.switchBuild",
      "ascribe.checkEveryBuild",
      "ascribe.search",
      "ascribe.projectMenu",
      "ascribe.toggleBuildLens",
      "ascribe.startReview",
      "ascribe.stopReview",
      "ascribe.changedPages",
      "ascribe.refreshComments",
      "ascribe.submitReview",
      "ascribe.openSourceChanges",
      "ascribe.review.openPage",
      "ascribe.review.openThread",
      "ascribe.review.pageSourceChanges",
      "ascribe.review.showAdvice",
      "ascribe.review.hideAdvice",
      "ascribe.showOutput",
      "ascribe.restartServer",
      "ascribe.switchProject",
      "ascribe.sidebar.showOutput",
      "ascribe.sidebar.restart",
      "ascribe.pages.placeHere",
      "ascribe.pages.renameLabel",
      "ascribe.pages.openPreview",
      "ascribe.pages.copyLink",
      "ascribe.pages.renameOrMove",
      "ascribe.pages.findReferences",
      "ascribe.pages.revealInExplorer",
      "ascribe.pages.newPageHere",
      "ascribe.pages.orphansOnly",
      "ascribe.pages.showAll",
      "ascribe.model.findReferences",
      "ascribe.model.rename",
      "ascribe.model.insert",
      "ascribe.model.unusedOnly",
      "ascribe.model.showAll",
      "ascribe.review.replyNow",
      "ascribe.review.addToReview",
      "ascribe.review.resolve",
      "ascribe.review.reopen",
      "ascribe.review.promptAgent",
      "ascribe.walkthrough.preview",
      "ascribe.walkthrough.actions",
      "ascribe.walkthrough.lens",
    ]);
    const properties = manifest.contributes.configuration.properties;
    expect(Object.keys(properties).sort()).toEqual([
      "ascribe.agents.promptTarget",
      "ascribe.formatOnSave",
      "ascribe.maxCrashes",
      "ascribe.path",
      "ascribe.preview.scrollEditorWithPreview",
      "ascribe.preview.scrollPreviewWithEditor",
      "ascribe.projects.exclude",
      "ascribe.review.sourceComments",
      "ascribe.startServers",
      "ascribe.trace.server",
    ]);
    expect(properties["ascribe.formatOnSave"]?.default).toBe(false);
    expect(properties["ascribe.trace.server"]?.default).toBe("off");
    expect(properties["ascribe.startServers"]?.default).toBe("onDemand");
    expect(properties["ascribe.projects.exclude"]?.default).toEqual([]);
    expect(properties["ascribe.review.sourceComments"]?.default).toBe("auto");
    expect(properties["ascribe.review.sourceComments"]?.enum).toEqual(["auto", "on", "off"]);
    expect(properties["ascribe.agents.promptTarget"]?.default).toBe("clipboard");
    expect(properties["ascribe.agents.promptTarget"]?.enum).toEqual([...PROMPT_TARGETS]);
  });

  it("offers no command that runs ascribe observe: the advice's quick fix only copies it", () => {
    for (const command of manifest.contributes.commands) {
      expect(command.command).not.toMatch(/observe/i);
    }
    const source = new URL("../../src/", import.meta.url);
    const files = readdirSync(source, { recursive: true, encoding: "utf8" }).filter((f) =>
      f.endsWith(".ts"),
    );
    expect(files.length).toBeGreaterThan(0);
    for (const file of files) {
      const text = readFileSync(new URL(file, source), "utf8");
      expect(text, file).not.toMatch(/ascribe observe|["'`]observe["'`]/);
    }
    // The quick fix's command copies the line the server gives it.
    const extension = read("../../src/extension.ts");
    expect(extension).toMatch(
      /registerCommand\("ascribe\.copyCommand"[\s\S]*?clipboard\.writeText\(line\)/,
    );
  });

  it("lets startServers be on demand or all, per window", () => {
    const setting = manifest.contributes.configuration.properties["ascribe.startServers"];
    expect(setting?.enum).toEqual(["onDemand", "all"]);
    expect(setting?.scope).toBe("window");
  });

  it("injects its grammars into markdown, and the files exist", () => {
    for (const grammar of manifest.contributes.grammars) {
      expect(grammar.injectTo).toEqual(["text.html.markdown"]);
      expect(existsSync(new URL(`../../${grammar.path}`, import.meta.url)), grammar.path).toBe(
        true,
      );
      const scopeName = (JSON.parse(read(`../../${grammar.path}`)) as { scopeName: string })
        .scopeName;
      expect(scopeName).toBe(grammar.scopeName);
    }
  });
});

/**
 * The language server's semantic token legend is a contract (see
 * crates/ascribe-lsp/README.md): the extension must map every type and
 * modifier it declares to a theme scope. Skipped if that README has no
 * legend.
 */
describe("semantic tokens and the server's legend", () => {
  const readme = new URL("../../../../crates/ascribe-lsp/README.md", import.meta.url);
  const legend = existsSync(readme) ? readFileSync(readme, "utf8") : "";
  const types = [...legend.matchAll(/^\| \d+ \| `(ascribe\w+)` \|.*\| `([^`]+)` \|$/gm)].map(
    (match) => ({ id: match[1] ?? "", scope: match[2] ?? "" }),
  );
  const modifiers = [...legend.matchAll(/^\| \d+ \| `(\w+)` \| An `ascribe/gm)].map(
    (match) => match[1] ?? "",
  );

  it.skipIf(types.length === 0)("declares every token type, mapped to the suggested scope", () => {
    const declared = manifest.contributes.semanticTokenTypes.map((type) => type.id);
    const scopes = manifest.contributes.semanticTokenScopes.find(
      (entry) => entry.language === "markdown",
    )?.scopes;
    for (const { id, scope } of types) {
      expect(declared, id).toContain(id);
      expect(scopes?.[id], id).toContain(scope);
    }
    expect(declared.sort()).toEqual(types.map((type) => type.id).sort());
  });

  it.skipIf(modifiers.length === 0)("declares every token modifier", () => {
    const declared = manifest.contributes.semanticTokenModifiers.map((modifier) => modifier.id);
    expect(declared.sort()).toEqual([...modifiers].sort());
  });

  it("offers ascribe mcp to VS Code's agents, through one provider", () => {
    // The id is the one src/agents/mcp.ts registers.
    const mcp = read("../../src/agents/mcp.ts");
    expect(manifest.contributes.mcpServerDefinitionProviders).toEqual([
      { id: "ascribe.mcp", label: MCP_LABEL },
    ]);
    expect(mcp).toContain('const MCP_PROVIDER = "ascribe.mcp";');
  });

  it("declares the tools the extension registers, each one chat can name", () => {
    const tools = read("../../src/agents/tools.ts");
    const registered = [...tools.matchAll(/^ {2}\w+: "(ascribe_\w+)",$/gm)].map((m) => m[1]);
    const declared = manifest.contributes.languageModelTools;
    expect(declared.map((t) => t.name)).toEqual(registered);
    for (const tool of declared) {
      expect(tool.toolReferenceName).toBe(tool.name);
      expect(tool.canBeReferencedInPrompt).toBe(true);
      expect(tool.when).toBe("ascribe.active");
      // Optional input only: a tool with none acts on the active editor's project.
      expect(tool.inputSchema).toMatchObject({ type: "object", properties: { path: {} } });
      expect(tool.inputSchema.required).toBeUndefined();
    }
    // The problems tool says what it doesn't cover, and what does.
    const problems = declared.find((t) => t.name === "ascribe_editor_problems");
    expect(problems?.modelDescription).toContain("editor's build only");
    expect(problems?.modelDescription).toContain("ascribe_check");
  });

  it("offers chat the skill the build copies from its one home, where no synced copy is", () => {
    expect(manifest.contributes.chatSkills).toEqual([
      { path: "./dist/skills/ascribe/SKILL.md", when: `ascribe.active && !${SYNCED_KEY}` },
    ]);
    // The build copies `@ascribed/cli`'s folder there, and the package keeps it.
    const build = read("../../esbuild.mjs");
    expect(build).toContain('new URL("../cli/skills/ascribe/", import.meta.url)');
    expect(build).toContain('cpSync(skill, "dist/skills/ascribe", { recursive: true });');
    expect(read("../../.vscodeignore").split("\n")).toContain("!dist/skills/**");
    // Chat takes a skill whose folder is named as its `name` says.
    expect(read("../../../cli/skills/ascribe/SKILL.md")).toMatch(/^---\nname: ascribe\n/);
  });
});

describe("languages", () => {
  // The names are the model crate's: one home per fact, and this compares.
  const declared = (file: string, constant: string) => {
    const match = new RegExp(`pub const ${constant}: &str = "([^"]+)";`).exec(
      read(`../../../../crates/ascribe-model/src/${file}`),
    );
    if (!match?.[1]) throw new Error(`no ${constant} in ${file}`);
    return match[1];
  };

  it("gives Ascribe's TOML files without a .toml extension to TOML", () => {
    const toml = manifest.contributes.languages.find((language) => language.id === "toml");
    expect(toml?.filenames).toEqual([
      declared("nav.rs", "NAV_FILE"),
      declared("permalinks.rs", "PERMALINKS_FILE"),
      declared("lock.rs", "LOCK_FILE"),
    ]);
  });
});
