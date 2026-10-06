import { mkdirSync, mkdtempSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { copySchema, schemaPath } from "../src/schema.js";

function dirs(): { siteRoot: string; codegen: string } {
  const root = mkdtempSync(path.join(tmpdir(), "ascribe-schema-"));
  const siteRoot = path.join(root, "site");
  const codegen = path.join(root, ".astro", "integrations", "_ascribed_astro");
  mkdirSync(path.join(siteRoot, "_ascribe"), { recursive: true });
  mkdirSync(codegen, { recursive: true });
  return { siteRoot, codegen };
}

describe("copySchema", () => {
  it("copies the site output's schema, and rewrites it only when it changes", () => {
    const { siteRoot, codegen } = dirs();
    const copy = path.join(codegen, "schema.ts");
    writeFileSync(schemaPath(siteRoot), 'import { z } from "astro/zod";\n');
    expect(copySchema(siteRoot, codegen)).toBe(true);
    expect(readFileSync(copy, "utf8")).toBe('import { z } from "astro/zod";\n');

    const written = statSync(copy).mtimeMs;
    expect(copySchema(siteRoot, codegen)).toBe(false);
    expect(statSync(copy).mtimeMs).toBe(written);

    writeFileSync(schemaPath(siteRoot), "export const schema = 1;\n");
    expect(copySchema(siteRoot, codegen)).toBe(true);
    expect(readFileSync(copy, "utf8")).toBe("export const schema = 1;\n");
  });

  it("does nothing when the site output has no schema", () => {
    const { siteRoot, codegen } = dirs();
    expect(copySchema(siteRoot, codegen)).toBe(false);
  });
});
