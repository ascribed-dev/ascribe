// @ascribed/astro: the Astro integration for Ascribe. Written against Astro
// 7.3.5.
//
//   // astro.config.mjs
//   import ascribe from "@ascribed/astro";
//   export default defineConfig({
//     // `site`, `base`, and `trailingSlash` come from ascribe.toml's
//     // [consumer] when they're left out here; set here, they must agree.
//     integrations: [ascribe({ build: "site" })],
//   });
//
//   // src/content.config.ts
//   import { defineCollection } from "astro:content";
//   import { ascribeCollection } from "@ascribed/astro/content";
//   // The generated schema. With the project outside the Astro root, import the
//   // copy inside it instead: "../.astro/integrations/_ascribed_astro/schema.ts".
//   import { schema } from "../.ascribe/build/site/site/_ascribe/schema.ts";
//   export const collections = { docs: defineCollection(ascribeCollection({ schema })) };
//
//   // a layout
//   import Elements from "@ascribed/astro/Elements.astro";           // <Elements /> in <head>
//   import Availability from "@ascribed/astro/Availability.astro";   // the page's availability
//   import Head from "@ascribed/astro/Head.astro";                   // <Head entry={entry} /> in <head>
//   import Versions from "@ascribed/astro/Versions.astro";           // the lines that have the page
//
//   // the sidebar: the project's navigation, joined to the collection
//   import { sidebar } from "@ascribed/astro/content";
//   const groups = sidebar(await getCollection("docs"));
//
//   // src/pages/404.astro
//   import MissingPermalink from "@ascribed/astro/MissingPermalink.astro"; // a permalink that isn't here
//
// Several projects in one site, each entry named, and each collection naming its entry:
//
//   ascribe({
//     projects: [
//       { name: "docs", project: "../docs", build: "site" },
//       { name: "home", project: "home", build: "site" },
//     ],
//     review: "docs",
//   })
//   ascribeCollection({ schema, name: "docs" })
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { AstroIntegration, IntegrationResolvedRoute } from "astro";
import { findBinary } from "./binary.js";
import { codeTitles } from "./code-titles.js";
import { SeenFiles, watchDev, type DevBuilds } from "./dev.js";
import {
  agentsMiddleware,
  copyAgentFiles,
  copyPublishedFiles,
  filesMiddleware,
  folderUnder,
  inOrder,
} from "./files.js";
import { WARN as HEAD_WARN } from "./head-check.js";
import { headersFor, headersOutcome, mergeRules, readHostHeaders } from "./headers.js";
import { mergeRedirects, readHostRedirects, redirectsOutcome } from "./redirects.js";
import { helpPatterns } from "./help.js";
import { forwardingRoutes, hasPermalinks, permalinkPattern, readPermalinks } from "./permalinks.js";
import {
  consumerMismatches,
  homeRoute,
  normalizeBase,
  readProject,
  supplyBuildFormat,
  supplyRouting,
  type AstroRouting,
} from "./project.js";
import {
  basePathClashes,
  nameProblems,
  pagesUnderOtherPaths,
  permalinkProblems,
  type ProjectEntry,
  type ResolvedEntry,
} from "./projects.js";
import {
  pageList,
  readRecords,
  routeAs,
  routeKey,
  sitemapRoute,
  type ProjectRobots,
} from "./records.js";
import rehypeAscribeAttributes from "./rehype.js";
import { removeDevFile, siteUrl, writeDevFile } from "./review/devfile.js";
import { runDiff, runPrompt } from "./review/diff.js";
import { APP_ID } from "./review/protocol.js";
import { readRoutes, type RoutedPage } from "./review/routes.js";
import { channelProblem, ReviewServer, type ToolbarChannel } from "./review/server.js";
import {
  devRouted,
  devSitePages,
  rootFiles,
  rootFilesMiddleware,
  type RootFileSources,
} from "./root-files.js";
import { anchorsFor, hasWarnings, reviewFor, runBuild } from "./run.js";
import { satteriAscribeAttributes } from "./satteri.js";
import { copySchema } from "./schema.js";
import { linesOf, olderLines } from "./versions.js";

