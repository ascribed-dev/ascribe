// The module the integration provides to `content.ts` (through Vite), so the
// collection helper needs nothing but the integration's own options.
declare module "virtual:tessera/site" {
  /** The absolute path of the configured build's site output root. */
  export const siteRoot: string;
}
