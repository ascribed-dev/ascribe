import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

// Review's parts from the workspace package's source, as tsconfig.json has them.
const review = fileURLToPath(new URL("../review/src/", import.meta.url));

export default defineConfig({
  resolve: {
    alias: {
      "@ascribed/review/github": `${review}github/index.ts`,
      "@ascribed/review/marks": `${review}marks/index.ts`,
      "@ascribed/review/overlay": `${review}overlay/index.ts`,
      "@ascribed/review/place": `${review}place/index.ts`,
    },
  },
  test: {
    include: ["test/**/*.test.ts"],
    testTimeout: 30_000,
  },
});
