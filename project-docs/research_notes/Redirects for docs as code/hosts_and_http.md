# Redirects on static hosts and web servers, and the HTTP semantics behind them

Researched 2026-10-23. Dates in parentheses are the "last updated" dates the pages carry, where they show one.

## Formats and limits for each host

### Takeaway
Every host takes a list of from/to rules, but the files, the defaults, and the limits all differ. Rule caps range from 50 (S3 routing rules) and 1,000 (GitLab) through 2,100 (Cloudflare `_redirects`) and 2,048 (Vercel `redirects`) up to millions on Cloudflare and Vercel bulk redirects. GitHub Pages has no server-side redirects at all, so it needs HTML stubs.

### Cited Findings

**Netlify** (`_redirects` and `netlify.toml`)
- `_redirects` is a plain text file with no extension, placed in the publish directory. Each line is a from path, a to path or URL, and optional options; `#` starts a comment. "Paths are case-sensitive and special characters in paths must be url-encoded." — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
- In `netlify.toml`, each rule is a `[[redirects]]` table with `from`, `to`, `status` ("`301` by default"), `force` ("Whether to override any existing content in the path or not; `false` by default"), `query`, `conditions`, `headers`, and `signed`. — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
- Order: "The redirects engine will process the first matching rule it finds, reading from top to bottom. Rules in the `_redirects` file are always processed first, followed by rules in the Netlify configuration file." Edge Functions run before redirects. — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
- Limits: there is no numeric cap. Netlify says "If the size of this output is too large, the deploy might fail," and "If you need to set up 10,000 redirects or more, we recommend using wildcards or placeholders as much as possible", or Edge Functions. — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
- Status codes:
  - Netlify documents 301 (the default), 302, 404 (shows a custom page without changing the URL), and 200 (a rewrite or proxy).
  - For 302, it says: "Use this status code instead of `307`, which is currently unsupported."
  - The page doesn't mention 410, 303, or 308.
  — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
