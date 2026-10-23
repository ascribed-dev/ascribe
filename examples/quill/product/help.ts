// A product's help, from the docs' build: the title, summary, and address
// of the permalink a product names, read from `_ascribe/help.json`.
//
//   node help.ts <help.json> <name> [--choose <dimension>=<value>]
//
// With `--choose`, for a reader the product knows uses that value, such as
// `pm=pnpm`, it also prints the section with that arm chosen, when the
// section has a switcher on the dimension, and opens the address with it.
//
// It reads the file and nothing else. A name the file doesn't have exits 1,
// so a product's CI fails on a link the docs no longer have.
// :snippet-start: help
import { readFileSync } from "node:fs";

interface Entry {
  title: string;
  summary?: string;
  text: string;
  choices?: Record<string, Record<string, string>>;
  address?: string;
  source: string;
}

interface Help {
  version: number;
  help: Record<string, Entry>;
}

const [file, name, ...options] = process.argv.slice(2);
const choose = options[0] === "--choose" ? options[1]?.split("=") : undefined;
if (file === undefined || name === undefined || (options.length > 0 && choose?.length !== 2)) {
  console.error("Usage: node help.ts <help.json> <name> [--choose <dimension>=<value>]");
  process.exit(2);
}
const help = JSON.parse(readFileSync(file, "utf8")) as Help;
if (help.version !== 1) {
  throw new Error(`help.json version ${help.version} isn't one this product reads`);
}
const entry = Object.hasOwn(help.help, name) ? help.help[name] : undefined;
if (entry === undefined) {
  console.log(`No help for ${name}`);
  process.exit(1);
}
console.log(entry.title);
if (entry.summary !== undefined) {
  console.log(entry.summary);
}
let address = entry.address;
if (choose !== undefined) {
  const [dimension, value] = choose;
  const chosen = entry.choices?.[dimension]?.[value];
  if (chosen !== undefined) {
    console.log(`\n${chosen}\n`);
    address = address === undefined ? undefined : `${address}?${dimension}=${value}`;
  }
}
if (address !== undefined) {
  console.log(`Read more: ${address}`);
}
// :snippet-end:
