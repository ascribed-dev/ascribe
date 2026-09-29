import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/unit/**/*.test.ts", "test/webview/**/*.test.ts"],
    globalSetup: ["test/webview/global-setup.ts"],
    testTimeout: 30_000,
    hookTimeout: 120_000,
  },
});
