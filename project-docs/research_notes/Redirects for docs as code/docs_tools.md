# Redirects in documentation tools and static site generators

Scope: how authors declare and manage redirects in Docusaurus, MkDocs (mkdocs-redirects, Material, mike), Sphinx (sphinx-reredirects, sphinxext-rediraffe), Hugo/Docsy, Jekyll, Astro/Starlight, Next.js/Nextra, VitePress, Mintlify, ReadMe, GitBook, Read the Docs, Antora, Fern, plus mdBook (found along the way; it is the one tool with documented fragment redirects). Researched October 2026. Code blocks below are quoted as the source shows them; where a fetch returned only a paraphrase, the shape is described in prose instead.

## Where a redirect is declared, its syntax and forms, what is emitted, and the resulting status

### Takeaway
Two camps: a central map in the tool's config (Docusaurus, mkdocs-redirects, both Sphinx extensions, Astro, Next.js/Nextra, Mintlify, Fern, GitBook, mdBook) or the old path written into the moved page's own front matter (Hugo `aliases`, Jekyll `redirect_from`, Antora `page-aliases`). Hosted platforms (Read the Docs, ReadMe, GitBook site redirects) keep redirects in a dashboard, not the repository. Static generators mostly emit meta-refresh/JS stub pages, so no real HTTP 3xx. Only Hugo (through a template), Antora (through `redirect_facility`) and Astro (through adapters) also emit host config files. Hosted platforms and Next.js return real 301/302/307/308 responses.

### Cited Findings

