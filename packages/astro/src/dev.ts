import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import type { AstroIntegration } from "astro";
import { consumerMismatches, readProject, type ProjectInfo } from "./project.js";

type ServerOptions = Parameters<NonNullable<AstroIntegration["hooks"]["astro:server:setup"]>>[0];

/**
 * The assets a build copied from outside the content root, as absolute paths,
 * read from the site output's manifest. Watching the content root doesn't
 * cover them. A manifest that's missing or unreadable gives none: the next
 * successful build writes one.
 */
export function outsideAssets(project: ProjectInfo, build: string): string[] {
  const manifest = path.join(path.dirname(project.siteRoot(build)), "site.manifest.json");
  let files: unknown;
  try {
    files = (JSON.parse(readFileSync(manifest, "utf8")) as { files?: unknown }).files;
  } catch {
    return [];
  }
  if (!Array.isArray(files)) return [];
  const assets = new Set<string>();
  for (const entry of files as { kind?: unknown; source?: unknown }[]) {
    if (entry.kind !== "asset" || typeof entry.source !== "string") continue;
    const absolute = path.resolve(project.contentRoot, entry.source);
    if (!absolute.startsWith(project.contentRoot + path.sep)) assets.add(absolute);
  }
  return [...assets].sort();
}

/**
 * Each watched file's size and modification time, as last seen. A watcher
 * can report an edit late, after a build already read it: macOS delivers
 * edits made just before the dev server started some seconds after it
 * started. An event that changes neither is no edit, and rebuilding for it
 * would reload the page under the reader.
 */
export class SeenFiles {
  private readonly stamps = new Map<string, string>();

  /** Records the files under `paths` (files or folders) as they are now. */
  prime(paths: readonly string[]): void {
    for (const at of paths) {
      let entries: string[];
      try {
        entries = statSync(at).isDirectory()
          ? readdirSync(at, { recursive: true, encoding: "utf8" }).map((f) => path.join(at, f))
          : [at];
      } catch {
        continue;
      }
      for (const file of entries) {
        const stamp = stampOf(file);
        if (stamp !== undefined && !this.stamps.has(file)) this.stamps.set(file, stamp);
      }
    }
  }

  /** Whether `file` differs from when it was last seen (or wasn't seen), and records it. */
  changed(file: string): boolean {
    const stamp = stampOf(file);
    const before = this.stamps.get(file);
    if (stamp === undefined) this.stamps.delete(file);
    else this.stamps.set(file, stamp);
    return stamp === undefined || stamp !== before;
  }
}

function stampOf(file: string): string | undefined {
  try {
    const stat = statSync(file);
    return stat.isFile() ? `${stat.size}:${stat.mtimeMs}` : undefined;
  } catch {
    return undefined;
  }
}

/** What `watchDev` gives back. */
export interface DevBuilds {
  /** Runs `task` in turn with the compiler runs, never during one. */
  inTurn(task: () => Promise<void>): Promise<void>;
}

/** Watch source files, serialize compiler runs, and refresh Astro after successful output. */
export function watchDev(options: {
  server: ServerOptions["server"];
  refreshContent: () => Promise<void>;
  logger: ServerOptions["logger"];
  project: ProjectInfo;
  astro: { base: string; trailingSlash: string; site: string | undefined };
  build: string;
  rebuild: () => Promise<void>;
  /** Called after each successful rebuild, before Astro reloads the page. */
  onBuilt?: () => void;
  /** The sources as the first build read them. */
  seen?: SeenFiles;
}): DevBuilds {
  const { server, refreshContent, logger, rebuild } = options;
  let project = options.project;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let pending: Promise<void> = Promise.resolve();
  let dirty = false;
  let running = false;
  let failed = false;
  let assets = new Set<string>();
  const seen = options.seen ?? new SeenFiles();
  const watchAssets = () => {
    assets = new Set(outsideAssets(project, options.build));
    if (assets.size > 0) server.watcher.add([...assets]);
    seen.prime([...assets]);
  };

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
      options.onBuilt?.();
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
        watchAssets();
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
    if (!["add", "change", "unlink"].includes(event)) return;
    const absolute = path.resolve(file);
    const source =
      absolute === project.configPath ||
      assets.has(absolute) ||
      (absolute.startsWith(project.contentRoot + path.sep) &&
        !absolute.startsWith(project.outputRoot + path.sep));
    if (!source) return;
    // A late report of an edit the last build already had.
    if (event !== "unlink" && !seen.changed(absolute)) return;
    dirty = true;
    schedule();
  };
  server.watcher.add(project.contentRoot);
  server.watcher.add(project.configPath);
  seen.prime([project.contentRoot, project.configPath]);
  watchAssets();
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
  return {
    inTurn: (task) => {
      const run = pending.then(task);
      pending = run.catch(() => undefined);
      return run;
    },
  };
}
