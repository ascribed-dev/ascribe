// @ascribed/astro: the Astro integration for Ascribe. Written against Astro
// 7.3.5.
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
import { watchDev } from "./dev.js";
import { copyPublishedFiles, filesMiddleware } from "./files.js";
import { consumerMismatches, readProject } from "./project.js";
import rehypeAscribeAttributes from "./rehype.js";
import { removeDevFile, siteUrl, writeDevFile } from "./review/devfile.js";
import { runDiff } from "./review/diff.js";
import { APP_ID } from "./review/protocol.js";
import { readRoutes } from "./review/routes.js";
import { channelProblem, ReviewServer, type ToolbarChannel } from "./review/server.js";
import { anchorsFor, reviewFor, runBuild } from "./run.js";
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
   * The `ascribe` binary, relative to the Astro root. Default: `ASCRIBE_BIN`, then
   * the platform binary installed with `@ascribed/cli`.
   */
  binary?: string;
  /**
   * Mark each block of the site output with the source lines it came from
   * (source anchors), for review: `"dev"` in `astro dev` only, `true` always.
   * Default: `false`, though `review` has them in `astro dev`.
   */
  anchors?: boolean | "dev";
  /**
   * Review in the site preview: an **Ascribe review** app in `astro dev`'s
   * toolbar, which marks the page's changes and shows the pull request's
   * comments, with source anchors in `astro dev`. Default `true`; `false`
   * removes the app. `astro build` never has it.
   */
  review?: boolean;
}

/** The toolbar app's icon: a speech bubble over a page. */
const ICON =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3h8l4 4v6"/><path d="M14 3v4h4"/><path d="M6 3v15"/><path d="M11 14h10v6h-6l-3 2v-2h-1z"/></svg>';

/** Runs `ascribe build`, checks the site's routing, adds the markdown plugin, and serves published files. */
export default function ascribe(options: AscribeOptions): AstroIntegration {
  let siteRoot = "";
  let devProject: ReturnType<typeof readProject> | undefined;
  let devConfig:
    | {
        base: string;
        trailingSlash: string;
        site: string | undefined;
      }
    | undefined;
  let rebuild: (() => Promise<void>) | undefined;
  let binary: string | undefined;
  let reviewing = false;
  let devHttps = false;
  let review: ReviewServer | undefined;
  return {
    name: "@ascribed/astro",
    hooks: {
      "astro:config:setup": async ({ config, command, logger, updateConfig, addDevToolbarApp }) => {
        const root = fileURLToPath(config.root);
        const project = readProject(path.resolve(root, options.project ?? "."));
        // An unknown build is `ascribe build`'s to report: it knows the implicit `site` build.
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

        // Where a markdown plugin goes depends on the processor Astro 7.3 runs.
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

        devConfig = {
          base: config.base,
          trailingSlash: config.trailingSlash,
          site: config.site,
        };

        reviewing = reviewFor(options.review, command);
        if (reviewing) {
          addDevToolbarApp({
            id: APP_ID,
            name: "Ascribe review",
            icon: ICON,
            entrypoint: new URL("./toolbar/app.js", import.meta.url),
          });
        }

        if (command !== "preview") {
          const found = findBinary({ binary: options.binary, root });
          binary = found;
          rebuild = async () => {
            logger.info(`running ${path.basename(found)} build for "${options.build}"`);
            const result = await runBuild({
              binary: found,
              configPath: project.configPath,
              build: options.build,
              cwd: project.dir,
              anchors: anchorsFor(options.anchors, command, reviewing),
              // While review is on, the JSON output says which page is at each route.
              outputs: review?.active ? ["site", "json"] : ["site"],
            });
            if (result.diagnostics !== "") logger.warn(result.diagnostics);
            if (result.summary !== "") logger.info(result.summary);
          };
          await rebuild();
          if (command === "dev") devProject = project;
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
      "astro:server:setup": ({ server, refreshContent, logger, toolbar }) => {
        if (devProject && rebuild && devConfig) {
          if (!refreshContent)
            throw new Error("@ascribed/astro: this Astro version does not support refreshContent.");
          const project = devProject;
          devHttps = Boolean(server.config.server.https);
          const builds = watchDev({
            server,
            refreshContent: () => refreshContent({}),
            logger,
            project,
            astro: devConfig,
            build: options.build,
            rebuild,
            onBuilt: () => review?.rebuilt(),
          });
          if (reviewing && binary !== undefined) {
            const found = binary;
            const run = { binary: found, configPath: project.configPath, cwd: project.dir };
            const jsonRoot = path.join(path.dirname(siteRoot), "json");
            review = new ReviewServer({
              channel: toolbar as ToolbarChannel,
              logger,
              build: options.build,
              contentRoot: project.contentRoot,
              channelProblem: channelProblem(server.config),
              diff: (base) => runDiff({ ...run, build: options.build, base }),
              connect: async () => (await import("./review/github.js")).connect(project.dir),
              writeRoutes: () =>
                builds.inTurn(async () => {
                  await runBuild({ ...run, build: options.build, outputs: ["json"] });
                }),
              readRoutes: () => readRoutes(jsonRoot),
            });
          }
        }
      },
      "astro:server:start": ({ address, logger }) => {
        if (!devProject || !devConfig) return;
        const dir = devProject.dir;
        try {
          writeDevFile(dir, {
            url: siteUrl(address, { https: devHttps, base: devConfig.base }),
            build: options.build,
            pid: process.pid,
          });
          // Vite exits on a signal before `astro:server:done` runs.
          process.once("exit", () => removeDevFile(dir, process.pid));
        } catch (error) {
          logger.warn(
            `couldn't write .ascribe/dev.json, which the editor's Open Site Preview reads: ${error instanceof Error ? error.message : String(error)}`,
          );
        }
      },
      "astro:server:done": () => {
        review?.dispose();
        review = undefined;
        if (devProject) removeDevFile(devProject.dir, process.pid);
      },
      "astro:build:done": async ({ dir, logger }) => {
        if (await copyPublishedFiles(siteRoot, dir)) logger.info("copied _ascribe/files/");
      },
    },
  };
}
