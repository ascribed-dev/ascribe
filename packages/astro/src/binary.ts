import { resolveBinary } from "@ascribed/cli/binary";
import { existsSync } from "node:fs";
import path from "node:path";

/** The path of the binary to run. Throws an error saying how to point at one. */
export function findBinary(options: { binary?: string | undefined; root: string }): string {
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
  return resolveBinary();
}
