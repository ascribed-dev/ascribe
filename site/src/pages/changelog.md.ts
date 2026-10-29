// The changelog as Markdown, for agents: /changelog.md, the releases and
// entries src/pages/changelog.astro shows, the unreleased ones included, with
// its anchors and its links made absolute (`changelogMarkdown`). The page's
// head names it as its alternate, and the edge function answers /changelog/
// asked for as Markdown with it.
import type { APIRoute } from "astro";
import { getCollection, getEntry } from "astro:content";
import { changelogId, changelogMarkdown, kindUnder } from "../changelog.ts";
import { contentDir, docsPath, repo } from "../site.ts";

export const GET: APIRoute = async ({ site, url }) => {
  const entry = await getEntry("changelog", changelogId);
  if (entry?.rendered === undefined || entry.body === undefined) {
    throw new Error("the changelog collection has no rendered entry");
  }
  // Each docs page by its source, at its route, as the page points them.
  const base = import.meta.env.BASE_URL;
  const pages = new Map(
    (await getCollection("docs")).map((page) => [
      `${contentDir}/${page.data.source}`,
      `${base}${docsPath}${page.id === "index" ? "" : `${page.id}/`}`,
    ]),
  );
  const kind = kindUnder(entry.data.repository);
  const text = await changelogMarkdown(
    entry.body,
    entry.rendered.html,
    { pages, repo, kind },
    site ?? url,
  );
  return new Response(text, { headers: { "content-type": "text/markdown; charset=utf-8" } });
};
