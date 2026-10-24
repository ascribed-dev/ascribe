// A help entry's own address: one entry of a line's `_ascribe/help.json`, as
// JSON (`help.ts`). The integration adds this route once for each folder of
// entries, `<path>[id].json` for the current line and `<line>/go/[id].json`
// under the base path for each older one; which folder this is comes from
// the route's pattern. Any origin may read it, as the build's headers say
// for a host.
import type { APIRoute, GetStaticPaths } from "astro";
import { base, permalinks } from "virtual:ascribe/site";
import { helpPaths } from "./help.js";

export const getStaticPaths = (({ routePattern }) =>
  helpPaths(
    permalinks.map((source) => source.siteRoot),
    base,
    routePattern,
  )) satisfies GetStaticPaths;

export const GET: APIRoute = ({ props }) =>
  new Response((props as { body: string }).body, {
    headers: {
      "Content-Type": "application/json; charset=utf-8",
      "Access-Control-Allow-Origin": "*",
    },
  });
