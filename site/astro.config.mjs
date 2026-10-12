import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";
import { unindexed } from "./src/site.ts";

// Three Ascribe projects in one site: the site's own pages (home/, at the
// root), the docs (../docs, under /docs/), and the landing page's
// demonstration (demo/, whose one page the landing page draws; it has no
// route). `site` and `trailingSlash` come from the projects' [consumer], so
// the site's address is written only in their ascribe.toml, and `base`,
// Astro's default `/`, is the root of every base path.
export default defineConfig({
  // Code follows the system's theme, as the rest of the site does. The
  // `-default` themes are the ones whose colors pass 4.5:1 on the code's background.
  markdown: {
    shikiConfig: {
      themes: { light: "github-light-default", dark: "github-dark-default" },
    },
  },
  integrations: [
    ascribe({
      projects: [
        { name: "docs", project: "../docs", build: "site" },
        { name: "home", project: "home", build: "site" },
        { name: "demo", project: "demo", build: "site" },
      ],
      // Review in `astro dev` covers the docs, the pages that get reviewed.
      review: "docs",
      // One sitemap for the site, and robots.txt: the docs' pages and home/'s
      // that are listed, and the site's own, the changelog among them. The
      // demo's page has no route, so it isn't in it.
      sitemap: { exclude: unindexed },
    }),
  ],
});
