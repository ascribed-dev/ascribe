# Brainstorm: what Ascribe could do next

Ideas for Ascribe that nobody has built, planned, or started. This is exploration, not a plan: nothing here is decided. Each idea says what it is, how it fits Ascribe, what's hard about it, and the questions to answer before designing it.

Written on 2 October 2026, after 0.1.1, and cut back on 11 October 2026 to what's still open. Most of what it held then has been built, and the rest of it has a plan or a proposal of its own, so it's gone from here:

- **Built.** The guides say what exists: the [editor](../docs/content/editor/index.md), [agents](../docs/content/agents/index.md) that write docs and the outputs agents read, [review](../docs/content/review/index.md), [snippets and drift](../docs/content/examples/index.md), [Vale](../docs/content/checks/vale.md), [reports](../docs/content/checks/report.md) (the inventory, overdue reviews, and external links), [permalinks](../docs/content/permalinks/index.md), [page templates](../docs/content/templates/index.md), [tested examples](../docs/content/examples/tested.md), [memos](../docs/content/writing/memos.md), [versions](../docs/content/versions/index.md), [navigation](../docs/content/navigation/index.md), [the glossary as pages](../docs/content/content-model/glossary.md), the editor's [sidebar](../docs/content/editor/sidebar.md), [search](../docs/content/command/questions.md#ascribe-search), and what a search engine and a crawler are told, in [Publishing](../docs/content/publishing/index.md).
- **Proposed.** [The rest of the cache](proposals/cache.md).

The sections keep the numbers they had, since other documents name them, so the numbers have gaps.

Three groups of ideas:

