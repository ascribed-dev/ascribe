import { defineConfig } from "astro/config";
import ascribe from "@ascribed/astro";

// The integration takes `site` and `trailingSlash` from ascribe.toml's
// [consumer]. `base` is the site's: [consumer] base-path is this path or one
// under it, and the integration fails the build if it isn't.
export default defineConfig({
  base: "/docs",
  integrations: [ascribe({ build: "site" })],
});
