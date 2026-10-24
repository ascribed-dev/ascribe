## Outputs
@memo: Generated from tests/conformance/diagnostics.toml by tests/conformance/tests/docs.rs; edit the registry, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs` to rewrite it.

### ASC163 `description-too-long`
@id {permalink=true}: ASC163
@summary: The project publishes `llms.txt` (`[consumer] agents = true`), and a page's description, or a content type's default for its description field, is longer than 300 characters as `llms.txt` writes it.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** The project publishes `llms.txt` (`[consumer] agents = true`), and a page's description, or a content type's default for its description field, is longer than 300 characters as `llms.txt` writes it.

**Message:** this description is \{length} characters, and its line in llms.txt reads best under \{limit}; shorten it to one sentence

**Fix:** Shorten the description to one sentence that says what the page is for. It's the page's line in `llms.txt`, which an agent reads to choose a page, so the whole index should stay short.

**Read more:** [Field roles](../content-model/index.md#54-field-roles) in the `ascribe.toml` reference.

### ASC164 `llms-section-large`
@id {permalink=true}: ASC164
@summary: The project publishes `llms.txt` (`[consumer] agents = true`), and one of a build's `llms.txt` files is longer than 50,000 characters even with the index split by folder. Reported at the page the file is about.

Advice · page level · next step: write · configurable in `[checks]` · [SPEC §9.4]({repo}/blob/main/SPEC.md#94-outputs)

**When:** The project publishes `llms.txt` (`[consumer] agents = true`), and one of a build's `llms.txt` files is longer than 50,000 characters even with the index split by folder. Reported at the page the file is about.

**Message:** \{file} is \{size} characters in build `{build}`, over the \{limit} an agent reads in one fetch; move some of its pages into folders of their own, or shorten their descriptions

**Fix:** Each folder of pages gets its own `llms.txt` once the index is too long for one file, so a file this long is one folder, or the pages outside any folder, with more pages than an agent reads in one fetch. Move some of them into subfolders of their own, or shorten their descriptions. It's the delivery spec's `llms-txt-size` check, made from the build before a deploy; `ascribe report agents` names this check for it.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC184 `listing-disallowed`
@id {permalink=true}: ASC184
@summary: A page is `unlisted`, and what the project says in `[crawlers]` keeps a search crawler from fetching it: its own lines, for every crawler or in a group that names the crawler, or a disallowed purpose.

Advice · file level · next step: fix · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** A page is `unlisted`, and what the project says in `[crawlers]` keeps a search crawler from fetching it: its own lines, for every crawler or in a group that names the crawler, or a disallowed purpose.

**Message:** this page is unlisted, and `{line}` in `[crawlers] lines` disallows its address, so a crawler never fetches it to read that it isn't to be listed; a search engine can still list the address alone

**Fix:** Remove one of the two. A crawler reads `noindex` only on a page it may fetch, so to keep a page out of search results, let crawlers fetch it: remove the `Disallow` line, or narrow it so it doesn't cover the page, or allow the purpose. If the page only needs to stay out of what crawlers fetch, remove `listing: unlisted`. The editor's quick fix removes `listing`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

### ASC190 `crawlers-disallowed`
@id {permalink=true}: ASC190
@summary: `[crawlers]` disallows `search` or `on-request`, and the site's pages are listed unless they say otherwise.

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §7.2]({repo}/blob/main/SPEC.md#72-declarations)

**When:** `[crawlers]` disallows `search` or `on-request`, and the site's pages are listed unless they say otherwise.

**Message:** `{purpose} = "disallow"` keeps AI search products from indexing the site, so its pages aren't in their answers, while the pages are listed for every other search engine

**Fix:** Choose what the site wants. To be in AI search answers, and fetched when a person asks, `allow` the purpose or remove it. To keep it, turn this check off in `[checks]`, saying so in a comment: the choice is the project's, and this only says what it costs. A purpose's group asks only the crawlers Ascribe knows, and some of them say they may not follow `robots.txt`.

**Read more:** [`[crawlers]`](../content-model/checks.md#21-crawlers) in the `ascribe.toml` reference.

## Prose, through Vale

### ASC165 `prose`
@id {permalink=true}: ASC165
@summary: The project has `[checks.vale]`, and Vale reports an alert about the prose of a page or a fragment, or of a phrase's text. Reported where the alert is in the source, or at the phrase.

Advice · file level · next step: write · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** The project has `[checks.vale]`, and Vale reports an alert about the prose of a page or a fragment, or of a phrase's text. Reported where the alert is in the source, or at the phrase.

**Message:** \{rule}: \{message}

**Fix:** Vale reported this about the prose, under the rule the message starts with. Change the text, or take the replacement the editor offers when the rule gives one; for text from a phrase, change the phrase's value in `ascribe.toml`. A rule that's wrong for the project is turned off in Vale's configuration, or, for the `quiet` preset, in `[checks.vale] off`; one place is quieted with Vale's own comments (`<!-- vale Rule = NO -->`). See [Prose, through Vale](../checks/vale.md).

**Read more:** [`[checks.vale]`](../content-model/checks.md#checksvale) in the `ascribe.toml` reference.

### ASC166 `prose-not-checked`
@id {permalink=true}: ASC166
@summary: The project has `[checks.vale]`, and Vale couldn't be run, failed, or didn't finish in time, or the tool checking the project doesn't start it, as the agents' tools don't. Reported once, at `[checks.vale]`.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §2.1]({repo}/blob/main/SPEC.md#21-files)

**When:** The project has `[checks.vale]`, and Vale couldn't be run, failed, or didn't finish in time, or the tool checking the project doesn't start it, as the agents' tools don't. Reported once, at `[checks.vale]`.

**Message:** the prose wasn't checked: `{command}` couldn't be run (\{reason}). Install Vale, or set `[checks.vale] command` to where it is

**Fix:** Install [Vale](https://vale.sh) 3.16 or later so that the command `[checks.vale] command` names (`vale` by default) runs, or fix what Vale's own message says about its configuration. Nothing else is checked differently while it can't run. A project that checks prose only in CI can turn this off on other machines with `prose-not-checked = "off"` in `[checks]`. The agents' tools never start Vale; the agents' hook never starts a `command` that's a program in the project, and the editor asks before it does. Run `ascribe check --vale` in a terminal to check the prose then.

**Read more:** [`[checks.vale]`](../content-model/checks.md#checksvale) in the `ascribe.toml` reference.

## External links

### ASC168 `link-external-broken`
@id {permalink=true}: ASC168
@summary: `ascribe report links`: the link checker found that an external link answers with an error, doesn't answer in time, or can't be reached.

Advice · file level · next step: review · configurable in `[checks]` · acknowledged with `@intended` above the block, or under the heading for a heading · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: the link checker found that an external link answers with an error, doesn't answer in time, or can't be reached. An address that answers with a server error (`5xx`) is asked about again after a short wait, and reported only when it fails again. A host `[checks.links] ignore` names isn't checked.

**Message:** `{url}` answered \{status}, so the link is broken for readers

**Fix:** Open the address. When the page is gone, link to where its content went, or remove the link. A site that turns link checkers away, or one that's down for a while, may be fine as it is: acknowledge the link with `@intended {check=link-external-broken}: <why>` directly above its block, or leave the whole host out with `[checks.links] ignore`. Only `ascribe report links` checks external links; `ascribe check` never reaches the network.

**Read more:** [`[checks.links]`](../content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

### ASC169 `link-external-moved`
@id {permalink=true}: ASC169
@summary: `ascribe report links`: an external link's address redirects permanently (`301` or `308`) to another one that answers. A site's root that redirects to another of its own pages, a landing page, hasn't moved.

Advice · file level · next step: choose · configurable in `[checks]` · [SPEC §5.2]({repo}/blob/main/SPEC.md#52-links)

**When:** `ascribe report links`: an external link's address redirects permanently (`301` or `308`) to another one that answers. A site's root that redirects to another of its own pages, a landing page, hasn't moved.

**Message:** `{url}` moved permanently to `{to}`; link to the new address

**Fix:** Link to the address it redirects to; the fix writes it. Check first that it's the page you meant: some sites send every old address to their home page, and then the link needs a new target, not this one. Only `ascribe report links` checks external links.

**Read more:** [`[checks.links]`](../content-model/checks.md#checkslinks) in the `ascribe.toml` reference.

## The published site

### ASC170 `delivery-hosting`
@id {permalink=true}: ASC170
@summary: `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that its hosting decides, such as status codes, caching, or content negotiation.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that its hosting decides, such as status codes, caching, or content negotiation.

**Message:** the site fails the delivery spec's `{check}` check: \{result}. \{setting}

**Fix:** Change the setting the message names where the site is hosted; no page needs editing. [Serve the Markdown versions](../astro/serve-markdown.md) says what to set on the hosts it names. Only `ascribe report agents` checks a published site.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

### ASC171 `delivery-output`
@id {permalink=true}: ASC171
@summary: `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that `llms.txt`, the Markdown pages, or the pointer on each page should pass, which Ascribe writes with `[consumer] agents = true`.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §9.5]({repo}/blob/main/SPEC.md#95-consumer-profile)

**When:** `ascribe report agents`: the published site fails, or warns on, a check of the Web Documentation Delivery Spec that `llms.txt`, the Markdown pages, or the pointer on each page should pass, which Ascribe writes with `[consumer] agents = true`.

**Message:** the site fails the delivery spec's `{check}` check, which what Ascribe writes should pass: \{result}. Report it at \{issues}

**Fix:** First check that the site publishes what the build wrote, as it wrote it: `llms.txt` and the Markdown pages from the plain output, and each page's pointer from the site output. When it does, this is a bug in Ascribe: report it at the issue tracker the message names, with the report's output. No page needs editing.

**Read more:** [`[consumer]`](../content-model/output.md#consumer) in the `ascribe.toml` reference.

## Tested examples

### ASC199 `example-stale`
@id {permalink=true}: ASC199
@summary: A `@snippet` shows a tested example whose file has changed since the observation that lists it was made.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** A `@snippet` shows a file that its source's `ascribe.examples.toml` makes an example, and the file has changed since the observation that lists it was made. For a source in another repository, the config, the observation, and the file are the copies `ascribe sources fetch` made. Reported once for each snippet of the file, whichever region it shows.

**Message:** `{path}` changed after its last test run, so the page doesn't say what it was tested with; run `ascribe observe` in `{folder}` to test it again

**Fix:** Run `ascribe observe` in the folder named, where `ascribe.examples.toml` is, and commit the observation it writes. It runs the folder's test command, and writes the observation only when the command passes. An observation covers a whole file: an edit anywhere in it, even outside the region a page shows, needs a new test run. For a source in another repository, that's done in the code's repository, and `ascribe sources update` copies the new observation here; nothing in this repository runs the test.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC200 `example-unobserved`
@id {permalink=true}: ASC200
@summary: A `@snippet` shows a file that its source's `ascribe.examples.toml` makes an example, and no observation lists the file. For a source in another repository, the config and the observation are the copies `ascribe sources fetch` made. Reported once for each snippet of the file.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §4.8]({repo}/blob/main/SPEC.md#48-snippet)

**When:** A `@snippet` shows a file that its source's `ascribe.examples.toml` makes an example, and no observation lists the file. For a source in another repository, the config and the observation are the copies `ascribe sources fetch` made. Reported once for each snippet of the file.

**Message:** `{path}` is an example, and no test run has covered it, so the page doesn't say what it was tested with; run `ascribe observe` in `{folder}`

**Fix:** Run `ascribe observe` in the folder named, where `ascribe.examples.toml` is, and commit the observation it writes. When the file isn't meant to be an example, add it to the config's `ignore`, or narrow its `include`. For a source in another repository, both are done in the code's repository, and `ascribe sources update` copies the result here.

**Read more:** [`@snippet`](../writing/directives.md#snippet) in the directive reference.

### ASC201 `examples-config-invalid`
@id {permalink=true}: ASC201
@summary: A source has an `ascribe.examples.toml`, or an observation beside it, that can't be read: in its folder, or, for a source in another repository, in its copies. Reported at the source in ascribe.toml, once for each problem in each file.

Warning · file level · next step: outside · configurable in `[checks]` · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** A source has an `ascribe.examples.toml`, or an observation beside it, that can't be read: in its folder, or, for a source in another repository, in its copies. Reported at the source in ascribe.toml, once for each problem in each file.

**Message:** the examples of source `{source}` can't be checked: `{file}` can't be read: \{detail}

**Fix:** Fix the file named, in the source's folder: `ascribe.examples.toml` is written by the code's owners, and an observation by `ascribe observe`, which writes it again when it's run there. Until it can be read, no snippet of the source says what it was tested with.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.

### ASC202 `observation-file-missing`
@id {permalink=true}: ASC202
@summary: An observation beside the `ascribe.examples.toml` of a source in the same repository lists a file that isn't in the source's folder. Reported at the file's line in the observation, once for each file.

Advice · file level · next step: outside · configurable in `[checks]` · [SPEC §7.3]({repo}/blob/main/SPEC.md#73-sources)

**When:** An observation beside the `ascribe.examples.toml` of a source in the same repository lists a file that isn't in the source's folder. Reported at the file's line in the observation, once for each file.

**Message:** `{observation}` lists `{path}`, which isn't in `{folder}`; run `{command}` there to write the observation again

**Fix:** The example was usually deleted or renamed without a second test run. Run the command named, in the folder named, and commit the observation it writes, which lists the examples as they are. When nothing writes the observation any more, delete its file.

**Read more:** [`[sources.<name>]`](../content-model/output.md#18-sourcesname) in the `ascribe.toml` reference.
