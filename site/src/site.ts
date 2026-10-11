// What the layout says about the site itself.

export const name = "Ascribe";

/** The repository, for the header link and each page's edit link. */
export const repo = "https://github.com/ascribed-dev/ascribe";

/** Where the docs pages' sources are in the repository. */
export const contentDir = "docs/content";

/**
 * The docs' route: `[consumer] base-path` in ../docs/ascribe.toml less Astro's
 * `base`, and the folder of their route in src/pages/. test/projects.test.ts
 * checks it against the base path.
 */
export const docsPath = "docs/";
