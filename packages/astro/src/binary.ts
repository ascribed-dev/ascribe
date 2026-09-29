// Finding the `ascribe` binary. npm distribution is phase 22's; for now the
// integration uses a binary someone built (Q152), looked for in this order:
//
//   1. the `binary` option,
//   2. the `ASCRIBE_BIN` environment variable,
//   3. `target/release/ascribe` or `target/debug/ascribe` in the project's
//      directory or the nearest parent that has one (the Ascribe workspace's
//      `cargo build -p tessera-cli`), the newer of the two.
import { existsSync, statSync } from "node:fs";
import path from "node:path";

const EXE = process.platform === "win32" ? "ascribe.exe" : "ascribe";

/** The path of the binary to run. Throws an error saying how to point at one. */
export function findBinary(options: {
  binary?: string | undefined;
  projectDir: string;
  root: string;
}): string {
  if (options.binary !== undefined) {
    const binary = path.resolve(options.root, options.binary);
    if (!existsSync(binary))
      throw new Error(
        `@ascribed/astro: the \`binary\` option names ${binary}, which doesn't exist.`,
      );
    return binary;
  }
  const fromEnv = process.env["ASCRIBE_BIN"];
  if (fromEnv !== undefined && fromEnv !== "") {
    const binary = path.resolve(fromEnv);
    if (!existsSync(binary))
      throw new Error(`@ascribed/astro: ASCRIBE_BIN names ${binary}, which doesn't exist.`);
    return binary;
  }
  for (let dir = path.resolve(options.projectDir); ; dir = path.dirname(dir)) {
    const found = ["release", "debug"]
      .map((profile) => path.join(dir, "target", profile, EXE))
      .filter((candidate) => existsSync(candidate))
      .sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs)[0];
    if (found !== undefined) return found;
    if (path.dirname(dir) === dir) break;
  }
  throw new Error(
    "@ascribed/astro can't find the `ascribe` binary. Build it with `cargo build -p tessera-cli`, then " +
      "set the `binary` option of the integration or the ASCRIBE_BIN environment variable to it.",
  );
}
