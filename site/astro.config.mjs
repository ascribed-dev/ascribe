import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

// `site`, `base`, and `trailingSlash` repeat ../docs/ascribe.toml's [consumer];
// the integration fails the build if they disagree.
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
  integrations: [ascribe({ project: "../docs", build: "site" })],
});