export { default as rehypeAscribeAttributes } from "./rehype.js";
export { satteriAscribeAttributes } from "./satteri.js";
export type { ProjectEntry } from "./projects.js";

/** The virtual module `content.ts` reads the resolved site output roots from. */
const SITE_MODULE = "virtual:ascribe/site";

/** Options every form of the integration takes. */
interface SharedOptions {
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
   * removes the app. `astro build` never has it. With several projects, it
   * reviews one: the entry a name names, or the first for `true`.
   */
  review?: boolean | string;
  /**
   * Show a code block's title (`title="…"` in its info string, as `@snippet`
   * writes it), and what a tested example was tested with (`tested="…"`),
   * above it, with a Shiki transformer that puts the block in a
   * `<figure class="code-title">` with a `<figcaption>`. Default `true`.
   */
  codeTitles?: boolean;
  /**
   * The site's one sitemap, at `<base>sitemap.xml`, of every listed page of
   * every project and every page of the site's own. `exclude` lists routes,
   * from the host's root, of the site's own pages to leave out of it, such
   * as one that carries `noindex`.
   */
  sitemap?: { exclude?: string[] };
}

/** Options of the integration with one project. */
export interface OneProjectOptions extends SharedOptions {
  /** The build whose site output is the collection (a name in `ascribe.toml`). */
  build: string;
  /** The directory holding `ascribe.toml`, relative to the Astro root. Default: the root. */
  project?: string;
  projects?: never;
}

/** Options of the integration with several projects, each entry named. */
export interface ProjectsOptions extends SharedOptions {
  /** The projects, each a named build of a project; two entries may share a project. */
  projects: ProjectEntry[];
  build?: never;
  project?: never;
}

/** Options of the integration. */
export type AscribeOptions = OneProjectOptions | ProjectsOptions;

/** The toolbar app's icon: a speech bubble over a page. */
const ICON =
  '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"><path d="M6 3h8l4 4v6"/><path d="M14 3v4h4"/><path d="M6 3v15"/><path d="M11 14h10v6h-6l-3 2v-2h-1z"/></svg>';

/** An entry as the hooks use it. */
interface Entry extends ResolvedEntry {
  siteRoot: string;
  /** The plain output's root, when the project publishes it for agents. */
  agentsRoot: string | undefined;
  /** Where the generated schema's copy goes, inside the Astro root. */
  schemaDir: string;
}

/** A project with the entries that build it, which a dev watcher follows together. */
interface DevGroup {
  project: ResolvedEntry["project"];
  entries: Entry[];
  rebuild: () => Promise<void>;
  builds?: DevBuilds;
}

function fail(problems: string[], what: string): void {
  if (problems.length > 0) throw new Error(`@ascribed/astro: ${what}:\n- ${problems.join("\n- ")}`);
}

/**
 * Runs `ascribe build` for each project, checks the site's routing, adds the markdown plugin, adds a
 * page for each permalink's address and an answer for each line's help entries, and serves published files, and, with `[consumer] agents = true`,
 * `llms.txt` and each page's Markdown, each at its project's base path; and writes the site's one
 * sitemap and `robots.txt` from every project's page records, and, with `[consumer] host =
 * "netlify"`, its `_headers` and `_redirects` from the host's files each build writes.
 */
