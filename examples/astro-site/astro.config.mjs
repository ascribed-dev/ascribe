import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

// The integration gives Astro its `site`, `base`, and `trailingSlash` from
// ascribe.toml's [consumer]. Any of them set here must agree with it, or the
// integration fails the build.
export default defineConfig({
  integrations: [ascribe({ build: "site" })],
});
