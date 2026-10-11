// The sidebar's list: its groups, in order, each with its pages' source files,
// relative to the content root. `npx @ascribed/astro init` wrote it from the
// pages there were then; it's yours to reorder, relabel, and add to. Ascribe's
// content model has no page order or grouping yet, so the site keeps the list,
// and the layout's `sidebar()` keeps it complete: the build fails on a page the
// list leaves out, and on a file it names that isn't a page.
import type { NavGroup } from "@ascribed/astro/content";

export const nav: NavGroup[] = [
  { label: "Start", pages: ["index.md"] },
  { label: "Guides", pages: ["Guides/My Setup.md"] },
  { label: "Reference", pages: ["Reference/Options.md"] },
];
