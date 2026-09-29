import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { beforeAll, describe, expect, it } from "vitest";
import { INITIAL, Registry, parseRawGrammar, type IGrammar } from "vscode-textmate";
import { loadWASM, createOnigScanner, createOnigString } from "vscode-oniguruma";

const require = createRequire(import.meta.url);

/**
 * VS Code's own markdown grammar (MIT, from microsoft/vscode, unchanged apart
 * from formatting), so the tests see the scopes the injection really meets.
 */
const markdown = readFileSync(
  new URL("../fixtures/markdown.tmLanguage.json", import.meta.url),
  "utf8",
);

let grammar: IGrammar;

beforeAll(async () => {
  const wasm = readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm"));
  await loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
  const read = (name: string) =>
    readFileSync(new URL(`../../syntaxes/${name}.tmLanguage.json`, import.meta.url), "utf8");
  const injections: Record<string, string> = {
    "tessera.injection": read("tessera.injection"),
    "tessera.injection.nested": read("tessera.injection.nested"),
  };
  const registry = new Registry({
    onigLib: Promise.resolve({ createOnigScanner, createOnigString }),
    loadGrammar: async (scope) => {
      if (scope === "text.html.markdown")
        return parseRawGrammar(markdown, "markdown.tmLanguage.json");
      const injection = injections[scope];
      if (injection) return parseRawGrammar(injection, `${scope}.json`);
      return null;
    },
    getInjections: (scope) =>
      scope === "text.html.markdown" ? ["tessera.injection", "tessera.injection.nested"] : [],
  });
  const loaded = await registry.loadGrammar("text.html.markdown");
  if (!loaded) throw new Error("the markdown grammar didn't load");
  grammar = loaded;
});

/** A highlighted piece of a line: its text, and its Tessera scopes without the `.ascribe` suffix. */
type Piece = [text: string, scopes: string[]];

function tokenize(source: string): Piece[][] {
  let state = INITIAL;
  return source.split("\n").map((line) => {
    const result = grammar.tokenizeLine(line, state);
    state = result.ruleStack;
    return result.tokens.map((token): Piece => {
      const scopes = token.scopes
        .filter((scope) => scope.endsWith(".ascribe"))
        .map((scope) => scope.replace(/\.ascribe$/, ""));
      return [line.slice(token.startIndex, token.endIndex), scopes];
    });
  });
}

/** The text of each piece that has the scope. */
function textsWith(line: Piece[], scope: string): string[] {
  return line.filter(([, scopes]) => scopes.includes(scope)).map(([text]) => text);
}

function highlighted(line: string): boolean {
  return (tokenize(line)[0] ?? []).some(([, scopes]) => scopes.length > 0);
}

const first = (source: string) => tokenize(source)[0] ?? [];

