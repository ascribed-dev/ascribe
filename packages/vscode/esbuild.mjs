// Bundles the extension into one file the extension host can load, and the
// integration tests into the files the test runner loads.
import process from "node:process";
import { URL, fileURLToPath } from "node:url";
import { build } from "esbuild";

const common = {
  bundle: true,
  platform: "node",
  target: "node24",
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

// The preview's webview runs in a browser: the element library (`@ascribed/elements`,
// script and stylesheet) and the preview's own script and stylesheet, each one file
// the webview loads from the extension's `dist/webview/`. Classic scripts, so the
// content security policy needs no nonce. The element library is bundled from
// the workspace package's own source and stylesheet (the same files its `dist/`
// and `style.css` export are built from), so bundling doesn't wait for, or race,
// the package's build.
const elements = fileURLToPath(new URL("../elements/", import.meta.url));
const webview = {
  // The package marks only its `dist/index.js` as having side effects; bundling
  // its source, registering the elements is the whole point of the import.
  ignoreAnnotations: true,
  alias: {
    "@ascribed/elements/style.css": `${elements}css/style.css`,
    "@ascribed/elements": `${elements}src/index.ts`,
  },
  bundle: true,
  platform: "browser",
  target: "es2022",
  format: "iife",
  sourcemap: false,
  logLevel: "info",
  minify: process.argv.includes("--minify"),
  outdir: "dist/webview",
};
await build({
  ...webview,
  entryPoints: {
    elements: "src/webview/elements.ts",
    preview: "src/webview/preview.ts",
  },
});
await build({
  ...webview,
  entryPoints: { elements: "src/webview/elements.css", preview: "src/webview/preview.css" },
});

if (process.argv.includes("--tests")) {
  await build({
    ...common,
    entryPoints: {
      run: "test/integration/run.ts",
      "suite/index": "test/integration/suite/index.ts",
      "suite/activation.it": "test/integration/suite/activation.it.ts",
      "suite/monorepo.it": "test/integration/suite/monorepo.it.ts",
      "suite/preview.it": "test/integration/suite/preview.it.ts",
      "suite/quill.it": "test/integration/suite/quill.it.ts",
      "suite/stub.it": "test/integration/suite/stub.it.ts",
    },
    outdir: "out/integration",
    outExtension: { ".js": ".cjs" },
    format: "cjs",
    external: ["vscode", "mocha"],
  });
}
