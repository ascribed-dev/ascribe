import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

// `site`, `base`, and `trailingSlash` repeat ascribe.toml's [consumer]; the
// integration fails the build if they disagree.
export default defineConfig({
  site: "https://docs.example.com",
  base: "/docs",
  trailingSlash: "never",
  integrations: [ascribe({ build: "site" })],
});
