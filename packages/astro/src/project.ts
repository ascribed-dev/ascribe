// What the integration needs from `ascribe.toml`: where a build's site output
// goes, and the consumer settings that must agree with `astro.config`.
// The compiler validates the whole file; this reads
// only what it compiled. A file with `[from]` takes keys from another
// project's, which only the binary follows, so then the binary is asked.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";
import { findBinary } from "./binary.js";

/** The parts of an Ascribe project the integration reads. */
export interface ProjectInfo {
  /** The directory holding `ascribe.toml`. */
  dir: string;
  /** The path of `ascribe.toml`. */
  configPath: string;
  /** Source files and assets watched during development. */
  contentRoot: string;
  /** Generated output, never treated as a source change. */
  outputRoot: string;
  /** `[consumer]`, with the profile's defaults. */
  consumer: {
    site: string | undefined;
    basePath: string;
    trailingSlash: "always" | "never";
    /** Whether builds publish what agents read: `llms.txt` and each page's Markdown, in the plain output. */
    agents: boolean;
    /** The host the site is deployed to, whose files the compiler writes: `"netlify"`, or none. */
    host: "netlify" | undefined;
  };
  /** The names of `[builds]`'s tables, in order: empty when there are none, and the one build is `site`. */
  builds: string[];
  /** The site output's root for a build: `<output-dir>/<build>/site`. */
  siteRoot(build: string): string;
  /** The plain output's root for a build: `<output-dir>/<build>/plain`. */
  plainRoot(build: string): string;
}

/** Where to find the binary, as the integration's `binary` option and Astro's root say. */
export interface BinaryOption {
  binary?: string | undefined;
  root: string;
}

/**
 * Reads `ascribe.toml` in `dir`. Throws a readable error if it can't be read.
 * When it has `[from]`, what it takes is the other file's, which the binary
 * reads: `[consumer]` and `[builds]` are then the binary's answer, from
 * `ascribe model --format json`, run with the binary `binary` finds
 * (`findBinary`).
 */
export function readProject(dir: string, binary?: BinaryOption): ProjectInfo {
  const configPath = path.join(dir, "ascribe.toml");
  let table: Record<string, unknown>;
  try {
    table = parse(readFileSync(configPath, "utf8"));
  } catch (error) {
    throw new Error(
      `@ascribed/astro can't read ${configPath}: ${error instanceof Error ? error.message : String(error)}`,
      { cause: error },
    );
  }
  const project = section(table, "project");
  let consumer = section(table, "consumer");
  let builds = Object.keys(section(table, "builds"));
  if (Object.keys(section(table, "from")).length > 0) {
    const model = askModel(configPath, findBinary(binary ?? { root: dir }));
    consumer = Object.fromEntries(
      model.consumer.filter((key) => key.set).map((key) => [key.key, key.value]),
    );
    if (takes(table, "builds")) builds = model.builds.map((build) => build.name);
  }
  const outputDir =
    typeof project["output-dir"] === "string" ? project["output-dir"] : ".ascribe/build";
  const contentDir = typeof project["content-root"] === "string" ? project["content-root"] : "docs";
  const trailingSlash = consumer["trailing-slash"] === "never" ? "never" : "always";
  const site = typeof consumer["site"] === "string" ? consumer["site"] : undefined;
  return {
    dir,
    configPath,
    contentRoot: path.resolve(dir, contentDir),
    outputRoot: path.resolve(dir, outputDir),
    consumer: {
      site,
      basePath: normalizeBase(
        typeof consumer["base-path"] === "string" ? consumer["base-path"] : "/",
      ),
      trailingSlash,
      // `true` with a site unless it says otherwise, as the binary reads it.
      agents: typeof consumer["agents"] === "boolean" ? consumer["agents"] : site !== undefined,
      host: consumer["host"] === "netlify" ? "netlify" : undefined,
    },
    builds,
    siteRoot: (build) => path.resolve(dir, outputDir, build, "site"),
    plainRoot: (build) => path.resolve(dir, outputDir, build, "plain"),
  };
}

/** What the integration reads of `ascribe model --format json`. */
interface ModelReport {
  consumer: { key: string; value?: unknown; set: boolean }[];
  builds: { name: string; line?: string }[];
}

