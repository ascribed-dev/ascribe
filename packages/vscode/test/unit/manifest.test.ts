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
    menus: { commandPalette: { command: string; when: string }[] };
    configuration: { properties: Record<string, { default: unknown }> };
    grammars: { path: string; injectTo: string[]; scopeName: string }[];
    semanticTokenTypes: { id: string }[];
    semanticTokenModifiers: { id: string }[];
    semanticTokenScopes: { language: string; scopes: Record<string, string[]> }[];
  };
}

const manifest = JSON.parse(read("../../package.json")) as Manifest;

describe("package.json", () => {
  it("is private, so nothing can be published by accident", () => {
    expect(manifest.private).toBe(true);
  });

  it("activates only for workspaces that contain ascribe.toml", () => {
    expect(manifest.activationEvents).toEqual(["workspaceContains:**/ascribe.toml"]);
  });

  it("doesn't support untrusted workspaces, since it runs the project's binary", () => {
    expect(manifest.capabilities.untrustedWorkspaces.supported).toBe(false);
  });

  it("hides every command from the palette until a workspace has ascribe.toml", () => {
    const hidden = manifest.contributes.menus.commandPalette;
    expect(hidden.map((entry) => entry.command).sort()).toEqual(
      manifest.contributes.commands.map((command) => command.command).sort(),
    );
    for (const entry of hidden) expect(entry.when).toBe("ascribe.active");
  });

  it("declares the commands and settings the extension reads", () => {
    expect(manifest.contributes.commands.map((command) => command.command)).toEqual([
      "ascribe.restartServer",
      "ascribe.showOutput",
    ]);
    const properties = manifest.contributes.configuration.properties;
    expect(Object.keys(properties).sort()).toEqual([
      "ascribe.formatOnSave",
      "ascribe.maxCrashes",
      "ascribe.path",
      "ascribe.trace.server",
    ]);
    expect(properties["ascribe.formatOnSave"]?.default).toBe(false);
    expect(properties["ascribe.trace.server"]?.default).toBe("off");
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
 * crates/tessera-lsp/README.md): the extension must map every type and
 * modifier it declares to a theme scope. Skipped while that README has no
 * legend (phase 15 not merged).
 */
describe("semantic tokens and the server's legend", () => {
  const readme = new URL("../../../../crates/tessera-lsp/README.md", import.meta.url);
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