- Splats: `/news/* /blog/:splat`. An asterisk must end a path segment (`/jobs/*.html` isn't allowed), and a splat can't exclude paths; you use rule order instead. — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
- Placeholders: `/news/:month/:date/:year/:slug /blog/:year/:month/:date/:slug`. "A placeholder either matches a path segment from one `/` to the next `/` or matches a final path segment including a file extension but excluding a query string." — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
- Query matching: `/store id=:id /blog/:id 301` matches only when the query has exactly `id`. "If the URL contains other parameters in addition to or instead of `id`, the request doesn't match that rule." — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
- Domain-level rules are allowed, for example `http://blog.yoursite.com/* https://www.yoursite.com/blog/:splat 301!`, but "Domains must be assigned to the site." — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
- "Infinitely looping rules, where the 'from' and 'to' resolve to the same location, are incorrect and will be ignored." Netlify limits internal rewrites to one hop by default. — [Netlify: Rewrites and proxies](https://docs.netlify.com/manage/routing/redirects/rewrites-proxies/)

**Vercel** (`vercel.json` and bulk redirects)
- A `redirects` entry has these fields — [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json) (2026-08-14):
  - `source`: "A pattern that matches each incoming pathname (excluding querystring)".
  - `destination`.
  - `permanent`: "default `true`". When true the status is 308; when false it is 307.
  - `statusCode`: for any code other than 307 or 308; it can't be combined with `permanent`.
  - `has` and `missing`: conditions on request properties.
- Limits on configuration redirects: "Number of redirects in the array: 2,048"; "String length for `source` and `destination`: 4,096". The `/.well-known` path is reserved and can't be redirected. — [Vercel: Configuration redirects](https://vercel.com/docs/routing/redirects/configuration-redirects) (2026-08-11)
- Bulk redirects:
  - They're configured with `bulkRedirectsPath` in `vercel.json`, which points to a CSV, JSON, or JSONL file or a folder of them. Files may be generated at build time.
  - Fields:
    - `source` is a path or full URL, excluding the query string, at most 2,048 characters.
    - `destination` is at most 2,048 characters.
    - `permanent` defaults to `false`.
    - `statusCode` may be 301, 302, 303, 307, or 308; the default is 307.
    - `caseSensitive` defaults to `false`.
    - `preserveQueryParams` defaults to `false`.
  - Bulk redirects support no wildcards and no header matching. Vercel "processes them before any other route specified in your deployment."
  — [Vercel: Bulk redirects](https://vercel.com/docs/routing/redirects/bulk-redirects) (2026-09-21); [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
- Bulk redirect limits and pricing:
  - Pro and Enterprise only; they're "not available on the Hobby plan."
  - Included: Pro 1,000, Enterprise 10,000. Extra capacity is sold in groups of 25,000.
  - "A maximum of 1,000,000 bulk redirects can be configured per project."
  - The price shown is "$0.002/month per additional 25,000". That figure looks implausibly low; it may be a rendering error on the page, so verify it before quoting.
  — [Vercel: Bulk redirects](https://vercel.com/docs/routing/redirects/bulk-redirects)
- Other ways to redirect on Vercel:
  - Middleware with Global Config, for redirects that change without a redeploy.
  - Domain redirects, set in the dashboard.
  - Firewall redirects, which "execute before CDN configuration redirects." A firewall redirect made in the dashboard is a 307.
  — [Vercel: Redirects](https://vercel.com/docs/routing/redirects) (2026-09-18)
- Vercel's descriptions of the status codes:
  - 307: "Not cached by client, the method and body never changed."
  - 302: "Not cached by client, the method may or may not be changed to `GET`."
  - 308: "Cached by client, the method and body never changed."
  - 301: "Cached by client, the method may or may not be changed to `GET`."
  — [Vercel: Redirects](https://vercel.com/docs/routing/redirects)
- Next.js on Vercel: "Redirects are checked before the filesystem which includes pages and `/public` files." Next.js adds a `Refresh` header to 308 responses "to ensure IE11 compatibility." — [Next.js: redirects](https://nextjs.org/docs/app/api-reference/config/next-config-js/redirects)

**Cloudflare Pages and Workers static assets** (`_redirects`)
- The format is `[source] [destination] [code?]`, one rule per line. Invalid lines are ignored. The code defaults to **302**, and 301, 302, 303, 307, and 308 are supported. A 200 proxy works only for relative URLs; other rewrite codes such as 404 aren't supported. — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- Limits: "2,000 static redirects and 100 dynamic redirects, for a combined total of 2,100 redirects." "Each redirect declaration has a 1,000-character limit." — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- Splats and placeholders:
  - A URL can have one splat (`*`). It is greedy and is referenced as `:splat`.
  - Placeholders match `:[A-Za-z]\w*`, and each can be referenced only once.
  - "Fragments in the source are not evaluated." The destination can include fragments, query strings, splats, and placeholders.
  — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- Not supported: domain-level redirects; matching on query parameters; and redirects by country, language, or cookie. Redirects also don't apply to requests that Pages Functions serve. — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- Workers static assets accept the same `_redirects` file with the same limits: 2,000 static, 100 dynamic, 1,000 characters per rule, default 302. Rules apply to static asset responses and not to responses from Worker code. — [Cloudflare Workers: Static assets redirects](https://developers.cloudflare.com/workers/static-assets/redirects/)
- Bulk Redirects are account-level and are recommended for more than 2,100 rules. They run in front of Pages and Workers, so they win when both have a rule for the same path. — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- Bulk Redirects quotas per account:

  | | Free | Pro | Business | Enterprise |
  |---|---|---|---|---|
  | Bulk Redirect Rules | 15 | 15 | 15 | 50 |
  | Lists | 5 | 5 | 5 | 25 |
  | URL redirects across lists | 10,000 | 25,000 | 50,000 | 1,000,000 (default; "several million" on request) |

  Single Redirects allow 10, 25, 50, and 300 rules per zone on the same four plans. — [Cloudflare: URL forwarding availability](https://developers.cloudflare.com/rules/url-forwarding/)
- Bulk Redirects use static matching only: they "do not support string replacement operations or regular expressions." They run after the WAF. — [Cloudflare: Bulk Redirects](https://developers.cloudflare.com/rules/url-forwarding/bulk-redirects/)
- Bulk redirect parameters:
  - `source_url` and `target_url`: each at most 32 KB.
  - `status_code`: 301, 302, 307, or 308; the default is 301. 303 isn't listed.
  - `preserve_query_string`: default `false`. When it is enabled, "Any query string on the target URL is discarded."
  - `include_subdomains`: default `false`.
  - `subpath_matching`: default `false`.
  - `preserve_path_suffix`: default `true`; it applies only with subpath matching.
  - A source URL without a scheme matches both http and https. A list can't hold two redirects with the same source URL.
  — [Cloudflare: Bulk redirect parameters](https://developers.cloudflare.com/rules/url-forwarding/bulk-redirects/reference/parameters/)
- Pages redirects HTML files to their extensionless URLs: `/contact.html` goes to `/contact`, and `/about/index.html` goes to `/about/`. The page doesn't give the status code. — [Cloudflare Pages: Serving Pages](https://developers.cloudflare.com/pages/configuration/serving-pages/)

**GitHub Pages**
- GitHub's Jekyll page lists the plugins that are always enabled; `jekyll-redirect-from` isn't one of them. "GitHub Pages cannot build sites using unsupported plugins." The page says nothing about redirects or `.htaccess`. Publishing from a custom GitHub Actions workflow is now "the recommended approach." — [GitHub Docs: About GitHub Pages and Jekyll](https://docs.github.com/en/pages/setting-up-a-github-pages-site-with-jekyll/about-github-pages-and-jekyll)
- The supported-versions list for GitHub Pages includes `jekyll-redirect-from` 0.16.0, alongside `github-pages` 232 and Jekyll 3.10.0. — [pages.github.com/versions.json](https://pages.github.com/versions.json)
- How jekyll-redirect-from works:
  - "Redirects are performed by serving an HTML file with an HTTP-REFRESH meta tag which points to your destination." It writes no `.htaccess` or nginx configuration.
  - `redirect_from` in a page's front matter takes a list. An entry with a trailing slash produces `dir/index.html`; an entry without one produces a file with no extension.
  - `redirect_to` accepts only relative paths or `http(s)` URLs; `javascript:` and other schemes are ignored with a warning.
  - It writes a `redirects.json` file by default, "for automated testing or server-side redirects."
  — [jekyll-redirect-from README](https://github.com/jekyll/jekyll-redirect-from)
- The plugin's whole template — [redirect.html](https://github.com/jekyll/jekyll-redirect-from/blob/master/lib/jekyll-redirect-from/redirect.html):
  ```html
  <!DOCTYPE html>
  <html lang="en-US">
    <meta charset="utf-8">
    <title>Redirecting&hellip;</title>
    <link rel="canonical" href="{{ page.redirect.to | escape }}">
    <script>location={{ page.redirect.to_js }}</script>
    <meta http-equiv="refresh" content="0; url={{ page.redirect.to | escape }}">
    <meta name="robots" content="noindex">
    <h1>Redirecting&hellip;</h1>
    <a href="{{ page.redirect.to | escape }}">Click here if you are not redirected.</a>
  </html>
  ```

**AWS S3 static website hosting and CloudFront**
- S3 offers three mechanisms — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html):
  1. Redirect every request on a bucket to another host.
  2. Routing rules, written as JSON in the console or XML in the API. A `Condition` holds `KeyPrefixEquals` and/or `HttpErrorCodeReturnedEquals`. A `Redirect` holds `Protocol`, `HostName`, either `ReplaceKeyPrefixWith` or `ReplaceKeyWith` (not both), and `HttpRedirectCode`.
  3. Object metadata: setting `x-amz-website-redirect-location` on an object makes S3 "interpret[] the object as a 301 redirect".
- The rule cap: "Amazon S3 has a limitation of 50 routing rules per website configuration. If you require more than 50 routing rules, you can use object redirect." — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html)
- Object redirects work only on the website endpoint. "REST endpoint – Amazon S3 doesn't redirect the page request. It returns the requested object." — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html)
- When `HostName` and `ReplaceKeyWith` are combined, S3 inserts the slash between them, so `ReplaceKeyWith` should have no leading slash. — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html)
- CloudFront Functions quotas: 100 functions per account. A function is at most **10 KB** ("This quota isn't adjustable. To store additional data… create a key value store") and has 2 MB of memory. — [CloudFront quotas](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cloudfront-limits.html)
- CloudFront KeyValueStore quotas:
  - Key at most 512 bytes; value at most 1 KB.
  - Store at most **5 MB**.
  - One store per function, and 200 stores per account.
  - An update request changes at most "50 keys or 3 MB payload".
  - Functions that read a store need JavaScript runtime 2.0. AWS lists "URL rewrites or redirects" as a use case.
  — [CloudFront quotas](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cloudfront-limits.html); [CloudFront KeyValueStore](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/kvs-with-functions.html)
- Lambda@Edge quotas: the compressed function and libraries are at most 50 MB. A response generated on viewer request is at most 40 KB; on origin request, at most 1 MB. The timeout is 30 s. CloudFront's maximum URL length is 8,192 bytes. — [CloudFront quotas](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/cloudfront-limits.html)

**Azure Static Web Apps** (`staticwebapp.config.json`)
- Routes form a `routes` array that is "evaluated in the order as they appear", and "Rule evaluation stops at the first match." A route has `route`, optional `methods`, and `redirect` or `rewrite` (mutually exclusive) with `statusCode`. — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration) (2026-03-25)
- For redirects, the "Default response code is a `302`… but you can override with a `301`." The docs mention no other redirect codes. — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration)
- Wildcards work only at the end of a path: `/calendar/*` doesn't match `/calendar`, but `/calendar*` does. A route of `/profile/index.html` also matches `/profile` and `/profile/`; a folder route doesn't match the file. — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration)
- Restrictions: "Max file size is 20 KB"; "Max of 50 distinct roles." — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration)

**Firebase Hosting** (`firebase.json`)
- Each redirect has a `source` glob (recommended) or a `regex` (RE2), a `destination`, and a `type` of 301 or 302. A glob captures a segment with `:name` and the rest of the path with `:name*`; a regex uses named groups (`?P`) or `:1`. The first matching rule wins. — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)
- Priority order:
  1. Reserved `/__/*` paths.
  2. Configured redirects.
  3. Exact-match static content.
  4. Configured rewrites.
  5. The custom 404 page.
  6. The default 404 page.
  — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)
- The configuration page gives no count limits for redirects, rewrites, or headers. — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)

**GitLab Pages** (`_redirects`)
- The file is `public/_redirects`, in a subset of Netlify's syntax. Status codes are 301 (the default), 302, and 200. Every path must start with `/`. — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)
- Limits are configurable per instance; the defaults are a "Maximum file size is 64 KB" and a "Maximum number of rules is 1,000". Only the first rules, up to the maximum, are processed. — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)
- Not supported: query parameters, non-200 rewrites, country, language, and role. Domain-level redirects have been generally available since GitLab 17.4, with 301 and 302. On the default Pages domain, every rule must be prefixed with the project slug. — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)
- Matching:
  - A splat matches anything, including an empty string, and only the first splat can be referenced.
  - A placeholder is `:[a-zA-Z]+`. It matches a single segment and doesn't match an empty string.
  - You can debug a site's rules by visiting `/_redirects` on it.
  — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)

