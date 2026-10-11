import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

// Two Ascribe projects in one site: the site's own pages (home/, at the root)
// and the docs (../docs, under /docs/). `site` and `trailingSlash` repeat each
// project's [consumer], and `base`, Astro's default `/`, is the root of both
// base paths; the integration fails the build if they disagree.
export default defineConfig({
  site: "https://ascribed-dev.com",
  trailingSlash: "always",
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
      ],
      // Review in `astro dev` covers the docs, the pages that get reviewed.
      review: "docs",
    }),
  ],
});
