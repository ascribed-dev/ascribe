---
type: how-to
title: Deploy the site
description: "Build an Astro site of Ascribe docs in CI and publish it on GitHub Pages or Netlify, with what each host, Cloudflare Pages, and Vercel need: the base path, the trailing slash, and the host's files."
---

`astro build` writes the site to `dist/`, a folder of static files any host serves. This guide builds it in CI on each push to `main` and publishes it, with a GitHub Actions workflow for GitHub Pages and one for Netlify, and says what each host needs from `ascribe.toml` first.

## Before you start

- A site [set up with `@ascribed/astro`](index.md) that `npx astro build` builds on your computer, with its `package-lock.json` committed.
- The address the site will have. Ascribe writes every link from `[consumer]`'s `site` and `base-path`, so they're the deployed site's, not your computer's.

## What the host needs

A host serves `dist/` at the root of the address it gives the site, so where the docs are and what the host reads decide `[consumer]`:

| Host | The docs' address | `_headers` and `_redirects` | A page at `x`, without a slash |
|---|---|---|---|
| GitHub Pages | `https://<owner>.github.io/<repository>/`, or a domain of your own | Not read | Served from `x.html` |
| Netlify | A domain of your own, or `<name>.netlify.app` | Read, with `host = "netlify"` | Served from `x.html` |
| Cloudflare Pages | A domain of your own, or `<name>.pages.dev` | Read, with `host = "netlify"`, up to 100 header rules | Served from `x.html` |
| Vercel | A domain of your own, or `<name>.vercel.app` | Not read | Served from `x.html` only with `cleanUrls` |