/** The content model as the binary reads it, `[from]` followed. Throws a readable error when it can't. */
function askModel(configPath: string, binary: string): ModelReport {
  try {
    const out = execFileSync(binary, ["model", "--format", "json", "--config", configPath], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const report = JSON.parse(out) as Partial<ModelReport>;
    return {
      consumer: report.consumer ?? [],
      builds: (report.builds ?? []).filter((build) => build.line === undefined),
    };
  } catch (error) {
    const stderr = (error as { stderr?: unknown }).stderr;
    const detail =
      typeof stderr === "string" && stderr.trim() !== ""
        ? stderr.trim()
        : error instanceof Error
          ? error.message
          : String(error);
    throw new Error(
      `@ascribed/astro can't read ${configPath}, whose [from] takes keys from another file, with \`ascribe model\`: ${detail}`,
      { cause: error },
    );
  }
}

/** Whether a `take` list of `[from]` names `name`, or a key in it. */
function takes(table: Record<string, unknown>, name: string): boolean {
  return Object.values(section(table, "from")).some((from) => {
    const take = typeof from === "object" && from !== null ? (from as { take?: unknown }).take : [];
    return (
      Array.isArray(take) &&
      take.some((key) => typeof key === "string" && (key === name || key.startsWith(`${name}.`)))
    );
  });
}

function section(table: Record<string, unknown>, name: string): Record<string, unknown> {
  const value = table[name];
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

/** A base path with a leading and a trailing `/`, as Ascribe's router has it. */
export function normalizeBase(base: string): string {
  let result = base.trim();
  if (!result.startsWith("/")) result = `/${result}`;
  if (!result.endsWith("/")) result += "/";
  return result;
}

/**
 * The route of a base path, ending the way `trailingSlash` says the site's
 * routes do: the site's root for Astro's `base`, a project's root for its
 * `[consumer] base-path`. (`import.meta.env.BASE_URL` isn't replaced in a
 * package's components.)
 */
export function homeRoute(base: string, trailingSlash: "always" | "never" | "ignore"): string {
  const root = normalizeBase(base);
  return trailingSlash === "never" && root !== "/" ? root.slice(0, -1) : root;
}

/** Astro's routing settings, as the integration reads and supplies them. */
export interface AstroRouting {
  base: string;
  trailingSlash: "always" | "never" | "ignore";
  site: string | undefined;
}

/**
 * Astro's routing with what astro.config leaves unset taken from `[consumer]`,
 * and the settings taken, for `updateConfig`. Astro gives an integration its
 * configuration with its defaults filled in, so unset is the default: no
 * `site`, `trailingSlash: "ignore"`, and `base: "/"`. `site` and
 * `trailingSlash` come from the first project that has them, since every
 * project must agree with them anyway. `base` comes from the first project's
 * `base-path` only with `supplyBase`, as with one project; with several, it
 * stays the site's, the root every base path is under. A base Astro has is
 * rewritten to end the way a supplied `trailingSlash` says, as Astro writes
 * it when the setting is its own.
 */
export function supplyRouting(
  projects: readonly ProjectInfo[],
  astro: AstroRouting,
  supplyBase: boolean,
): { routing: AstroRouting; supplied: Partial<AstroRouting> } {
  const supplied: Partial<AstroRouting> = {};
  if (astro.site === undefined) {
    const site = projects.find((project) => project.consumer.site !== undefined)?.consumer.site;
    if (site !== undefined) supplied.site = site;
  }
  const first = projects[0];
  if (first === undefined) return { routing: astro, supplied };
  if (astro.trailingSlash === "ignore") supplied.trailingSlash = first.consumer.trailingSlash;
  const trailingSlash = supplied.trailingSlash ?? astro.trailingSlash;
  const base = homeRoute(
    supplyBase && normalizeBase(astro.base) === "/" ? first.consumer.basePath : astro.base,
    trailingSlash,
  );
  if (base !== astro.base) supplied.base = base;
  return { routing: { ...astro, ...supplied }, supplied };
}

/** Astro's `build.format`: how a page's route becomes the file a static host serves it from. */
type BuildFormat = "file" | "directory" | "preserve";

/**
 * Astro's `build.format` for routes that end as `trailingSlash` says, and
 * the format to supply for `updateConfig`. A static host serves `x/index.html`
 * at `x/` and `x.html` at `x`, so `"never"` needs `"file"` and `"always"`
 * needs `"directory"`; any other format serves each page behind a redirect.
 * Astro can't tell an integration a format left out from its default,
 * `"directory"`, so that one is replaced for `"never"`; any other that
 * disagrees is a problem, as a sentence.
 */
export function supplyBuildFormat(
  trailingSlash: "always" | "never" | "ignore",
  format: BuildFormat,
): { supplied: BuildFormat | undefined; problems: string[] } {
  if (trailingSlash === "ignore") return { supplied: undefined, problems: [] };
  const wanted = trailingSlash === "never" ? "file" : "directory";
  if (format === wanted) return { supplied: undefined, problems: [] };
  if (format === "directory") return { supplied: wanted, problems: [] };
  return {
    supplied: undefined,
    problems: [
      `[consumer] trailing-slash is "${trailingSlash}", but Astro's \`build.format\` is "${format}", which a static host serves behind a redirect. Use "${wanted}", or leave it out.`,
    ],
  };
}

/**
 * The ways `ascribe.toml`'s `[consumer]` disagrees with Astro's configuration,
 * as sentences. Ascribe writes every link with its own settings, so a site
 * whose routes differ would have broken links.
 */
export function consumerMismatches(
  project: ProjectInfo,
  astro: { base: string; trailingSlash: string; site: string | undefined },
): string[] {
  const problems: string[] = [];
  const { basePath, trailingSlash, site } = project.consumer;
  // A base path under Astro's `base` leaves the rest of the site to other pages, such as a home page.
  if (!basePath.startsWith(normalizeBase(astro.base))) {
    problems.push(
      `[consumer] base-path is "${basePath}", but Astro's \`base\` is "${astro.base}". Set base-path to that path or a path under it.`,
    );
  }
  // Astro's "ignore" serves both forms, so Ascribe's links work either way.
  if (astro.trailingSlash !== "ignore" && astro.trailingSlash !== trailingSlash) {
    problems.push(
      `[consumer] trailing-slash is "${trailingSlash}", but Astro's \`trailingSlash\` is "${astro.trailingSlash}".`,
    );
  }
  if (site !== undefined && astro.site !== undefined && origin(site) !== origin(astro.site)) {
    problems.push(`[consumer] site is "${site}", but Astro's \`site\` is "${astro.site}".`);
  }
  return problems;
}

function origin(url: string): string {
  try {
    return new URL(url).origin;
  } catch {
    return url;
  }
}
