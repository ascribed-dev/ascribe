// The workspace's configuration, plus the directories Astro and Ascribe generate.
import { defineConfig, globalIgnores } from "eslint/config";
import workspace from "../../eslint.config.js";

export default defineConfig([globalIgnores([".astro/", ".ascribe/", ".e2e-tmp/"]), workspace]);
