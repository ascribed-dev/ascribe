// The Zod modules the site output generates (SPEC §9.6): they type-check
// (`pnpm typecheck` compiles the generated files under the workspace's strict
// options), and they accept the frontmatter the site output writes for the
// Quill project and reject what the content model doesn't allow.
//
// `generated/` is written by `ASCRIBE_BLESS=1 cargo test -p tessera-emit --test
// zod`, which also fails when it's out of date.

import { describe, expect, it } from "vitest";
import { z } from "astro/zod";

import frontmatter from "./generated/quill.frontmatter.json" with { type: "json" };
import * as full from "./generated/full.schema.ts";
import * as quill from "./generated/quill.schema.ts";

describe("the Quill schema", () => {
  it("accepts the frontmatter of every page of the site build", () => {
    const pages = Object.entries(frontmatter);
    expect(pages.map(([path]) => path)).toEqual(["install-agent.md", "keys.md", "quickstart.md"]);
    for (const [path, data] of pages) {
      const result = quill.schema.safeParse(data);
      expect(result.success, `${path}: ${JSON.stringify(result.error?.issues)}`).toBe(true);
    }
  });

  it("gives a layout page-level availability as a list of targets", () => {
    const page = quill.pageSchema.parse(frontmatter["install-agent.md"]);
    expect(page.available).toEqual([
      { target: "cloud", dimension: "deployment", states: ["ga"], text: "Quill Cloud (GA)" },
      {
        target: "self-managed",
        dimension: "deployment",
        states: ["preview"],
        versions: ["3.3"],
        text: "self-managed (preview, 3.3+)",
      },
    ]);
  });

  it("rejects an unknown key, a missing title, and a source-style available", () => {
    expect(quill.pageSchema.safeParse({ title: "T", surprise: 1 }).success).toBe(false);
    expect(quill.pageSchema.safeParse({ description: "no title" }).success).toBe(false);
    expect(quill.pageSchema.safeParse({ title: "T", available: "cloud" }).success).toBe(false);
  });

  it("takes variant as a mapping to a value or a list", () => {
    const ok = quill.pageSchema.safeParse({ title: "T", variant: { pm: ["npm", "yarn"], deployment: "cloud" } });
    expect(ok.success).toBe(true);
    expect(quill.pageSchema.safeParse({ title: "T", variant: "cloud" }).success).toBe(false);
  });

  it("lists the content types", () => {
    expect(Object.keys(quill.schemas)).toEqual(["page"]);
    expect(quill.contentTypes.page).toEqual({ default: true, files: [] });
    const title: z.infer<typeof quill.pageSchema>["title"] = "typed";
    expect(title).toBe("typed");
  });
});

describe("the full content model's schema", () => {
  const guide = {
    title: "Guide",
    description: "d",
    order: 2,
    featured: true,
    updated: "2026-02-03",
    level: "beginner",
    tags: ["a", "b"],
    category: "Concepts",
    author: { name: "Ann" },
    related: [{ title: "T", url: "https://example.com" }],
    variant: { pm: "npm" },
  };

  it("accepts every field type and applies defaults", () => {
    const parsed = full.guideSchema.parse(guide);
    expect(parsed.draft).toBe(false);
    expect(parsed.status).toBe("published");
    expect(parsed.updated).toBeInstanceOf(Date);
    expect(full.guideSchema.safeParse({ ...guide, available: [] }).success).toBe(true);
  });

  it("rejects values of the wrong type or outside an enumeration", () => {
    for (const bad of [
      { ...guide, order: "2" },
      { ...guide, level: "expert" },
      { ...guide, category: "Other" },
      { ...guide, tags: [1] },
      { ...guide, author: { name: "Ann", phone: "1" } },
      { ...guide, related: [{ title: "T" }] },
      { ...guide, updated: "not a date" },
      { ...guide, unknown: true },
    ]) {
      expect(full.guideSchema.safeParse(bad).success, JSON.stringify(bad)).toBe(false);
    }
  });

  it("has a schema for each type, and one for any of them", () => {
    expect(Object.keys(full.schemas)).toEqual(["guide", "reference"]);
    expect(full.contentTypes.reference.files).toEqual(["reference/**", "api/**/*.md"]);
    const reference = { title: "R", "api-version": "3", formatted: { title: "<code>R</code>" } };
    expect(full.schema.safeParse(reference).success).toBe(true);
    expect(full.schema.safeParse({ nope: 1 }).success).toBe(false);
  });

  it("has the formatted form of a field that sets inline", () => {
    const page = { title: "ascribe.toml reference", formatted: { title: "<code>ascribe.toml</code> reference" } };
    expect(full.referenceSchema.parse(page).formatted.title).toBe("<code>ascribe.toml</code> reference");
    // The site output always writes it beside the title.
    expect(full.referenceSchema.safeParse({ title: "R" }).success).toBe(false);
    expect(full.referenceSchema.safeParse({ ...page, formatted: { title: "R", nope: "" } }).success).toBe(false);
  });
});