describe("directive lines", () => {
  it("highlights the sigil, name, attribute block, colon, and primary", () => {
    const line = first("@note {type=caution}: Back up your database first.");
    expect(textsWith(line, "punctuation.definition.directive")).toEqual(["@"]);
    expect(textsWith(line, "keyword.control.directive")).toEqual(["note"]);
    expect(textsWith(line, "meta.attributes").join("")).toBe("{type=caution}");
    expect(textsWith(line, "entity.other.attribute-name")).toEqual(["type"]);
    expect(textsWith(line, "string.unquoted.attribute-value")).toEqual(["caution"]);
    expect(textsWith(line, "punctuation.section.attributes.begin")).toEqual(["{"]);
    expect(textsWith(line, "punctuation.section.attributes.end")).toEqual(["}"]);
    expect(textsWith(line, "punctuation.separator.directive")).toEqual([":"]);
    expect(textsWith(line, "meta.primary").join("")).toBe("Back up your database first.");
  });

  it("highlights a directive with no attributes or primary", () => {
    for (const name of ["steps", "details"]) {
      const line = first(`@${name}`);
      expect(textsWith(line, "keyword.control.directive")).toEqual([name]);
    }
  });

  it("highlights the colon that opens a container, with no primary", () => {
    for (const source of ["@note {type=tip}:", "@variant {pm=npm}:", "@note:", "@note:   "]) {
      const line = first(source);
      expect(textsWith(line, "punctuation.separator.directive"), source).toEqual([":"]);
      expect(textsWith(line, "meta.primary").join(""), source).toBe("");
    }
  });

  it("highlights @end alone on its line", () => {
    const line = first("@end");
    expect(textsWith(line, "keyword.control.end")).toEqual(["end"]);
    expect(textsWith(line, "punctuation.definition.directive")).toEqual(["@"]);
    expect(highlighted("@end   ")).toBe(true);
    expect(highlighted("@end \t")).toBe(true);
  });

  it("doesn't highlight anything after @end", () => {
    expect(highlighted("@end of the road")).toBe(false);
    expect(highlighted("@end:")).toBe(false);
  });

  it("colors an identifier primary as one token (@id, @include)", () => {
    const id = first("@id: try-in-browser");
    expect(textsWith(id, "string.unquoted.identifier.primary")).toEqual(["try-in-browser"]);
    const include = first("@include {heading=false}: guides/setup.md#install");
    expect(textsWith(include, "string.unquoted.identifier.primary")).toEqual([
      "guides/setup.md#install",
    ]);
    expect(textsWith(include, "entity.other.attribute-name")).toEqual(["heading"]);
  });

  it("doesn't highlight a second word after an identifier primary", () => {
    expect(highlighted("@id: two words")).toBe(false);
  });

  it("colors an availability spec as one line primary (@available)", () => {
    const line = first("@available: cloud ga 3.4, on-prem preview");
    expect(textsWith(line, "constant.other.availability")).toEqual([
      "cloud ga 3.4, on-prem preview",
    ]);
  });

  it("doesn't read phrases or markdown inside a line primary", () => {
    const line = first("@available: {feature}");
    expect(textsWith(line, "meta.phrase")).toEqual([]);
  });

  it("highlights project widgets, whose names contain a hyphen", () => {
    const line = first("@api-endpoint {method=get}: Lists workspaces.");
    expect(textsWith(line, "entity.name.function.widget")).toEqual(["api-endpoint"]);
    expect(textsWith(line, "entity.other.attribute-name")).toEqual(["method"]);
    expect(textsWith(line, "meta.primary").join("")).toBe("Lists workspaces.");
    expect(highlighted("@code-sample")).toBe(true);
  });

  it("highlights phrases in a text primary", () => {
    const line = first("@note: Sign in to {cloud} first.");
    expect(textsWith(line, "variable.other.phrase")).toEqual(["cloud"]);
  });

  it("stops at the first line of a text primary (the rest is the server's)", () => {
    const [head, rest] = tokenize("@note: Back up your database\nbefore you upgrade.");
    expect(textsWith(head ?? [], "meta.primary").join("")).toBe("Back up your database");
    expect(rest?.every(([, scopes]) => scopes.length === 0)).toBe(true);
  });

  describe("indentation (Q1)", () => {
    it("allows up to three extra spaces before a directive line and @end", () => {
      for (const indent of ["", " ", "  ", "   "]) {
        expect(highlighted(`${indent}@note: Hello.`), JSON.stringify(indent)).toBe(true);
        expect(highlighted(`${indent}@end`), JSON.stringify(indent)).toBe(true);
      }
    });

    it("leaves indented code (four spaces or a tab) alone", () => {
      expect(highlighted("    @note: Hello.")).toBe(false);
      expect(highlighted("\t@end")).toBe(false);
    });

    it("follows blockquote markers", () => {
      expect(highlighted("> @note: Hello.")).toBe(true);
      expect(highlighted("> > @end")).toBe(true);
      expect(highlighted(">@end")).toBe(true);
    });

    it("accepts the deeper indentation of a directive inside a list item", () => {
      // TextMate can't see the item's content column, so inside a list or a
      // quote any indentation is taken.
      const [, ordered, , , , nested] = tokenize(
        "1. Install\n   @note: Keep the key safe.\n\n- a\n  - b\n      @note: Deeper.",
      );
      expect(ordered?.some(([, scopes]) => scopes.includes("keyword.control.directive"))).toBe(
        true,
      );
      expect(nested?.some(([, scopes]) => scopes.includes("keyword.control.directive"))).toBe(true);
    });

    it("takes a directive line in a blockquote, and still leaves top-level code alone", () => {
      const lines = tokenize("> Quote\n> @note: Hello.\n\n    @end");
      expect(lines[1]?.some(([, scopes]) => scopes.includes("keyword.control.directive"))).toBe(
        true,
      );
      expect(lines[3]?.every(([, scopes]) => scopes.length === 0)).toBe(true);
    });
  });

  describe("what isn't a directive (§3.2)", () => {
    it("ignores unknown names", () => {
      for (const source of ["@astrojs/react", "@timestamp", "@warning: Careful.", "@Note: Hi."]) {
        expect(highlighted(source), source).toBe(false);
      }
    });

    it("ignores an @ that isn't at line start", () => {
      expect(highlighted("Write to support@example.com.")).toBe(false);
      expect(highlighted("Use @note: like this.")).toBe(false);
    });

    it("ignores text that fits no part of a directive line", () => {
      expect(highlighted("@steps foo")).toBe(false);
      expect(highlighted("@note hello: text")).toBe(false);
    });

    it("ignores directive lines in fenced code", () => {
      const lines = tokenize("```\n@note: Hello.\n@end\n```");
      for (const line of lines) expect(line.every(([, scopes]) => scopes.length === 0)).toBe(true);
    });

    it("ignores directive-shaped text in a code span", () => {
      expect(highlighted("`@note: Hello.`")).toBe(false);
    });
  });

  describe("attribute blocks (§3.3)", () => {
    it("highlights quoted strings, escapes, and sets", () => {
      const line = first(String.raw`@note {label="Say \"hi\"", platform=cloud|on-prem}: Text.`);
      expect(textsWith(line, "string.quoted.double").join("")).toContain(String.raw`Say \"hi\"`);
      expect(textsWith(line, "constant.character.escape")).toEqual([
        String.raw`\"`,
        String.raw`\"`,
      ]);
      expect(textsWith(line, "entity.other.attribute-name")).toEqual(["label", "platform"]);
      expect(textsWith(line, "punctuation.separator.attributes")).toEqual([",", "|"]);
      expect(textsWith(line, "string.unquoted.attribute-value")).toEqual(["cloud", "on-prem"]);
    });

    it("keeps a brace inside a quoted string in the block", () => {
      const line = first('@note {label="a}b"}: Text.');
      expect(textsWith(line, "meta.attributes").join("")).toBe('{label="a}b"}');
      expect(textsWith(line, "meta.primary").join("")).toBe("Text.");
    });

    it("runs an unclosed block to the end of the line (§3.3)", () => {
      const line = first("@note {type=caution: Text");
      expect(textsWith(line, "keyword.control.directive")).toEqual(["note"]);
      expect(textsWith(line, "meta.attributes").join("")).toBe("{type=caution: Text");
      expect(textsWith(line, "punctuation.separator.directive")).toEqual([]);
    });

    it("accepts an empty block and spacing anywhere", () => {
      expect(textsWith(first("@note {}: Text."), "meta.attributes").join("")).toBe("{}");
      const spaced = first("@note   { type = tip }  :   Text.");
      expect(textsWith(spaced, "entity.other.attribute-name")).toEqual(["type"]);
      expect(textsWith(spaced, "meta.primary").join("")).toBe("Text.");
      const tight = first("@note{type=tip}:Text.");
      expect(textsWith(tight, "keyword.control.directive")).toEqual(["note"]);
      expect(textsWith(tight, "meta.primary").join("")).toBe("Text.");
    });
  });
});

describe("phrases (§5.1)", () => {
  it("highlights {key} in prose, against punctuation and words", () => {
    const line = first("Sign in to {cloud} and {cloud}'s {cloud}-hosted, or {api-v2}.");
    expect(textsWith(line, "meta.phrase")).toEqual([
      "{",
      "cloud",
      "}",
      "{",
      "cloud",
      "}",
      "{",
      "cloud",
      "}",
      "{",
      "api-v2",
      "}",
    ]);
    expect(textsWith(line, "variable.other.phrase")).toEqual(["cloud", "cloud", "cloud", "api-v2"]);
  });

  it("ignores an escaped phrase", () => {
    expect(highlighted(String.raw`Write \{cloud} literally.`)).toBe(false);
  });

  it("ignores text that isn't shaped like a key", () => {
    for (const source of ["{Cloud}", "{1x}", "{}", "{a b}", "{a=b}", "{ cloud }"]) {
      expect(highlighted(`x ${source} y`), source).toBe(false);
    }
  });

  it("ignores phrases in code spans, fenced code, and indented code", () => {
    expect(highlighted("Use `{cloud}` here.")).toBe(false);
    for (const line of tokenize("```yaml\nname: {cloud}\n```")) {
      expect(line.every(([, scopes]) => scopes.length === 0)).toBe(true);
    }
    expect(highlighted("    name: {cloud}")).toBe(false);
  });
});
