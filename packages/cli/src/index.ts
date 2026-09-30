#!/usr/bin/env node
import { spawn } from "node:child_process";
import { realpathSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolveBinary } from "./binary.js";

export { nativeBinaryPath, nativePackageName, resolveBinary } from "./binary.js";
export type { BinaryPlatform } from "./binary.js";

if (process.argv[1] && realpathSync(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const child = spawn(resolveBinary(), process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: false,
  });
  child.on("error", (error) => {
    console.error(`@ascribed/cli could not start the native binary: ${error.message}`);
    process.exitCode = 1;
  });
  child.on("exit", (code) => {
    process.exitCode = code ?? 1;
  });
}
