// The sidebar. Ascribe's content model has no page order or grouping, so the
// site lists its pages here, by their source files under docs/content/.
// test/nav.test.ts fails when a published page is missing from this list, or
// when it names a page that doesn't exist.

export interface NavGroup {
  label: string;
  /** Source files, relative to the content root, in order. */
  pages: string[];
}

export const nav: NavGroup[] = [
  { label: "Start", pages: ["index.md", "getting-started.md"] },
  {
    label: "Guides",
    pages: ["guides/astro.md", "guides/editor.md", "guides/review.md", "guides/drift.md"],
  },
  {
    label: "Reference",
    pages: [
      "reference/cli.md",
      "reference/content-model.md",
      "reference/directives.md",
      "reference/diagnostics.md",
    ],
  },
  {
    label: "Contracts",
    pages: [
      "contracts/content-model.md",
      "contracts/site-render.md",
      "contracts/output-layout.md",
      "contracts/assets.md",
    ],
  },
];

/**
 * A source file's entry id in the collection: Astro's glob loader drops the
 * extension and slugs each segment, which for these lowercase names leaves
 * them as they are.
 */
export function entryId(file: string): string {
  return file.replace(/\.md$/, "");
}
