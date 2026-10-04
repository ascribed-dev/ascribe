import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const review = fileURLToPath(new URL("../review/src/", import.meta.url));

export default defineConfig({
  resolve: {
    // Review's parts from the workspace package's source, as the bundle has them.
    alias: {
      "@ascribed/review/github": `${review}github/index.ts`,
      "@ascribed/review/place": `${review}place/index.ts`,
    },
  },
  test: {
    include: ["test/unit/**/*.test.ts", "test/webview/**/*.test.ts"],
    globalSetup: ["test/webview/global-setup.ts"],
    testTimeout: 30_000,
    hookTimeout: 120_000,
  },
});
