// The sidebar. Ascribe's content model has no page order or grouping, so the
// site lists its pages here, by their source files under docs/content/. Each
// group is a folder of docs/content/, a feature or a subject with every type
// of page about it, but for Start, the pages at the content root. In a group,
// the folder's index.md is first, then the pages by type, in the order
// docs/ascribe.toml declares its types, then by title, with a page's parts,
// the pages in the folder named for it, after it. Contracts keep an order of
// their own. test/nav.test.ts fails when a published page is missing from
// this list, or when it names a page that doesn't exist.

export interface NavGroup {
  label: string;
  /** Source files, relative to the content root, in order. */
  pages: string[];
}

export const nav: NavGroup[] = [
  { label: "Start", pages: ["index.md", "getting-started.md"] },
  { label: "Writing pages", pages: ["writing/directives.md", "writing/variants.md"] },
  {
    label: "The content model",
    pages: [
      "content-model/index.md",
      "content-model/checks.md",
      "content-model/output.md",
      "content-model/content.md",
    ],
  },
  {
    label: "Checks and reports",
    pages: [
      "checks/check.md",
      "checks/diagnostics.md",
      "checks/diagnostics/content-checks.md",
      "checks/diagnostics/directives.md",
      "checks/diagnostics/pages.md",
      "checks/diagnostics/content-model.md",
      "checks/vale.md",
      "checks/report.md",
    ],
  },
  { label: "Review", pages: ["review/index.md", "review/diff.md", "review/troubleshooting.md"] },
  { label: "The editor", pages: ["editor/index.md", "editor/settings.md"] },
  { label: "Agents", pages: ["agents/index.md", "agents/commands.md", "agents/measuring.md"] },
  {
    label: "Examples and drift",
    pages: ["examples/index.md", "examples/tested.md", "examples/sources.md"],
  },
  {
    label: "Publishing with Astro",
    pages: ["astro/index.md", "astro/reference.md", "astro/elements.md"],
  },
  { label: "Permalinks", pages: ["permalinks/index.md"] },
  { label: "Page templates", pages: ["templates/index.md"] },
  {
    label: "The command",
    pages: ["command/index.md", "command/questions.md", "command/project-files.md"],
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
      "contracts/tested-examples.md",
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
 * them as they are, and gives a folder's `index.md` the folder's id. The
 * root's `index.md` stays `index`.
 */
export function entryId(file: string): string {
  return file.replace(/\.md$/, "").replace(/\/index$/, "");
}
