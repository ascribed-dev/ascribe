// The starter, `npx @ascribed/astro init`: writes into an Astro project, once,
// the files every site built with Ascribe writes again: the collection, the
// route, and a layout that loads the elements and draws the sidebar from the
// project's navigation. From then on they're the site's own, as `ascribe
// templates eject` leaves a project's templates; `init` never writes over a
// file.
//
// The files are in `starter/`, as `examples/astro-site/src` has them; the
// example's are what `init` writes for its project, which a test holds. One
// thing differs from one project to the next: the schema's path.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { readProject, type ProjectInfo } from "./project.js";

/** The files `init` writes, by path from the Astro root. */
export const STARTER_FILES = [
  "src/content.config.ts",
  "src/pages/[...slug].astro",
  "src/layouts/Docs.astro",
] as const;

/** The starter's files, in the package beside `src/` and `dist/`. */
const STARTER = fileURLToPath(new URL("../starter/", import.meta.url));

/** The import of the schema in the starter's `content.config.ts`, whose path `init` replaces. */
const SCHEMA_IMPORT = /^(import \{ schema \} from )"[^"]*";$/m;

/** What `init` is told. */
export interface InitOptions {
  /** The Astro root, where the files go. */
  root: string;
  /** The directory holding `ascribe.toml`, relative to the root. Default: the root. */
  project?: string | undefined;
  /** The build whose site output is the collection. Default: the project's one build. */
  build?: string | undefined;
}

/** What `init` did. */
export interface InitResult {
  /** The build whose site output is the collection, which the integration's `build` option names. */
  build: string;
  /** The files it wrote, by path from the root. */
  written: string[];
}

/**
 * Writes the starter's files into the Astro project at `root`, for a build of
 * the project. Writes nothing, and throws, when any of them is there already,
 * or when the build can't be chosen.
 */
export async function init(options: InitOptions): Promise<InitResult> {
  const root = path.resolve(options.root);
  const project = readProject(path.resolve(root, options.project ?? "."));
  const build = chooseBuild(project, options.build);
  const there = STARTER_FILES.filter((file) => existsSync(path.join(root, file)));
  if (there.length > 0) {
    throw new Error(
      `@ascribed/astro init writes only files that aren't there, and ${there.join(", ")} ${there.length === 1 ? "is" : "are"}. It wrote nothing. Move ${there.length === 1 ? "it" : "them"} aside to start from the starter.`,
    );
  }
  const files = starterFiles({ schema: schemaImport(root, project, build) });
  for (const [file, text] of files) {
    mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    writeFileSync(path.join(root, file), text);
  }
  return { build, written: [...files.keys()] };
}

/** The build the site shows: the one asked for, or the project's only one. */
export function chooseBuild(project: ProjectInfo, asked: string | undefined): string {
  const builds = project.builds.length === 0 ? ["site"] : project.builds;
  if (asked !== undefined) {
    if (builds.includes(asked)) return asked;
    throw new Error(
      `@ascribed/astro init: ${project.configPath} has no build "${asked}". Its builds are ${quoted(builds)}.`,
    );
  }
  const [only] = builds;
  if (builds.length === 1 && only !== undefined) return only;
  throw new Error(
    `@ascribed/astro init: ${project.configPath} has several builds, ${quoted(builds)}. Say which the site shows with --build.`,
  );
}

/**
 * The path `src/content.config.ts` imports the schema from: the generated file
 * in the build's site output when the project is inside the Astro root, and
 * the integration's copy inside the root when it's outside, where TypeScript
 * can't find `astro/zod` from the original.
 */
export function schemaImport(root: string, project: ProjectInfo, build: string): string {
  const schema = path.join(project.siteRoot(build), "_ascribe", "schema.ts");
  const fromRoot = path.relative(root, schema);
  const inside = !fromRoot.startsWith("..") && !path.isAbsolute(fromRoot);
  const target = inside ? schema : path.join(root, ".astro/integrations/_ascribed_astro/schema.ts");
  return path.relative(path.join(root, "src"), target).split(path.sep).join("/");
}

/** The starter's files, by path from the Astro root, with the schema's path filled in. */
export function starterFiles(options: { schema: string }): Map<string, string> {
  const files = new Map<string, string>();
  for (const file of STARTER_FILES) {
    let text = readFileSync(path.join(STARTER, file), "utf8");
    if (file === "src/content.config.ts")
      text = fill(
        text,
        SCHEMA_IMPORT,
        (_, start: string) => `${start}${JSON.stringify(options.schema)};`,
      );
    files.set(file, text);
  }
  return files;
}

/** `text` with the one match of `pattern` replaced. The starter's files are the package's own, so a miss is its bug. */
function fill(
  text: string,
  pattern: RegExp,
  replacement: (match: string, ...groups: string[]) => string,
): string {
  if (!pattern.test(text))
    throw new Error(`@ascribed/astro init: the starter has no ${pattern.source}`);
  return text.replace(pattern, replacement);
}

function quoted(names: readonly string[]): string {
  return names.map((name) => `"${name}"`).join(", ");
}
