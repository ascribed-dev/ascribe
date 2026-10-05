// What the layout says about the site itself.

export const name = "Ascribe";

/** The repository, for the header link and each page's edit link. */
export const repo = "https://github.com/ascribed-dev/ascribe";

/** Where the pages' sources are in the repository. */
export const contentDir = "docs/content";

/**
 * The page's source file, relative to the content root, from the path the
 * collection loaded it from: the site output mirrors the content root's paths.
 */
export function sourceFile(filePath: string): string {
  const match = /\.ascribe\/build\/[^/]+\/site\/(.+)$/.exec(filePath.replaceAll("\\", "/"));
  if (!match?.[1]) throw new Error(`not a page of the site output: ${filePath}`);
  return match[1];
}
