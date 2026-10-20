---
type: how-to
title: Serve the Markdown versions
description: Configure your host to serve the Markdown versions of an Astro site's pages and llms.txt as the Web Documentation Delivery Spec asks, with their type, their response headers, and content negotiation.
---

The outputs for agents are static files, so any host serves them, but a few things the [Web Documentation Delivery Spec](https://agentdocsspec.com/spec/web/) asks are the server's, not the build's. This guide sets them on your host, once the site [publishes for AI agents](agents.md).

## What your host does
@id: what-your-host-does
@available: ascribe 0.3.0

### 1. Check the files' type

Serve the `.md` files as `text/markdown; charset=utf-8`, and `llms.txt` as `text/plain; charset=utf-8`, as `astro dev` does. Check what your host sends, and set the header in its configuration if it's something else:

```sh
curl -I https://docs.example.com/docs/guides/setup.md
```

### 2. Send each Markdown version's headers

A Markdown version has no head, so the rule its page's head gives a crawler is a response header: `X-Robots-Tag` for a page that isn't listed, and a canonical `Link` to its page. Each build lists them in `_ascribe/headers.json` ([response headers](reference.md#response-headers)).

**On Netlify,** set `headers = "netlify"` in `[consumer]`, and `astro build` writes them into the built site's `_headers`.

**On Cloudflare Pages,** which reads a `_headers` file in the same form as Netlify's, set `headers = "netlify"` too. Cloudflare takes at most 100 rules, and there's one for each Markdown version, so a larger site gives the rules to a Worker instead.

**On nginx,** write a `map` for each header after the build, from a Markdown version's path to the header's value, for the `http` block to include:

```js
// After `astro build`: headers.conf, a map for each header Ascribe's rules give.
import { readFileSync, writeFileSync } from "node:fs";
const { headers } = JSON.parse(
  readFileSync(".ascribe/build/site/site/_ascribe/headers.json", "utf8"),
);
const names = [...new Set(headers.flatMap((rule) => Object.keys(rule.headers)))];
const maps = names.map((name) => {
  const variable = `$ascribe_${name.toLowerCase().replaceAll("-", "_")}`;
  const lines = headers
    .filter((rule) => name in rule.headers)
    .map((rule) => `  '${rule.path}' '${rule.headers[name]}';`);
  return `map $uri ${variable} {\n  default "";\n${lines.join("\n")}\n}\n`;
});
writeFileSync("headers.conf", maps.join(""));
```

The server then sends each header from its variable, and nginx leaves out a header whose value is empty, so every other file is answered as before:

```nginx
http {
  include headers.conf;

  server {
    add_header Link $ascribe_link;
    add_header X-Robots-Tag $ascribe_x_robots_tag;
  }
}
```

A `location` with `add_header` lines of its own, such as one that gives `.md` files their type, doesn't inherit the server's, so it repeats these two.

**On another host,** give each path in `headers.json` its headers in the host's own configuration.

### 3. Answer content negotiation

An agent may ask for a page with `Accept: text/markdown`. Answering it with the page's `.md` needs a rule on the server, such as an edge function or middleware that rewrites to `<path>.md` when that header asks for Markdown. Without it, agents still find the `.md` URLs through `llms.txt` and the pointer.

On Netlify, an edge function does it: this site's is [`markdown.ts`]({repo}/blob/main/site/netlify/edge-functions/markdown.ts). Whatever answers this way passes on the `.md` file's headers, `X-Robots-Tag` and `Link`: the answer is at the page's own address, where no rule for the `.md` path applies, and on Netlify a function's response gets none of `_headers` at all.

### 4. Answer a missing page with 404

A missing page should answer 404, not 200 with a page saying so, and the `.md` files and `llms.txt` should be cached no longer than the pages.

### 5. Let agents past bot protection

A rule that challenges unfamiliar clients blocks agents too; let them read `llms.txt` and the `.md` files.

## See also

- [Response headers](reference.md#response-headers), in the `@ascribed/astro` reference: what `astro dev` sends, and how the build treats a site's own `_headers`.
- [`[consumer] headers`](../content-model/output.md#consumer): the host whose format the headers are written in.
- `ascribe report agents`, which checks a published site against the spec: [the report](../checks/report.md).
