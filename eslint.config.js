// Shared ESLint configuration for every package in the pnpm workspace.
// Packages run `eslint .`, which finds this file by walking up.
import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import prettier from "eslint-config-prettier";
import tseslint from "typescript-eslint";

export default defineConfig([
  globalIgnores(["**/dist/", "**/out/", "**/coverage/", "**/node_modules/", "target/"]),
  js.configs.recommended,
  tseslint.configs.strict,
  tseslint.configs.stylistic,
  // CommonJS scripts, such as the release smoke test's suite, which VS Code
  // loads with require().
  {
    files: ["**/*.cjs"],
    languageOptions: {
      sourceType: "commonjs",
      globals: {
        require: "readonly",
        exports: "writable",
        module: "writable",
        process: "readonly",
        setTimeout: "readonly",
      },
    },
    rules: { "@typescript-eslint/no-require-imports": "off" },
  },
  // Formatting is Prettier's job; turn off rules that conflict with it.
  prettier,
]);
