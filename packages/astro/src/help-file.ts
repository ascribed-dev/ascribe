// A build's `_ascribe/help.json` (output-layout contract §1.7): each
// permalink's section. `help.ts` serves its entries at their own addresses,
// and `permalinks.ts` links each forwarding page to them.
import { readFileSync } from "node:fs";
import path from "node:path";

/** The file's format version this reads. */
const VERSION = 1;

/** A permalink's section, as `_ascribe/help.json` has it. */
interface HelpEntry {
  title: string;
  summary?: string;
  text: string;
  address?: string;
  source: string;
}

/** `_ascribe/help.json`. */
export interface HelpFile {
  version: number;
  /** The build's name, as `ascribe.toml` declares it. */
  build: string;
  /** For a line of a versioned build, the line. */
  line?: string;
  help: Record<string, HelpEntry>;
}

/** The file in a build's site output, or `undefined` when there isn't one. */
export function readHelp(siteRoot: string): HelpFile | undefined {
  const file = path.join(siteRoot, "_ascribe", "help.json");
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    return undefined;
  }
  const help = JSON.parse(text) as HelpFile;
  if (help.version !== VERSION) {
    throw new Error(
      `@ascribed/astro: ${file} is version ${String(help.version)} of the help file, and this version of the integration reads version ${VERSION}. Update @ascribed/astro and @ascribed/cli together.`,
    );
  }
  return help;
}
