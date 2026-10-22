// Every surface that says what Ascribe is says it in the words of their one
// home, design/words.toml. A surface generated from the home is generated; one
// that can't be is compared with it here, so changing a line in the home
// fails this file until every surface agrees. design/README.md lists them.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { parse } from "smol-toml";
import { describe, expect, test } from "vitest";
import { readSources } from "./assets.ts";
import { ROOT } from "./specimen.ts";
import { author, readWords, SHORT_MAX, words, WORDS } from "./words.ts";

const read = (file: string): string => readFileSync(join(ROOT, file), "utf8");
const home = words();

/** A package's manifest, as far as it says what the package is. */
interface Manifest {
  name: string;
  private?: boolean;
  description?: string;
  keywords?: string[];
  homepage?: string;
  author?: unknown;
}

const manifest = (file: string): Manifest => JSON.parse(read(file)) as Manifest;

/** The npm packages a release publishes: every manifest under packages/ that isn't private. */
function published(): string[] {
  return execFileSync("git", ["ls-files", "--", "packages/**package.json"], {
    cwd: ROOT,
    encoding: "utf8",
  })
    .split("\n")
    .filter((file) => file !== "" && manifest(file).private !== true);
}

/** A Markdown file's first paragraph: its first block after the frontmatter that isn't HTML. */
function firstParagraph(markdown: string): string {
  const body = markdown.replace(/\r\n/g, "\n").replace(/^---\n[\s\S]*?\n---\n/, "");
  const block = body
    .split(/\n\s*\n/)
    .map((each) => each.trim())
    .find((each) => each !== "" && !each.startsWith("<"));
  return (block ?? "").replace(/\s*\n\s*/g, " ");
}

/** The address of each permalink the docs publish: `https://ascribed-dev.com/go/<name>/`. */
function permalinks(): Set<string> {
  const docs = parse(read("docs/ascribe.toml")) as {
    consumer: { site: string; "permalink-path": string };
  };
  const list = parse(read("docs/ascribe.permalinks")) as { permalinks: Record<string, string> };
  const at = `${docs.consumer.site.replace(/\/$/, "")}${docs.consumer["permalink-path"]}`;
  return new Set(Object.keys(list.permalinks).map((name) => `${at}${name}/`));
}

/** The short form, then the package's job in a sentence or more of its own. */
function startsWithTheShortForm(description: string | undefined): boolean {
  return (
    description !== undefined &&
    description.startsWith(`${home.short} `) &&
    /^[A-Z].*\.$/.test(description.slice(home.short.length + 1))
  );
}

describe("design/words.toml", () => {
  test("holds the story's headline and sentence, as the story quotes them", () => {
    const story = read("project-docs/brand/story.md");
    expect(story).toContain(`\n> ${home.headline}\n`);
    expect(story).toContain(`\n> ${home.sentence}\n`);
  });

  test("its short form is the sentence's first clause, as a line", () => {
    const clause = home.short.replace(/\.$/, "");
    expect(home.short).toMatch(/\.$/);
    expect(home.sentence).toMatch(
      new RegExp(`^Ascribe is ${clause[0]?.toLowerCase() ?? ""}${clause.slice(1)}\\b`),
    );
    expect(home.short.length).toBeLessThan(SHORT_MAX);
  });

  test("a missing or too long line names the problem", () => {
    const text = read(WORDS);
    expect(() => readWords(text.replace(/^headline = .*$/m, ""))).toThrow(
      `${WORDS} headline: a line of text`,
    );
    expect(() => readWords(text.replace(/^short = .*$/m, `short = "${"a".repeat(120)}"`))).toThrow(
      `${WORDS} short: under ${SHORT_MAX} characters, not 120`,
    );
    expect(() => readWords(text.replace(/^url = .*$/m, ""))).toThrow(
      `${WORDS} maker.url: a line of text`,
    );
  });
});

const LANDING = "site/home/content/index.md";

// The landing page's frontmatter `description`, which is written as a
// double-quoted YAML string, since the sentence has a colon in it.
const landingDescription = (): string => {
  const line = read(LANDING)
    .split("\n")
    .find((l) => l.startsWith("description: "));
  const value = (line ?? "").slice("description: ".length);
  return value.startsWith('"') ? (JSON.parse(value) as string) : value;
};

describe("the headline", () => {
  test("is the social card's line", () => {
    expect(readSources().line).toBe(home.headline);
  });

  test("is the landing page's heading", () => {
    const heading = read(LANDING)
      .split("\n")
      .find((l) => l.startsWith("# "));
    expect(heading?.slice(2)).toBe(home.headline);
  });
});

describe("the sentence", () => {
  test("is the README's first paragraph", () => {
    expect(firstParagraph(read("README.md"))).toBe(home.sentence);
  });

  test("is the landing page's description", () => {
    expect(landingDescription()).toBe(home.sentence);
  });

  test("opens getting started", () => {
    expect(firstParagraph(read("docs/content/getting-started.md"))).toMatch(
      new RegExp(`^${home.sentence.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}( |$)`),
    );
  });
});

describe("the short form", () => {
  test("starts each npm package's description, then says the package's job", () => {
    const files = published();
    expect(files.length).toBeGreaterThan(5);
    expect(files.filter((file) => !startsWithTheShortForm(manifest(file).description))).toEqual([]);
  });

  test("starts the Marketplace's description, then says what the extension does", () => {
    expect(startsWithTheShortForm(manifest("packages/vscode/package.json").description)).toBe(true);
  });

  test("is ascribe-cli's crate description, which the command's help opens with", () => {
    // The binary takes it as `CARGO_PKG_DESCRIPTION`: crates/ascribe-cli/src/cli.rs
    // tests that `--help` opens with it, and plugin.rs starts the plugin's
    // description with it.
    const cargo = parse(read("crates/ascribe-cli/Cargo.toml")) as {
      package: { description: string };
    };
    expect(cargo.package.description).toBe(home.short);
  });

  test("starts the plugin's description in each of its manifests", () => {
    const claude = JSON.parse(read("plugins/ascribe/.claude-plugin/plugin.json")) as Manifest;
    const agentPlugins = JSON.parse(read("plugins/ascribe/plugin.json")) as Manifest;
    const marketplace = JSON.parse(read("plugins/ascribe/.claude-plugin/marketplace.json")) as {
      plugins: Manifest[];
    };
    for (const description of [
      claude.description,
      agentPlugins.description,
      marketplace.plugins[0]?.description,
    ]) {
      expect(startsWithTheShortForm(description), description).toBe(true);
    }
  });
});

describe("the keywords", () => {
  test("are in every package's and the plugin's", () => {
    const files = [
      ...published(),
      "packages/vscode/package.json",
      "plugins/ascribe/plugin.json",
      "plugins/ascribe/.claude-plugin/plugin.json",
    ];
    const missing = files.flatMap((file) =>
      home.keywords
        .filter((keyword) => !(manifest(file).keywords ?? []).includes(keyword))
        .map((keyword) => `${file}: ${keyword}`),
    );
    expect(missing).toEqual([]);
  });
});

test("every package's homepage is a page of the docs, by its permalink", () => {
  const addresses = permalinks();
  const files = [...published(), "packages/vscode/package.json"];
  expect(
    files
      .map((file) => `${file}: ${manifest(file).homepage ?? "none"}`)
      .filter((each) => !addresses.has(each.slice(each.indexOf(": ") + 2))),
  ).toEqual([]);
});

// The extension's publisher, `Ascribe` in packages/vscode/package.json, isn't
// compared: a Marketplace publisher can't be renamed, so it stays
// (project-docs/decisions/833h-bryo-studio-maker.md).
describe("the maker", () => {
  test("is every npm package's author", () => {
    const files = published();
    expect(
      files
        .map((file) => `${file}: ${String(manifest(file).author)}`)
        .filter((each) => !each.endsWith(`: ${author(home.maker)}`)),
    ).toEqual([]);
  });

  test("is the plugin's author, in each manifest, and its marketplace's owner", () => {
    const named = { name: home.maker.name, url: home.maker.url };
    expect(manifest("plugins/ascribe/plugin.json").author).toEqual(named);
    expect(manifest("plugins/ascribe/.claude-plugin/plugin.json").author).toEqual(named);
    const marketplace = JSON.parse(read("plugins/ascribe/.claude-plugin/marketplace.json")) as {
      owner: unknown;
    };
    expect(marketplace.owner).toEqual(named);
  });
});
