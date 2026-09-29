// The workspace's configuration, plus the directories Astro and Tessera generate.
import { defineConfig, globalIgnores } from "eslint/config";
import workspace from "../../eslint.config.js";

export default defineConfig([globalIgnores([".astro/", ".tessera/", ".e2e-tmp/"]), workspace]);
