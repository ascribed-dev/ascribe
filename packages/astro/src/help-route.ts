// A help entry's own addresses: one entry of a line's `_ascribe/help.json`, as
// JSON, and its section as Markdown (`help.ts`). The integration adds this
// route once for each folder of entries and each format, `<path>[id].json`
// and `<path>[id].md` for the current line and `<line>/go/[id].json` and
// `<line>/go/[id].md` under the base path for each older one; which folder
// and format this is comes from the route's pattern. Any origin may read
// it, and the JSON names the Markdown as its alternate, as the build's
// headers say for a host.
import type { APIRoute, GetStaticPaths } from "astro";
import { base, permalinks } from "virtual:ascribe/site";
import { helpPaths, type HelpAnswer } from "./help.js";

export const getStaticPaths = (({ routePattern }) =>
  helpPaths(
    permalinks.map((source) => source.siteRoot),
    base,
    routePattern,
  )) satisfies GetStaticPaths;

export const GET: APIRoute = ({ props }) => {
  const { body, type, alternate } = props as HelpAnswer;
  return new Response(body, {
    headers: {
      "Content-Type": `${type}; charset=utf-8`,
      "Access-Control-Allow-Origin": "*",
      ...(alternate === undefined
        ? {}
        : { Link: `<${alternate}>; rel="alternate"; type="text/markdown"` }),
    },
  });
};
