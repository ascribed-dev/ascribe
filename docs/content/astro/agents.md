---
type: how-to
title: Publish for AI agents
description: "Publish `llms.txt` and a Markdown version of each page with an Astro site, so an AI agent reading the docs gets Markdown and an index to choose pages from."
permalink: publish-for-agents
---

An AI agent reading your docs does better with Markdown than with HTML, and with an index to choose pages from. This guide turns on the outputs for agents, which each build then publishes with the site, as the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) lays them out.

## Before you start

- A site [set up with `@ascribed/astro`](index.md).

## Turn on the outputs for agents
@id: publish-for-ai-agents

@steps
1. In `ascribe.toml`'s `[consumer]`, set `site`, the site's address. With it, the outputs for agents are on; the example says so with `agents = true`, which is the default once `site` is set:

   @snippet: code:examples/astro-site/ascribe.toml#agents

   `site` is required, because everything an agent reads links by absolute URL. A site that publishes nothing for agents sets `agents = false`.

2. Write each page's `description`, the field its type marks `role = "description"`, as one sentence saying what the page is for. It's what an agent picks a page by.
3. Run `ascribe check`. It advises on a page whose description is too long for its line in `llms.txt`, and on an `llms.txt` that's still too long when split.
4. Build the site, with `npx astro dev` or `npx astro build`. Each build publishes:

   - **`llms.txt`** at the base path: every page, in the sections and order of the project's [navigation](../navigation/index.md), as its title, a link to its Markdown, and its description. Past 50,000 characters it's split into one `llms.txt` per top section, which the root file links to.
   - **Each page's Markdown** at its URL with `.md`: `/docs/guides/setup` is also `/docs/guides/setup.md`. It opens with a line pointing at `llms.txt`, and after the title says what the page is for, its type and its description, as `> How-to · Set up sync between two machines.`, so an agent that lands on the page from a search sees what the site shows under the title. Its links to other pages lead to their Markdown too, and a link to a heading to that heading's anchor there, so an agent that follows a link stays in Markdown.
   - **A pointer** at the top of each page to both, which the elements' stylesheet hides from sight but not from an agent reading the page.

   With [`[releases]`](../content-model/content.md#71-releasestarget), `llms.txt`'s summary and each `.md`'s opening line also say which release the build describes, `This documentation describes Quill 2.4. Anything marked planned isn't in that release yet.`, so an agent helping someone on that release knows that a page without an availability mark applies to them.

5. Open `llms.txt` at the base path, `http://localhost:4321/docs/llms.txt` in `astro dev`, and a page's `.md`.
6. [Serve the Markdown versions](serve-markdown.md): what your host sends with them is the rest of the spec.

Nothing is written at the host's root for a project under a path. With docs at `/docs/`, an agent reaches `/docs/llms.txt` through each page's pointer, and the spec's checker looks there as well as at `/llms.txt`. A site that wants a file at the root, such as one whose docs are somewhere else, puts its own in Astro's `public/`.

## See also

- [`[consumer] agents`](../content-model/output.md#consumer): how `llms.txt` is put together.
- [Serve the Markdown versions](serve-markdown.md): the files' type, their headers, and content negotiation.
- [What the integration does](reference.md#what-the-integration-does): the outputs for agents in `astro dev` and the built site.