**Render** (static sites)
- Rules are entered in the Render Dashboard. The redirects page doesn't mention `render.yaml` or a `_redirects` file. Redirects are "301 Moved Permanently"; no other code is documented. — [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)
- Matching: `*` matches any string from its position onward, and `:name` placeholders are supported. A rule can't apply to the domain root; each source needs at least one path component. — [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)
- Chaining: "If this process results in a redirect to another site path, the process repeats with the new path." — [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)

**nginx**
- `return code URL;`: "it is possible to specify either a redirect URL (for codes 301, 302, 303, 307, and 308)". 307 was "not treated as a redirect until versions 1.1.16 and 1.0.13", and 308 not "until version 1.13.0." — [nginx: ngx_http_rewrite_module](https://nginx.org/en/docs/http/ngx_http_rewrite_module.html)
- `rewrite regex replacement [flag];`: the `redirect` flag returns 302, and `permanent` returns 301. — [nginx: ngx_http_rewrite_module](https://nginx.org/en/docs/http/ngx_http_rewrite_module.html)
- Query strings under `rewrite`: "If a `replacement` string includes the new request arguments, the previous request arguments are appended after them." A trailing `?` on the replacement stops the append. — [nginx: ngx_http_rewrite_module](https://nginx.org/en/docs/http/ngx_http_rewrite_module.html)
- `map` builds a variable from a table and accepts `include file` for an external list ("There can be several inclusions"), which is the usual way to keep a large redirect list. Its size is governed by `map_hash_max_size` and `map_hash_bucket_size`. — [nginx: ngx_http_map_module](https://nginx.org/en/docs/http/ngx_http_map_module.html)
- Location matching "is performed against a normalized URI, after decoding the text encoded in the '%XX' form, resolving references to relative path components '.' and '..', and possible compression of two or more adjacent slashes into a single slash." `~*` makes a regex location case-insensitive; `~` makes it case-sensitive. — [nginx: ngx_http_core_module](https://nginx.org/en/docs/http/ngx_http_core_module.html)

**Apache httpd**
- `Redirect [status] URL-path URL` works in server and virtual host configuration, in `<Directory>`, and in `.htaccess` (needs `AllowOverride FileInfo`). The old URL-path "is a case-sensitive (%-decoded) path beginning with a slash." — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
- `Redirect` matches by prefix, and only on whole segments: "Additional path information beyond the matched URL-path will be appended to the target URL," and the query string is carried over. — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
- Status codes:
  - The default is 302.
  - Keywords: `permanent` (301), `temp` (302), `seeother` (303), and `gone` (410). With `gone`, "the URL argument should be omitted."
  - A numeric 3xx status needs a URL; any other status must have none.
  — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
- `RedirectMatch` takes a regex. All `Redirect` directives are processed before `Alias`, and "the first match taking precedence." — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
- `RewriteRule` flags — [Apache: RewriteRule flags](https://httpd.apache.org/docs/2.4/rewrite/flags.html):
  - `[R]` defaults to 302; `[R=301]` and symbolic names are allowed. Use it as `[R,L]`.
  - `[NC]` matches case-insensitively.
  - `[QSA]` merges query strings.
  - `[QSD]` discards the query string. It is available since 2.4.0 and wins over `[QSA]`.

### Inferences
- The hosts fall into four groups by what a toolchain would emit:
  1. A Netlify-style `_redirects` text file: Netlify, Cloudflare Pages and Workers, and GitLab Pages.
  2. A JSON configuration file: Vercel `vercel.json`, Azure `staticwebapp.config.json`, and Firebase `firebase.json`.
  3. Server configuration: nginx `map`, Apache `.htaccess`, and S3 routing rules (JSON or XML) or per-object metadata.
  4. No server rules, so HTML stubs are needed: GitHub Pages, and any host where server rules are absent or too limited.
- The `_redirects` dialects aren't interchangeable:
  - Default codes differ: Netlify and GitLab default to 301, and Cloudflare to 302.
  - Only Netlify accepts force (`!`) and query matching.
  - GitLab rejects rules without a leading `/` and needs the project-slug prefix on default domains.
  - A generator should always write the status code explicitly and use only plain path-to-path rules.
- The tightest common limits:
  - S3 allows 50 routing rules; past that, use one object per redirect.
  - Azure's configuration file is capped at 20 KB, which fits only a few hundred rules.
  - GitLab defaults to 1,000 rules and 64 KB.
  - Cloudflare allows 2,000 static rules plus 100 dynamic ones.
  - Vercel allows 2,048 configuration redirects.
  - A toolchain should count the rules and warn per host before a deploy fails or rules are silently dropped. Silent dropping is documented for GitLab, which processes only the first rules up to the maximum.

### Gaps
- No current Vercel page states a "1,024" limit. The configuration-redirects page says 2,048, and I couldn't establish whether 1,024 is an old figure.
- Netlify publishes no numeric rule or size limit, only "might fail" and the 10,000 advisory. The docs don't say whether Netlify supports 303, 308, or 410 (only that 307 is unsupported).
- Render: no rule-count limit, no codes other than 301, and no `render.yaml` support for redirects documented on the page I read.
- Firebase: no documented cap on redirect count; I didn't find it on the configuration page.
- Azure: the docs mention only 301 and 302. Whether 307 or 308 work isn't documented.

## Trailing slashes, case, query strings, encoded paths, and shadowing

### Takeaway
Hosts disagree about whether a file at the old path beats a redirect:
- **The file wins** (shadowing) on Netlify (unless you force it), GitLab, and Render.
- **The redirect wins** on Cloudflare, Firebase, and Vercel or Next.js.

Trailing-slash normalization and query-string handling also vary. A toolchain must therefore not leave content at the old path, and should emit or test both slash forms of each URL.

### Cited Findings
- **Shadowing**
  - **Netlify**: the file wins by default. "By default, you can't shadow a URL that actually exists within the site." To force a rule, append `!` (`/app/* /app/index.html 200!`) or set `force = true`. — [Netlify: Rewrites and proxies](https://docs.netlify.com/manage/routing/redirects/rewrites-proxies/); [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
  - **GitLab Pages**: the file wins. "Files take priority over redirects." Force isn't supported; `200!` doesn't change this. — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)
  - **Render**: the file wins. "Render does not apply redirect or rewrite rules to a path if a resource exists at that path." — [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)
  - **Cloudflare Pages and Workers**: the redirect wins. "Redirects are always followed, regardless of whether or not an asset matches the incoming request." — [Cloudflare Workers: Static assets redirects](https://developers.cloudflare.com/workers/static-assets/redirects/); [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
  - **Firebase**: the redirect wins. Redirects rank above exact-match static content. — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)
  - **Vercel and Next.js**: the redirect wins. "Redirects are checked before the filesystem." By contrast, Vercel's `rewrites` give "precedence… to the filesystem." — [Next.js: redirects](https://nextjs.org/docs/app/api-reference/config/next-config-js/redirects); [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
  - **S3**: an object carrying `x-amz-website-redirect-location` can keep its content, but the website endpoint still redirects. — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html)
- **Trailing slashes**
  - **Netlify**: rules match "regardless of whether or not they contain a trailing slash." You "cannot use a redirect rule to add or remove a trailing slash." `/blog/remove-my-slashes/ /blog/remove-my-slashes 301!` "will cause an infinite redirect." Pretty URLs, which are on by default, send `/about` and `/about.html` to `/about/`. — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
  - **Vercel**:
    - `trailingSlash: false` sends `/about/` to `/about` with a 308.
    - `true` sends `/about` to `/about/` with a 308, except for paths with a file extension.
    - Leaving it undefined serves both forms without redirecting.
    - `cleanUrls: true` sends `/about.html` to `/about` with a 308.
    — [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
  - **Vercel** also normalizes `//` to `/` with a 308 before any rules run. — [Vercel: Redirects](https://vercel.com/docs/routing/redirects)
  - **Azure**: `trailingSlash` can be `always`, `never`, or `auto`; each redirects with a **301**. With the setting omitted, `/about`, `/about/`, and `/about/index.html` all return 200. — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration)
  - **Firebase**: `trailingSlash: true` or `false` redirects to add or remove the slash. When it is unset, slashes are used "only for directory index files". `cleanUrls: true` sends `x.html` to `x` with a 301. — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)
  - **Cloudflare Pages**: `/about/index.html` goes to `/about/`, and `/contact.html` to `/contact`. — [Cloudflare Pages: Serving Pages](https://developers.cloudflare.com/pages/configuration/serving-pages/)
- **Case**
  - **Netlify**: paths are case-sensitive. — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
  - **Vercel's CDN**: it does "not normalize URL paths to lowercase", so `/About` and `/about` are different paths. Project-level routing rules match case-insensitively by default. A `vercel.json` that defines `redirects` makes those rules case-sensitive. — [Vercel: Redirects](https://vercel.com/docs/routing/redirects); [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
  - **Vercel bulk redirects**: `caseSensitive` defaults to `false`, so they match case-insensitively. — [Vercel: Bulk redirects](https://vercel.com/docs/routing/redirects/bulk-redirects)
  - **Apache**: the `Redirect` URL-path is case-sensitive; `[NC]` makes a `RewriteRule` case-insensitive. — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html); [Apache: RewriteRule flags](https://httpd.apache.org/docs/2.4/rewrite/flags.html)
- **Query strings**
  - **Netlify** "automatically passes on all query string parameters to destination paths for redirects with `200`, `301`, and `302` HTTP status." — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
  - **Vercel configuration redirects** pass the query through: "Query parameters pass through unless you explicitly replace them in the destination." Next.js says the same. — [Vercel: Redirects](https://vercel.com/docs/routing/redirects); [Next.js: redirects](https://nextjs.org/docs/app/api-reference/config/next-config-js/redirects)
  - **Bulk redirects drop the query by default**: Vercel's `preserveQueryParams` and Cloudflare's `preserve_query_string` both default to `false`. — [Vercel: Bulk redirects](https://vercel.com/docs/routing/redirects/bulk-redirects); [Cloudflare: Bulk redirect parameters](https://developers.cloudflare.com/rules/url-forwarding/bulk-redirects/reference/parameters/)
  - **Apache `Redirect`** preserves the query string. **`RewriteRule`** copies it when the target has none, unless `[QSD]` is set. — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html); [Apache: RewriteRule flags](https://httpd.apache.org/docs/2.4/rewrite/flags.html)
- **Encoded paths**
  - **Netlify**: "special characters in paths must be url-encoded." — [Netlify: Redirects overview](https://docs.netlify.com/manage/routing/redirects/overview/)
  - **Apache**: `Redirect` compares %-decoded paths. — [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
  - **nginx**: locations match the decoded, normalized URI. — [nginx: ngx_http_core_module](https://nginx.org/en/docs/http/ngx_http_core_module.html)
  - **Apache `RewriteRule`**: output is percent-encoded unless `[NE]` is set, so `#` becomes `%23` and "then result in a 404". — [Apache: RewriteRule flags](https://httpd.apache.org/docs/2.4/rewrite/flags.html)

### Inferences
- **Don't leave content at the old path.** A toolchain that writes both a server rule and an HTML stub at the old path gets different results by host. On Netlify (without `!`), GitLab, and Render, the stub shadows the rule, and the host serves a 200 meta-refresh instead of a 301. On Cloudflare, Firebase, and Vercel, the server rule wins. The toolchain should either:
  - emit stubs only for hosts that have no rules, or
  - emit Netlify rules with `!`, and for GitLab and Render, never emit stubs or old pages.
- **Hosts that normalize slashes need one rule per URL.** Netlify, and Azure, Vercel, and Firebase once configured, normalize the trailing slash. There, one rule per canonical form is enough. Hosts that don't normalize, or whose behavior is undocumented (Cloudflare's `_redirects`, S3, nginx, Apache), may need both `/a` and `/a/` emitted, or a `/a/*`-style rule.
- **Write sources in the case the site publishes.** Writing rules in exactly that case is safe everywhere. Relying on case-insensitive matching isn't portable.
- **The query string survives only on some hosts.** Netlify, Vercel configuration redirects, and Apache `Redirect` keep it. Bulk redirect products on both Vercel and Cloudflare drop it unless configured otherwise.

### Gaps
- Not documented: Cloudflare Pages' `_redirects` matching rules for trailing slashes and case; Bulk Redirects' URL-matching algorithm, which is described on a separate "how it works" page I didn't read; and the status code of Pages' `.html` redirect.
- How S3 orders routing rules against existing objects (does `KeyPrefixEquals` fire when the object exists?) isn't stated on the page I read.
- Whether nginx `return` appends the original query string isn't stated in the module docs. The common idiom, `return 301 /new$is_args$args;`, implies it doesn't, but I didn't find an official statement.
- How Netlify, GitLab, Firebase, and Azure handle URL-encoded or non-ASCII paths isn't documented beyond Netlify's "must be url-encoded."

## HTTP status semantics: 301, 308, 302, 307, 303, 404, and 410

### Takeaway
- **Permanent moves**: 301 and 308 both mean the move is permanent. 308 forbids changing the method, while 301 lets a POST become a GET; for a docs site of GET-only pages they are equivalent. Google treats them identically.
- **Temporary moves**: 302, 303, and 307 are temporary.
- **Removed content**: 410 says a page was removed on purpose and permanently, but Google treats it the same as 404.
- **Caching**: 301, 308, and 410 are heuristically cacheable, and Chromium stores 301 and 308 responses with infinite freshness when no explicit cache headers are sent. Without a `Cache-Control` on the redirect, a permanent redirect is effectively impossible to undo for browsers that have already seen it.

### Cited Findings
- **301 Moved Permanently**: "the target resource has been assigned a new permanent URI." The note to 301 reads: "For historical reasons, a user agent MAY change the request method from POST to GET… If this behavior is undesired, the 308 (Permanent Redirect) status code can be used instead." "A 301 response is heuristically cacheable." — [RFC 9110 §15.4.2](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4.2)
- **302 Found**: "resides temporarily under a different URI… the client ought to continue to use the target URI for future requests." The same POST-to-GET allowance applies, and RFC 9110 points to 307 as the alternative. — [RFC 9110 §15.4.3](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4.3)
- **303 See Other** means "an indirect response to the original request." It is "primarily used to allow the output of a POST action to redirect the user agent to a different resource," and the new URI "is not considered equivalent to the target URI." — [RFC 9110 §15.4.4](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4.4)
- **307 Temporary Redirect**: "the user agent MUST NOT change the request method if it performs an automatic redirection." — [RFC 9110 §15.4.8](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4.8)
- **308 Permanent Redirect** is permanent and preserves the method. It is heuristically cacheable. "This status code is much younger (June 2014) than its sibling codes and thus might not be recognized everywhere." — [RFC 9110 §15.4.9](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4.9)
- **History of the method change**: RFC 9110 says 301 and 302 were method-preserving in HTTP/1.0, but "Prevailing practice eventually converged on changing the method to GET." 307 and 308 "were later added to unambiguously indicate method-preserving redirects." — [RFC 9110 §15.4](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4)
- **410 Gone**:
  - Meaning: "no longer available at the origin server and that this condition is likely to be permanent. If the origin server does not know… the status code 404 (Not Found) ought to be used instead."
  - Purpose: it is "primarily intended to assist the task of web maintenance by notifying the recipient that the resource is intentionally unavailable and that the server owners desire that remote links to that resource be removed."
  - Duration: "It is not necessary to mark all permanently unavailable resources as 'gone' or to keep the mark for any length of time."
  - It is heuristically cacheable.
  — [RFC 9110 §15.5.11](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.5.11)
- **Heuristic caching**:
  - Caches may assign a heuristic freshness only to "heuristically cacheable" responses without explicit freshness, and "A cache MUST NOT use heuristics to determine freshness when an explicit expiration time is present."
  - "A typical setting of this fraction might be 10%" of the time since Last-Modified.
  — [RFC 9111 §4.2.2](https://www.rfc-editor.org/rfc/rfc9111.html#section-4.2.2)
- **Chromium's cache**: in `GetFreshnessLifetimes`, "These responses are implicitly fresh (unless otherwise overruled)." That applies to 300, 301, 308, and 410, and sets `lifetimes.freshness = base::TimeDelta::Max(); … // It should never be stale.` Explicit `Cache-Control` or `Expires` overrule it. — [Chromium source: net/http/http_response_headers.cc](https://github.com/chromium/chromium/blob/main/net/http/http_response_headers.cc)
- **Undoing a cached 301**: developer write-ups describe browsers keeping cached 301s until the user clears the cache, for example with DevTools' "Disable cache" or by clearing site data. A 2019 test found no browser cached them, so the anecdotal sources conflict. These are secondary and anecdotal sources; the Chromium code above is the primary evidence. — [neilwithdata: Make Chrome forget a permanent redirect](https://www.neilwithdata.com/chrome-redirect-forget); [Jaywing: Do modern browsers still cache 301 redirects?](https://www.jaywing.com/views/opinion/do-modern-browsers-still-cache-301-redirects/)
- **Google's treatment**:
  - 301 is a "strong" signal that the target should be canonical; 308 is "Equivalent to `301`".
  - 302 is a "weak" signal, and 303 and 307 are equivalent to 302.
  - Google notes that the codes are still semantically different, so you should use the right one.
  - All 4xx except 429 "are treated the same," 404 and 410 included: the URL is dropped from the index and crawled less often.
  — [Google: HTTP status codes and Google's crawlers](https://developers.google.com/crawling/docs/troubleshooting/http-status-codes) (2026-02-04)
- **Google's table**: 301, 308, a 0-second meta refresh, a 0-second HTTP refresh, a JavaScript `location` change, and "crypto redirects" are permanent. 302, 303, 307, and delayed refreshes are temporary. Google recommends "a permanent server-side redirect whenever possible," and permanent redirects "when you're sure that the redirect won't be reverted." — [Google Search Central: Redirects and Google Search](https://developers.google.com/search/docs/crawling-indexing/301-redirects) (2026-04-14)
- **Google on site moves**: "Keep the redirects for as long as possible… generally at least 1 year." — [Google Search Central: Site moves with URL changes](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes) (2026-08-20)
- **MDN's guidance by code**:
  - 301: "Reorganization of a website."
  - 308: the same, "with non-GET links/operations."
  - 302 and 307 are temporary, and 307 is "Better than `302` when non-`GET` operations are available."
  - 303 is used after a PUT or POST.
  — [MDN: Redirections in HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections)
- **Vercel's guidance**: Vercel recommends 307 and 308 "to avoid the ambiguity of non `GET` methods". Its `vercel.json` default is 308. — [Vercel: Redirects](https://vercel.com/docs/routing/redirects)

### Inferences
- **The status for a moved docs page**: use 301 for the widest host support. Everything below lists 301 except where noted. Use 308 only where the host supports it; all docs-site requests are GETs, so nothing is lost. Netlify says only that 307 is unsupported, and lists 301, 302, 404, and 200.
- **The status for a removed page with no successor**: 410 is semantically right, but it gives no search benefit over 404. Only some hosts support 410 as a rule: Apache (`gone`), nginx (`return 410`), and Azure through a route's `statusCode`. A real 404 page, or a redirect to the nearest parent page, is the portable choice.
- **Hosts can supply their own long-lived caching**: emitting `Cache-Control: max-age=…` on redirects caps how long a mistaken 301 lives in Chromium. Where a host doesn't let you set headers on redirect responses, a mistaken 301 should be treated as irreversible for visitors who have already seen it.

### Gaps
- I found no first-party Firefox or Safari documentation of how long they cache 301s.
- I found no documentation of how CDNs such as Netlify, Cloudflare, and Vercel cache redirect responses at the edge (TTL and purge on deploy).

## Client-side redirects: meta refresh and JavaScript

### Takeaway
A 0-second meta refresh, with a JavaScript `location` change, a canonical link, and a visible link, is the accepted fallback where server redirects are impossible. jekyll-redirect-from's template is the reference. Google counts an instant meta refresh or a JavaScript redirect as permanent, but prefers server redirects. WCAG accepts an instant meta refresh as sufficient and fails a delayed one.

### Cited Findings
- **Order of execution**: "HTTP redirects always execute first." "Somewhat surprisingly, JavaScript redirects execute next, before HTML redirects," because a meta refresh runs after the page has loaded. MDN warns against using both: "If someone changes the HTTP redirects but forgets to change the HTML redirects… could cause an infinite loop." — [MDN: Redirections in HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections)
- **MDN on the delay**: for meta refresh, "Always set it to `0` for accessibility compliance". It works only for HTML, not for images or other resources. — [MDN: Redirections in HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections)
- **HTML's syntax for the attribute**: the content must be a non-negative integer, optionally followed by `;`, whitespace, `URL=`, and "a valid URL string that does not start with a literal U+0027 APOSTROPHE (') or U+0022 QUOTATION MARK (")". — [HTML Standard: the meta element's Refresh state](https://html.spec.whatwg.org/multipage/semantics.html#attr-meta-http-equiv-refresh)
- **Google on meta refresh**: Google interprets "instant `meta refresh` redirects as permanent redirects" and "delayed `meta refresh` redirects as temporary redirects."
- **Google on JavaScript redirects**: use them only when server-side and meta refresh options aren't available. Google may miss them if rendering fails.
- **Google's fallback order**: client-side redirects are a "last resort" in a site move.
  — [Google Search Central: Redirects and Google Search](https://developers.google.com/search/docs/crawling-indexing/301-redirects); [Google: Site moves](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- **WCAG H76**, "Using meta refresh to create an instant client-side redirect": "It is important that the time-out is set to zero"; "The page containing the redirect code should only contain information related to the redirect." It is sufficient for SC 3.2.5, Change on Request, together with G110. It notes that "Redirects are preferably implemented on the server side" and refers to SVR1. — [W3C: H76](https://www.w3.org/WAI/WCAG22/Techniques/html/H76)
- **WCAG F41**: a meta refresh used to refresh or redirect after a delay is a failure of SC 2.2.1, 2.2.4, and 3.2.5. Its test flags refresh values from 1 to 72,000 seconds, so 0 isn't flagged. — [W3C: F41](https://www.w3.org/WAI/WCAG22/Techniques/failures/F41)
- **jekyll-redirect-from's stub** combines these pieces: `<link rel="canonical">`, an inline `<script>location=…</script>`, `<meta http-equiv="refresh" content="0; url=…">`, `<meta name="robots" content="noindex">`, and a visible "Click here if you are not redirected" link. — [jekyll-redirect-from: redirect.html](https://github.com/jekyll/jekyll-redirect-from/blob/master/lib/jekyll-redirect-from/redirect.html)

### Inferences
- **What a stub should contain**:
  - A 0-second meta refresh.
  - A canonical link to the target.
  - `noindex`.
  - A JavaScript `location.replace`, so the stub stays out of the back-button history, which an assigned `location` doesn't. It also allows carrying `location.hash`; see Fragments.
  - A visible link to the target.
  - A `<title>` and `lang`, with no other content.
- **A stub doesn't replace a server redirect**: it always returns 200, so link checkers and non-browser clients see a page, not a redirect.

### Gaps
- No first-party source quantifies how Bing or other engines treat meta refresh.
- I found no screen-reader studies of 0-second redirects.

## Fragments

### Takeaway
The fragment never reaches the server, so a server redirect can't route by fragment. Browsers carry the original fragment onto the new URL when the `Location` has none, and a `Location` with its own fragment replaces it. Sending `/old#a` and `/old#b` to different pages is possible only client-side, with JavaScript reading `location.hash` on a stub or on the target page.

### Cited Findings
- RFC 9110: "If the Location value provided in a 3xx (Redirection) response does not have a fragment component, a user agent MUST process the redirection as if the value inherits the fragment component of the URI reference used to generate the target URI." In its example, `/index.html#larry` with `Location: http://www.example.net/index.html` goes to `…/index.html#larry`. `Location: /People.html#tim` sets a fragment explicitly. — [RFC 9110 §10.2.2](https://www.rfc-editor.org/rfc/rfc9110.html#section-10.2.2)
- The Fetch standard's location URL algorithm: "If location is a URL whose fragment is null, then set location's fragment to requestFragment." — [WHATWG Fetch: location URL](https://fetch.spec.whatwg.org/#concept-response-location-url)
- The target URI "excludes the reference's fragment component, if any, since fragment identifiers are reserved for client-side processing." — [RFC 9110 §7.1](https://www.rfc-editor.org/rfc/rfc9110.html#section-7.1)
- On Cloudflare, "Fragments in the source are not evaluated," while destinations may include fragments. — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
- On Apache, a `#` in a `RewriteRule` target needs `[NE]`, or it becomes `%23` and gives a 404. The example `/anchor/xyz` goes to `/bigpage.html#xyz`. — [Apache: RewriteRule flags](https://httpd.apache.org/docs/2.4/rewrite/flags.html)

### Inferences
- **Moving a whole page**: emit no fragment in the target. Deep links like `/old#install` then land on `/new#install`, which works when the anchor IDs are unchanged.
- **Moving or renaming a section's anchor**: the old anchor ID has to be handled on the target page. A small script maps old IDs to new ones, or an empty element keeps the old ID.
- **Splitting a page**: section `#a` moving to another page needs a client-side map on the old URL. A stub reads `location.hash` and picks a target. A server rule can't do this, and on a host where a server rule exists for that path, the stub never runs. On Cloudflare, Firebase, and Vercel, redirects win over files.
- **Setting a fragment in the target**: a server target such as `/new#section` overrides any fragment the visitor arrived with. That is correct for "this page became a section of that page".

### Gaps
- None material.

## Redirect chains and loops

### Takeaway
Browsers stop at 20 redirects: the Fetch standard, Chromium, and Firefox all use 20. Googlebot follows up to 10 hops. Google advises redirecting straight to the final URL, with chains "ideally no more than 3 and fewer than 5". Each hop costs a full round trip.

### Cited Findings
- Fetch: "If request's redirect count is 20, then return a network error." — [WHATWG Fetch: HTTP-redirect fetch](https://fetch.spec.whatwg.org/#http-redirect-fetch)
- Chromium: `static constexpr int kMaxRedirects = 20;` — [Chromium source: net/url_request/url_request.h](https://github.com/chromium/chromium/blob/main/net/url_request/url_request.h)
- Firefox: `pref("network.http.redirection-limit", 20);` — [Firefox source: modules/libpref/init/all.js](https://github.com/mozilla-firefox/firefox/blob/main/modules/libpref/init/all.js)
- RFC 9110: "A client SHOULD detect and intervene in cyclical redirections." "An earlier version of this specification recommended a maximum of five redirections… some clients might implement such a fixed limitation." — [RFC 9110 §15.4](https://www.rfc-editor.org/rfc/rfc9110.html#section-15.4)
- Googlebot: "By default, Google's crawlers follow up to 10 redirect hops." Specific products may differ, and Google's Inspection Tools don't follow redirects. — [Google: HTTP status codes and Google's crawlers](https://developers.google.com/crawling/docs/troubleshooting/http-status-codes)
- Google on site moves: "Avoid chaining redirects"; redirect "to the final destination directly"; if that isn't possible, keep chains "ideally no more than 3 and fewer than 5." — [Google: Site moves](https://developers.google.com/search/docs/crawling-indexing/site-move-with-url-changes)
- MDN on performance: "as performance is impacted with every redirect, their use should be kept to a minimum."
- MDN on loops: browsers show an error ("redirecting… in a way that will never terminate"), or the server returns a 500.
- MDN on fallbacks: meta and HTTP redirects that disagree can loop.
  — [MDN: Redirections in HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Redirections)
- Host safeguards:
  - Netlify ignores rules whose from and to resolve to the same location, and warns that a slash-removal rule loops.
  - nginx stops internal rewrite cycles after 10, with a 500.
  - Render re-runs rules after an internal redirect.
  — [Netlify: Rewrites and proxies](https://docs.netlify.com/manage/routing/redirects/rewrites-proxies/); [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/); [nginx: ngx_http_rewrite_module](https://nginx.org/en/docs/http/ngx_http_rewrite_module.html); [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)

### Inferences
- **Flatten chains at build time.** Given A→B and B→C, the toolchain should emit A→C and B→C. It should detect cycles (A→B→A) and self-redirects, and report them as errors. Doing this at build time keeps every hop count at 1 for both browsers and Googlebot.
- **Check that every target exists.** Each target should resolve to a page that exists in the build, or to an external URL. Otherwise the redirect lands on a 404, or on another redirect, which makes a chain.
- **Normalize the slash form of each target.** A host that redirects for trailing slashes (Vercel, Azure, Firebase, and Netlify's Pretty URLs) adds a hop when a target is written in the non-canonical form. Each target should be emitted in the form the site publishes.

### Gaps
- I found no first-party Safari or WebKit limit; it isn't confirmed here.
- I found no first-party latency figures per hop beyond "an additional round-trip."

## One source of truth, and the lowest common denominator across hosts

### Takeaway
A list of exact `from → to` path pairs, plus one status per pair, can be compiled to every host. The portable subset is exact paths, absolute-path or full-URL targets, status 301, no query matching, no reliance on case-insensitivity, and no fragment in the source. Splats are portable only among the `_redirects` hosts, Vercel, Firebase, nginx, and Apache, and each spells them differently. GitHub Pages, and any host where rule limits are exceeded, need generated HTML stubs.

### Cited Findings
- **Status codes each host documents**:
  - Netlify: 301, 302, 404, and 200; 307 is unsupported. — [Netlify: Redirect options](https://docs.netlify.com/manage/routing/redirects/redirect-options/)
  - Cloudflare: 301, 302, 303, 307, and 308. — [Cloudflare Pages: Redirects](https://developers.cloudflare.com/pages/configuration/redirects/)
  - Vercel: 301, 302, 303, 307, and 308. — [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
  - GitLab: 301 and 302. — [GitLab Docs: Pages redirects](https://docs.gitlab.com/user/project/pages/redirects/)
  - Firebase: 301 and 302. — [Firebase: Configure Hosting behavior](https://firebase.google.com/docs/hosting/full-config)
  - Azure: 301 and 302. — [Microsoft Learn: Configure Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/configuration)
  - Render: 301. — [Render: Redirects and rewrites](https://render.com/docs/redirects-rewrites)
  - S3 object redirects: 301. — [AWS: Configuring a webpage redirect](https://docs.aws.amazon.com/AmazonS3/latest/userguide/how-to-page-redirect.html)
  - nginx and Apache: all codes. — [nginx: ngx_http_rewrite_module](https://nginx.org/en/docs/http/ngx_http_rewrite_module.html); [Apache: mod_alias](https://httpd.apache.org/docs/2.4/mod/mod_alias.html)
- **Generated rule files are supported**:
  - Vercel's bulk redirect files "can be generated at build time." — [Vercel: vercel.json](https://vercel.com/docs/project-configuration/vercel-json)
  - jekyll-redirect-from writes `redirects.json` "for automated testing or server-side redirects." — [jekyll-redirect-from README](https://github.com/jekyll/jekyll-redirect-from)
  - nginx `map` can `include` a generated file. — [nginx: ngx_http_map_module](https://nginx.org/en/docs/http/ngx_http_map_module.html)
  - CloudFront KeyValueStore is populated from S3 or the API. — [CloudFront KeyValueStore](https://docs.aws.amazon.com/AmazonCloudFront/latest/DeveloperGuide/kvs-with-functions.html)

### Inferences
- **The one status every host supports is 301.** It is the safe default for "moved". A per-pair `temporary` flag maps to 302, the only temporary code common to all hosts. 307 and 308 should be opt-in per host.
- **What to emit for each host**, from one list of `{from, to, status}`:
  - Netlify `_redirects`: `from to 301!`. Use `!` only if the toolchain also leaves a page or stub at `from`; otherwise leave it off.
  - Cloudflare `_redirects`: `from to 301`. Always write the code, since the default is 302. Check against 2,000 static and 100 dynamic rules and 1,000 characters per line.
  - GitLab `_redirects`: `from to 301`, with a leading `/`. Check against 1,000 rules and 64 KB. Never leave a file at `from`, because files win.
  - Vercel: `redirects: [{source, destination, permanent:true}]`, giving a 308. Use `statusCode: 301` for parity. Check against 2,048 rules. Above that, emit a `bulkRedirectsPath` file (Pro or Enterprise only) with `preserveQueryParams` and `caseSensitive` set explicitly.
  - Azure: `routes: [{route, redirect, statusCode:301}]`. Check that the file stays under 20 KB.
  - Firebase: `hosting.redirects: [{source, destination, type:301}]`.
  - S3: routing rules for up to 50 entries, or else one zero-byte object per `from` with `x-amz-website-redirect-location`. Those work only on the website endpoint.
  - CloudFront: a KeyValueStore of `from → to` (key at most 512 bytes, value at most 1 KB, store at most 5 MB), plus a viewer-request function of at most 10 KB.
  - nginx: a `map $uri $redirect { include redirects.map; }` block with `if ($redirect) { return 301 $redirect; }`.
  - Apache: `.htaccess` lines of the form `Redirect 301 /from /to`. Remember that `Redirect` matches by prefix, so a pair can catch child paths; `RedirectMatch 301 ^/from$ /to` matches exactly.
  - GitHub Pages and other hosts without rules: an HTML stub at `from` (`from/index.html`, or `from.html` to match the site's URL style), in the jekyll-redirect-from template's form.
- **Checks to run before emitting**:
  - Flatten chains and reject cycles.
  - Check that `from` doesn't collide with a published page. It would shadow the rule on Netlify, GitLab, and Render, and be shadowed by it on Cloudflare, Firebase, and Vercel.
  - Check that `to` exists.
  - Normalize slashes to the site's canonical form.
  - Count rules and bytes against each host's limit.
- **Leave out of the portable subset**:
  - Query-string matching (only Netlify).
  - Language, country, or cookie conditions.
  - Regular expressions.
  - Mid-path wildcards (Netlify forbids them).
  - More than one splat (Cloudflare allows one per rule).
  - Fragment-based routing, which is only possible client-side.

### Gaps
- No single source compares every host. The matrix above is built from each host's own documentation and is current as of the dates noted. Undocumented behaviors, listed in the Gaps sections above, would need testing against live deployments.
