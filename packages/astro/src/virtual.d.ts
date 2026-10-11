// The module the integration provides to `content.ts` (through Vite), so the
// collection helper needs nothing but the integration's own options.
declare module "virtual:ascribe/site" {
  /** The absolute path of the first entry's site output root. */
  export const siteRoot: string;
  /** The route of the site's root: Astro's `base`, ending as the site's routes do. */
  export const home: string;
  /** Each entry's site output root, by its name. Empty with one project and no name. */
  export const collections: Record<string, string>;
  /** Each entry's site output root and its project's root route, in the entries' order. */
  export const permalinks: { siteRoot: string; home: string }[];
  /** Each entry's site output root, and its project's base path and trailing slash, for its pages' routes. */
  export const routes: {
    siteRoot: string;
    basePath: string;
    trailingSlash: "always" | "never";
  }[];
}
