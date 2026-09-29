import { defineConfig } from "astro/config";
import tessera from "@tessera/astro";

// `site`, `base`, and `trailingSlash` repeat tessera.toml's [consumer]; the
// integration fails the build if they disagree.
export default defineConfig({
  site: "https://docs.example.com",
  base: "/docs",
  trailingSlash: "never",
  integrations: [tessera({ build: "site" })],
});
