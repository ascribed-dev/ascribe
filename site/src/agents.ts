// What the site adds for agents to the docs' llms.txt: the changelog, which
// isn't a page of the docs project, so the docs' build can't list it. It's
// listed under "Optional", the llms.txt format's section for what an agent can
// skip when it's short of room, after the build copies the docs' llms.txt.
import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AstroIntegration } from "astro";
import { docsPath } from "./site.ts";

/** The changelog's line, with the site's address. */
export function changelogLine(site: URL): string {
  const address = new URL("changelog.md", site).href;
  return (
    `- [Changelog](${address}): every release, and what's in \`main\` not yet ` +
    `released: what changed, and which version a fix is in`
  );
}

/** `llms.txt` with an "Optional" section listing the changelog, after the rest. */
export function withChangelog(llms: string, site: URL): string {
  if (/^## Optional$/m.test(llms)) {
    throw new Error("the docs' llms.txt already has an Optional section");
  }
  return `${llms.trimEnd()}\n\n## Optional\n\n${changelogLine(site)}\n`;
}

/** Lists the changelog in the built docs' `llms.txt`. */
export function changelogInLlms(): AstroIntegration {
  let site: URL | undefined;
  let base = "/";
  return {
    name: "changelog-in-llms",
    hooks: {
      "astro:config:done": ({ config }) => {
        site = config.site === undefined ? undefined : new URL(config.site);
        base = config.base;
      },
      "astro:build:done": async ({ dir, logger }) => {
        if (site === undefined) throw new Error("the site has no address for llms.txt's links");
        const file = path.join(fileURLToPath(dir), docsPath, "llms.txt");
        const root = new URL(base.endsWith("/") ? base : `${base}/`, site);
        await writeFile(file, withChangelog(await readFile(file, "utf8"), root));
        logger.info(`listed the changelog in /${docsPath}llms.txt`);
      },
    },
  };
}
