// The starter: what `npx @ascribed/astro init` writes, and that
// examples/astro-site's collection, route, and layout are what it writes for
// that project. Run with ASCRIBE_BLESS=1 to rewrite the example's from the
// starter in starter/.
import { existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { chooseBuild, init, schemaImport, STARTER_FILES, starterFiles } from "../src/init.js";
import { readProject } from "../src/project.js";

const BLESS = "ASCRIBE_BLESS=1 pnpm --filter @ascribed/astro exec vitest run test/init.test.ts";
const example = fileURLToPath(new URL("../../../examples/astro-site/", import.meta.url));
const projectFiles = fileURLToPath(
  new URL("../../../docs/content/command/project-files.md", import.meta.url),
);

function project(toml: string): string {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-init-"));
  writeFileSync(path.join(dir, "ascribe.toml"), toml);
  return dir;
}

describe("the example", () => {
  it("has the files init writes for it", () => {
    const files = starterFiles({ schema: schemaImport(example, readProject(example), "site") });
    for (const [file, text] of files) {
      const at = path.join(example, file);
      if (process.env["ASCRIBE_BLESS"]) writeFileSync(at, text);
      expect(readFileSync(at, "utf8"), `${file} is out of date: run ${BLESS}`).toBe(text);
    }
  });
});

describe("Files Ascribe writes", () => {
  it("lists the files init writes", () => {
    const rows = readFileSync(projectFiles, "utf8")
      .split(/\r?\n/)
      .filter((line) => line.startsWith("| ") && line.includes("@ascribed/astro init"));
    expect(rows, "one row of the page's table names `npx @ascribed/astro init`").toHaveLength(1);
    const file = rows[0]?.split(" | ")[0] ?? "";
    const listed = [...file.matchAll(/`([^`]+)`/g)].map((match) => match[1]);
    expect(listed).toEqual([...STARTER_FILES]);
  });
});

describe("chooseBuild", () => {
  it("takes the one build, `site` when there's no [builds]", () => {
    expect(chooseBuild(readProject(project('spec = "0.1"\n')), undefined)).toBe("site");
    const one = readProject(project('spec = "0.1"\n[builds.web]\n'));
    expect(chooseBuild(one, undefined)).toBe("web");
  });

  it("needs --build when there are several, and one of them", () => {
    const several = readProject(project('spec = "0.1"\n[builds.web]\n[builds.pdf]\n'));
    expect(chooseBuild(several, "pdf")).toBe("pdf");
    expect(() => chooseBuild(several, undefined)).toThrow(/several builds, "web", "pdf".*--build/);
    expect(() => chooseBuild(several, "site")).toThrow(
      /has no build "site". Its builds are "web", "pdf"/,
    );
  });
});

describe("schemaImport", () => {
  it("imports the generated schema when the project is inside the Astro root", () => {
    const root = project('spec = "0.1"\n[project]\noutput-dir = "out"\n');
    expect(schemaImport(root, readProject(root), "web")).toBe("../out/web/site/_ascribe/schema.ts");
  });

  it("imports the integration's copy when the project is outside it", () => {
    const docs = project('spec = "0.1"\n');
    const root = mkdtempSync(path.join(tmpdir(), "ascribe-astro-site-"));
    expect(schemaImport(root, readProject(docs), "site")).toBe(
      "../.astro/integrations/_ascribed_astro/schema.ts",
    );
  });
});

describe("init", () => {
  it("writes the three files, and no list of the pages", async () => {
    const root = project('spec = "0.1"\n');
    const result = await init({ root });
    expect(result).toEqual({ build: "site", written: [...STARTER_FILES] });
    expect(existsSync(path.join(root, "src/nav.ts"))).toBe(false);
    expect(readFileSync(path.join(root, "src/content.config.ts"), "utf8")).toContain(
      'import { schema } from "../.ascribe/build/site/site/_ascribe/schema.ts";',
    );
    const layout = readFileSync(path.join(root, "src/layouts/Docs.astro"), "utf8");
    expect(layout).toContain("sidebar(entries, { entry })");
    expect(layout).toContain("neighbors(entry, entries)");
  });

  it("writes nothing when one of the files is there", async () => {
    const root = project('spec = "0.1"\n');
    mkdirSync(path.join(root, "src/layouts"), { recursive: true });
    writeFileSync(path.join(root, "src/layouts/Docs.astro"), "<!-- mine -->\n");
    await expect(init({ root })).rejects.toThrow(/src\/layouts\/Docs\.astro is\. It wrote nothing/);
    expect(existsSync(path.join(root, "src/content.config.ts"))).toBe(false);
    expect(readFileSync(path.join(root, "src/layouts/Docs.astro"), "utf8")).toBe("<!-- mine -->\n");
  });
});
