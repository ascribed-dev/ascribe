import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const read = (relative: string) => readFileSync(new URL(relative, import.meta.url), "utf8");

interface Manifest {
  private?: boolean;
  capabilities: { untrustedWorkspaces: { supported: boolean } };
  main: string;
  activationEvents: string[];
  contributes: {
    commands: { command: string }[];
    menus: {
      commandPalette: { command: string; when: string }[];
      "editor/title": { command: string; when: string }[];
    };
    configuration: {
      properties: Record<string, { default: unknown; enum?: string[]; scope?: string }>;
    };
    grammars: { path: string; injectTo: string[]; scopeName: string }[];
    semanticTokenTypes: { id: string }[];
    semanticTokenModifiers: { id: string }[];
    semanticTokenScopes: { language: string; scopes: Record<string, string[]> }[];
    keybindings: { command: string; key: string; mac?: string; when: string }[];
  };
}

const manifest = JSON.parse(read("../../package.json")) as Manifest;

describe("package.json", () => {
  it("is private, so nothing can be published by accident", () => {
    expect(manifest.private).toBe(true);
  });

  it("activates for workspaces that contain ascribe.toml, and to restore a preview", () => {
    expect(manifest.activationEvents).toEqual([
      "workspaceContains:**/ascribe.toml",
      "onWebviewPanel:ascribe.preview",
    ]);
  });

  it("doesn't support untrusted workspaces, since it runs the project's binary", () => {
    expect(manifest.capabilities.untrustedWorkspaces.supported).toBe(false);
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
      // So do the Projects view's buttons, which act on their project.
      else if (entry.command.startsWith("ascribe.projects.")) expect(entry.when).toBe("false");
      // So do the walkthrough's buttons, which bring back a page first.
      else if (entry.command.startsWith("ascribe.walkthrough.")) expect(entry.when).toBe("false");
      // An action, and the actions bar, are for a page, so they're shown in a
      // Markdown file of a project.
      else if (entry.command === "ascribe.actions" || entry.command.startsWith("ascribe.action.")) {
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
      "ascribe.restartServer",
      "ascribe.showOutput",
      "ascribe.openPagePreview",
      "ascribe.openPreview",
      "ascribe.openSitePreview",
      "ascribe.selectPreviewBuild",
      "ascribe.switchBuild",
      "ascribe.projectMenu",
      "ascribe.toggleBuildLens",
      "ascribe.projects.refresh",
      "ascribe.projects.showOutput",
      "ascribe.projects.restart",
      "ascribe.startReview",
      "ascribe.stopReview",
      "ascribe.changedPages",
      "ascribe.refreshComments",
      "ascribe.review.replyNow",
      "ascribe.review.addToReview",
      "ascribe.review.resolve",
      "ascribe.review.reopen",
      "ascribe.actions",
      "ascribe.walkthrough.preview",
      "ascribe.walkthrough.actions",
      "ascribe.walkthrough.lens",
    ]);
    const properties = manifest.contributes.configuration.properties;
    expect(Object.keys(properties).sort()).toEqual([
      "ascribe.formatOnSave",
      "ascribe.maxCrashes",
      "ascribe.path",
      "ascribe.preview.scrollEditorWithPreview",
      "ascribe.preview.scrollPreviewWithEditor",
      "ascribe.review.sourceComments",
      "ascribe.startServers",
      "ascribe.trace.server",
    ]);
    expect(properties["ascribe.formatOnSave"]?.default).toBe(false);
    expect(properties["ascribe.trace.server"]?.default).toBe("off");
    expect(properties["ascribe.startServers"]?.default).toBe("onDemand");
    expect(properties["ascribe.review.sourceComments"]?.default).toBe("auto");
    expect(properties["ascribe.review.sourceComments"]?.enum).toEqual(["auto", "on", "off"]);
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
});
