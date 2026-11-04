# Style guides, AI writing tells, and which tells can be checked mechanically

Research notes, gathered 2026-11-03. Pages were fetched live where a URL is given. Microsoft pages carry `ms.date` and `updated_at` metadata, so their dates are noted. Google pages show no date in the fetched text.

## What the established guides say about the topics AI-tell lists touch

### Takeaway
The big guides already ban most of the vocabulary on AI-tell lists: "simply", "just", "easy", "leverage", "utilize", "please note", "via", and superlatives. They disagree with each other on em dashes, on "let's", and on self-referential openings. Their formatting rules mostly endorse things the AI-tell lists flag, such as bold term-plus-definition lists and lists generally. On those points AI-tell lists and doc conventions really do conflict.

### Cited Findings

**Word lists: Google**
- easy, easily: "What might be easy for you might not be easy for others. Try eliminating this word from the sentence." — [Google word list](https://developers.google.com/style/word-list)
- simple, simply: "What might be simple for you might not be simple for others. Try eliminating this word from the sentence because usually the same meaning can be conveyed without it." — [Google word list](https://developers.google.com/style/word-list)
- just: "Avoid. Usually, just is a filler word that you can delete without affecting your meaning." — [Google word list](https://developers.google.com/style/word-list)
- leverage: "Avoid using if you mean use. If possible, use a more precise term. For example, use, build on, or take advantage of." — [Google word list](https://developers.google.com/style/word-list)
- utilize: "Don't use *utilize* when you mean *use*. It's OK to use *utilize* or *utilization* when referring to the quantity of a resource being used." — [Google word list](https://developers.google.com/style/word-list)
- please / please note: "Don't use *please* in the normal course of explaining how to use a product…"; "Don't use the phrase *please note*." — [Google word list](https://developers.google.com/style/word-list)
- via: "Don't use." e.g.: "Don't use. Instead, use phrases like for example or such as." in order to: "Avoid in order to; instead, use to." etc.: "Avoid using etc., and so forth, and and so on wherever possible." — [Google word list](https://developers.google.com/style/word-list)
- will: "Avoid. Applies equally to its past tense, *would*." should: "Generally avoid." once: "If you mean *after*, then use *after* instead of *once*." may: "reserve for official policy or legal considerations." — [Google word list](https://developers.google.com/style/word-list)
- this document: "use this document, and not this article, this topic, this doc, or this page." The guide therefore assumes a page may refer to itself. — [Google word list](https://developers.google.com/style/word-list)
- generative AI: "Spell out generative… Don't use gen AI or Gen AI." — [Google word list](https://developers.google.com/style/word-list); the addition is logged on [What's new](https://developers.google.com/style/whats-new)
- ensure / guarantee and superlatives: "avoid superlatives like *best*, *simplest*, *fastest*, *never*, and *always*"; "be careful about words like *ensure* and *guarantee* and use them only when something can truly be ensured or guaranteed." An excessive claim is one that "Makes a statement about performance or cost that isn't easily verifiable with data that's available to the reader." — [Google: Excessive claims](https://developers.google.com/style/excessive-claims)
- No Google word-list entry was found for *robust*, *seamless*, or *powerful*. Google leaves promotional adjectives to the excessive-claims page and the tone page ("Buzzwords or technical jargon"). — [Google word list](https://developers.google.com/style/word-list), [Google tone](https://developers.google.com/style/tone)

**Word lists: Microsoft**
- "Use simple words with precise meanings, and remove words that don't add substance." Its table says to use *use* in place of "utilize, make use of", *to* in place of "in order to, as a means to", and *also* in place of "in addition". — [Microsoft: Use simple words, concise sentences](https://learn.microsoft.com/en-us/style-guide/word-choice/use-simple-words-concise-sentences) (ms.date 2018-01-19, updated 2026-10-15)
- "Omit unnecessary adverbs… Unless they're important to the meaning of a statement, leave them out." Its examples are *quite, very, quickly, easily, effectively*. — [Microsoft: Use simple words, concise sentences](https://learn.microsoft.com/en-us/style-guide/word-choice/use-simple-words-concise-sentences)
- "Avoid weak phrasing like *there is*, *there are*, and *there were*." "Edit out *you can* when it isn't necessary." — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice) (ms.date 2026-07-02)
- A search found no Microsoft entry that bans "simply" or "just" by name. Microsoft covers them through the adverb rule. — search result; see Gaps

**GitLab (a docs team's public guide)**
- Its "Building trust" section says to avoid words like "easily" and "simply". Its "Self-referential writing" section says to avoid writing about the document itself. Phrases like "This page shows…" and "This guide explains…" slow readers down, so a page should get straight to the point. — [GitLab documentation style guide](https://docs.gitlab.com/development/documentation/styleguide/)

**Em dashes and punctuation**
- Google: "To indicate a break in the flow of a sentence—or an interruption—use an em dash… Don't put a space before or after it." En dash: "Don't use. Instead, use a hyphen or the word to." Google sets no limit on how often em dashes appear. — [Google: Dashes](https://developers.google.com/style/dashes)
- Microsoft: "Em dashes can be an effective alternative to colons, semicolons, and parentheses, but don't overuse them. … em dashes can affect readability if they are used excessively, by interrupting the flow of the text and making sentences overly long and complex. Don't use spaces around em dashes." — [Microsoft: Dashes and hyphens](https://learn.microsoft.com/en-us/style-guide/punctuation/dashes-hyphens/emes) (ms.date 2025-04-11)
- Microsoft's "Don't be spacey" tip shows em dashes as the approved form: "Use pipelines—logical groups of activities—to consolidate…" — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)
- GitLab bans both: don't use the en dash or em dash; use separate sentences or commas. — [GitLab documentation style guide](https://docs.gitlab.com/development/documentation/styleguide/)
- Google's tone page lists "Exclamation marks" among things to avoid. — [Google tone](https://developers.google.com/style/tone)
- Microsoft requires the Oxford comma ("Remember the last comma"). — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)

**Bold text and inline labels in lists**
- Microsoft prescribes bold term lists. "If you must define, describe, or explain a set of terms or concepts, a common alternative to a table is a list where each item consists of a term followed by a definition." "For the term, use sentence case and bold… Use a period between the term and the definition. The period is in plain text." "The use of bold in term lists is an exception to the general guideline of using italic for emphasis." Its example is `- **Draft**. You created the document, but you're still working on it.` — [Microsoft: Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists) (ms.date 2023-06-14, updated 2026-10-15)
- Microsoft uses italic for emphasis, "sparingly", not bold. — [Microsoft: Formatting common text elements](https://learn.microsoft.com/en-us/style-guide/text-formatting/formatting-common-text-elements) (ms.date 2025-03-26)
- Google endorses run-in headings in lists: "Start the run-in heading with a capital letter. End the run-in heading with a period or a colon, but be consistent within the list." Description lists are for when "you want to draw attention to two or more terms (such as a glossary)." — [Google: Lists](https://developers.google.com/style/lists)
- GitLab doesn't use bold for keywords or emphasis. It keeps bold for UI elements and navigation paths. — [GitLab documentation style guide](https://docs.gitlab.com/development/documentation/styleguide/)

**Headings and title case**
- Google: "Use sentence case for all headings and titles." "For a task-based heading, start with a bare infinitive." "When possible, avoid using -ing verb forms as the first word in any heading or title." "Make sure headings are followed by content." — [Google: Headings](https://developers.google.com/style/headings)
- Microsoft: "Default to sentence-style capitalization… Don't use title-style capitalization (Like This)." "Don't use a period or a colon at the end of titles, headings, subheadings, and UI titles." — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)
- GitLab: topic titles use sentence case, and feature names are lowercase. — [GitLab documentation style guide](https://docs.gitlab.com/development/documentation/styleguide/)

**List use**
- Microsoft: "A list should have at least two items but (if possible) no more than seven items… Make all the items in a list consistent in structure." "If you introduce a list with a heading, don't use explanatory text after the heading." Items get no end period "if all items have three or fewer words". — [Microsoft: Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists)
- Google: "Introduce a list with a complete sentence, not a partial one that's completed by the list items." "Use the same syntax/structure for all list items." "Don't use a list to show only one item." — [Google: Lists](https://developers.google.com/style/lists)

**Signposting and notes**
- Google's notices are "Note", "Caution", "Warning", and "Success". "Don't use too many notices. When you use multiple notices on a page, they begin to lose their visual distinctiveness." "Where possible, avoid grouping two or more notices together." — [Google: Notices](https://developers.google.com/style/notices)
- Google: "use this document" (word list, above). GitLab: avoid "This page shows…" (above).
- Microsoft: "Lead with what's most important. Front-load keywords for scanning." — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)
- Diátaxis tutorials endorse a stated goal up front: "In this tutorial we will create and deploy a scalable web application." Diátaxis calls "In this tutorial you will learn…" "presumptuous" and a very poor pattern. Its tutorial phrases include "We …", "Notice that … Remember that … Let's check …", and "You have built a secure, three-layer hylomorphic stasis engine…" (a mild note of admiration). — [Diátaxis: Tutorials](https://diataxis.fr/tutorials/)

**Voice, tone, and promotional language**
- Google wants "conversational, friendly, and respectful without using slang or being overly colloquial or frivolous", like "a knowledgeable friend". It lists things to avoid: "Buzzwords or technical jargon", "Being too cutesy", "Avoid figurative language", "Placeholder phrases like *please note* and *at this time*", "Choppy or long-winded sentences", "Starting all sentences with the same phrase", "Exclamation marks", "Phrasing in terms of *let's* do something", and "Using phrases like *simply*, *It's that simple*, *It's easy*, or *quickly* in a procedure." — [Google tone](https://developers.google.com/style/tone)
- Microsoft says the opposite about *let's*: "Use contractions like *it's*, *you'll*, *you're*, *we're*, and *let's*." Its other tips are "Write like you speak", "Shorter is always better", and "Prune every excess word." — [Microsoft: Top 10 tips](https://learn.microsoft.com/en-us/style-guide/top-10-tips-style-voice)
- Diátaxis on reference: "It should be **austere**." "*Neutral description* is the key imperative of technical reference." Natural tendencies "to explain, instruct, discuss, opine… run counter to the needs of technical reference." — [Diátaxis: Reference](https://diataxis.fr/reference/)
- Write the Docs: "A consistent tone and style makes your content easier to read." For error messages: "Write like a human, not a robot." Its style-guides page says nothing about "simply"/"just", sentence length, linters, or AI. — [Write the Docs: Style guides](https://www.writethedocs.org/guide/writing/style-guides/)

**Sentence length**
- Google gives no word count. It says "Write shorter sentences. The shorter the sentence, the easier it is to translate," and "Longer sentences can impair understanding." — [Google: Writing for a global audience](https://developers.google.com/style/translation)
- Microsoft's prose guide gives no number either. Its Vale package uses 30 words: `Microsoft/SentenceLength` ("Try to keep sentences short (< 30 words)."). — [errata-ai/Microsoft SentenceLength.yml](https://github.com/errata-ai/Microsoft/blob/master/Microsoft/SentenceLength.yml)
- Google warns against "Choppy or long-winded sentences" and "Starting all sentences with the same phrase". That is guidance on rhythm, which is close to the AI "uniform rhythm" tell. — [Google tone](https://developers.google.com/style/tone)

**Attribution and claims**
- Google: avoid claims "that [aren't] easily verifiable with data that's available to the reader". — [Google: Excessive claims](https://developers.google.com/style/excessive-claims)
- Diátaxis reference "should be wholly authoritative" with "no doubt or ambiguity". — [Diátaxis: Reference](https://diataxis.fr/reference/)

### Inferences
- Most of a vocabulary tell list ("simply", "just", "easy", "leverage", "utilize", "please note", "in order to", superlatives) can be justified by Google, Microsoft, and GitLab rules alone. A project skill can cite those guides rather than "sounds like AI" for these. The fluff vocabulary of AI-tell lists ("delve", "tapestry", "pivotal", "underscore") is not in either word list. It falls under general bans on "buzzwords" and "figurative language".
- The guides disagree among themselves on em dashes (Google allows, Microsoft allows sparingly, GitLab bans), *let's* (Google avoids, Microsoft encourages, Diátaxis uses in tutorials), and self-reference (Google's word list assumes "this document", GitLab bans "This page shows…"). A project has to pick a side; no guide settles these.

### Gaps
- **Apple Style Guide:** No entry was retrieved. Searches returned no pages from the Apple Style Guide (support.apple.com/guide/applestyleguide), and the guide's per-entry pages weren't reached. Nothing here reports Apple's position on em dashes, "simply", or headings.
- **Microsoft A–Z entries for *leverage*, *ensure*, *simply*, *just*:** not fetched. Only the general word-choice pages were confirmed.
- **Google's bold-for-emphasis rule** (text-formatting page) was not fetched.
- The Write the Docs pages fetched had no guidance on word lists or sentence length. Other Write the Docs pages may.

## Where the guides endorse what AI-tell lists flag, and how to resolve each conflict

### Takeaway
At least five common tells are standard doc conventions: bold run-in labels in term lists, "In this tutorial we will…" openings, lists generally (including three-item lists), em dashes, and Note callouts. The AI signal isn't any one of these. It is overuse, or using them where the convention doesn't apply. A skill should check rates and placement, not the construct itself.

### Cited Findings
- Wikipedia's list of AI signs includes negative parallelisms ("Not just X, but also Y", "Not X, but Y", "Y rather than X"), the rule of three, "Overuse of em dashes", excessive boldface and "canned-format lists", vague attribution ("attribute opinions or claims to some vague authority"), superficial "-ing" analyses, "Challenges and future prospects" sections, section summaries, elegant variation, and AI vocabulary. — [Wikipedia: Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- The same page warns that these are "only potential signs of a problem, not the problem itself". It notes that human writing shows the traits too, since models train on it, and that negative parallelisms are "common among human writers". It also warns against relying on detection tools, and against fixing only the surface, since that makes detection harder. — [Wikipedia: Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- On vocabulary, the page says one occurrence may be coincidence, but many in one edit is a strong signal. That argues for counts rather than single hits. — [Wikipedia: Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- **Bold run-in labels:** Microsoft prescribes `**Term**. Definition.` for term lists, and Google allows run-in headings ending in a period or colon (see section 1). — [Microsoft: Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists), [Google: Lists](https://developers.google.com/style/lists)
- **"This page explains…" openings:** Google's word list assumes self-reference ("use this document"). GitLab forbids "This page shows…". Diátaxis endorses "In this tutorial we will…" for tutorials and rejects "you will learn". — [Google word list](https://developers.google.com/style/word-list), [GitLab style guide](https://docs.gitlab.com/development/documentation/styleguide/), [Diátaxis: Tutorials](https://diataxis.fr/tutorials/)
- **Lists of three:** Microsoft's list rule is 2–7 items. It sets no minimum beyond two and has no rule against three. Its examples include three-item lists. — [Microsoft: Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists)
- **Em dashes:** Google and Microsoft allow them; Microsoft adds "don't overuse them". GitLab bans them. — [Google: Dashes](https://developers.google.com/style/dashes), [Microsoft: Dashes](https://learn.microsoft.com/en-us/style-guide/punctuation/dashes-hyphens/emes), [GitLab style guide](https://docs.gitlab.com/development/documentation/styleguide/)
- **"Note:" callouts:** Google defines Note as a standard notice type, then says "Don't use too many notices" and avoid grouping them. — [Google: Notices](https://developers.google.com/style/notices)
- **"For example" lists:** Google tells writers to use "for example or such as" instead of "e.g." and "etc.". Vale's `occurrence` docs use a rule that *requires* "for example" in paragraphs (`min`) as their sample. Linters can push toward examples as well as away from them. — [Google word list](https://developers.google.com/style/word-list), [Vale: occurrence](https://docs.vale.sh/checks/occurrence)
- **Tutorial praise:** Diátaxis allows "a mild note of admiration" at a tutorial's end ("You have built a secure, three-layer … engine"). AI-tell lists would read that as a punchy or puffed closer. — [Diátaxis: Tutorials](https://diataxis.fr/tutorials/)

### Inferences
How to resolve each conflict for docs. These are recommendations drawn from the findings above.
- **Bold run-in labels:** Keep them in true term or definition lists, where the bold is a term the reader scans for, and where the term and definition are separated by a period (Microsoft) or a colon (Google), consistently. Flag them when the "label" is a restated summary of its own sentence ("**Fast setup:** Setup is fast."), when every list on the page uses them, or when a paragraph would read better. A check can measure the share of list items that open with bold. Whether a label earns its place needs judgment.
- **Self-referential openings:** Choose by page type. For a tutorial, a goal statement ("In this tutorial we will…" or a plain goal sentence) follows Diátaxis. For reference and how-to pages, cut "This page describes…" (GitLab, Microsoft's "Get to the point fast") and lead with the content. A regex can find the opener; whether it belongs depends on the page type, which a doc system with typed pages (such as Ascribe's content model) can supply to the check.
- **Three-item lists:** A triad is a tell only in prose, as rhetorical padding ("fast, flexible, and reliable"), and mainly when it recurs. A real list with three items is fine. Don't flag list length; flag adjective triplets in running prose, by count per page.
- **Em dashes:** Pick a project rule. Either GitLab's ban (simple, fully mechanical) or Microsoft's "don't overuse" as a per-paragraph or per-page cap. A cap keeps legitimate dashes and catches the AI rate. Spacing around dashes is already a mechanical rule in both Vale packages.
- **Notes:** Keep the Note type, and flag stacked notices, notes on every section, and "Note that…" in body prose (Google's "please note" ban covers the phrase).
- **Tutorial closers:** Allow one goal-recap sentence at a tutorial's end (Diátaxis). Flag an applause or "In conclusion" closer on reference or how-to pages.

### Gaps
- No guide addresses negative parallelism ("not X, but Y") or "-ing" significance clauses directly. Google's ban on "figurative language" and Diátaxis's "neutral description" are the closest.
- No style guide found treats the rule of three as a fault.

## Published guidance on AI-assisted documentation from docs teams, 2023–2026

### Takeaway
Microsoft has the most explicit public policy: AI-generated content on Microsoft Learn is human-reviewed, its code is tested, and each such article is labeled. GitLab's docs style guide has an "AI-generated content" section aimed at contributors. Google's developer style guide only adds terminology, though Google Search has separate guidance on AI-generated web content. No Red Hat or MDN docs-specific policy was found.

### Cited Findings
- **Microsoft Learn**, "Principles for AI-Generated Content" (ms.date 2025-03-05, updated 2026-05-26):
  - "We understand that AI-generated content isn't always accurate. We test and review AI-generated content before we publish it."
  - "All articles that contain any AI-generated content include text acknowledging the role of AI. You'll see this text at the end of the article."
  - In the authoring model, the author plans the article, uses AI to generate part of it, then "reviews and revises the AI-generated content".
  - Articles pass the standard validation for formatting and for "terms and language are appropriate and inclusive".
  - "The author tests all AI-generated code before publishing."

  — [Microsoft Learn: Principles for AI-Generated Content](https://learn.microsoft.com/en-us/principles-for-ai-generated-content)
- **GitLab:** "When you use AI tools to generate or assist with documentation, check the output carefully before you submit it for review." It lists what to check for:
  - Repetition: content that restates the page or a linked topic.
  - Vague or unverifiable claims: don't invent command syntax, API parameters, or UI element names.
  - Incorrect scope: a new page for something an existing page covers.

  — [GitLab documentation style guide](https://docs.gitlab.com/development/documentation/styleguide/)
- GitLab also keeps a page on AI agent instruction files for documentation. It names the docs style guide as the single source of truth for documentation standards. — [GitLab: AI agent instruction files for documentation](https://docs.gitlab.com/development/documentation/ai-instruction-files-documentation/)
- **Google:** the developer style guide's AI content is word-list terminology (*AI*, *generative AI*). — [Google word list](https://developers.google.com/style/word-list). Separately, Google Search's guidance on AI-generated web content tells publishers to manually fact-check and review AI content, and suggests disclosing how content was made. — [Google Search: Using gen AI content](https://developers.google.com/search/docs/fundamentals/using-gen-ai-content), [Search Central blog, Feb 2023](https://developers.google.com/search/blog/2023/02/google-search-and-ai-content)
- **Mozilla:** the support site (SUMO, not MDN) has a contributor policy on generative AI built on accountability, authenticity, and transparency. It asks for attribution when "a significant portion" of content is AI-generated, and repeated violations can lead to moderation action. — [Mozilla SUMO: Contributor policy, generative AI](https://support.mozilla.org/de/kb/contributor-policy-generative-ai-usage)
- **Red Hat:** the supplementary style guide frames AI as a reader. Short descriptions on every module provide a summary for readers and for AI-powered search, and the guide is published in Markdown for machine reading. No writer-facing AI-assistance rules were found. — [Red Hat supplementary style guide](https://redhat-documentation.github.io/supplementary-style-guide/)
- **Practitioner guidance:** a passo.uno post (2025-11-10) recommends that a docs AI policy:
  - holds the human responsible for AI output;
  - lists where human expertise is required;
  - reviews AI contributions like any others;
  - asks for non-punitive disclosure (tool, prompts, how much editing was needed, what the AI got wrong);
  - relies on deterministic checks (linters, link checkers, build tests), extended to catch hallucinated commands and APIs.

  — [passo.uno: You need an AI policy for your docs](https://passo.uno/ai-docs-policy-contributions/)

### Inferences
- The published policies focus on accuracy (invented APIs, untested code), repetition, and scope. They don't focus on stylistic tells. That suggests a project skill should rank "every command, flag, and API name exists" and "nothing is restated" above vocabulary and punctuation tells.
- GitLab's three checks map to things a doc toolchain can verify. Repetition can be caught by near-duplicate detection across the page and the pages it links. Invented syntax can be caught by checking code spans against the real CLI or API surface. Scope can be checked by comparing a new page against existing titles.

### Gaps
- No MDN (developer.mozilla.org) policy on AI-generated contributions was found. Only Mozilla SUMO's.
- Red Hat's internal "Policy on the Use of AI Technology" and its code-assistant guidelines are referenced elsewhere but were not accessible. No docs-specific Red Hat policy was found.
- Stripe: nothing public was found. (Not searched in depth.)
- Apple: nothing found.

## Mechanical checks: Vale rule types, which tells each expresses, false positives, and what needs judgment

### Takeaway
Fixed words, phrases, punctuation, heading case, and per-sentence or per-paragraph counts are all mechanical. Vale's `existence`, `substitution`, `occurrence`, `capitalization`, and `metric` cover them. `sequence` and `script` (Tengo) stretch to simple syntactic patterns such as "not X, but Y". Tells that depend on meaning or placement need a human or LLM pass: puffery about significance, vague attribution, empty "-ing" significance clauses, restated summaries, unearned closers, and whether a list or label belongs. Field reports show that rules a team can't trust get ignored.

### Cited Findings

**Vale check types**
- `existence` "looks for the presence of particular tokens". Its parameters are `tokens`, `raw`, `ignorecase`, `nonword`, `exceptions`, `vocab`, `action`, and `affix` (v3.25.0+). The example is `extends: existence`, `message: Consider removing '%s'`, `tokens: [appears to be, arguably]`. — [Vale: existence](https://docs.vale.sh/checks/existence)
- `substitution` "associates a string with a preferred form." — [Vale: substitution](https://docs.vale.sh/checks/substitution)
- `occurrence` caps (`max`) or requires (`min`) a token's count per `scope`. Its examples are a limit of three commas per sentence, and a rule that flags paragraphs lacking "for example". Per-scope positions for `min` need v3.20.0+. — [Vale: occurrence](https://docs.vale.sh/checks/occurrence)
- `repetition` "looks for repeated occurrences of its tokens" (e.g., "'the' is repeated!"). — [Vale: repetition](https://docs.vale.sh/checks/repetition)
- `consistency` keeps one of two forms per file (e.g., "Inconsistent spelling of 'adviser'"). — [Vale: consistency](https://docs.vale.sh/checks/consistency)
- `conditional` requires that one pattern appear only if another does. The typical use is an acronym that must be defined. — [Vale: conditional](https://docs.vale.sh/checks/conditional)
- `capitalization` "checks that the text in the specified scope matches the case of match", with styles like `$sentence` and `$title` (AP or Chicago). — [Vale: capitalization](https://docs.vale.sh/checks/capitalization)
- `metric` evaluates a `formula` against a `condition`. Its variables:
  - counts: `words`, `sentences`, `characters`, `syllables`, `complex_words`, `long_words`, `quote_words`, `sentence_length_sd` (v3.23.0+);
  - structure: `paragraphs`, `heading.h{n}`, `list`, `blockquote`, `pre`;
  - readability scores: Flesch–Kincaid, Gunning fog, and others.

  With `scope` (v3.21.0+), it measures each block. — [Vale: metric](https://docs.vale.sh/checks/metric)
- `sequence` matches part-of-speech token sequences: `pattern`, `tag`, `upos`, `negate`, `skip`, `min`, `target`. "Every rule needs at least one `pattern`" as an anchor. Rules are sentence-scoped, and results depend on the POS tagger. — [Vale: sequence](https://docs.vale.sh/checks/sequence)
- `spelling` is Hunspell-based spell checking. — [Vale: spelling](https://docs.vale.sh/checks/spelling)
- `script` runs a Tengo script once per scope block. The script receives `scope` text and fills `matches` with `begin`/`end` offsets and an optional message, and is limited to Go regex syntax. Rules can carry `tests` run by `vale test`. — [Vale: script](https://docs.vale.sh/checks/script)

**Existing packages that already encode guide rules**
- errata-ai/Google has 36 rules, including `EmDash`, `ExcessiveClaims`, `Headings`, `Will`, `Passive`, `WordList`, `Exclamation`, `FirstPerson`, `We`, `Latin`, and `Colons`. — [errata-ai/Google](https://github.com/errata-ai/Google/tree/master/Google)
- `Google/EmDash` flags only *spaced* dashes (`'\s[—–]\s'`). It doesn't limit em dashes. — [Google/EmDash.yml](https://github.com/errata-ai/Google/blob/master/Google/EmDash.yml)
- errata-ai/Microsoft has 47 rules, including `Adverbs` (a long list that includes *easily*, *effectively*), `Wordiness` (a substitution list: `utilize|make use of → use`, `in order to → to`), `SentenceLength` (occurrence, max 30 words per sentence), `Dashes`, `Headings`, `Avoid`, and `Negative`. — [errata-ai/Microsoft](https://github.com/errata-ai/Microsoft/tree/master/Microsoft)
- `Microsoft/SentenceLength` is `extends: occurrence`, `scope: sentence`, `max: 30`, `token: \b(\w+)\b`. — [SentenceLength.yml](https://github.com/errata-ai/Microsoft/blob/master/Microsoft/SentenceLength.yml)
- `Google/Headings` uses `extends: capitalization`, `scope: heading`, `match: $sentence`, with a hand-kept list of proper-noun exceptions (Kubernetes, macOS, TypeScript…). — [Google/Headings.yml](https://github.com/errata-ai/Google/blob/master/Google/Headings.yml)

**False-positive evidence**
- A comment in the current `Google/ExcessiveClaims.yml` says *never*, *always*, and *ensure* were dropped from the rule because in technical writing they are "usually legitimate instructions ('never commit secrets')". They "accounted for 125 of 142 hits on a 950-file corpus" (about 88% of that rule's hits). It also excludes "best practices" as a fixed term. The rule keeps `best(?! practices?)`, `simplest`, `fastest`, and `guarantees?`. — [Google/ExcessiveClaims.yml](https://github.com/errata-ai/Google/blob/master/Google/ExcessiveClaims.yml). The corpus isn't named in the comment, so this figure can't be independently checked.
- GitLab MR !54 (2026-08-04), "scope vale to what it can honestly judge". Vale findings dropped from 308 to 17 in a repository where the Vale job ran with `allow_failure: true`, so "nobody acted on the output": "A linter whose output is permanently ignored is not help, it is noise with a CI cost."
  - 107 of the 308 came from one link-format rule misapplied to ordinary relative links.
  - CHANGELOG.md was excluded because rewording a dated record "edits history".
  - The Oxford-comma rule was found to fire whether or not the comma was present.
  - 252 real findings were fixed.
  - The remaining 17 hits were judged context-dependent. One example is whether "self-managed" names a product tier or describes the reader's environment.

  — [GitLab MR !54](https://gitlab.com/gitlab-com/public-sector/reference/-/merge_requests/54)
- A GitLab issue reports that some Vale rules produced so many warnings that they are often ignored. The proposed fix was downgrading them to suggestions. — [GitLab issue 382360](https://gitlab.com/gitlab-org/gitlab/-/issues/382360)
- Meilisearch: "Vale will find false positives." Start with a handful of rules and adjust over time. There's no formula for choosing suggestion, warning, or error. — [Meilisearch: Prose linting with Vale](https://www.meilisearch.com/blog/prose-linting-with-vale)
- Elastic's shared Vale rules add filters for camelCase identifiers, uppercase acronyms, CLI flags, and file extensions to cut false positives in technical docs. — [elastic/vale-rules](https://github.com/elastic/vale-rules) (via search summary, not fetched)
- An AI-tell Vale ruleset (ammil industries, 2025-12-08, credited to Christian Miles) sorts rules into three tiers:
  - **error:** "Definite AI artifacts", such as chatbot phrases, placeholders, and tracking URLs;
  - **warning:** "Likely AI patterns", such as hedging clusters, knowledge-cutoff references, and enumeration style;
  - **suggestion:** "Suspicious but common in human writing", such as vocabulary, transitions, passive voice, and symbolic language.

  The post says Vale can't flag stray Markdown syntax only outside Markdown contexts. It gives no false-positive data. — [ammil industries: Signs of AI writing, a Vale ruleset](https://ammil.industries/signs-of-ai-writing-a-vale-ruleset)

### Inferences

**Classifying the common tells.** Each line names the tell, how it can be checked, the Vale type, and notes. The Vale snippets below are illustrative, written for these notes from the documented parameters, not copied from a package.

**1. Regex or Vale rule (deterministic, low judgment)**
- **Filler and condescending words** ("simply", "just", "easy/easily", "obviously", "of course"): `existence`. Google and Microsoft both back it (cited above). Expect false positives from "just" as in "just-in-time", and from "easy" inside names. Add `exceptions`.
- **Wordy or inflated verbs** ("leverage", "utilize", "in order to", "make use of", "via", "e.g.", "etc."): `substitution`. `Microsoft/Wordiness` already covers several. "utilize" for resource quantity is allowed by Google, so make it a suggestion.
- **Chatbot artifacts** ("As an AI…", "I hope this helps", "Certainly!", "Great question", knowledge-cutoff phrases, `[insert …]` placeholders, `utm_source=chatgpt.com`): `existence`, at error level. Near-zero false positives in docs (ammil's "definite" tier).
- **AI vocabulary** ("delve", "tapestry", "testament", "pivotal", "underscore", "seamless", "robust", "landscape", "realm"): `existence` at suggestion level, or better `occurrence` with `max` per document, since one hit means little (per Wikipedia).
- **Puffery adjectives and superlatives** ("best", "fastest", "powerful", "seamless", "cutting-edge", "world-class"): `existence`. Google's ExcessiveClaims data shows absolute words ("never", "always", "ensure") are mostly legitimate instructions in docs. Leave them out.
- **"Please note", "Note that", "It's worth noting", "It is important to note"**: `existence`. Google bans "please note".
- **Self-referential openers** ("This page describes", "This guide explains", "In this article"): `existence`, with `scope` limited to the first paragraph if possible. This is a conflict case: allow a tutorial goal statement per Diátaxis, so condition the rule on page type in the toolchain.
- **Summary or conclusion openers** ("In summary", "In conclusion", "To sum up", "Overall,"): `existence`.
- **Title-case headings**: `capitalization` with `match: $sentence`, `scope: heading`. Needs a proper-noun exception list (see `Google/Headings`).
- **Heading end punctuation** (period or colon): `existence` with `scope: heading`. Both packages have one.
- **Spaced em dashes or `--`**: `existence`. Already in both packages.
- **Exclamation marks in prose**: `existence`. `Google/Exclamation` exists.
- **Emoji in headings or list bullets**: `existence` with a Unicode range and `nonword: true`.
- **Curly vs. straight quotes, inconsistent spelling**: `consistency`.
- **Undefined acronyms**: `conditional`.

```yaml
# Filler words (Google word list, Microsoft adverbs)
extends: existence
message: "Remove '%s'; what's easy for you may not be for the reader."
level: warning
ignorecase: true
tokens: [simply, just, easily, obviously, of course]
exceptions: [just-in-time]
---
# Wordy verbs
extends: substitution
message: "Use '%s' instead of '%s'."
level: suggestion
ignorecase: true
swap:
  leverage: use
  utilize: use
  in order to: to
  via: through|by|with
---
# Self-referential opener (scope to the first paragraph where the toolchain allows)
extends: existence
message: "Start with the content, not with what the page does."
level: suggestion
ignorecase: true
tokens:
  - this (?:page|guide|document|article|section) (?:describes|explains|covers|shows|walks you through)
  - in this (?:article|guide)
```

**2. Heuristic count (Vale `occurrence`, `metric`, `script`, or a custom pass; tune thresholds per corpus)**
- **Em-dash density**: `occurrence` with `scope: paragraph`, `max: 1`, `token: '—'`. This follows Microsoft's "don't overuse"; GitLab-style projects use `existence` instead.
- **Sentence length**: `occurrence` with `scope: sentence`, `max: 30` (Microsoft package default).
- **Uniform sentence rhythm** (low "burstiness"): `metric` with `sentence_length_sd`, e.g. `formula: sentence_length_sd`, `condition: "< 4.0"`, scoped per section. The threshold is unvalidated: no source gives a value. Google's "Choppy or long-winded sentences" and "Starting all sentences with the same phrase" support the idea.
- **Repeated sentence starts**: `script` (Tengo), counting first words per paragraph.
- **Bold-label share in lists** (share of list items that start with `**…**:`): `script`, or the doc toolchain's AST. A high share across every list on a page is the tell. A single term list is Microsoft's convention.
- **Notice density and stacked notices**: `script` or an AST check (two adjacent admonitions; more than N per page). Google's notices rule backs it.
- **Adjective or noun triplets in prose** ("fast, reliable, and secure"): `sequence` (`tag: JJ`, `tag: JJ`, `pattern: and`, `tag: JJ` with skips) plus a count per page. Expect noise from legitimate enumerations, so use this as a suggestion only.
- **Hedging clusters** ("may", "might", "could", "potentially", "arguably" in one sentence): `occurrence` with `scope: sentence`, `max: 2`, and a `(may|might|could|potentially|perhaps|arguably)` token.
- **Negative parallelism** ("not just X, but Y", "it's not X, it's Y", "isn't about X—it's about Y"): `existence` with `raw` regexes for the common templates catches the fixed forms. The free forms (two clauses in separate sentences, "rather than") need `sequence` or judgment. Wikipedia notes humans use it often, so count it rather than ban it.
- **Trailing "-ing" significance clauses** (", highlighting its importance", ", underscoring…", ", reflecting…"): `existence` on `, (highlighting|underscoring|emphasizing|reflecting|showcasing|ensuring|demonstrating)` catches the template. Whether the clause adds a real fact is judgment.

```yaml
extends: occurrence
message: "More than one em dash in this paragraph."
level: suggestion
scope: paragraph
max: 1
token: '—'
---
extends: existence
message: "Negative parallelism ('%s'): state the point directly."
level: suggestion
ignorecase: true
raw:
  - '\bnot (?:just|only|merely) [^.;]{1,60}?,? but(?: also)?\b'
  - "\\b(?:it|this|that)(?:'s| is) not (?:about )?[^.;]{1,40}?[,—;] (?:it|this|that)(?:'s| is)\\b"
```

**3. Human or agent judgment (no reliable pattern)**
- **Puffery about significance** ("plays a pivotal role in the evolving landscape"): the vocabulary is catchable; whether the claim is empty isn't.
- **Vague attribution** ("experts say", "many developers find", "it is widely considered"): `existence` can find the phrases, but in docs the real fix is a source or a deletion. A judge decides.
- **Restating or summary paragraphs** and repetition across a page and its linked pages (GitLab's first AI check): needs semantic comparison. An LLM pass or embedding similarity, not regex.
- **Punchy one-line closers and applause lines**: length and position are measurable (a last-paragraph sentence under N words), but whether it's earned (the Diátaxis tutorial recap) is judgment.
- **Invented syntax, flags, API names** (GitLab, Microsoft's tested-code rule): best checked mechanically against the real surface (CLI help, schema), not by a prose linter. This is a toolchain check, and it ranks above style.
- **Whether a list, table, bold label, or note belongs** at all, and whether a triad is padding.
- **Elegant variation** (renaming one concept several ways). Microsoft: "Use one term consistently". `consistency` handles known pairs; unknown synonyms need judgment.
- **Tone fit to page type** (austere reference vs. friendly tutorial, per Diátaxis).

**False positives, overall**
- Rules on fixed artifact phrases and wordy substitutions are low-noise. Rules on common words ("never", "always", "ensure", "will", passive voice) are high-noise in technical docs: the ExcessiveClaims data shows 88% of those words' hits were legitimate. Run any rule on the project's own docs and inspect the hits before making it a warning.
- Make vocabulary and structure tells suggestions with per-page counts. Keep errors for artifacts and project-decided bans, such as em dashes if the project chooses GitLab's rule. The GitLab MR shows that a linter people ignore costs more than it gives.

### Gaps
- No published, quantified false-positive rate was found for AI-tell rule sets specifically (ammil gives none). The only number found is the ExcessiveClaims comment (125/142 on an unnamed 950-file corpus) and the GitLab MR counts (308 → 17 after fixes and scoping, with 107 from one misapplied rule).
- No source gives a validated threshold for `sentence_length_sd` or for em-dash density that separates AI from human docs. The thresholds in the snippets above are placeholders to tune on the project's corpus.
- Vale's `scope` options for "first paragraph of a document" weren't confirmed in the fetched docs. A toolchain-level check (such as Ascribe's own checker, which knows page types) may be needed for position-dependent tells.
- The repetition, consistency, conditional, and capitalization docs pages were confirmed only for their one-line descriptions, not their full parameter tables.
