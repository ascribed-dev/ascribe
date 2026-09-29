// @ascribed/astro: the Astro integration for Ascribe (phase 21's slice; phase 22
// completes it). Written against Astro 7.3.5.
//
//   // astro.config.mjs
//   import ascribe from "@ascribed/astro";
//   export default defineConfig({
//     base: "/docs", trailingSlash: "never",     // as in ascribe.toml's [consumer]
//     integrations: [ascribe({ build: "site" })],
//   });
//
//   // src/content.config.ts
//   import { defineCollection } from "astro:content";
//   import { ascribeCollection } from "@ascribed/astro/content";
//   import { schema } from "../.ascribe/build/site/site/_ascribe/schema.ts";
//   export const collections = { docs: defineCollection(ascribeCollection({ schema })) };
//
//   // a layout's <head>
//   import Elements from "@ascribed/astro/Elements.astro";   // <Elements />
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AstroIntegration } from "astro";
import { findBinary } from "./binary.js";
import { copyPublishedFiles, filesMiddleware } from "./files.js";
import { consumerMismatches, readProject } from "./project.js";
import rehypeAscribeAttributes from "./rehype.js";
import { runBuild } from "./run.js";
import { satteriAscribeAttributes } from "./satteri.js";

export { default as rehypeAscribeAttributes } from "./rehype.js";
export { satteriAscribeAttributes } from "./satteri.js";

/** The virtual module `content.ts` reads the resolved site output root from. */
const SITE_MODULE = "virtual:ascribe/site";

/** Options of the integration. */
export interface AscribeOptions {
  /** The build whose site output is the collection (a name in `ascribe.toml`). */
  build: string;
  /** The directory holding `ascribe.toml`, relative to the Astro root. Default: the root. */
  project?: string;
  /**
   * The `ascribe` binary, relative to the Astro root. Default: `ASCRIBE_BIN`, then the
   * nearest `target/release` or `target/debug` build above the project (Q152).
   */
  binary?: string;
}

/** Runs `ascribe build`, checks the site's routing, adds the markdown plugin, and serves published files. */
export default function ascribe(options: AscribeOptions): AstroIntegration {
  let siteRoot = "";
  return {
    name: "@ascribed/astro",
    hooks: {
      "astro:config:setup": async ({ config, command, logger, updateConfig }) => {
        const root = fileURLToPath(config.root);
        const project = readProject(path.resolve(root, options.project ?? "."));
        // An unknown build is `ascribe build`'s to report: it knows the implicit `site` build (content-model.md §17).
        siteRoot = project.siteRoot(options.build);

        // `ascribe.toml`'s routing must be Astro's, or every link Ascribe writes is wrong.
        const problems = consumerMismatches(project, {
          base: config.base,
          trailingSlash: config.trailingSlash,
          site: config.site,
        });
        if (problems.length > 0) {
          throw new Error(
            `@ascribed/astro: ascribe.toml and astro.config disagree:\n- ${problems.join("\n- ")}`,
          );
        }

        // Where a markdown plugin goes depends on the processor Astro 7.3 runs (Q151).
        const processor = config.markdown.processor;
        const pluginLists = processor.options as {
          rehypePlugins?: unknown[];
          hastPlugins?: unknown[];
        };
        if (processor.name === "satteri" && pluginLists.hastPlugins !== undefined) {
          pluginLists.hastPlugins.push(satteriAscribeAttributes());
        } else if (processor.name === "unified" && pluginLists.rehypePlugins !== undefined) {
          pluginLists.rehypePlugins.push(rehypeAscribeAttributes);
        } else {
          throw new Error(
            `@ascribed/astro: the markdown processor "${processor.name}" is not one it can add its plugin to. ` +
              "Use Astro's default (satteri) or unified(), or add `rehypeAscribeAttributes` to your own.",
          );
        }

        if (command !== "preview") {
          const binary = findBinary({ binary: options.binary, projectDir: project.dir, root });
          logger.info(`running ${path.basename(binary)} build for "${options.build}"`);
          const result = await runBuild({
            binary,
            configPath: project.configPath,
            build: options.build,
            cwd: project.dir,
          });
          if (result.diagnostics.split("\n").length > 1) logger.warn(result.diagnostics);
          if (result.summary !== "") logger.info(result.summary);
        }

        // Dev serving of `_ascribe/files/`; the build copies them in `astro:build:done`.
        updateConfig({
          vite: {
            // `Elements.astro` is Astro source, so Vite must compile it, not load it as a Node module.
            ssr: { noExternal: ["@ascribed/astro"] },
            plugins: [
              {
                name: "@ascribed/astro:site",
                resolveId: (id) => (id === SITE_MODULE ? `\0${SITE_MODULE}` : undefined),
                load: (id) =>
                  id === `\0${SITE_MODULE}`
                    ? `export const siteRoot = ${JSON.stringify(siteRoot)};`
                    : undefined,
              },
              {
                name: "@ascribed/astro:files",
                configureServer(server) {
                  server.middlewares.use(
                    filesMiddleware(
                      siteRoot,
                      config.base.endsWith("/") ? config.base : `${config.base}/`,
                    ),
                  );
                },
              },
            ],
          },
        });
      },
      "astro:build:done": async ({ dir, logger }) => {
        if (await copyPublishedFiles(siteRoot, dir)) logger.info("copied _ascribe/files/");
      },
    },
  };
}
