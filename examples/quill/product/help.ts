// A product's help, from the docs' build: the title, summary, and address
// of the permalink a product names, read from `_ascribe/help.json`.
//
//   node help.ts <help.json> <name> [<folder>]
//
// With a folder, the address its line's entries are served under, such as
// https://docs.quill.dev/3.4/go/, it fetches the entry from the docs first,
// and uses the bundled file offline, on any error, or when the docs no longer
// publish its line. A name neither has exits 1, so a product's CI fails on a
// link the docs no longer have.
// :snippet-start: help
import { readFileSync } from "node:fs";

interface Entry {
  title: string;
  summary?: string;
  text: string;
  address?: string;
  source: string;
}

interface Help {
  version: number;
  line?: string;
  help: Record<string, Entry>;
}

const [file, name, folder] = process.argv.slice(2);
if (file === undefined || name === undefined) {
  console.error("Usage: node help.ts <help.json> <name> [<folder>]");
  process.exit(2);
}
const bundled = JSON.parse(readFileSync(file, "utf8")) as Help;
if (bundled.version !== 1) {
  throw new Error(`help.json version ${bundled.version} isn't one this product reads`);
}

/** The entry the docs publish now for this line, or `undefined` for any reason not to use it. */
async function fetched(folder: string, name: string): Promise<Entry | undefined> {
  try {
    const response = await fetch(new URL(`${name}.json`, folder), {
      signal: AbortSignal.timeout(2000),
    });
    if (!response.ok) return undefined;
    const help = (await response.json()) as Help;
    // Another line's text describes another release: only this line's will do.
    if (help.version !== 1 || help.line !== bundled.line) return undefined;
    return Object.hasOwn(help.help, name) ? help.help[name] : undefined;
  } catch {
    return undefined;
  }
}

const entry =
  (folder === undefined ? undefined : await fetched(folder, name)) ??
  (Object.hasOwn(bundled.help, name) ? bundled.help[name] : undefined);
if (entry === undefined) {
  console.log(`No help for ${name}`);
  process.exit(1);
}
console.log(entry.title);
if (entry.summary !== undefined) {
  console.log(entry.summary);
}
if (entry.address !== undefined) {
  console.log(`Read more: ${entry.address}`);
}
// :snippet-end:
