import path from "node:path";
import type { AstroIntegration } from "astro";
import { consumerMismatches, readProject, type ProjectInfo } from "./project.js";

type ServerOptions = Parameters<NonNullable<AstroIntegration["hooks"]["astro:server:setup"]>>[0];

/** Watch source files, serialize compiler runs, and refresh Astro after successful output. */
export function watchDev(options: {
  server: ServerOptions["server"];
  refreshContent: () => Promise<void>;
  logger: ServerOptions["logger"];
  project: ProjectInfo;
  astro: { base: string; trailingSlash: string; site: string | undefined };
  build: string;
  rebuild: () => Promise<void>;
}): void {
  const { server, refreshContent, logger, rebuild } = options;
  let project = options.project;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: Promise<void> = Promise.resolve();
  let dirty = false;
  let running = false;
  let failed = false;

  const drain = async () => {
    running = true;
    const previousProject = project;
    const previousSiteRoot = previousProject.siteRoot(options.build);
    while (dirty) {
      dirty = false;
      try {
        await rebuild();
        failed = false;
      } catch (error) {
        failed = true;
        logger.error(error instanceof Error ? error.message : String(error));
      }
    }
    if (!failed) {
      try {
        const reloaded = readProject(project.dir);
        if (reloaded.siteRoot(options.build) !== previousSiteRoot) {
          throw new Error(
            "ascribe.toml changed output-dir; restart astro dev to reload the collection.",
          );
        }
        const mismatches = consumerMismatches(reloaded, options.astro);
        if (mismatches.length > 0) {
          throw new Error(
            `@ascribed/astro: ascribe.toml and astro.config disagree:\n- ${mismatches.join("\n- ")}`,
          );
        }
        project = reloaded;
        server.watcher.add(project.contentRoot);
        server.watcher.add(project.configPath);
        await refreshContent();
        server.ws.send({ type: "full-reload" });
      } catch (error) {
        failed = true;
        logger.error(
          `could not refresh Ascribe content: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
    running = false;
  };
  const schedule = () => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = undefined;
      pending = pending.then(drain);
    }, 50);
  };
  const onChange = (event: string, file: string) => {
    const absolute = path.resolve(file);
    if (absolute === project.configPath && ["add", "change", "unlink"].includes(event)) {
      dirty = true;
      schedule();
      return;
    }
    if (
      !["add", "change", "unlink"].includes(event) ||
      !absolute.startsWith(project.contentRoot + path.sep) ||
      absolute.startsWith(project.outputRoot + path.sep)
    )
      return;
    dirty = true;
    schedule();
  };
  server.watcher.add(project.contentRoot);
  server.watcher.add(project.configPath);
  server.watcher.on("all", onChange);
  server.httpServer?.once("close", () => {
    if (timer) clearTimeout(timer);
    server.watcher.off("all", onChange);
  });
  server.middlewares.use((_req, res, next) => {
    if (!dirty && !running && !failed) return next();
    if (timer) {
      clearTimeout(timer);
      timer = undefined;
      pending = pending.then(drain);
    }
    void pending.then(() => {
      if (failed) {
        res.statusCode = 503;
        res.end("Ascribe build failed; fix the source and save to retry.");
      } else next();
    });
  });
}
