import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    testTimeout: 60_000,
    hookTimeout: 180_000,
    // The tests share one site directory, and Ascribe's output directory is locked.
    fileParallelism: false,
  },
});
