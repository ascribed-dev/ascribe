// The sidebar. Ascribe's content model has no page order or grouping, so the
// site lists its pages here, by their source files under docs/content/. Each
// group is a type in docs/ascribe.toml, but for Start, the home page, and
// Contracts, reference pages kept apart since they're for implementers.
// test/nav.test.ts fails when a published page is missing from this list, or
// when it names a page that doesn't exist.

export interface NavGroup {
  label: string;
  /** Source files, relative to the content root, in order. */
  pages: string[];
}

export const nav: NavGroup[] = [
  { label: "Start", pages: ["index.md"] },
  { label: "Tutorials", pages: ["getting-started.md", "guides/variants.md"] },
  {
    label: "How-to guides",
    pages: [
      "guides/astro.md",
      "guides/review.md",
      "guides/drift.md",
      "guides/agents.md",
      "guides/vale.md",
      "guides/report.md",
      "guides/permalinks.md",
      "guides/templates.md",
    ],
  },
  { label: "Concepts", pages: ["guides/measuring.md"] },
  { label: "Troubleshooting", pages: ["guides/review/troubleshooting.md"] },
  {
    label: "Reference",
    pages: [
      "reference/cli.md",
      "reference/cli/check.md",
      "reference/cli/diff.md",
      "reference/cli/questions.md",
      "reference/cli/agents.md",
      "reference/cli/sources.md",
      "reference/content-model.md",
      "reference/content-model/content.md",
      "reference/content-model/output.md",
      "reference/content-model/checks.md",
      "reference/directives.md",
      "reference/diagnostics.md",
      "reference/diagnostics/directives.md",
      "reference/diagnostics/pages.md",
      "reference/diagnostics/content-model.md",
      "reference/diagnostics/content-checks.md",
      "reference/elements.md",
      "reference/astro.md",
      "reference/project-files.md",
      "guides/editor.md",
      "guides/editor/settings.md",
    ],
  },
  {
    label: "Contracts",
    pages: [
      "contracts/content-model.md",
      "contracts/content-model/content.md",
      "contracts/content-model/output.md",
      "contracts/content-model/checks.md",
      "contracts/content-model/validation.md",
      "contracts/site-render.md",
      "contracts/output-layout.md",
      "contracts/assets.md",
      "contracts/json-reports.md",
      "contracts/json-reports/diff.md",
      "contracts/json-reports/report.md",
      "contracts/json-reports/sources.md",
      "contracts/json-reports/templates.md",
      "contracts/json-reports/questions.md",
      "contracts/permalinks-list.md",
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