export default function ascribe(options: AscribeOptions): AstroIntegration {
  /** With `projects`; otherwise the one entry has no name a collection gives. */
  const named = options.projects !== undefined;
  let entries: Entry[] = [];
  let groups: DevGroup[] = [];
  let routing: AstroRouting | undefined;
  /** The dev server's sources as its first build read them. */
  const seen = new SeenFiles();
  let binary: string | undefined;
  let reviewed: Entry | undefined;
  let devCommand = false;
  let devHttps = false;
  let review: ReviewServer | undefined;
  /** The site's routes, as Astro resolved them, for what `astro dev` serves of the site's own. */
  let astroRoutes: IntegrationResolvedRoute[] = [];
  /** The site's `public/` folder, where its own `robots.txt` would be. */
  let publicDir = "";

  /** What the sitemap and `robots.txt` are drawn from, given the site's pages and its routes. */
  const rootSources = (
    sitePages: readonly string[],
    routed: (route: string) => boolean,
  ): RootFileSources => {
    const base = routing?.base ?? "/";
    const records = entries.map((entry) => readRecords(entry.siteRoot));
    // Entries of one project share its `ascribe.toml`, and so what it says to crawlers.
    const robots = new Map<string, ProjectRobots>();
    for (const [i, entry] of entries.entries()) {
      if (!robots.has(entry.project.dir))
        robots.set(entry.project.dir, {
          name: entry.name,
          robots: records[i]?.robots ?? { lines: [] },
        });
    }
    const root = normalizeBase(base);
    return {
      projects: [
        ...records.map(pageList),
        // An older line's pages are the site's too: those the current line has point at it.
        ...entries.flatMap((entry) =>
          olderLines(entry.siteRoot).map((line) => pageList(readRecords(line.siteRoot))),
        ),
      ],
      routed,
      sitePages,
      excluded: [
        ...forwardingRoutes(entries.map((entry) => readPermalinks(entry.siteRoot))),
        `${root}404`,
        `${root}500`,
        ...(options.sitemap?.exclude ?? []),
      ],
      robots: [...robots.values()],
      site: routing?.site,
      base,
      ownRobots: existsSync(path.join(publicDir, "robots.txt")),
      ownSitemap: existsSync(path.join(publicDir, "sitemap.xml")),
    };
  };
  return {
    name: "@ascribed/astro",
    hooks: {
      "astro:config:setup": async ({
        config,
        command,
        logger,
        updateConfig,
        addDevToolbarApp,
        createCodegenDir,
        injectRoute,
        addMiddleware,
      }) => {
        const root = fileURLToPath(config.root);
        publicDir = fileURLToPath(config.publicDir);
        const list: ProjectEntry[] = options.projects ?? [
          {
            name: "",
            build: options.build,
            ...(options.project === undefined ? {} : { project: options.project }),
          },
        ];
        if (named) fail(nameProblems(list), "the `projects` option is wrong");
        const codegen = command === "preview" ? "" : fileURLToPath(createCodegenDir());
        entries = list.map((entry) => {
          const project = readProject(path.resolve(root, entry.project ?? "."), {
            binary: options.binary,
            root,
          });
          return {
            name: entry.name,
            build: entry.build,
            project,
            // An unknown build is `ascribe build`'s to report: it knows the implicit `site` build.
            siteRoot: project.siteRoot(entry.build),
            agentsRoot: project.consumer.agents ? project.plainRoot(entry.build) : undefined,
            schemaDir: named ? path.join(codegen, entry.name) : codegen,
          };
        });

        // What astro.config leaves unset comes from [consumer]: `site` and `trailingSlash` from the
        // first project that has them, and with one project, `base` from its base path. With
        // `projects`, `base` stays the site's, the root every base path is under.
        const { routing: astro, supplied } = supplyRouting(
          entries.map((entry) => entry.project),
          { base: config.base, trailingSlash: config.trailingSlash, site: config.site },
          !named,
        );
        // `ascribe.toml`'s routing must be Astro's, or every link Ascribe writes is wrong.
        for (const entry of entries) {
          fail(
            consumerMismatches(entry.project, astro),
            named
              ? `"${entry.name}"'s ascribe.toml and astro.config disagree`
              : "ascribe.toml and astro.config disagree",
          );
        }
        // Each page's file must be served at the route Ascribe publishes for it, not behind a redirect.
        const format = supplyBuildFormat(astro.trailingSlash, config.build.format);
        fail(format.problems, "ascribe.toml and astro.config disagree");
        fail(basePathClashes(entries), "two projects share a base path");

        if (typeof options.review === "string" && !entries.some((e) => e.name === options.review)) {
          throw new Error(
            `@ascribed/astro: \`review\` names "${options.review}", and no entry of \`projects\` has that name.`,
          );
        }
        if (reviewFor(options.review, command)) {
          reviewed =
            typeof options.review === "string"
              ? entries.find((e) => e.name === options.review)
              : entries[0];
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

        routing = astro;

        if (reviewed !== undefined) {
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
          const buildEntry = async (entry: Entry) => {
            logger.info(
              `running ${path.basename(found)} build for "${entry.build}"${named ? ` (${entry.name})` : ""}`,
            );
            const run = {
              binary: found,
              configPath: entry.project.configPath,
              cwd: entry.project.dir,
              anchors: anchorsFor(options.anchors, command, entry === reviewed),
            };
            // `llms.txt` and each page's Markdown, which agents read.
            const plain = entry.agentsRoot === undefined ? [] : (["plain"] as const);
            const result = await runBuild({
              ...run,
              build: entry.build,
              outputs: [
                "site",
                ...plain,
                // While review is on, the JSON output says which page is at each route.
                ...(entry === reviewed && review?.active ? (["json"] as const) : []),
              ],
            });
            if (hasWarnings(result.diagnostics)) logger.warn(result.diagnostics);
            else if (result.diagnostics !== "") logger.info(result.diagnostics);
            if (result.summary !== "") logger.info(result.summary);
            // A versioned build's other lines, which the build's `versions.json` names, once the
            // current line's build has written it. An error in one fails this run as it would the
            // first; `ascribe check` reports every line's warnings, so only what was built is said.
            const older = olderLines(entry.siteRoot).map((line) => line.build);
            if (older.length > 0) {
              const lines = await runBuild({
                ...run,
                build: older,
                outputs: [
                  "site",
                  ...plain,
                  ...(entry === reviewed && review?.active ? (["json"] as const) : []),
                ],
              });
              if (lines.summary !== "") logger.info(lines.summary);
            }
            // The schema's copy inside the Astro root, for content.config.ts to import.
            mkdirSync(entry.schemaDir, { recursive: true });
            copySchema(entry.siteRoot, entry.schemaDir);
          };
          // What the projects share, which no one project's build can see.
          const checkShared = () => {
            if (entries.length < 2) return;
            const current = entries.map((entry) => ({
              ...entry,
              project: readProject(entry.project.dir, { binary: options.binary, root }),
            }));
            fail(basePathClashes(current), "two projects share a base path");
            fail(pagesUnderOtherPaths(current), "a page is under another project's base path");
            fail(
              permalinkProblems(
                current,
                current.map((entry) => readPermalinks(entry.siteRoot)),
              ),
              "the projects' permalinks can't share one route",
            );
          };
          // One group for each project: a change rebuilds the project it's in.
          const byDir = new Map<string, Entry[]>();
          for (const entry of entries) {
            byDir.set(entry.project.dir, [...(byDir.get(entry.project.dir) ?? []), entry]);
          }
          groups = [...byDir.values()].map((group) => ({
            project: (group[0] as Entry).project,
            entries: group,
            rebuild: async () => {
              for (const entry of group) await buildEntry(entry);
              checkShared();
            },
          }));
          // The sources as the first build reads them: the watcher may report their edits late.
          if (command === "dev") {
            for (const group of groups)
              seen.prime([group.project.contentRoot, group.project.configPath]);
          }
          for (const entry of entries) await buildEntry(entry);
          checkShared();
          devCommand = command === "dev";
          if (devCommand) {
            // A tag of the head written twice, by Head.astro and the layout, is warned about.
            (globalThis as Record<symbol, unknown>)[HEAD_WARN] = (message: string) =>
              logger.warn(message);
            addMiddleware({
              entrypoint: new URL("./head-check.js", import.meta.url),
              order: "post",
            });
          }

          // Each permalink's address, a page that forwards to its heading. One route serves every
          // list, at the path of the first that names anything.
          const lists = entries.flatMap((entry) => readPermalinks(entry.siteRoot) ?? []);
          const served = lists.find(hasPermalinks) ?? lists[0];
          if (served !== undefined) {
            injectRoute({
              pattern: permalinkPattern(served.path, astro.base),
              entrypoint: new URL("../src/Permalink.astro", import.meta.url),
              prerender: true,
            });
          }
          // Each permalink's help, at `<name>.json` beside its address, and each older line's
          // under the line's base path. One route for each folder: the current line's, then each
          // older line's.
          for (const pattern of helpPatterns(
            entries.map((entry) => entry.siteRoot),
            astro.base,
          )) {
            injectRoute({
              pattern,
              entrypoint: new URL("./help-route.js", import.meta.url),
              prerender: true,
            });
          }
        }

        const projectHome = (entry: Entry) =>
          homeRoute(entry.project.consumer.basePath, astro.trailingSlash);
        // Dev serving of `_ascribe/files/`, and of the plain output for agents; the build copies them in `astro:build:done`.
        updateConfig({
          ...supplied,
          ...(format.supplied === undefined ? {} : { build: { format: format.supplied } }),
          ...(options.codeTitles === false
            ? {}
            : { markdown: { shikiConfig: { transformers: [codeTitles] } } }),
          vite: {
            // `Elements.astro` is Astro source, so Vite must compile it, not load it as a Node module.
            ssr: { noExternal: ["@ascribed/astro"] },
            plugins: [
              {
                name: "@ascribed/astro:site",
                resolveId: (id) => (id === SITE_MODULE ? `\0${SITE_MODULE}` : undefined),
                load: (id) =>
                  id === `\0${SITE_MODULE}`
                    ? `export const siteRoot = ${JSON.stringify(entries[0]?.siteRoot ?? "")};\n` +
                      `export const astroRoot = ${JSON.stringify(root)};\n` +
                      `export const home = ${JSON.stringify(homeRoute(astro.base, astro.trailingSlash))};\n` +
                      `export const base = ${JSON.stringify(astro.base)};\n` +
                      `export const collections = ${JSON.stringify(
                        named ? Object.fromEntries(entries.map((e) => [e.name, e.siteRoot])) : {},
                      )};\n` +
                      `export const permalinks = ${JSON.stringify(
                        entries.map((e) => ({ siteRoot: e.siteRoot, home: projectHome(e) })),
                      )};\n` +
                      `export const routes = ${JSON.stringify(
                        entries.map((e) => ({
                          siteRoot: e.siteRoot,
                          basePath: e.project.consumer.basePath,
                          trailingSlash: e.project.consumer.trailingSlash,
                        })),
                      )};`
                    : undefined,
              },
              {
                name: "@ascribed/astro:files",
                configureServer(server) {
                  for (const entry of entries) {
                    const base = entry.project.consumer.basePath;
                    server.middlewares.use(filesMiddleware(entry.siteRoot, base));
                    // Each older line's, at its own base path; the lines are read as each request comes.
                    server.middlewares.use((req, res, next) =>
                      inOrder(
                        olderLines(entry.siteRoot).map((line) =>
                          filesMiddleware(line.siteRoot, line.base),
                        ),
                        req,
                        res,
                        next,
                      ),
                    );
                    if (entry.agentsRoot !== undefined) {
                      const siteRoot = entry.siteRoot;
                      server.middlewares.use(
                        agentsMiddleware(entry.agentsRoot, base, (pathname) =>
                          headersFor(siteRoot, pathname),
                        ),
                      );
                      // Each older line's outputs for agents, at its own base path.
                      server.middlewares.use((req, res, next) =>
                        inOrder(
                          olderLines(siteRoot).map((line) =>
                            agentsMiddleware(line.plainRoot, line.base, (pathname) =>
                              headersFor(line.siteRoot, pathname),
                            ),
                          ),
                          req,
                          res,
                          next,
                        ),
                      );
                    }
                  }
                  // The sitemap and robots.txt, as the build would write them now.
                  server.middlewares.use(
                    rootFilesMiddleware(astro.base, () =>
                      rootFiles(
                        rootSources(
                          devSitePages(astroRoutes, astro),
                          devRouted(astroRoutes, astro),
                        ),
                      ),
                    ),
                  );
                },
              },
            ],
          },
        });
      },
      "astro:routes:resolved": ({ routes }) => {
        astroRoutes = routes;
      },
      "astro:server:setup": ({ server, refreshContent, logger, toolbar }) => {
        if (!devCommand || !routing) return;
        if (!refreshContent)
          throw new Error("@ascribed/astro: this Astro version does not support refreshContent.");
        devHttps = Boolean(server.config.server.https);
        for (const group of groups) {
          group.builds = watchDev({
            server,
            seen,
            refreshContent: () => refreshContent({}),
            logger,
            project: group.project,
            astro: routing,
            builds: group.entries.map((entry) => entry.build),
            rebuild: group.rebuild,
            ...(binary === undefined ? {} : { binary: { binary, root: group.project.dir } }),
            onBuilt: () => {
              if (reviewed !== undefined && group.entries.includes(reviewed)) review?.rebuilt();
            },
          });
        }
        const group = groups.find((g) => reviewed !== undefined && g.entries.includes(reviewed));
        if (reviewed !== undefined && group?.builds !== undefined && binary !== undefined) {
          const entry = reviewed;
          const builds = group.builds;
          const run = { binary, configPath: entry.project.configPath, cwd: entry.project.dir };
          // A versioned build is reviewed in every line, the current one first, each line's
          // routes from its own JSON output.
          const lines = () =>
            linesOf(entry.siteRoot)?.map((line) => ({
              build: line.build,
              siteRoot: line.siteRoot,
            })) ?? [{ build: entry.build, siteRoot: entry.siteRoot }];
          const lineBuilds = () => lines().map((line) => line.build);
          review = new ReviewServer({
            channel: toolbar as ToolbarChannel,
            logger,
            build: entry.build,
            contentRoot: entry.project.contentRoot,
            channelProblem: channelProblem(server.config),
            diff: (base) => runDiff({ ...run, build: lineBuilds(), base }),
            prompt: (page, base) => runPrompt({ ...run, build: lineBuilds(), base, path: page }),
            projectDir: entry.project.dir,
            connect: async () => (await import("./review/github.js")).connect(entry.project.dir),
            writeRoutes: () =>
              builds.inTurn(async () => {
                await runBuild({ ...run, build: lineBuilds(), outputs: ["json"] });
              }),
            readRoutes: async () => {
              const all = new Map<string, RoutedPage>();
              const versioned = lines().length > 1;
              for (const line of lines()) {
                const jsonRoot = path.join(path.dirname(line.siteRoot), "json");
                for (const [route, page] of await readRoutes(jsonRoot)) {
                  if (!all.has(route))
                    all.set(route, versioned ? { ...page, build: line.build } : page);
                }
              }
              return all;
            },
          });
        }
      },
      "astro:server:start": ({ address, logger }) => {
        review?.listening(address.address);
        if (!devCommand) return;
        // Each project's folder gets its own file, with its own address: its base path.
        for (const group of groups) {
          const entry = group.entries[0] as Entry;
          const dir = group.project.dir;
          try {
            writeDevFile(dir, {
              url: siteUrl(address, { https: devHttps, base: entry.project.consumer.basePath }),
              build: entry.build,
              pid: process.pid,
            });
            // Vite exits on a signal before `astro:server:done` runs.
            process.once("exit", () => removeDevFile(dir, process.pid));
          } catch (error) {
            logger.warn(
              `couldn't write ${groups.length > 1 ? path.join(dir, ".ascribe", "dev.json") : ".ascribe/dev.json"}, which the editor's Open Site Preview reads: ${error instanceof Error ? error.message : String(error)}`,
            );
          }
        }
      },
      "astro:server:done": () => {
        review?.dispose();
        review = undefined;
        if (devCommand) for (const group of groups) removeDevFile(group.project.dir, process.pid);
      },
      "astro:build:done": async ({ dir, pages, logger }) => {
        const base = routing?.base ?? "/";
        const trailingSlash = routing?.trailingSlash ?? "ignore";
        // Every page Astro built, by its route from the host's root. Astro lists
        // pages only, not endpoints such as a feed, so a dot in a page's last
        // segment, as in `/v1.2`, is still a page.
        const built = pages.map((page) =>
          routeAs(`${normalizeBase(base)}${page.pathname.replace(/^\//, "")}`, trailingSlash),
        );
        const builtKeys = new Set(built.map(routeKey));
        const files = rootFiles(rootSources(built, (route) => builtKeys.has(routeKey(route))));
        fail(files.problems, "the site's robots.txt");
        for (const note of files.notes) logger[note.level](note.text);
        const out = fileURLToPath(dir);
        if (files.sitemap !== undefined) {
          await writeFile(path.join(out, "sitemap.xml"), files.sitemap);
          logger.info(`wrote ${sitemapRoute(base)}`);
        }
        if (files.robots !== undefined) {
          await writeFile(path.join(out, "robots.txt"), files.robots);
          logger.info("wrote /robots.txt");
        }
        // The host's files, from the projects that name it: each build's,
        // and each of its older lines'.
        const hostRoots = entries
          .filter((entry) => entry.project.consumer.host === "netlify")
          .flatMap((entry) => [
            entry.siteRoot,
            ...olderLines(entry.siteRoot).map((line) => line.siteRoot),
          ]);
        const ownHeaders = path.join(publicDir, "_headers");
        const headers = headersOutcome({
          rules: mergeRules(hostRoots.map((siteRoot) => readHostHeaders(siteRoot) ?? [])),
          base,
          own: existsSync(ownHeaders) ? readFileSync(ownHeaders, "utf8") : undefined,
        });
        if (headers.kind === "fail") throw new Error(`@ascribed/astro: ${headers.text}`);
        if (headers.kind === "note") logger.info(headers.text);
        if (headers.kind === "write") {
          await writeFile(path.join(out, "_headers"), headers.text);
          logger.info("wrote /_headers");
        }
        const ownRedirects = path.join(publicDir, "_redirects");
        const redirects = redirectsOutcome({
          redirects: mergeRedirects(hostRoots.map(readHostRedirects)),
          base,
          published: (route) => builtKeys.has(routeKey(route)),
          own: existsSync(ownRedirects) ? readFileSync(ownRedirects, "utf8") : undefined,
        });
        if (redirects.kind === "fail") throw new Error(`@ascribed/astro: ${redirects.text}`);
        if (redirects.kind === "note") logger.info(redirects.text);
        if (redirects.kind === "write") {
          await writeFile(path.join(out, "_redirects"), redirects.text);
          logger.info("wrote /_redirects");
        }
        // Entries of one project publish at one base path: the first entry's copies go last, so they win.
        for (const entry of [...entries].reverse()) {
          const folder = folderUnder(entry.project.consumer.basePath, base);
          const where = folder === "" ? "" : ` to ${folder}`;
          if (await copyPublishedFiles(entry.siteRoot, dir, folder))
            logger.info(`copied _ascribe/files/${where}`);
          for (const line of olderLines(entry.siteRoot)) {
            const lineFolder = folderUnder(line.base, base);
            if (await copyPublishedFiles(line.siteRoot, dir, lineFolder))
              logger.info(`copied ${line.build}'s _ascribe/files/ to ${lineFolder}`);
          }
          if (
            entry.agentsRoot !== undefined &&
            (await copyAgentFiles(entry.agentsRoot, dir, folder))
          )
            logger.info(`copied llms.txt and each page's Markdown${where}`);
          if (entry.agentsRoot !== undefined) {
            for (const line of olderLines(entry.siteRoot)) {
              const lineFolder = folderUnder(line.base, base);
              if (await copyAgentFiles(line.plainRoot, dir, lineFolder))
                logger.info(
                  `copied ${line.build}'s llms.txt and each page's Markdown to ${lineFolder}`,
                );
            }
          }
        }
      },
    },
  };
}
