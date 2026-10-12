// `astro dev`'s check of each page's head: a tag `Head.astro` writes that the
// head has twice, because the layout writes it too, is warned about once,
// naming the tag. Not an error: a site may have a reason. The integration adds
// this middleware in `astro dev` only.
import type { MiddlewareHandler } from "astro";
import { repeatedHeadTags } from "./head.js";

/** Where the integration leaves its logger's `warn` for this middleware, which Vite loads apart from it. */
export const WARN = Symbol.for("@ascribed/astro:head-warn");

/** The tags already warned about. */
const warned = new Set<string>();

/** The warnings for `html`, the page at `route`: one per repeated tag not yet warned about. */
export function headWarnings(html: string, route: string, seen: Set<string> = warned): string[] {
  return repeatedHeadTags(html)
    .filter((tag) => {
      if (seen.has(tag)) return false;
      seen.add(tag);
      return true;
    })
    .map(
      (tag) =>
        `${route}'s head has ${tag} twice. Head.astro writes it from the page's record, so the layout doesn't need to.`,
    );
}

export const onRequest: MiddlewareHandler = async (context, next) => {
  const response = await next();
  if (!(response.headers.get("content-type") ?? "").startsWith("text/html")) return response;
  const html = await response.text();
  const warn =
    ((globalThis as Record<symbol, unknown>)[WARN] as ((message: string) => void) | undefined) ??
    ((message: string) => console.warn(`[@ascribed/astro] ${message}`));
  for (const message of headWarnings(html, context.url.pathname)) warn(message);
  return new Response(html, response);
};
