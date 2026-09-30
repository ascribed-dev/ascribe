import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/parity/**/*.test.ts"],
    testTimeout: 120_000,
    hookTimeout: 300_000,
    // The tests build copies of the example site, which share Ascribe's locked output directory.
    fileParallelism: false,
  },
});
