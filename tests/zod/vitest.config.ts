import { defineConfig } from "vitest/config";

// The generated modules import `z` from `astro/zod`, which re-exports
// `zod/v4` in the Astro version the site output targets (7.3.5). Depending
// on `zod` itself keeps this package small; the Astro example runs the real
// thing.
export default defineConfig({
  resolve: { alias: { "astro/zod": "zod/v4" } },
});