- **The base path is where `dist/` is served.** With [`build`](reference.md#options), the integration gives Astro's `base` the project's `base-path`, and `astro build` writes a site that works only there. GitHub Pages serves a repository's site under its name, so `base-path` is `/<repository>/`. A host that serves `dist/` at the domain's root takes the docs at `base-path = "/"`, or under a path of a site with [a home page in front of the docs](more-than-docs.md#a-home-page-in-front-of-the-docs), whose `base` stays `/`.
- **The trailing slash.** With `trailing-slash = "never"`, the integration writes each page as `x.html`, which every host above serves at `x`; on Vercel, set `"cleanUrls": true` in `vercel.json`. With `"always"`, each page is `x/index.html`, which any host serves at `x/`.
- **The host's files.** With `host = "netlify"`, the build writes `_headers` and `_redirects` at the root of `dist/` ([the host's files](../publishing/site.md#the-hosts-files)): the response headers that keep the Markdown versions of pages that aren't listed out of search, and the redirects from addresses [`ascribe.addresses`](../publishing/redirects.md) recorded a move from. Netlify and Cloudflare Pages read them. They're written only when Astro's `base` is `/`, since the host reads them only at its root; with another base, `astro build` prints their lines instead. A host that reads neither, such as GitHub Pages, sends no headers and forwards no moved address: [Serve the Markdown versions](serve-markdown.md) says what each needs.
- **`robots.txt`** is read only at the host's root, so it's written only when Astro's `base` is `/` ([`robots.txt`](../publishing/site.md#robotstxt)).

Permalinks work on every host: each permalink's address is a page that forwards to its section, and on Netlify a redirect does it in one step ([permalinks on Netlify](../publishing/redirects.md#permalinks-on-netlify)).

## Deploy to GitHub Pages

### 1. Give the docs GitHub Pages' address

For a repository's site, `site` is the account's address and `base-path` the repository's name:

```toml
# ascribe.toml
[consumer]
profile = "astro"
site = "https://loom-dev.github.io"
base-path = "/loom/"
trailing-slash = "never"
```

With a domain of your own, set `site` to it and `base-path` to `"/"`, and add the domain in the repository's **Settings** → **Pages**.

### 2. Publish from Actions

In the repository's **Settings** → **Pages**, set **Source** to **GitHub Actions**. A site published from Actions is served as the workflow uploads it, so the files under `_ascribe/` are served as they are.

### 3. Add the workflow
@id: github-pages-workflow

Add `.github/workflows/deploy.yml`. It builds the site on each push to `main`, uploads `dist/`, and publishes it:

```yaml phrases=true
name: Deploy

on:
  push:
    branches: [main]
  workflow_dispatch:

# One deployment at a time, and one that's started is finished.
concurrency:
  group: pages
  cancel-in-progress: false

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: {node}
      - run: npm ci
      - run: npx astro build
      - uses: actions/upload-pages-artifact@v4
        with:
          path: dist

  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
      url: ${{ steps.deployment.outputs.page_url }}
    steps:
      - id: deployment
        uses: actions/deploy-pages@v4
```

`astro build` fails when `ascribe build` reports an error, so a page with an error is never deployed. With pnpm, run `pnpm install --frozen-lockfile` and `pnpm exec astro build` after `pnpm/action-setup`. With the Astro project in a folder of the repository, give the `run` steps its `working-directory`, and `upload-pages-artifact` its `dist/`.

### 4. Push
@id: github-pages-push

Push to `main`, and the workflow's **deploy** job links to the site when it's published.

## Deploy to Netlify

### 1. Write Netlify's files

Set `host = "netlify"` in `[consumer]`, so the build writes `_headers` and `_redirects`, and give the docs the root of Netlify's address, or put them under a path with [a home page in front of them](more-than-docs.md#a-home-page-in-front-of-the-docs):

```toml
# ascribe.toml
[consumer]
profile = "astro"
site = "https://docs.loom.example"
base-path = "/"
trailing-slash = "never"
host = "netlify"
```

Cloudflare Pages reads the same files, so this step is the same there.

### 2. Give the workflow Netlify's keys

Create the site in Netlify, without connecting a repository, since the workflow deploys it. Then add two secrets in the repository's **Settings** → **Secrets and variables** → **Actions**:

- `NETLIFY_AUTH_TOKEN`: a personal access token, from your Netlify user settings.
- `NETLIFY_SITE_ID`: the site's **Project ID**, from its configuration.

### 3. Add the workflow
@id: netlify-workflow

Add `.github/workflows/deploy.yml`. It builds the site on each push to `main` and deploys `dist/` as the production site:

```yaml phrases=true
name: Deploy

on:
  push:
    branches: [main]
  workflow_dispatch:

concurrency:
  group: netlify
  cancel-in-progress: false

permissions:
  contents: read

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
      - uses: actions/setup-node@949feb2413d6458794dcd2491c4babbbce0c15c1 # v7.1.0
        with:
          node-version: {node}
      - run: npm ci
      - run: npx astro build
      # `--no-build` deploys dist/ as it was built above.
      - run: npx --yes netlify-cli@27.11.0 deploy --prod --no-build --dir dist
        env:
          NETLIFY_AUTH_TOKEN: ${{ secrets.NETLIFY_AUTH_TOKEN }}
          NETLIFY_SITE_ID: ${{ secrets.NETLIFY_SITE_ID }}
```

To have Netlify build the site instead, connect the repository in Netlify, and give it the build in a `netlify.toml` beside `package.json`, with no workflow:

```toml
[build]
  command = "npm ci && npx astro build"
  publish = "dist"
```

### 4. Push
@id: netlify-push

Push to `main`. The deploy step prints the site's address, and Netlify's **Deploys** lists the deploy.

## Check the deployed site

Open the site at its address, and follow a link or two: a link that leads to the host's 404 page means `site` or `base-path` isn't where the host serves the site. With `host = "netlify"`, check that a Markdown version gets its headers, as `astro dev` sends them:

```sh
curl -sI https://docs.loom.example/guides/setup.md | grep -i '^link'
```

Each pull request is checked before it reaches `main` by [Checks in CI](../checks/ci.md), which fails on warnings too, and on an address the docs published that a change lost.

## See also

- [Astro](index.md#ways-to-publish): the ways to publish, and setting up the site.
- [Serve the Markdown versions](serve-markdown.md): what a host does for agents, on Netlify, Cloudflare Pages, and nginx.
- [Keep the addresses you've published](../publishing/redirects.md): the redirects the build writes when a page moves.
- [Search engines and crawlers](../publishing/index.md): what the deployed site tells search engines and AI crawlers.