- [Product features](#product-features): what's left of snippets and Vale, editable previews, OpenAPI, redirects, and other things a docs system does, embedded help, what the landing page showed about widgets, what's left of the editor's views, a page in any output from the command line, and passages written twice.
- [Beyond the pages](#beyond-the-pages): what's left of the cache.
- [Paid features](#paid-features): what could be charged for, if Ascribe ever goes that way. A list to keep, not a plan.

## What the ideas are judged against

- **Markdown stays the source of truth.** SPEC principle 1: source readability wins. A feature that writes source must write the source a person would.
- **Everything is checkable.** The content model declares what's allowed, and mistakes are diagnostics, in the editor as you type and in CI.
- **One model, many outputs.** Builds, variants, and availability make different documentation sets from the same source.
- **Projects are self-contained.** A project's files stay inside its folder, and a workspace can hold several projects.
- **The language server is the engine.** The editor, the preview, and the CLI share it, so a capability added to the server reaches all of them.

---

# Product features

## 1. Code snippets: what's left

`@snippet`, sources, and `ascribe drift` are built ([Examples and drift](../docs/content/examples/index.md)), and Ascribe's own docs take their examples that way. Knowing whether an example's code was tested, and with what, is built: [tested examples](../docs/content/examples/tested.md). So is part of moving between a page and its code: a code file says which pages show it, in a lens, and a `@snippet`'s hover gives its address ([the editor](../docs/content/editor/writing.md#code-files-a-page-shows)). What's left:

- **The rest of Bluehawk's tags.** `snippet` and `remove` work as they are. `replace`, `uncomment`, `state`, and `emphasize` are reserved and do nothing. States are the interesting one: versions of one example, such as a tutorial's start and finish, which map onto Ascribe's own model as `@snippet {state=start}` inside a `@variant` arm, or a build that picks a state for a whole tutorial. The tested examples plan leaves new tags to a proposal of their own, designed fresh for a project starting clean, so whether to finish Bluehawk's set is a question about who's migrating.
- **The rest of the editor's part, filed:** the preview and a page's problems following an edit to the code ([#791](https://github.com/ascribed-dev/ascribe/issues/791)), Go to Definition from a `@snippet` to its region and the code in its hover ([#792](https://github.com/ascribed-dev/ascribe/issues/792)), and the tags' problems shown in the code file ([#793](https://github.com/ascribed-dev/ascribe/issues/793)).

## 2. Editable previews

### The idea

Edit in the preview instead of in Markdown, with changes written back to the source file.

### The risk

A general WYSIWYG editor that serializes back to Markdown tends to produce source no one wants to read or review: escaped characters, reflowed lines, reordered attributes. That breaks principle 1. So build it in layers, each useful on its own, and stop before a full rich-text editor unless writers ask for one.

### Layers

1. **Structural editing.** Click an element in the preview to change it with a small control:
   - a note's type, from a menu;
   - the order of steps or tabs, by dragging;
   - an availability badge, in a small form ("cloud, self-hosted beta 2.4");
   - whether a `@details` is open, a tab's label, an image's width.

   Each action becomes a source edit sent through VS Code, so undo, the dirty marker, and git behave as for typing. Most of the parts exist: the preview already maps elements to source positions (it scrolls to the cursor), the server already makes edits (code actions, rename), and `fmt` keeps the result canonical.
2. **Insert from a palette.** Add a note, steps, a tab group, an include, or a snippet from a menu in the preview, written in canonical form at the right place. This removes most of the need to remember directive syntax.
3. **Text editing in place.** Double-click a paragraph, heading, or list item to edit its text, limited to plain text and inline marks, mapped back to that block's source range.
4. **A full rich-text editor** (ProseMirror or similar). Only if writers need it; the hard part is a lossless mapping between the editor's model and Ascribe's syntax tree.

### Questions

- Who would use it: writers who avoid Markdown, or everyone, for structure?
- How should a preview edit and a concurrent edit in the text editor reconcile? (VS Code's document versions help; the server already rejects stale versions.)

## 3. OpenAPI

### Levels

1. **Checked links into the spec.** `[Get a user](../api/openapi.yaml#getUser)`, checked like heading links, with completion of operation ids and hover showing the operation. Cheap and immediately useful.
2. **Embeds.** `@openapi-operation: getUser` in a hand-written page renders the method, path, parameters, request and response schemas, and examples. Writers keep the narrative; the spec supplies the facts.
3. **Generated reference pages.** Virtual pages, one per tag or operation, built from the spec at build time and never written to disk, with hooks for authored prose around them.

### What Ascribe could do that other tools don't

- **Check examples against schemas.** JSON in a page's code blocks that claims to be a request or response is validated against the spec, so a stale example is an error.
- **Coverage.** Report operations no page documents.
- **Versions.** Map API versions onto builds and variants; availability from a spec extension such as `x-ascribe-available`.

### Questions

- Embedded rendering needs elements in `@ascribed/elements` (operation, schema, example), and an Astro story.
- Start with OpenAPI 3.1 only? GraphQL and AsyncAPI would follow the same design later.
- Generated pages raise navigation and URL questions (see [navigation](#navigation-and-site-structure)).

## 5. Vale: what's left

Ascribe runs Vale on a page's prose, with phrases substituted and directives left out, reports each alert at its place in the source, and writes the project's own words into a Vale vocabulary ([Vale](../docs/content/checks/vale.md)). Two things from the first idea aren't built:

- **Linting what a build's readers see.** The text with variants and availability resolved, for a team whose rules should apply to an output and not to the source. Nobody has asked for it.
- **Keeping results between runs,** by the hash of a page's prose, so a hook or a check lints only what changed. The [cache proposal](proposals/cache.md#vale-results) designs it, and holds it until a project's full Vale run is slow enough to matter (its Settled 4).

## 6. Other things a docs system does

Review dates, the inventory, external link checking, and the checks for unused and oversized images are built. These aren't.

### Navigation and site structure

Built: [navigation](../docs/content/navigation/index.md). Sections are folders, their order and their pages' order are in a file beside `ascribe.toml`, and the editor reorders by dragging.

### Redirects

Renaming or moving a page updates the links to it, and a permalink gives a product an address that doesn't move. Neither helps a bookmark or a link from another site to a page's old address. Ascribe could record a redirect from the old published address and emit it with the site. The Web Documentation Delivery Spec asks for same-host redirects when content moves.

It needs something Ascribe doesn't have: a record of what was published before, read from an earlier run. Two finished or planned pieces of work stopped at that seam on purpose. What a crawler is told is written without it, so a sitemap has addresses and no dates. And the versions plan's `retire` reports each old address and where it should go, as a list, and writes nothing for a host. Redirects would take that list as their first input.

### Differences between builds

"What does the self-hosted build leave out compared with the site build?" A report of pages, sections, and variant arms for each build, for release reviews. The build machinery already computes the answer.

The versions plan has the nearest thing, `ascribe versions changes`, which says what differs between two lines of one target. It doesn't compare two builds that differ in any other way.

### Translation

A translation's status for each page, from a hash of the source: a translation is marked stale when the source page changes, with the changed sections listed.

### A PDF

A PDF or print output of a build. The versions plan gives an archive a print stylesheet and one document with every page, so a browser can make a PDF of a frozen version. Nothing does it for a live site.

## 17. Embedded help

Added 2026-10-09. Not planned. It builds on permalinks, which are built since ([Permalinks](../docs/content/permalinks/index.md)).

### The idea

A permalink lets a product link to a section of its docs by a name that doesn't change. Embedded help goes one step further: the product shows the section's text itself, in a tooltip, a side panel, or beside an error, without sending anyone to a browser.

Help-authoring tools have had this for years. MadCap Flare calls it micro content: short answers written once and shown in the product, in search results, and in the docs.

### How it fits Ascribe

Most of the parts exist:

- **A stable name for the content:** a permalink.
- **The content, resolved and structured:** the JSON output has every page as blocks, and the plain output has it as Markdown, per build, with phrases, variants, and availability already applied.
- **The right version of it:** a build filters for a target and version, so a product at 3.4 can be given the 3.4 text.

What's missing is the join: an output keyed by permalink, holding each named section's text in a form a product can show. A product would bundle that file at build time, or fetch it from the docs site.

### What Ascribe adds that a help-authoring tool can't

- **It's the same source as the docs,** so the tooltip and the page can't disagree.
- **It's checked.** A product asking for an id that doesn't exist fails `ascribe permalinks verify` in its own CI.
- **It's per version and per variant** without a second copy.

### Hard parts and questions

- **Size.** A section written for a docs page is usually too long for a tooltip. Either authors mark a short form (the section's first paragraph, or a summary of its own), or the product truncates, which reads badly. Does this need a construct in the language, or is "the first paragraph" enough?
- **Format.** A product can't render Ascribe's custom elements. The plain Markdown output is the safe form, and loses notes' styling and tabs.
- **Bundled or fetched?** Bundled text is as old as the product's build, and works offline. Fetched text is current, and needs the docs site to be reachable and to allow the request. Ascribe hosts nothing, so fetching means a static file on the docs' own host.
- **Links inside the text** point at docs pages. In a product they'd need to be absolute, as the plain output's already are when `[consumer] site` is set.
- **Who is it for?** A product team has to build the part that shows the text. Ascribe can supply the file and a small example of reading it, and no more.

---

## 18. Widgets for a page's layout

The landing page (`site/home/content/index.md`) was the experiment of the [website plan](https://github.com/ascribed-dev/ascribe/blob/e80d764ef544f7fb617dec8084248a3e161a6d23/project-docs/website/README.md): whether a handful of project widgets carry a marketing page without changing the language. **They carried part of it.** Three widgets did: the hero, a claim with its example, and a page's source beside its result. The lists, the install commands, and the status needed none, and an availability mark inside a claim worked as it does on any page.

What didn't fit:

- **A widget can't name a page.** `@source-and-result {page=install}` says which page to draw with a string attribute. Ascribe doesn't know it's a page, so it doesn't check that it exists, follow it when the page is renamed, or count it as a use; the site's own test does. A page-reference attribute type, checked as a link is, would close that gap. It's a change to SPEC §7.2's attribute types.
- **The site draws another project's page inside a widget by cutting HTML.** The widget's element holds only the source; the route finds the element's end tag in the page's rendered HTML and draws the other page before it (`site/src/demo.ts`). It works because widgets don't nest and the element's name is unique, and it's the one place the site edits Ascribe's HTML rather than styling it.
- **A widget's title isn't a heading.** A title line becomes the element's `heading` attribute, which a stylesheet can show but which isn't in the page's outline or table of contents. Each claim's name is a `###` heading inside the widget instead.
- **The drawn page's headings join the page's outline.** The demonstrated page's `##` headings are the landing page's too, inside the demonstration. The stylesheet makes them smaller; nothing makes them a level lower.

## 19. One selector for a whole site

Added 2026-10-11. Not planned.

A reader choosing "Cloud" or "Self-managed 3.5" is choosing between a target with no versions and a version of another. That's one case of a wider need: choosing a platform, an environment, a framework, or a language, and having the whole site hold that choice.

Ascribe has the parts. Dimensions name the choices, variants are shown as tabs, and a reader's choice of tab holds across pages. What it doesn't have is one design for selectors and filters across a site: which choices are offered where, what a page that doesn't vary shows, and what an address says about the choice.

The [versions plan](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/versions/README.md) carves it out. Its list of a page's lines is for one target, and is built so a general selector can take it over.

## 20. `ascribe.toml` in more than one file

Added 2026-10-11. Not planned.

TOML has no include, and everything in the content model is in one file. The table that most needed splitting was the glossary, and the [glossary proposal](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/proposals/glossary-pages.md) moves that into pages instead. `[phrases]`, `[features]`, and the `[[intended]]` acknowledgements can grow long too.

Two designs are usual. A key that names a file for one table (`[phrases] file = "phrases.toml"`) is narrow and easy to explain, and each table that wants it needs its own key. A general `include` at the top is one mechanism for everything, and needs rules for a key set in two files.

Either touches the editor, whose diagnostics and hovers assume one file, and every command that edits `ascribe.toml` in place, which would have to know which file to write to.

## 21. The editor's views: what's left

Added 2026-10-18, from what the [editor's views plan](https://github.com/ascribed-dev/ascribe/blob/395fe80d/project-docs/editor-ui/README.md) left for later. Not planned.

- **A webview for a view.** Answered for This page: it becomes a webview, and the other views stay trees, in its [proposal](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/proposals/this-page-webview.md). Whether the Review view follows is the question left, once This page has been used.
- **A drop handler that asks for alt text.** Dropping or pasting an image into a page is VS Code's Markdown extension's, which writes the image's link with no alt text, where **Insert an image** asks for it. A handler of Ascribe's own would ask as that action does, and put the image where the project keeps its images. [Decision #544h](decisions/544h-editor-leaves-out.md) holds it back until what VS Code's drop writes fails `ascribe check`, which nobody has yet checked by hand: drop and paste an image, and drop a page, into a page of `examples/quill` with the settings at their defaults, then run `ascribe check`.

## 22. A page in any output, from the command line

Added 2026-10-20. Not planned.

`ascribe render` writes one page as a build's plain output writes it, and [#665](https://github.com/ascribed-dev/ascribe/issues/665) renames it `ascribe resolve`, since what it does is SPEC §9's resolution, then the plain emitter. The rename leaves room for what the old name promised: a page in the other outputs, without building the project.

- **`--emit site`:** the page as the site output writes it, Markdown with Ascribe's elements (`<ascribe-note>`, `<ascribe-tabs>`, availability badges), as `@ascribed/astro` receives it. The option is `ascribe build`'s own word, so `ascribe resolve PAGE --emit plain` is today's answer and `--emit site` the new one. It shows what a site's integration is given, which the plain output can't: what a switcher, a badge, or a glossary link becomes before a site draws it.
- **`--emit json`:** the page as the JSON output writes it, blocks and all, for a tool that reads one page.
- **HTML,** from `render_site_html`, the renderer the editor's preview uses and the Astro plugin is held to: the page's content as HTML, without a site's frame, so an agent or a test can see what the elements produce without a browser. It's the one case where "render" was the right word, and it might be a command of its own rather than an output, since no build writes HTML.
- **For agents:** an `emit` argument on `ascribe_resolve`, not a tool of its own, since a tool takes its command's arguments under the same names (the command checklist). `site` first: it's the view an agent without a shell can't get otherwise, of what a badge or a switcher becomes. `json` only if a use turns up, since the tool's JSON would hold the page's JSON in a field, and HTML not at all, since it costs an agent far more than the Markdown it came from and says nothing the site output doesn't. With `emit: site`, the tool's default text is the site output's Markdown. An argument added to a tool is still a recorded decision (decision 56), though no caller breaks.

What it would serve: the [docs audit](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/docs-audit.md)'s "where a screenshot would do the work" question, which needed a browser to see tabs and badges; an agent checking what a reader of the site gets, not only a reader of the Markdown version; and tests of a site's integration that start from one page.

Questions:

- **Is HTML an output or a separate thing?** Every other `--emit` value is an output a build writes. HTML isn't, so `--emit html` would make the option mean two things.
- **The JSON document.** The command's JSON wraps the page's text in `text`; a site or JSON page in it needs a field that says which output it is, which decision 4 lets the document add.
- **Elements without their script.** The site output's elements are custom elements that `@ascribed/elements` defines. Their HTML is readable without it, but tabs show every panel at once, as a page without JavaScript does.

## 23. Passages written twice

Added 2026-10-20, from the [docs audit](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/docs-audit.md). Not planned.

The audit found a dozen rules stated three to six times across Ascribe's own pages, and option paragraphs that say again what the generated tables under them say. It found them by reading and with a script: no tool says "this paragraph is on three pages". A writer who repeats a passage instead of including a fragment gets copies that drift apart, and a reader gets the same rule in three wordings.

Ascribe can find this where a prose linter can't, because it knows every page's blocks as a build publishes them. `ascribe diff` already fingerprints each block's content without its position (`crates/ascribe-diff/src/tree.rs`), and the [search plan](https://github.com/ascribed-dev/ascribe/blob/62bf76a9/project-docs/search/README.md)'s reading gives each block's text with its place in the source. A report of blocks whose text is the same, or nearly, on more than one page, each with its places, would point at the feature built for the problem: a fragment, written once and included where it's needed.

- **A report, not a check.** Some repetition is right: a warning that belongs on every page it applies to, or a sentence a page needs to stand alone. A diagnostic on each copy would push writers to fragment everything. A section of `ascribe report` that lists what's repeated, largest first, leaves the judgment to the writer.
- **What's the same.** Exact matches are cheap with fingerprints. Near matches, a rule reworded, are what the audit mostly found, and need a measure of similarity: the shingles a near-duplicate detector uses, over the reading's text. Exact first, and near only if exact finds too little.
- **What isn't a copy.** A fragment's text on the pages that include it is one passage, not several. Text a phrase fills in is the phrase's. The reading already knows both, as search's results do.

Questions:

- **The threshold.** How long must a passage be to count? A sentence repeated is often fine; a paragraph is the case the audit meant.
- **A fix.** The report could offer to move a passage into a fragment and include it in each place, which the editor's actions could do one passage at a time. Whether that's worth building depends on how often the copies are really the same.

---

# Beyond the pages

## 15. The cache: what's left

This section held three ideas that share no layer. Two are built: [memos](../docs/content/writing/memos.md), for what people know about a page and have nowhere to put, and [search](../docs/content/command/questions.md#ascribe-search), which was measured and needs no index.

The third was a cache, and its first use is built: `ascribe report links` keeps the answers it reuses under `.ascribe/cache/links/`, with an expiry, and holds a link that answered before until it keeps failing. The rule it's built on stays the rule for anything added: deleting the cache changes nothing but time, so nothing may need it to be correct. CI starts with none.

The rest has a proposal of its own, [the rest of the cache](proposals/cache.md), and its first steps are filed: differences between runs for the whole report ([#787](https://github.com/ascribed-dev/ascribe/issues/787), [#788](https://github.com/ascribed-dev/ascribe/issues/788)), and a baseline of a project's existing advice, a committed record rather than a cache ([#789](https://github.com/ascribed-dev/ascribe/issues/789), [#790](https://github.com/ascribed-dev/ascribe/issues/790)). Vale's results wait for a project that needs them, and the delivery spec's turned out not to fit a cache.

---

# Paid features

## 16. What could be charged for

Added 2026-10-09. A place to collect ideas, so they're on record if the question is ever opened. Nothing here is planned, and the research is against starting now: [Ascribe fit for docs pain points](reports/Ascribe%20fit%20for%20docs%20pain%20points.md) concludes that a paid tier is premature and its price ceiling low.

### What an idea is judged against

- **Everything local and deterministic stays free:** every check, snippets, the rendered diff, the report, the agent interfaces. Charging for any of them means maintaining two editions, and they're what would earn Ascribe its users.
- **The paid unit is what a file in a repository can't provide:** state, identity, and compute that someone else runs. The report found that the open-core services that last charge for something metered, stateful, or heavy that customers can't easily host themselves.
- **Ascribe hosts nothing today,** and calls no model. A paid service reverses the first, on purpose.
- **Holding a company's private content has fixed costs** that don't depend on the size of the service: becoming a data processor, security reviews, and a first SOC 2 audit quoted at $25,000 to $50,000.
- **The ceiling is low.** The nearest comparison charges $29 an organization a month. Reviewers and readers are free everywhere.

### The list

| Idea | What a team would pay for | The catch | From |
|---|---|---|---|
| Guest reviewers | People without a GitHub account or a checkout reading a change as pages and commenting | The market prices reviewers at zero | Review; the report |
| Hosted previews with comments | A deployed preview of each pull request, with the review overlay, and no local setup | Netlify and Vercel give this away for the page; only the resolved diff and block anchors are Ascribe's | Review |
| Sign-off | A required status check that a named person approved the rendered change, for AI-written pull requests | The one competitor moved to exactly this pitch, which suggests review alone didn't sell | The report |
| Notes for people outside git | Subject experts and support leaving notes on published pages, visible to the team at once | Small audience; committed notes already serve teams in git | Section 15 |
| Project history | Check and report results kept across CI runs: trends, "new since last week", a dashboard | A CI artifact and a committed file get most of the way for free | Section 15; the content checks plan |
| Scheduled link and site checks | External links and the delivery spec checked on a schedule, with alerts | A scheduled workflow in the team's own CI does this already | The content checks plan |
| A docs service for agents | A hosted endpoint that answers an agent's question with a small, correct piece of the docs for the product and version in use | The most metered and stateful idea here, and the least proven; see the agent-first docs experiment | The agent-first docs research |
| Proposed doc updates | An agent that drafts changes when code or an API changes | Needs a model, so it has a cost per use and no published accuracy; a team's own agent can do it through Ascribe's commands at no cost to anyone | The report |
| Single sign-on and audit logs | What a company's security team requires before buying anything above | Only exists if something above does | The report |
| Support or a membership | Priority answers, a say in what's next, a named contact | Not a product; income tracks the maintainer's hours | The report |

### What the list suggests

- **Most of these are thin.** Each row's catch is usually "the free way already works for teams in git". The ideas with real substance are the ones that bring in people who aren't in git (guest reviewers, notes for experts) and the one that serves agents rather than authors.
- **A docs service for agents is the outlier:** metered, stateful, hard to host yourself, and aimed at a need that's growing. It's also the furthest from what Ascribe is today, and unmeasured.
- **Support or a membership is the cheapest experiment,** since it needs no service at all.

### Questions

- Is the buyer a docs team, or the engineering team that owns the product?
- Would a hosted service be a separate product built on Ascribe, rather than a tier of it?
- What number of teams using Ascribe on private repositories would make this worth opening? The report's answer is "more than none", which is where things stand.

---

# Suggested order

Ahead of everything here are the three plans in progress and the four proposals named at the top. Navigation is the one of those with the most waiting on it: the docs' own sidebar, `llms.txt`'s sections, the archive's way around, and generated pages each work around its absence. Of what's left in this document:

1. **Redirects** (section 6), once something records what was published. The versions plan's `retire` gives it a first input.
2. **OpenAPI links and embeds** (section 3, levels 1 and 2).
3. **Structural editing in the preview** (section 2, layers 1 and 2).
4. **Embedded help** (section 17), which needs only an output keyed by permalink.

# Open questions

- **Who are the main users?** Writers on a docs team favor preview editing and the features of a docs system; engineers documenting their own code favor snippets, OpenAPI, review, and agent support. The research found engineers feel drift most and do the reviewing.
- **Bluehawk's remaining tags:** finish the set for teams migrating, or leave them reserved and design fresh ones with tested examples?
- **Should Ascribe apply more edits for an agent?** `ascribe check --fix` applies the fixes marked safe. Everything else returns edits for the harness to apply with its own approvals. The next candidate is a `rename` that applies its changes, with a dry run.
- **Review beyond a checkout:** is a hosted review service (guest reviewers, deployed previews) ever in scope, and would it be paid? Section 16 lists it with the other ideas.
- **Review hosts:** GitHub only, or GitLab too?
- **The delivery spec is a draft.** Ascribe follows 0.6.0. Its version is pinned, and its changes need watching.
