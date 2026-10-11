// The starter, `npx @ascribed/astro init`: writes into an Astro project, once,
// the files every site built with Ascribe writes again: the collection, the
// route, a layout that loads the elements, and the sidebar's list. From then
// on they're the site's own, as `ascribe templates eject` leaves a project's
// templates; `init` never writes over a file.
//
// The files are in `starter/`, as `examples/astro-site/src` has them; the
// example's are what `init` writes for its project, which a test holds. Two
// things differ from one project to the next: the schema's path, and the
// sidebar's pages, which `init` lists from a build.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { findBinary } from "./binary.js";
import { navFor, type NavGroup } from "./nav.js";
import { readProject, type ProjectInfo } from "./project.js";
import { sitePages } from "./projects.js";
import { runBuild } from "./run.js";

/** The files `init` writes, by path from the Astro root. */
export const STARTER_FILES = [
  "src/content.config.ts",
  "src/pages/[...slug].astro",
  "src/layouts/Docs.astro",
  "src/nav.ts",
] as const;

/** The starter's files, in the package beside `src/` and `dist/`. */
const STARTER = fileURLToPath(new URL("../starter/", import.meta.url));

/** The import of the schema in the starter's `content.config.ts`, whose path `init` replaces. */
const SCHEMA_IMPORT = /^(import \{ schema \} from )"[^"]*";$/m;

/** The list in the starter's `nav.ts`, which `init` replaces. */
const NAV_LIST = /^export const nav: NavGroup\[\] = \[\n[\s\S]*?^\];$/m;

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
  /** The build it listed the pages of, which the integration's `build` option names. */
  build: string;
  /** The files it wrote, by path from the root. */
  written: string[];
}

/**
 * Writes the starter's files into the Astro project at `root`, with the
 * sidebar's list from a build of the project. Writes nothing, and throws, when
 * any of them is there already, when the build can't be chosen, or when the
 * build fails.
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
  await runBuild({
    binary: findBinary({ root }),
    configPath: project.configPath,
    build,
    cwd: project.dir,
  });
  const files = starterFiles({
    schema: schemaImport(root, project, build),
    nav: navFor(sitePages(project, build)),
  });
  for (const [file, text] of files) {
    mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    writeFileSync(path.join(root, file), text);
  }
  return { build, written: [...files.keys()] };
}

/** The build `init` lists: the one asked for, or the project's only one. */
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

/** The starter's files, by path from the Astro root, with the schema's path and the sidebar's list filled in. */
export function starterFiles(options: {
  schema: string;
  nav: readonly NavGroup[];
}): Map<string, string> {
  const files = new Map<string, string>();
  for (const file of STARTER_FILES) {
    let text = readFileSync(path.join(STARTER, file), "utf8");
    if (file === "src/content.config.ts")
      text = fill(
        text,
        SCHEMA_IMPORT,
        (_, start: string) => `${start}${JSON.stringify(options.schema)};`,
      );
    if (file === "src/nav.ts") text = fill(text, NAV_LIST, () => navList(options.nav));
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

/** The sidebar's list as `nav.ts` declares it, laid out as Prettier would, at 100 columns. */
function navList(nav: readonly NavGroup[]): string {
  const lines = ["export const nav: NavGroup[] = ["];
  for (const group of nav) {
    const pages = `[${group.pages.map((page) => JSON.stringify(page)).join(", ")}]`;
    const line = `  { label: ${JSON.stringify(group.label)}, pages: ${pages} },`;
    if (line.length <= 100) {
      lines.push(line);
      continue;
    }
    lines.push("  {", `    label: ${JSON.stringify(group.label)},`);
    if (`    pages: ${pages},`.length <= 100) lines.push(`    pages: ${pages},`);
    else
      lines.push(
        "    pages: [",
        ...group.pages.map((page) => `      ${JSON.stringify(page)},`),
        "    ],",
      );
    lines.push("  },");
  }
  lines.push("];");
  return lines.join("\n");
}

function quoted(names: readonly string[]): string {
  return names.map((name) => `"${name}"`).join(", ");
}
