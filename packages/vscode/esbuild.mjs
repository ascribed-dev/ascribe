// Bundles the extension into one file the extension host can load, and the
// integration tests into the files the test runner loads.
import process from "node:process";
import { build } from "esbuild";

const common = {
  bundle: true,
  platform: "node",
  target: "node22",
  sourcemap: true,
  logLevel: "info",
};

await build({
  ...common,
  entryPoints: ["src/extension.ts"],
  outfile: "dist/extension.cjs",
  format: "cjs",
  external: ["vscode"],
  minify: process.argv.includes("--minify"),
});

if (process.argv.includes("--tests")) {
  await build({
    ...common,
    entryPoints: {
      run: "test/integration/run.ts",
      "suite/index": "test/integration/suite/index.ts",
      "suite/quill.it": "test/integration/suite/quill.it.ts",
      "suite/stub.it": "test/integration/suite/stub.it.ts",
    },
    outdir: "out/integration",
    outExtension: { ".js": ".cjs" },
    format: "cjs",
    external: ["vscode", "mocha"],
  });
}
