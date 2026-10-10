// The module the integration provides to `content.ts` (through Vite), so the
// collection helper needs nothing but the integration's own options.
declare module "virtual:ascribe/site" {
  /** The absolute path of the configured build's site output root. */
  export const siteRoot: string;
  /** The route of the site's root: Astro's `base`, ending as the site's routes do. */
  export const home: string;
}
