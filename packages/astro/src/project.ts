// What the integration needs from `ascribe.toml`: where a build's site output
// goes, and the consumer settings that must agree with `astro.config`.
// The compiler validates the whole file; this reads
// only what it compiled.
import { readFileSync } from "node:fs";
import path from "node:path";
import { parse } from "smol-toml";

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
  consumer: { site: string | undefined; basePath: string; trailingSlash: "always" | "never" };
  /** The site output's root for a build: `<output-dir>/<build>/site`. */
  siteRoot(build: string): string;
}

/** Reads `ascribe.toml` in `dir`. Throws a readable error if it can't be read. */
export function readProject(dir: string): ProjectInfo {
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
  const consumer = section(table, "consumer");
  const outputDir =
    typeof project["output-dir"] === "string" ? project["output-dir"] : ".ascribe/build";
  const contentDir = typeof project["content-root"] === "string" ? project["content-root"] : "docs";
  const trailingSlash = consumer["trailing-slash"] === "never" ? "never" : "always";
  return {
    dir,
    configPath,
    contentRoot: path.resolve(dir, contentDir),
    outputRoot: path.resolve(dir, outputDir),
    consumer: {
      site: typeof consumer["site"] === "string" ? consumer["site"] : undefined,
      basePath: normalizeBase(
        typeof consumer["base-path"] === "string" ? consumer["base-path"] : "/",
      ),
      trailingSlash,
    },
    siteRoot: (build) => path.resolve(dir, outputDir, build, "site"),
  };
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
  if (normalizeBase(astro.base) !== basePath) {
    problems.push(
      `[consumer] base-path is "${basePath}", but Astro's \`base\` is "${astro.base}". Set them to the same path.`,
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
