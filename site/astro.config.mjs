import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";
import { codeTitles } from "./src/code-titles.ts";

// `site`, `base`, and `trailingSlash` repeat ../docs/ascribe.toml's [consumer];
// the integration fails the build if they disagree.
export default defineConfig({
  site: "https://ascribed-dev.com",
  trailingSlash: "always",
  // Code follows the system's theme, as the rest of the site does, and shows
  // its title.
  markdown: {
    shikiConfig: {
      themes: { light: "github-light", dark: "github-dark" },
      transformers: [codeTitles],
    },
  },
  integrations: [ascribe({ project: "../docs", build: "site" })],
});
