// A product's help, from the docs' build: the title, summary, and address
// of the permalink a product names, read from `_ascribe/help.json`.
//
//   node help.ts <help.json> <name>
//
// It reads the file and nothing else. A name the file doesn't have exits 1,
// so a product's CI fails on a link the docs no longer have.
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
  help: Record<string, Entry>;
}

const [file, name] = process.argv.slice(2);
if (file === undefined || name === undefined) {
  console.error("Usage: node help.ts <help.json> <name>");
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
if (entry.address !== undefined) {
  console.log(`Read more: ${entry.address}`);
}
// :snippet-end:
