# Proposal: embedded help

A proposal, not a plan. It says what would be built and in what order, so the idea can be judged before anything is designed in detail. It picks up section 17 of the [brainstorm](../brainstorm.md#17-embedded-help). The plan for building it is [Embedded help](../embedded-help/README.md).

## The goal

A product shows a section of its docs inside itself, in a tooltip, a side panel, or beside an error, by the same permalink it already links to. The text is the docs' own, for the product's version, so the tooltip and the page can't disagree, and a product that asks for a name the docs don't have fails its own CI.

Ascribe supplies a file and says how to read it. It hosts nothing, ships no widget, and calls no model: the product's team builds the part that shows the text.

## What exists

- **Permalinks** (SPEC §2.1, §4.1): `permalink: options` on a page, `@id {permalink=true}: weave-config` on a heading. A build lists them in the site output's `_ascribe/permalinks.json`, each with its route, fragment, and source (output-layout contract §1.2), and `@ascribed/astro` publishes an address for each, `go/<name>/`, that forwards to it.
- **A product's CI checks its names** against the committed `ascribe.permalinks` with `ascribe permalinks verify --against … --from …`, and a removed permalink is a warning in the docs (`permalink-removed`) until it's retired.
- **The plain output** (SPEC §9.4) is each page resolved to CommonMark with no HTML: phrases substituted, includes and the glossary resolved, availability written as text, a switcher's arms as labeled sections, and links absolute when `[consumer] site` is set.
- **A build is already per version and per variant:** a versioned build writes one output per live line (`site`, `site@3.4`), and a build's `variants` and `availability` select what it keeps.
- **A page's description** has a role (`description`), and the page records (`_ascribe/pages.json`) carry it as plain text.

What's missing is the join: the text of each permalink's section, on its own, keyed by its name.

## The design

### One file per build, keyed by permalink

Each build writes `_ascribe/help.json` in its site output's root, beside `permalinks.json`: every permalink the build publishes, as `permalinks.json` lists them, with the text a product shows. It's written for every build, with an empty table when there are none, as `permalinks.json` is.

```json
{
  "version": 1,
  "build": "site",
  "line": "3.5",
  "help": {
    "weave-config": {
      "title": "Configure the weaver",
      "summary": "The weaver reads `weave.toml` from the project's root, or the file `--config` names.",
      "text": "The weaver reads `weave.toml` …\n\nSee [the options](https://docs.example.com/docs/reference/options/#weave).\n",
      "address": "https://docs.example.com/docs/go/weave-config/",
      "source": "guides/weaving.md"
    }
  }
}
```

| Field | Meaning |
|---|---|
| `version` | The format's version, `1`. A reader rejects versions it doesn't know; a later version may add keys, and doesn't change these. |
| `build`, `line` | The build, and its release line when it's a line of a versioned build, so a product can check it bundled the docs for its own version. `line` is absent for a build that isn't versioned, as `address` is without `site`: a key the build has nothing for is left out. |
| `help.*.title` | The heading's text, or the page's title, as plain text. |
| `help.*.summary` | Short enough for a tooltip, in plain-output Markdown. For a heading: the paragraph the section marks as its summary (phase 2), else its first block when that's a paragraph; absent when neither, as for a section that opens with a list, a table, or a note. For a page: its description (Settled 2). |
| `help.*.text` | The whole section as the plain output writes it: from the heading to the next heading at its level or above, without the heading itself. A page's permalink gives the page's opening, before its first heading. An image is left out, its alt text kept in its place: the plain output writes an image as an address on the docs' host, which a product showing a bundled copy offline can't load, and the page keeps the picture. |
| `help.*.address` | The permalink's address, absolute, for "read more". Absent when `[consumer] site` isn't set. |
| `help.*.source` | The page's source path, as in `permalinks.json`. A permalink in a fragment has its heading on every page that includes it; its entry follows the one record `permalinks.json` keeps, and its section is cut from the page that record names. |

An alias is listed under its own name with its target's entry, and a retired permalink isn't listed, so a product that asks for one finds nothing, as `permalinks verify` would have told it.

**The file is as large as the sections are.** For Ascribe's own docs it's about 330 entries, some of them a page's worth of text: fine to bundle, and more than a product that wants one entry needs, which phase 3's entry at a time answers.

**Availability is text, as the build has it.** A build that shows badges writes each mark as an "Available: …" line in the section, including marks for versions a product on an older release doesn't have, where a versioned build's older line leaves out what came after it. So a product bundles the file from its own line's build, which `line` lets it check.

### Why these choices

- **Plain-output Markdown, not HTML or the JSON output's blocks.** A product can't render Ascribe's elements. The plain output is already the form for a reader that isn't the site: CommonMark, absolute links, a note as a blockquote, availability as text. A product renders Markdown with whatever it has, or shows `summary` as text.
- **In the site output's root, beside `permalinks.json`.** It's keyed by the same names, and the site output is written by every build, where `@ascribed/astro` writes the plain output only with `[consumer] agents = true`. A product's build copies it from the docs' build; publishing it where a product can fetch it is phase 3's.
- **The summary needs a marker, and phase 2 adds one.** A section's first paragraph is how most prose sections open, and the thread starts from it. It fails on the docs' largest set of permalinks: each diagnostic's section opens with its metadata line ("Advice · page level · next step: write · SPEC §9.4"), and the text a tooltip on `ASC149` wants is the **When** paragraph two blocks down. Text written for a page also often leans on the heading above it, so short isn't the same as self-contained, and products will want to choose the paragraph sooner than later. A marker is cheap once the diagnostics' generator needs it anyway, and it's the summary's first real use.
- **No setting.** The file costs one pass over the permalinks a build already lists, and exists only where a project declared permalinks. A project that doesn't want it publishes none.

## The thread

- **The use case:** a product's build copies `help.json` from the docs' build, and its settings screen shows the `weave-config` section's summary in a tooltip, with "Read more" opening the permalink's address.
- **The layers:** resolve (a heading's section, cut from its resolved page), emit (the plain emitter writing a section on its own, and the file), the CLI (`ascribe build` writes it, and its manifest lists it), the output-layout contract, and the docs.
- **The seam it tests:** a section rendered on its own, outside its page. The plain emitter writes whole pages; a section has to come out with its links absolute and right from where it now stands: a link to another heading on the same page (`#other`) has to become the page's address and the fragment, an include inside the section resolved, a switcher's arms as the plain output writes them, and nothing of the page around it. If that can't be done from the resolved tree without rendering the page and cutting the text, the design changes to a pass over the plain output's text, which changes how the summary is found.
- **Measured on Ascribe's own docs.** Phase 1 builds the docs' file too, and records what the first-paragraph rule picks across their permalinks: its lengths, the sections with no paragraph first, and the diagnostics' metadata lines. That's phase 2's input.
- **Not Ascribe's own surfaces as the product.** The editor links a diagnostic to `go/ASC149/`, and a hover could show its **When** paragraph, but the binary already carries that text (`ascribe explain`), from the registry the docs' diagnostics pages are generated from. The editor needs no help file for it, so the thread's product is an example on the docs page, and the docs are the corpus the design is measured against.
- **Left out, and where it comes back:** the summary's marker, a check of the summary, and seeing the help in the editor (phase 2); publishing each entry where a product can fetch it, with the headers a cross-origin fetch needs (phase 3); a product choosing its reader's arm of a switcher (phase 4); agents asking for a section by permalink (phase 5).

## The phases

| Phase | What a user can do after it | Layers |
|---|---|---|
| 1. The file | Copy `_ascribe/help.json` from a build and show a permalink's summary and text, with "Read more" | resolve, emit, CLI, contract, docs |
| 2. The right summary | Mark which paragraph is a section's summary, see in the editor what a product will show, and be told when a summary is too long or missing | language, resolve, check, the docs' generator, language server, extension |
| 3. Fetched from the docs | Fetch one permalink's entry for the product's own release line from the docs site at run time | emit, `@ascribed/astro`, headers, docs |
| 4. The reader's arm | Show the `pnpm` arm of a section to a reader the product knows uses pnpm | emit, contract, docs |
| 5. For agents | Ask for a section by permalink from the command line and the MCP server | CLI, MCP |

**Phase 1** builds the thread. `ascribe build` in `examples/quill`, given a heading permalink (it has none yet), writes the file; that heading's section has a link to another heading on its page, an include, and a phrase. A conformance case holds the entry, and the output-layout contract gains §1.7. The docs gain a page, "Show the docs in your product", with a short TypeScript example that reads the file at the product's build, and `ascribe permalinks verify` in the product's CI from the permalinks guide. *Done when:* `ascribe build` writes `help.json` for `site`, its `weave-config` entry's text has the same-page link made absolute, and the example on the docs page, run against that file, prints the summary and the address; and the docs' own file is built, with what the first-paragraph rule picks for their permalinks recorded in the pull request.

**Phase 2** lets a section mark its summary, a contract change to the language that its own decision settles first (the open question below). The docs' diagnostics generator marks each **When** paragraph, so `ASC149`'s summary is what it should be. Advice, `help-summary`, reports a permalink whose summary is over a limit (`[checks.help-summary] limit`, 300 characters, as `description-too-long` measures) and one with none, a section that opens with something other than a paragraph and marks nothing. On the docs, the first-paragraph rule leaves about one summary in nine over 300 characters, and three sections with none. This page's Permalink row and the hover on `@id {permalink=true}` show the summary a product gets. Adding the marker changes no key of the file, so phase 1's shape holds.

**Phase 3** publishes an entry for each permalink from `@ascribed/astro`, under each live line's own base path: the current line's at `go/<name>.json`, beside its address, and an older line's under its number, `3.4/go/<name>.json`, since permalink addresses are published for the current line only and a product on 3.4 must not be shown 3.5's text. With `[consumer] headers`, it adds the header that lets a product on another origin fetch them. A product fetches its own line's entry, keeps the text current without a release of its own, and uses its bundled copy offline or when its line is no longer live. It's after the thread because the bundled file is the simpler contract, and the addresses are a second one.

**Phase 4** adds a key to each entry whose section has a switcher: the text for each value of the switcher's dimension, so a product that knows its reader's choice shows that arm alone. It's a key added to version 1, so phase 1's shape holds; it comes after phase 3 because which arm to show is a question for products that have shipped phase 1 and found the labeled sections too long.

**Phase 5** lets `ascribe resolve` take a permalink, `ascribe resolve go:weave-config --build site`, printing its entry, and `ascribe_resolve` the same, so an agent writing a product's error message can read what the docs will show.

## What this leaves out

- **A component that shows the text.** A product renders Markdown its own way; a widget would be a framework choice for every product.
- **Text written only for the product.** A help string that isn't in the docs is the product's own, in its own strings; embedded help is the docs' text, or it can't promise the two agree.
- **Translation.** The brainstorm's translation idea would give a build per language, and each would write its own file.

## Open questions

1. **How a section marks its summary** (phase 2): a container directive around the paragraph, `@summary:` … `@end`, which every output writes as its content, or an attribute on the section's heading that names a paragraph? Recommended: the container, since it marks the text itself, survives the paragraph moving within the section, and needs no counting; whichever, it's a language change with its own decision before phase 2.

## Settled

The three questions this proposal left open, answered by the maintainer on 22 October 2026.

1. **The file is in the site output's root, beside `permalinks.json`.** It's keyed by the same names as its sibling, and the site output is written by every build. The plain root is written only when a build emits the plain output, which `@ascribed/astro` does only with `[consumer] agents = true`, so a site that publishes nothing for agents would have no `help.json` to copy. "Reachable from the docs' host" depended on that same setting, and phase 3 publishes an entry at a time, with the headers a fetch needs, which a whole file in the plain root wouldn't have. _Why:_ written whenever the site is, and published on purpose rather than by a side effect of another setting.
2. **A page's permalink gives its description as the summary and its opening as the text.** A whole page is rarely what a tooltip or a panel wants, and the address gives the rest. Two edges, so a reader of the file never guesses: a page with no description takes its opening's first paragraph as the summary, and a page whose opening is empty, one that starts with a heading, has an empty `text` and the summary alone. _Why:_ the entry is the page's own words at the length a product shows, with the page a click away.
3. **Every permalink, aliases under their own names, retired ones left out.** A product already names the permalinks it uses, and `ascribe permalinks verify` already tells it which exist; a mark for help would be a second list kept beside `ascribe.permalinks`, and the file costs one pass over a list the build makes anyway. _Why:_ one list of names, which the product's CI already checks.
