# Ascribe's story

A draft. It says what Ascribe is, who it's for, and what it promises, in the words every surface will take from it: the website, the README, the package and Marketplace descriptions, the command's help, and the agents' plugin. The website's revamp follows from it.

## What's decided

These came from Kyle on 21 October 2026, after an inventory of how the surfaces describe Ascribe today.

1. **The audience is engineers, writers, and AI agents.** Engineers like tools made for writing, and Ascribe brings writing tools to the workspace they already use. Writers want to be close to the code, but miss the tools a help authoring tool (HAT) gives them. Agents both write and build docs, and read them.
2. **The promise is docs as code with the tools of a dedicated docs tool.** Reuse, conditional content, versions, a glossary, structured page types, and preview have mostly lived in HATs and hosted docs platforms. Ascribe puts them in Markdown files in the repository.
3. **Agents are a headline pillar,** not a supporting feature.
4. **The site names no other tools.** This keeps the decision the first website plan made.
5. **The headline is "Docs as code, with the tools docs need."**
6. **"Help authoring tool" stays out of the copy** unless a page has a very good reason to use it. Most engineers don't know the term, and the promise reads without it. This file uses it only to say where the tools come from.
7. **Bryo Studio is the maker.** Ascribe is a product, not an organization, so it isn't named as an author or an owner. The one exception is the VS Code Marketplace publisher, `Ascribe`, which can't be renamed and isn't worth replacing.

## The problem

There are two ways to keep documentation today, and each gives something up.

- **Docs as code** keeps docs in the repository, in Markdown, reviewed in pull requests and built in CI. Its tools come from code: a text editor and a static site generator. It gives up what writers rely on. Content isn't reused, so it's copied. There are no variants, so pages fork for each platform. Versions are copied folders. Links break when a page moves, and nothing checks the structure of a page. Reviewers read diffs of source, not pages.
- **A HAT or a docs platform** has those tools. It keeps the docs in its own editor and format, outside the code's workflow and out of reach of review, CI, and the agents working in the repository.

## The promise

Ascribe gives docs written as code the tools a docs team expects, and checks every page the way code is checked: in the editor, in CI, and for the AI agents that write and read the docs.

## What Ascribe is

**Recommendation: call it a toolchain, and don't call it a platform.**

"Platform" suggests hosting, accounts, and a place your docs live. Ascribe hosts nothing, and the Why page counts the low cost of leaving as a strength. A platform's breadth is better shown than claimed: one install gives the language, the checks, the editor, preview, review, and the outputs.

"Toolchain" is what a programming language comes with: a compiler, a formatter, a language server, and a linter. That's the right picture. Ascribe is a language for documentation, Markdown with a few directives, and the toolchain that comes with one. The sentence every surface opens with:

> Ascribe is a toolchain for documentation written as code: Markdown with the structure docs need, checked as you write, and built into a website, plain Markdown, and JSON.

"Language", "content model", and "compiler" stay in SPEC.md and the docs, where readers need them.

## Headline

> Docs as code, with the tools docs need.

The headline states the promise, and the sentence above says what Ascribe is. The headline doesn't say what "the tools" are, so whatever stands under it names them: reusable content, variants, versions, a glossary, and preview.

"Documentation written as code", the current tagline, stays in the sentence that says what Ascribe is. It's no longer the headline, because it names a practice that many tools claim.

## Pillars

Each pillar has its claim and the features that prove it. A claim with nothing built behind it doesn't go on a page.

1. **The tools of a docs team.**
   - Reuse with `@include` and phrases.
   - Variants by platform, product, or package manager with `@variant`, shown as tabs.
   - Availability and versions: one source, a build for each release line.
   - A glossary that links terms to their definitions.
   - Page types and templates, from a content model in `ascribe.toml`.
   - Notes, steps, and details.
   - A page preview and a site preview in VS Code.
   - Permalinks that keep working when a section moves.
2. **Checked like code.**
   - Broken links, missing headings, and undeclared names are errors, in the editor as you type and in CI.
   - Code examples are read from the code's own files with `@snippet`, and `ascribe drift` lists the pages whose examples changed.
   - Vale style checks run on the prose.
   - A pull request is reviewed as rendered pages, with what changed marked and comments beside the paragraph they're about.
3. **Built for agents, writing and reading.**
   - As writers, agents get the project's rules from `ascribe agents sync`, the model and the checks through the MCP server, and a hook that checks after each edit.
   - Memos keep the reason a page is written its way, for the next person or agent who changes it.
   - As readers, agents get `llms.txt`, a Markdown version of every page, JSON, and an index of the docs written into the product's `AGENTS.md`.
4. **Plain files you can leave.** This is the trust pillar, and it's quieter than the other three.
   - A page reads as Markdown without Ascribe.
   - The plain output is CommonMark.
   - Ascribe hosts nothing and calls no model.
   - It's open source under the MPL.

## For each audience

The pillars are the same for everyone. What leads changes with the reader.

1. **Engineers.** Lead with checked like code and examples from real code. They get writing tools without leaving the editor, the repository, or CI.
2. **Writers.** Lead with the tools of a docs team. They get reuse, variants, versions, a glossary, and preview, without leaving the repository where the engineers who review them work.
3. **Agents.** Lead with the rules, the checks, and the outputs. An agent writing pages has a deterministic check to run. An agent reading them gets Markdown and an index, not a page of HTML.

## Voice

The voice the site already has is kept. It's plain and exact, it shows real output rather than describing it, and it says what Ascribe doesn't do. It says "early" while Ascribe is before 1.0. It doesn't use superlatives, and it makes no claim that a command's output can't show.

## Words

Each concept gets one word in marketing copy. The docs keep their own reference terms.

1. Use "docs as code" in prose and "docs-as-code" only as a keyword.
2. Use "notes", not "callouts", since the directive is `@note`.
3. Use "reusable content", not "includes", in marketing; the docs say `@include`.
4. Use "toolchain", not "platform", "compiler", or "markup language", when saying what Ascribe is.
5. Use "AI agents" on first mention and "agents" after.

## After the story

1. **One home for the words.** The sentence and the headline are kept in one file. The README, the package descriptions, the Marketplace listing, the command's `about`, the plugin's description, and the site's meta description are generated from it or tested against it.
2. **Bryo Studio everywhere a maker is named.** The plugin's `author` and `owner` change from Ascribe to Bryo Studio. The VS Code publisher stays `Ascribe`, as the extension's id, `Ascribe.ascribe-vscode`, does: a publisher can't be renamed, and a new one isn't worth the trouble.
3. **The website.** The revamp starts from the pillars, and it's planned in this folder.