**Docusaurus (`@docusaurus/plugin-client-redirects`, docs v3.10.2)**
- Declared centrally as plugin options in `docusaurus.config.js`. Options: `fromExtensions`, `toExtensions`, `redirects` (a list of `{ to, from }` where `from` may be an array, e.g. `to: '/docs/newDoc',` / `from: '/docs/oldDoc',`), and `createRedirects`, a function called for every route Docusaurus created that returns the "from" path(s) for it. — [Docusaurus docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-client-redirects)
- No wildcard or placeholder syntax is documented. Pattern-like rules are written as code in `createRedirects` (e.g. Apache Doris rewrites a version prefix: `if (existingPath.startsWith('/docs/3.0/')) { const withoutVersion = existingPath.replace('/docs/3.0/', '/docs/');`). — [Docusaurus docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-client-redirects); [Apache Doris docusaurus.config.js](https://apache.googlesource.com/doris-website/+/refs/heads/dev_0827/docusaurus.config.js)
- Emits extra HTML pages that redirect "with JavaScript". It is "always inactive in development" and "only active in production". The docs recommend server-side redirects "whenever possible". No HTTP status, since the redirect runs client-side. — [Docusaurus docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-client-redirects)

**MkDocs: `mkdocs-redirects` (also the plugin Material for MkDocs users rely on)**
- Declared centrally in `mkdocs.yml`. Keys are the old *Markdown source* path relative to `docs_dir` ("the filename the doc had, or would have had"), not the rendered URL. Values are a Markdown path or an external URL. — [mkdocs-redirects README](https://github.com/mkdocs/mkdocs-redirects)
```yaml
plugins:
  - redirects:
      redirect_maps:
        'old.md': 'new.md'
        'old/file.md': 'new/file.md'
        'some_file.md': 'http://external.url.com/foobar'
```
- Exact mappings only. No wildcards are documented. — [mkdocs-redirects README](https://github.com/mkdocs/mkdocs-redirects)
- Emits one `.html` stub per old file. With `use_directory_urls`, `'old/dir/doc_name.md': 'new/dir/doc_name.md'` creates `$site_dir/old/dir/doc_name/index.html` redirecting to `../../new/dir/doc_name/`. — [mkdocs-redirects README](https://github.com/mkdocs/mkdocs-redirects)
- The stub contains `<link rel="canonical" href="{url}">`, a JS redirect `<script>var anchor=window.location.hash.substr(1);location.href="{url}"+(anchor?"#"+anchor:"")</script>`, and `<meta http-equiv="refresh" content="0; url={url}">`. — [plugin.py](https://raw.githubusercontent.com/mkdocs/mkdocs-redirects/master/mkdocs_redirects/plugin.py)
- I found no Material-specific redirect feature. Searches only turned up the standalone plugin. — [PyPI mkdocs-redirects](https://pypi.org/project/mkdocs-redirects)

**Sphinx: `sphinx-reredirects` (documatt)**
- Declared centrally as a `redirects = { "<source>": "<target>" }` dict in `conf.py`. Sources are docnames, without a suffix. — [usage.rst](https://raw.githubusercontent.com/documatt/sphinx-reredirects/main/docs/usage.rst)
- Wildcards: "Wildcards for selecting existing source documents know only `*`, `?`, `[seq]` and `[!seq]` patterns" (Python `fnmatch`, where `**` has no recursive meaning). Placeholder: "Matched document in the source, is available in the target as `$source` or `${source}` placeholder." — [usage.rst](https://raw.githubusercontent.com/documatt/sphinx-reredirects/main/docs/usage.rst)
- Emits HTML pages "with meta refresh redirects", and works only for `html` and `dirhtml` builders. With `dirhtml` the "target must be a folder" (e.g. `"install/"`). — [GitHub README](https://github.com/documatt/sphinx-reredirects); [usage.rst](https://raw.githubusercontent.com/documatt/sphinx-reredirects/main/docs/usage.rst)

**Sphinx: `sphinxext-rediraffe` (wpilibsuite)**
- Declared centrally as `rediraffe_redirects`, either a dict in `conf.py` or a file name. The file format is one pair per line, `#` comments, and optionally quoted paths: — [rediraffe docs](https://sphinxext-rediraffe.readthedocs.io/en/latest/)
```
'another file.rst' index.rst
another2.rst "another file.rst"
```
```
rediraffe_redirects = {
    'another.rst': 'index.rst',
    'another2.rst': 'another.rst',
}
```
- Paths are *source files* (`.rst`/`.md`), not URLs. Exact only. A Jinja `rediraffe_template` receives `from_file`, `to_file`, `from_url`, `to_url` and `rel_url`. — [rediraffe docs](https://sphinxext-rediraffe.readthedocs.io/en/latest/)
- The default template redirects with JavaScript (`window.location.href`, preserving query and hash). A `<meta http-equiv="refresh">` inside `<noscript>` and a link are the fallbacks. — [rediraffe.py](https://raw.githubusercontent.com/wpilibsuite/sphinxext-rediraffe/main/sphinxext/rediraffe.py)

**Hugo (and Docsy, a Hugo theme)**
- Declared on the moved page itself, in its front matter, as `aliases`. YAML form: `- /old-url`, `- old-name`, `- ../old/path`. TOML form: — [Hugo URL management](https://gohugo.io/content-management/urls/#aliases)
```toml
aliases = ['/old-url', 'old-name', '../old/path']
```
- Paths are site-relative (leading `/`) or page-relative, and may use `../`. Hugo adds "content dimension prefixes such as language, version, or role" (e.g. `/old-url` on `content/examples/example-1.en.md` resolves to `/en/old-url/`). — [Hugo](https://gohugo.io/content-management/urls/#aliases)
- By default Hugo emits "a small HTML file for every alias" with `meta http-equiv="refresh"` and a canonical link, at `public/<alias>/index.html`. You can override `layouts/alias.html`. For server-side redirects, set `disableAliases = true` and write your own template over the `Aliases` page method to emit `_redirects` (Netlify/Cloudflare/GitLab Pages) or `.htaccess`. Hugo ships no built-in output format for this. — [Hugo](https://gohugo.io/content-management/urls/#aliases)
- Docsy's content guide documents no redirect feature of its own, so it relies on Hugo aliases. — [Docsy content docs](https://www.docsy.dev/docs/adding-content/content/)

**Jekyll (`jekyll-redirect-from`)**
- Declared in the moved page's front matter: `redirect_from` (a single path or a list). `redirect_to` turns a page into a redirect to an external URL. A trailing slash produces `folder/index.html`. — [jekyll-redirect-from README](https://github.com/jekyll/jekyll-redirect-from)
- Emits HTML pages with "an HTTP-REFRESH meta tag". "No `.htaccess` file, nginx conf, xml file, or anything else will be generated." It also writes a `redirects.json` by default "for automated testing or server-side redirects", which `redirect_from: json: false` turns off. The layout can be overridden in `_layouts/redirect.html`, with `page.redirect.from`, `page.redirect.to` and `page.redirect.to_js` available. — [README](https://github.com/jekyll/jekyll-redirect-from)

**Astro and Starlight**
- Declared centrally in the Astro config's `redirects` map (old path to new path). Added in `astro@2.9.0`, with external `http(s)` targets since v5.2.0. Dynamic routes work when both sides have the same params (`/blog/[...slug]` → `/articles/[...slug]`). The default is 301, and an object `{ status, destination }` gives e.g. 302. — [Astro routing docs](https://docs.astro.build/en/guides/routing/#redirects)
- Static output emits meta-refresh HTML, and "the status code isn't used by the server". Supported adapters write the redirects into the host's config instead. File-based routes outrank redirects, so a redirect from a path that still has a page doesn't apply. — [Astro routing docs](https://docs.astro.build/en/guides/routing/#redirects)
- Starlight's pages guide doesn't mention redirects, so Starlight sites use Astro's `redirects`. — [Starlight pages guide](https://starlight.astro.build/guides/pages/)

**Next.js / Nextra**
- Next.js: declared centrally in `next.config.js` as `redirects()` returning `{ source, destination, permanent }`. Matching uses path-to-regexp: `:slug`, the modifiers `*`, `+` and `?`, regex `:slug(\\d{1,})`, and `has`/`missing` matching on header, cookie, host or query. `permanent: true` gives 308 and `false` gives 307, or set `statusCode` explicitly. "Redirects are checked before the filesystem." Added in v9.5.0. — [Next.js redirects](https://nextjs.org/docs/app/api-reference/config/next-config-js/redirects)
```js
{
  source: '/old-blog/:path*',
  destination: '/blog/:path*',
  permanent: false
}
```
- Nextra has no redirect feature of its own. Nextra sites (including Nextra's own) pass `redirects` through the Next.js config wrapped by `withNextra`. — [Nextra commit](https://redirect.github.com/shuding/nextra/commit/ad4823d) (secondary source; no official Nextra page found)

**VitePress**
- No built-in redirect feature. The routing guide only mentions a client-side redirect from `.html` paths to extensionless paths when `cleanUrls` is on. `rewrites` (e.g. `'packages/:pkg/src/:slug*': ':pkg/:slug*'`) remaps source files to output paths and doesn't redirect old URLs. — [VitePress routing](https://vitepress.dev/guide/routing). A maintainer suggested `rewrites` as a workaround in a permalinks issue. — [vitepress#285](https://github.com/vuejs/vitepress/issues/285)

**Mintlify**
- Declared centrally: a top-level `redirects` array of `{ source, destination }` objects in `docs.json`. The default is permanent (308), and `permanent: false` gives 307. The wildcard is `*` with named params like `/beta/:slug*` → `/v2/:slug*`, or a partial prefix like `/old/article-*`. "`*` is the only wildcard character". One wildcard per source, and no regex. Sources can't contain `#anchor` or `?query`, but destinations may contain an anchor. — [Mintlify redirects](https://www.mintlify.com/docs/create/redirects)

**Fern**
- Declared centrally in `docs.yml` `redirects`, either inline or in separate files, but not both. Each file needs "a top-level `redirects` list", and files are evaluated in order. — [Fern redirects](https://buildwithfern.com/learn/docs/seo/redirects)
```yaml
redirects:
  - source: /old-path
    destination: /new-path
  - source: /old-folder/:slug*
    destination: /new-folder/:slug*
```
- Params: `:slug` (one segment), `:slug*` (zero or more), `:slug+` (one or more), regex `:slug(regex)`, and an unnamed `(regex)` replayed as `:0`. `?` is unsupported. First match wins. Trailing slashes are normalized and matching is case-sensitive. The default is 308, and `permanent: false` gives 307. A redirected page's `.md`/`.mdx` "twin" URL also 308s to the destination's twin. — [Fern redirects](https://buildwithfern.com/learn/docs/seo/redirects); [Fern site settings](https://buildwithfern.com/learn/docs/configuration/site-level-settings)

**GitBook**
- Two declared layers plus one automatic layer. Space-level redirects go in the repo's `.gitbook.yaml` (Git Sync) and "only apply within that space": — [GitBook content configuration](https://gitbook.com/docs/getting-started/git-sync/content-configuration)
```yaml
redirects:
  previous/page: new-folder/page.md
```
- Site-level redirects are managed in the app (Site Settings > Redirects). The destination can be a page *reference*, which survives later moves, or a path. A trailing `*` wildcard can optionally append the matched text. Each redirect is 307 or 308, but 308 is sent "only to unauthenticated views of live public content". There are draft redirects and CSV import (max 500 rows, and on duplicate sources only the first row is used). URLs resolve case-insensitively. — [GitBook site redirects](https://gitbook.com/docs/publish/site-redirects)
- Lookup order: automatic redirects, then `.gitbook.yaml`, then site-level. — [GitBook site redirects](https://gitbook.com/docs/publish/site-redirects)

**ReadMe**
- Declared in the dashboard under "Settings > Errors & Redirects", one per line as `oldurl -> newurl`. Origins are relative, "cannot include a hash, and cannot match an existing page". JS-style regex must match the full URL, with `$1`, `$2` back-references and `\S+` as a catch-all. Top-to-bottom, first match wins. Redirects fire only on 404s. The status code isn't documented. — [ReadMe error pages](https://docs.readme.com/main/docs/error-pages)

**Read the Docs**
- Declared in the project dashboard (Admin > Redirects). No config-file option is documented. Types: Page Redirect (applies across all versions, so `From URL` omits `/<lang>/<version>`), Exact Redirect (the full path including language and version), and Clean URL to HTML / HTML to clean URL. A wildcard `*` may appear only at the end of `From URL`, reused as `:splat`. Examples: "From URL: /api/*" / "To URL: /api/v1/:splat", and "From URL: /dev/*" / "To URL: /en/latest/:splat". — [RTD user-defined redirects](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- By default redirects apply only when the page doesn't exist, and "Forced redirects allow you to apply redirects on existing pages". First match wins and the order can be changed in the dashboard. Community plans are "limited to 100 redirects per project". — [RTD](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- Built-in redirects: `/` goes to the default version, `/<lang>/` to the default version of that language, and `/page/<path>` to the default version of a page. — [RTD](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- The current page lists no "Prefix" type. RTD's redirects design doc proposed "Use `*` and `:splat` for wildcards" and "Merge prefix redirects with exact redirects". — [RTD redirects design doc](https://github.com/readthedocs/readthedocs.org/blob/6747f91a440937e2b5088e9168ad1b87d6611129/docs/dev/design/redirects.rst)

**Antora**
- Declared on the moved page itself, as a header attribute on the *target* page listing the old resource IDs: `:page-aliases: source-page-filename.adoc, version@component:module:source-page-filename.adoc`. Coordinates the alias omits are filled in from the target page. Only pages can be aliased (not partials, examples, images or attachments). An alias also works as an xref target, so "references to a deleted, renamed, or moved page don't need updating". — [Antora page-aliases](https://docs.antora.org/antora/latest/page/page-aliases/)
- Emission depends on the playbook's `urls.redirect_facility`: `static` (default; meta-refresh HTML at the old URL), `netlify` (`_redirects`), `gitlab` (`_redirects` without the trailing `!`), `httpd` (`.htaccess` with `Redirect 301`), `nginx` (`.etc/nginx/rewrite.conf`, which you must wire up yourself), or `disabled`. Page aliases and start pages use 301, and latest-version aliases use 302. No splat rules are documented. — [Antora redirect facility](https://docs.antora.org/antora/latest/playbook/urls-redirect-facility/)

**mdBook**
- Declared centrally in `book.toml` under `[output.html.redirect]`. Keys are output paths absolute from the build directory, and values are any URI. — [mdBook renderers config](https://rust-lang.github.io/mdBook/format/configuration/renderers.html)
```toml
[output.html.redirect]
"/appendices/bibliography.html" = "https://rustc-dev-guide.rust-lang.org/appendix/bibliography.html"
"/other-installation-methods.html" = "../infra/other-installation-methods.html"

# Fragment redirects also work.
"/some-existing-page.html#old-fragment" = "some-existing-page.html#new-fragment"
```

### Inferences
- Where the key lives decides what an old path *means*. mkdocs-redirects and rediraffe key on source file paths and Antora on resource IDs, so the tool can resolve them against its own model and check them. Mintlify, Fern, ReadMe, RTD and Next.js key on URLs, which allows patterns but makes the tool's model harder to check against.
- Front-matter declaration (Hugo, Jekyll, Antora) keeps the old path next to the page that replaced it. When the page moves again its aliases move with it, but it cannot express a redirect to a page that was deleted outright, or a pattern.
- Patterns (`:splat`, `:slug*`, fnmatch, regex) only appear in tools that either serve requests (hosted platforms, Next.js) or expand patterns against known source docs at build time (sphinx-reredirects). A purely static stub generator can't serve a wildcard.

### Gaps
- Astro's and Mintlify's exact config examples were only returned paraphrased by the fetch tool, so they aren't quoted verbatim here.
- ReadMe's and Read the Docs' HTTP status codes aren't stated on their pages.
- Which hosts Astro's adapters support for redirects wasn't listed in the fetched section.
- The removal date of RTD's former "Prefix" redirect type wasn't confirmed; only the design proposal and its absence from current docs were.

## Which tools validate redirects, and which fail the build

### Takeaway
Validation ranges from none (Jekyll, Hugo, VitePress, Mintlify's config) to strict. Antora *throws* on an alias that names an existing page, duplicates, or self-reference. Rediraffe errors on loops and duplicate sources, flattens chains, and sets a failing exit status for a missing target or a still-existing source. Docusaurus throws on a missing target and drops (with a report) redirects from existing pages and duplicates. Fern errors on loops and warns on published URLs that lost their page with no redirect. mkdocs-redirects only warns, which fails the build under `strict: true`.

### Cited Findings
- **Antora** `registerPageAlias` throws three errors: `Page cannot define alias that references itself: …`, `Page alias cannot reference an existing page: …` (naming both the source and the existing page), and `Duplicate alias: …`. An invalid alias spec returns silently, with the code comment `// QUESTION should we throw an error if alias is invalid?`. — [content-catalog.js](https://gitlab.com/antora/antora/-/raw/main/packages/content-classifier/lib/content-catalog.js)
- **Docusaurus** throws "You are trying to create client-side redirections to invalid paths." / "These paths are redirected to but do not exist:", and also throws on trailing-slash mismatches with `trailingSlash`. Redirects whose `from` is an existing route are filtered out and reported ("some redirects would override existing paths, and will be ignored:"). Duplicate `from`s are reported ("It is not possible to redirect the same pathname to multiple destinations:") and the first is kept. The severity of both reports follows `onDuplicateRoutes`. — [collectRedirects.ts](https://raw.githubusercontent.com/facebook/docusaurus/main/packages/docusaurus-plugin-client-redirects/src/collectRedirects.ts)
- **Rediraffe**: circular redirects log "rediraffe: A circular redirect exists. Links involved: …" and then raise `ExtensionError`. A source redirected twice ("… is redirected multiple times in the rediraffe_redirects file!") and invalid lines fail parsing with `ExtensionError`. "(broken) {from} redirects to {to} but {build_from} already exists!" and "(broken) … but {build_to} does not exist!" are warnings that skip the redirect and set exit status 1. Chains are silently flattened so "The end user will never experience more than 1 redirection." — [rediraffe.py](https://raw.githubusercontent.com/wpilibsuite/sphinxext-rediraffe/main/sphinxext/rediraffe.py); [rediraffe docs](https://sphinxext-rediraffe.readthedocs.io/en/latest/)
- **mkdocs-redirects** warns `"Redirect target '%s' does not exist!"` and `"redirects plugin: '%s' is not a valid markdown file!"`. The README notes this is useful with `strict: true`, which turns warnings into a failed build. — [plugin.py](https://raw.githubusercontent.com/mkdocs/mkdocs-redirects/master/mkdocs_redirects/plugin.py); [README](https://github.com/mkdocs/mkdocs-redirects)
- **sphinx-reredirects**: "For an existing document, the document's .html file will be overwritten with the redirect file." No warning is documented. "Checking redirects to another page in the same documentation is not supported yet" (linkcheck). — [usage.rst](https://raw.githubusercontent.com/documatt/sphinx-reredirects/main/docs/usage.rst)
- **Fern**: `check.rules.no-circular-redirects` (default `error`). `check.rules.missing-redirects` (default `warn`) "flags previously published URLs that no longer resolve" without a redirect, and "requires authentication and is skipped on first publish". Errors also cover source equal to destination, a missing redirects file, and a file without a top-level `redirects` key. — [Fern site settings](https://buildwithfern.com/learn/docs/configuration/site-level-settings); [Fern redirects](https://buildwithfern.com/learn/docs/seo/redirects)
- **Mintlify** documents no redirect validation. It points to `mint broken-links` for broken links and `mint dev` to test redirects. — [Mintlify](https://www.mintlify.com/docs/create/redirects)
- **Astro**: file routes beat redirects, so a redirect from an existing page silently doesn't apply. No validation is documented. — [Astro](https://docs.astro.build/en/guides/routing/#redirects)
- **Hugo**: no alias-collision error. Later output silently overwrites earlier output. `--printPathWarnings` (from v0.55 as `--path-warnings`) warns on duplicate target paths but exits 0, and `--panicOnWarning` makes the build fail. — [Hugo Discourse: make build fail](https://discourse.gohugo.io/t/make-build-fail-when-url-or-aliases-are-re-used-duplicated/53185); [Hugo Discourse: overrides](https://discourse.gohugo.io/t/how-to-get-warnings-when-hugo-overrides-my-files/23231)
- **Jekyll**: the only validation is that `redirect_to` must be relative or `http(s)`. Other schemes are ignored with a warning. — [README](https://github.com/jekyll/jekyll-redirect-from)
- **Read the Docs**: no save-time validation is documented. At runtime an infinite loop returns 404 ("no other redirects will be applied"). — [RTD](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- **ReadMe**: origins "cannot match an existing page" (enforced by redirects firing only on 404). — [ReadMe](https://docs.readme.com/main/docs/error-pages)
- **GitBook**: duplicate sources in a CSV import keep only the first row. No loop or conflict checks are documented. — [GitBook](https://gitbook.com/docs/publish/site-redirects)

### Inferences
- The full set of checks across tools: target exists (Docusaurus, mkdocs-redirects, rediraffe), source no longer exists (Antora, rediraffe, Docusaurus), duplicate source (Antora, Docusaurus, rediraffe), self-redirect (Antora, Fern), loops (rediraffe, Fern), chains flattened (rediraffe), and removed-without-redirect (rediraffe `checkdiff`, Fern `missing-redirects`). No single tool does all of them.
- Tools that key redirects on source files or resource IDs (Antora, rediraffe, mkdocs-redirects) can check "target exists" exactly. URL-keyed ones can't check pattern targets without expanding them.

### Gaps
- Docusaurus's `validateRedirect` messages (e.g. any self-redirect check) weren't fetched.
- No documented validation was found for Mintlify, ReadMe, GitBook or Next.js beyond what's above.

## Fragment (anchor) redirects

### Takeaway
Fragments never reach a server, so only client-side JS can redirect them. mdBook is the one tool here with first-class fragment redirects (keys like `"/page.html#old"`). mkdocs-redirects and rediraffe *preserve* the incoming hash when redirecting a whole page but can't remap it. Mintlify and ReadMe forbid `#` in sources, and Mintlify allows it in destinations.

### Cited Findings
- mdBook: "Fragment redirects also work." / "When fragment redirects are specified, the page must use JavaScript to redirect to the correct location." / "This is useful if you rename or move a section header." / "Fragment redirects work with existing pages and deleted pages." — [mdBook](https://rust-lang.github.io/mdBook/format/configuration/renderers.html)
- The mdBook implementation looks up `window.location.hash` in a fragment map and calls `window.location.replace()`. The commit notes that server-side handling would be ideal. — [mdBook commit mirror](https://repo.jesusperez.pro/jesus/mdbook/commit/15c93b56ed91b9bdeba64a2af34674f4f170e8ec)
- StrictDoc's "stable URI forwarder" reads the hash on `DOMContentLoaded` and listens for `hashchange`. — [StrictDoc forwarder](https://strictdoc.readthedocs.io/en/stable/assets/stable_uri_forwarder.js)
- mkdocs-redirects appends the visitor's current hash to the target (`location.href="{url}"+(anchor?"#"+anchor:"")`) and splits a `#` out of a configured target. — [plugin.py](https://raw.githubusercontent.com/mkdocs/mkdocs-redirects/master/mkdocs_redirects/plugin.py)
- Rediraffe's default template preserves query and hash. — [rediraffe.py](https://raw.githubusercontent.com/wpilibsuite/sphinxext-rediraffe/main/sphinxext/rediraffe.py)
- Mintlify: "Redirect **sources** cannot include URL anchors like `path#anchor`". Destinations may include them. — [Mintlify](https://www.mintlify.com/docs/create/redirects). ReadMe origins "cannot include a hash". — [ReadMe](https://docs.readme.com/main/docs/error-pages)
- WebKit bug: a fragment on the URL could be dropped after a server redirect, while other browsers keep it. — [WebKit 24175](https://bugs.webkit.org/show_bug.cgi?id=24175)

### Inferences
- A fragment redirect for a heading that moved *within a page that still exists* needs JS in the live page itself, not a stub. mdBook's wording ("work with existing pages") implies it injects the map into real pages.
- Tools that key on headings' IDs could generate such maps. None of the surveyed tools derive them automatically from a heading's old ID.

### Gaps
- No fragment-redirect support was found documented for Docusaurus, Hugo, Jekyll, Antora, Sphinx, GitBook, RTD or Fern.
- The mdBook version that added fragment redirects wasn't found.

## Redirects across versions of versioned docs

### Takeaway
Only Read the Docs (Page Redirect) and Antora (fully qualified `version@component:module:page` aliases) have a first-class cross-version form. Elsewhere authors write per-version rules or code: Docusaurus `createRedirects`, Fern/Mintlify path params. mike handles version *aliases* (e.g. `latest`) by redirect stubs or symlinks, not page moves.

### Cited Findings
- RTD Page Redirects "let you redirect a page across all versions", and the `From URL` omits `/<language>/<version>`. They "don't apply to translations or subprojects". An Exact Redirect for an old version "works only if that version is disabled, unless you use Force Redirect". — [RTD](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html)
- Antora aliases can cross components and versions, but "use them with care". A page moved to another component needs the fully qualified former ID. Aliases "are not meant for bulk redirects, such as removing or renaming a whole component or version" (use the host router or an extension). The version selector "doesn't link previous versions of the source page to the target page". Latest-version aliases are emitted as 302 rules. — [Antora page-aliases](https://docs.antora.org/antora/latest/page/page-aliases/); [Antora redirect facility](https://docs.antora.org/antora/latest/playbook/urls-redirect-facility/)
- mike `alias_type`: "`symlink`: Create a symbolic link from the alias to the base directory", "`redirect`: Create an HTML redirect for each page of the documentation", "`copy`: Copy all the files". `set-default` "_always_ uses a redirect". — [mike README](https://github.com/jimporter/mike)
- Hugo prefixes aliases with dimension prefixes ("language, version, or role"), so an alias is per-dimension. — [Hugo](https://gohugo.io/content-management/urls/#aliases)
- Docusaurus: version-prefix redirects are written as code in `createRedirects` (Doris example). — [Doris config](https://apache.googlesource.com/doris-website/+/refs/heads/dev_0827/docusaurus.config.js)
- ReadMe: links to the Main Version's versioned path redirect to the unversioned URL, and "Versioned links for non-main versions are not redirected." — [ReadMe redirect scenarios](https://docs.readme.com/main/docs/redirect-scenarios)

### Inferences
- Docusaurus's plugin operates per build, so redirects in an older version's snapshot are generated only if the rule (or `createRedirects`) matches that version's routes.

### Gaps
- No source confirmed how Docusaurus's own versioning interacts with `redirects` entries (whether they need a version prefix).
- No source found on Fern or Mintlify versioned-redirect specifics beyond generic path params.

## Automatic redirects on move/rename

### Takeaway
GitBook creates redirects automatically when a page moves (307). Fern compares against previously published URLs and warns about missing ones. Rediraffe diffs git against a branch, failing on deleted or renamed files without a redirect and (`rediraffewritediff`) appending detected renames to the redirects file. Front-matter tools (Hugo, Jekyll, Antora) "record the old path in the page itself", but the author still writes it. Mintlify explicitly does not auto-redirect.

### Cited Findings
- GitBook: when pages move or are renamed, it "automatically creates an HTTP 307 redirect from the old URL to the new one". Permanent 308s are not automatic. Site redirects that pick a page *reference* rather than a path keep working after later moves. — [GitBook site redirects](https://gitbook.com/docs/publish/site-redirects). An older docs version reportedly said 301. — [docs.gitbook.com (older)](https://docs.gitbook.com/published-documentation/site-redirects)
- Rediraffe: `rediraffecheckdiff` ensures "deleted or renamed files in the git repo are in your redirects", diffing against `rediraffe_branch` (e.g. `'main~1'`). `rediraffewritediff` "adds renamed files to your redirects file" when similarity ≥ `rediraffe_auto_redirect_perc` (default 100). Deleted files can't be auto-added, and it works only with a redirects file. Messages: "(broken) {file} was deleted but is not redirected!" plus "Hint: This file was renamed to {hint} with a similarity of {perc}%.", and "(okay) Renamed file {from} has been redirected to {to} in your redirects file!". — [rediraffe docs](https://sphinxext-rediraffe.readthedocs.io/en/latest/); [rediraffe.py](https://raw.githubusercontent.com/wpilibsuite/sphinxext-rediraffe/main/sphinxext/rediraffe.py)
- Fern `missing-redirects` compares against previously published URLs (needs auth, skipped on first publish). It warns and doesn't write. — [Fern](https://buildwithfern.com/learn/docs/configuration/site-level-settings)
- Mintlify: "When you change the path of a file in your docs folder, it also changes the URL path to that page." Redirects must be added manually. — [Mintlify](https://www.mintlify.com/docs/create/redirects)
- Antora page aliases double as xref targets, so links to the old ID keep resolving. — [Antora](https://docs.antora.org/antora/latest/page/page-aliases/)
- A Fedora docs thread asks for "automatic redirection in Antora". — [Fedora Discussion](https://discussion.fedoraproject.org/t/automatic-redirection-in-antora/43743)

### Inferences
- Two detection sources exist in practice: VCS history (rediraffe) and the previous deploy's URL set (Fern, GitBook's hosted state). A previous build's manifest is the host-independent equivalent of Fern's approach.

### Gaps
- Whether ReadMe auto-redirects on slug change wasn't found.
- No surveyed SSG offers a `move` command that renames a file and writes the redirect in one step.

## Known pain points

### Takeaway
The recurring complaints: client-side stubs aren't real redirects (no 3xx, dev-mode gaps, SPA navigation 404s), silent overwrites and conflicts, no wildcards in static plugins, dashboard-only redirects living outside the repo, and front-matter aliases misbehaving when duplicated.

### Cited Findings
- Docusaurus 3.1 added a broken-link checker that flagged links to redirect stubs. Redirect pages aren't SPA routes, so `@docusaurus/Link` navigation to them "shows a 404". The maintainer said: "You should prefer server-side redirects", "You should refactor your links to target the final destination page instead of redirect pages", and "we'd prefer to keep the system as is". — [docusaurus#9803](https://github.com/facebook/docusaurus/issues/9803)
- Docusaurus client redirects don't run in dev. — [Docusaurus docs](https://docusaurus.io/docs/api/plugins/@docusaurus/plugin-client-redirects)
- Hugo: duplicate URLs and aliases silently overwrite, and users asked for a failing build. — [Hugo Discourse](https://discourse.gohugo.io/t/make-build-fail-when-url-or-aliases-are-re-used-duplicated/53185). An alias pointing at a path with existing content was generated inconsistently (the poster's race-condition guess is unconfirmed). — [Hugo Discourse: Aliases are broken](https://discourse.gohugo.io/t/aliases-are-broken/12318)
- Antora: only the last `:page-aliases:` entry is honored when a page declares the attribute twice, and a double alias caused a 404. — [Fedora docs issue #160](https://pagure.io/fedora-docs/package-maintainer-docs/issue/160)
- Astro static output: meta refresh only, "the status code isn't used by the server". — [Astro](https://docs.astro.build/en/guides/routing/#redirects)
- sphinx-reredirects silently overwrites an existing doc's HTML, and linkcheck can't check internal redirects. — [usage.rst](https://raw.githubusercontent.com/documatt/sphinx-reredirects/main/docs/usage.rst)
- RTD: redirects live in the dashboard (not versioned with the docs), are capped at 100 on Community, and forced redirects have a performance cost, so they aren't the default. — [RTD](https://docs.readthedocs.com/platform/stable/user-defined-redirects.html); [RTD guide mirror](https://dokk.org/documentation/readthedocs/10.26.0/_sources/guides/redirects.rst.txt)
- ReadMe redirects may be cached by browsers, so changes might not show immediately. — [ReadMe](https://docs.readme.com/main/docs/error-pages)
- Mintlify warns that very large `redirects` arrays can slow deploys. — [Mintlify](https://www.mintlify.com/docs/create/redirects)
- Next.js: forgetting the `/` before `:` in params treats the path as literal and "you run the risk of causing infinite redirects". — [Next.js](https://nextjs.org/docs/app/api-reference/config/next-config-js/redirects)

### Inferences
- The most common structural complaint is that the generator emits stubs while the host (which actually serves the redirect) is configured elsewhere. Antora's `redirect_facility`, Hugo's `Aliases` template and Astro's adapters are the three tools that bridge one declaration to both.

### Gaps
- No engineering blog posts about migrating docs platforms (e.g. redirect-map migrations) were fetched within the call budget.
- Pain points specific to Mintlify, Fern, GitBook and Material for MkDocs (e.g. instant navigation and redirect stubs) weren't found.
