# Existing tools and skills for removing AI writing patterns

Researched 2026-11-03. Star counts are as the GitHub page showed them on that date. GitHub pages often hide the last-commit date from a fetch; where no date was visible, the notes say so.

## What agent skills, rules files, and prompts exist for removing AI writing patterns?

### Takeaway
A handful of Markdown skill files have most of the adoption: blader/humanizer (about 64k stars), hardikpandya/stop-slop (about 18.6k), and conorbronsdon/avoid-ai-writing (about 5.4k). Dozens of forks and directory copies descend from them. Almost all of them trace back to Wikipedia's "Signs of AI writing" page, and most are general-prose tools with a "technical" mode at most. The serious ones have moved from plain word lists to severity tiers, "weak alone" patterns, detect-only modes, and a deterministic scorer next to the prompt.

### Cited Findings

**The common source: Wikipedia, "Signs of AI writing" (WikiProject AI Cleanup)**
- It is an advice page from WikiProject AI Cleanup, not policy, and is semi-protected. Its sections are: Caveats; Content (inflated significance, promotional notability claims, superficial analysis, vague attributions); Language and grammar (word choice, copula avoidance, negative parallelisms, rule of three); Style; Communication intended for the user; Markup irregularities; Citations (broken links, invalid DOIs and ISBNs); comment and edit-summary indicators; Miscellaneous; Signs of human writing; Ineffective indicators; Historical indicators. — [Wikipedia:Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- The page calls itself "descriptive, not prescriptive". It says no single sign is proof, that the signs are symptoms and fixing surface features "can make detection harder", that detectors have real error rates, and that people judge authorship at about chance. — [Wikipedia:Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Its "signs of human writing" section is short. One of its stronger signals is whether the writer can explain their own editorial choices. — [Tech Brew, 2026-01-21](https://www.techbrew.com/stories/2026/01/21/ai-writing-claude-code-plugin)

**blader/humanizer, https://github.com/blader/humanizer**
- Author: blader. Press coverage names the author as Siqi Chen. MIT licence. About 64.4k stars, 5.1k forks, 103 commits. No date or version is visible; there is a CHANGELOG.md, and the Claude Code plugin needs Claude Code 2.1.142 or later. It is invoked as `/humanizer` and works in Claude Code, Codex, and Claude.ai. — [GitHub](https://github.com/blader/humanizer); [Tech Brew](https://www.techbrew.com/stories/2026/01/21/ai-writing-claude-code-plugin)
- The current README lists 26 patterns in six categories:
  - A. Staging instead of stating: "Not X but Y", one-line closers, sayings that sound deep, a staged run-up, arguing with no one.
  - B. Rhythm by rule: forced triads, repeated openings, and four patterns marked *weak alone*: dashes, stacked qualifiers, hyphenated pairs, and passive voice or missing subjects.
  - C. Inflation and borrowed authority: overused AI words, inflated significance, vague connections, shallow -ing riders, sales language, borrowed authority, avoiding "is/are/has".
  - D. Formatting by rule: decorative bold, decorative headings, and curly quotes (weak alone).
  - E. Chat and draft leftovers: chatbot residue, knowledge-limit disclaimers, a heading repeated in the first sentence, writing about the document instead of its subject.
  - F. Writing for the wrong reader: re-explaining what the reader already knows.
  
  — [GitHub](https://github.com/blader/humanizer)
- Structure:
  - A table per pattern with before and after examples.
  - A list of the "five strongest tells", the ones that justify an edit on a single sighting.
  - A worked rewrite of a whole post.
  - A process of mark tells, draft, critique what still sounds artificial, and write a final version. Given text, it shows all three stages.
  - It asks rather than invent a missing fact.
  - Voice calibration from a sample of the user's writing.
  
  — [GitHub](https://github.com/blader/humanizer)
- Scope: personal writing keeps its quirks, and technical and reference prose is kept "neutral and plain". When it edits files it changes only prose and leaves code, data, frontmatter, and link targets alone. — [GitHub](https://github.com/blader/humanizer)
- The README says it writes for human readers, not detectors, and that detectors still flag most of its output. — [GitHub](https://github.com/blader/humanizer)
- Its pattern count has changed between versions. Secondary sources say 24 or 29 patterns, and the current README lists 26. The forks wey-gu/humanizer and lanxkk/humanizer still use the older lists. — [Feedbagel summary](https://feedbagel.com/post/new-humanizer-tool-uses-wikipedias-ai-detection-guide-to-improve-ai-writing-qual); [GitHub search results: wey-gu](https://github.com/wey-gu/humanizer), [lanxkk](https://github.com/lanxkk/humanizer)
- Forks and variants:
  - Aboudjem/humanizer-skill lists 53 patterns, only scans and never rewrites, and reads a `humanizer-context.md` for the user's own samples and banned words.
  - jpeggdev/humanize-writing combines an eight-pass editing process with blader's patterns.
  
  — [Aboudjem/humanizer-skill](https://github.com/Aboudjem/humanizer-skill); [jpeggdev/humanize-writing](https://github.com/jpeggdev/humanize-writing)

**hardikpandya/stop-slop, https://github.com/hardikpandya/stop-slop**
- Author: Hardik Pandya, a product designer. MIT licence. About 18.6k stars, 1.4k forks, 12 commits; no date visible. It describes itself as "A skill file for removing AI tells from prose." — [GitHub](https://github.com/hardikpandya/stop-slop)
- Files: `SKILL.md` holds the core rules, with `references/phrases.md`, `references/structures.md`, and `references/examples.md` for before and after rewrites. — [GitHub](https://github.com/hardikpandya/stop-slop)
- Patterns:
  - Banned phrases: throat-clearing openers, emphasis crutches, business jargon, all adverbs, vague declaratives, meta-commentary.
  - Structural clichés: binary contrasts, negative listings, dramatic fragmentation, rhetorical setups, false agency, a narrator-from-a-distance voice, passive voice.
  - Sentence rules: no Wh- openers, no em dashes, no staccato fragments, no lazy extremes, active voice only.
  
  — [GitHub](https://github.com/hardikpandya/stop-slop)
- A rubric scores prose from 1 to 10 on Directness, Rhythm, Trust, Authenticity, and Density, and text under 35/50 gets revised. — [GitHub](https://github.com/hardikpandya/stop-slop)
- It targets general prose: essays, posts, and product writing. It says nothing about technical docs. — [GitHub](https://github.com/hardikpandya/stop-slop)

**conorbronsdon/avoid-ai-writing, https://github.com/conorbronsdon/avoid-ai-writing**
- Author: Conor Bronsdon. MIT licence. About 5.4k stars, 476 forks, 636 commits. The README refers to release v3.36.0; no date visible. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- Catalog: 74 pattern categories under content, language, structure, communication, and meta. Later additions cover hashtag stuffing, bare-noun bullet lists, tool fingerprints (placeholders, citation markup, UTM parameters), conversational register, and needless hyphenation. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- Word tiers: a replacement table of 112 entries plus 10 phrases.
  - Tier 1A holds frequency markers, such as "delve" and "tapestry".
  - Tier 1B holds clarity edits, such as "in order to" and "utilize", weighted lower "so wordiness fixes alone don't push a document toward an AI classification".
  - Tier 2 words count only when they cluster.
  - Tier 3 phrases count when one repeats or when three different ones appear.
  
  — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- Modes:
  - Rewrite (`--iterate 1|2`).
  - Detect: findings at severity P0, P1, or P2, with clear problems kept apart from judgment calls.
  - Edit: minimal changes in place. It refuses source code, config, and generated data, and leaves human-written passages alone.
  - Voice profiles: casual, professional, technical, warm, blunt.
  
  "Technical" is the default context for the detector gate. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- Tooling: a 0–100 detector with no dependencies, a CLI, a GitHub Action, a pre-commit hook that fails at 6 or more findings per file by default, an MCP server, and a "preservation validator" that checks code, links, and structure survive. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- Carve-outs: "real-time analytics" keeps its hyphen, changelogs and parameter docs may use bare noun phrases, and an honest contrast is kept. Ignore markers exclude quoted specimens. The README says flags are "signals, not proof", and that the validator doesn't check meaning. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)

**realrossmanngroup/no_ai_slop_writing_rules, https://github.com/realrossmanngroup/no_ai_slop_writing_rules**
- About 711 stars, 80 forks, 7 commits; no date visible. It is an Agent Skills package with `CLAUDE.md` holding 24 rules, `AGENTS.md`, and `skills/no-ai-slop/SKILL.md`, which gives WRONG and RIGHT examples and ends with a self-check. — [GitHub](https://github.com/realrossmanngroup/no_ai_slop_writing_rules)
- `references/ai-writing-detection.md` holds the banned lists: verbs, adjectives, hedges, academic tells, and structural and statistical patterns.
- A second skill imitates Louis Rossmann's voice, built from a 513,683-word corpus: "Claim-Mechanism-Reality" paragraphs, dense numbers, and contractions.
- Rule categories: em dashes, intensifiers, filler, hollow statements, fabricated facts, AI transition words, dramatic headings. — [GitHub](https://github.com/realrossmanngroup/no_ai_slop_writing_rules)

**Smaller skills and copies, seen only on directory pages and not verified at the source**
- jalaalrd/anti-ai-slop-writing: 50+ banned words, for Claude Code, Codex, Cursor, Gemini CLI, and others. — [vibed-lab listing](https://vibed-lab-agents.pages.dev/agent/jalaalrd-anti-ai-slop-writing)
- MohamedAbdallah-14/unslop, a plugin for several platforms. — [TomeVault](https://tomevault.io/tome/MohamedAbdallah-14/unslop)
- rohitg00 "deslop". — [TypingMind](https://www.typingmind.com/skills/rohitg00-deslop)
- Byk3y "no-slop". — [skillsdirectory](https://www.skillsdirectory.com/skills/byk3y-no-slop)
- reidbaker's "natural-writing" skill in the flutter/flutter agents folder. — [skillsmp](https://skillsmp.com/skills/flutter-flutter-agents-agents-reidbaker-agent-skills-natural-writing-skill-md)
- elithrar "anti-slop". — [ui-skills](https://www.ui-skills.com/skills/elithrar/anti-slop)
- cybertronai SutroYaro "Anti-Slop Guide". — [SutroYaro](https://cybertronai.github.io/SutroYaro/tooling/anti-slop-guide)
- Copies of stop-slop: jssblck/agents and b-open-io/prompts. Claude Market rates the jssblck copy 63/100 with about 1,000 installs. — [Claude Market](https://www.claudemarket.ai/skills/jssblck/agents/stop-slop)
- Cursor rules: no official Cursor guidance was found. Community `.mdc` rules repeat the same items, such as "delve → explore, dig into" and "no em dashes" or "at most one per paragraph". — [ClawHub avoid-ai-writing](https://clawhub.ai/conorbronsdon/avoid-ai-writing); [Will Francis](https://willfrancis-com.pages.dev/how-to-stop-claude-writing-like-an-ai/)

### Inferences
- The space has gathered around three files, and most of the "new" skills are forks or repackagings of humanizer or stop-slop. Being on a skill directory doesn't show that a skill is original.
- The way they're organized is converging:
  - Patterns grouped by family, with before and after pairs.
  - Weighting by severity or confidence.
  - A pass that drafts, critiques, and writes a final version.
  - A detect-only mode.
  - Increasingly, a deterministic scorer beside the prompt (avoid-ai-writing, Aboudjem's scan-only fork).
- Only avoid-ai-writing (the technical profile, its refusal to touch code, carve-outs for changelogs and parameter docs) and humanizer ("technical and reference prose neutral and plain", leaving code, frontmatter, and link targets alone) say anything about docs-as-code.

### Gaps
- Last-commit dates for humanizer, stop-slop, avoid-ai-writing, and the Rossmann repo weren't visible in the fetched pages. Each repo's CHANGELOG has them.
- No Anthropic first-party "anti-slop" skill was found in anthropics/skills during this pass, and the repo wasn't fetched directly.
- The contents of stop-slop's `phrases.md` and of the Rossmann banned-word file weren't fetched, so their exact word lists aren't recorded here.

## What Vale styles and other linters target AI tells, and which rule types do they use?

### Takeaway
At least four Vale packages target AI tells as of late 2026:
- tbhb/vale-ai-tells is the largest, with about 137 rules, and is aimed explicitly at technical docs.
- ammil-industries/vale-signs-of-ai-writing is built from the Wikipedia list.
- JMill's "Deslop".
- Vale's own Voices, which turns skill instructions into rules.

They mostly use existence and substitution rules, plus sequence, part-of-speech, and length rules. Outside Vale, the useful tools are new regex scorers (slop-guard, unsloppify, slopless, anti-slop) rather than proselint, write-good, or alex, which have no AI-specific rules.

### Cited Findings

**tbhb/vale-ai-tells, https://github.com/tbhb/vale-ai-tells**
- About 122 stars, 13 forks, 290 commits; no date visible. Every rule defaults to `error`. The README says the package targets **technical documentation** and is "less useful for creative or marketing writing". — [GitHub](https://github.com/tbhb/vale-ai-tells)
- About 137 rule files. Those visible include:
  - Vocabulary and phrasing: `OverusedVocabulary` ("delve", "comprehensive"), `OverusedVocabularyVerbs`, `AIAdjectiveNounPairs`, `AICompoundPhrases`, `FillerPhrases`, `FillerIntensifier`, `HedgingPhrases`, `DefensiveHedges`, `PromotionalPuffery`, `FormalTransitions`, `ConclusionMarkers`, `SequencingMarkers`.
  - Openers, closers, and headings: `ClosingPleasantries`, `HollowAcknowledgment`, `OpeningCliches`, `AnnouncementHeadings`, `ExplainerHeadings`, `MarketingHeadings`, `MicDrop`, `MicDropHeadings`, `ListIntroductions`.
  - Sentence shapes: `ContrastiveFormulas` ("It's not just X; it's Y"), `ContrastiveNegation`, `NegatedPair`, `PseudoCleft`, `RhetoricalSelfAnswer`, `ShellNounCopula`, `ServesAsDodge`, `LabelAndExplain`, `ParticipialPadding`, `NounString`.
  - Voice: `Metacommentary`, `SelfReference`, `AnthropomorphicCognition`, `AbsoluteAssertions`.
  - Punctuation: `EmDashUsage`, `SemicolonUsage`, `ColonUsage`.
  - A family of about 45 `Figurative*` rules: "lands", "carries", "rides", "surfaces", "sweeps".
  - Metaphor families: journey, growth, motion, and enforcement metaphors.
  
  — [GitHub](https://github.com/tbhb/vale-ai-tells)
- A second style, `ai-tells-commits`, holds 15 rules for commit messages, such as `CommitSelfReference`, `CommitTrailingJustification`, `CommitTestEnumeration`, `CommitMarketingAdjectives`, `CommitUnquantifiedClaims`, and `CommitFileListing`. — [GitHub](https://github.com/tbhb/vale-ai-tells)
- Rule types: `EmptyPadding` and `OverusedVocabularyVerbs` are described as sequence rules, and `NounString` uses part-of-speech tagging. The rest weren't stated in the fetched text. — [GitHub](https://github.com/tbhb/vale-ai-tells)
- False-positive notes in the rule descriptions:
  - `BareHolds` is "the loudest rule on ordinary technical writing", and `NamedAdjective` and `FigurativeNouns` are also heavy.
  - Several `Figurative*` rules say to switch them off for physics, finance, sports, or law.
  - `FillerIntensifier` and `EmptyPadding` "will flag literal uses".
  - `ContrastiveNegation` "may fire on terse spec lists".
  - It recommends writing your own `.vale.ini` rather than copying the repo's development config.
  
  — [GitHub](https://github.com/tbhb/vale-ai-tells)
- EveryDev lists a "vale-ai-tells" v1.0.0 by Krishna Sunkam with 17 rules (6 errors, 11 suggestions) and a bundled Claude Code skill, which flags em dashes, abstract triads, and antithesis flourishes. That doesn't match the tbhb repo's numbers, so it may be a different package under the same name. — [EveryDev](https://www.everydev.ai/tools/vale-ai-tells); conflicts with [GitHub tbhb](https://github.com/tbhb/vale-ai-tells)

**ammil-industries/vale-signs-of-ai-writing, https://github.com/ammil-industries/vale-signs-of-ai-writing**
- Author: Christian Miles (ammil industries), 2025-12-08. Licensed CC BY-SA 4.0 because it derives from the Wikipedia page. — [ammil industries blog](https://ammil.industries/signs-of-ai-writing-a-vale-ruleset)
- Three tiers, matched to how confident each rule is:
  - Error: definite artifacts, such as chatbot phrases, technical glitches, placeholders, and tracking URLs.
  - Warning: likely patterns, such as hedging clusters, knowledge-cutoff references, and enumeration style.
  - Suggestion: things common in human writing too, such as some vocabulary, transitions, passive voice, and symbolic language.
  
  — [ammil industries blog](https://ammil.industries/signs-of-ai-writing-a-vale-ruleset)
- Limitations it states:
  - Vale can't match stray Markdown syntax, so that sign was dropped.
  - The rules will go stale as models change.
  - Claude helped write the rules.
  
  — [ammil industries blog](https://ammil.industries/signs-of-ai-writing-a-vale-ruleset)

**Deslop (JMill)**
- A Vale package with one rule file per family of tells, such as `SlopVocab` and `HollowCloser`. Nothing ships as `error` by default. — [TomeVault listing via search](https://tomevault.io/tome/JMill/deslop). The page returned 503 when fetched, so this rests on the search snippet alone.

**Voices (Vale), https://github.com/jdkato/voices**
- Author: Joseph Kato, who maintains Vale. Announced 2026-09-02 with "Std". Needs Vale 3.20.0. — [Vale blog](https://vale.sh/blog/voices)
- It takes an AI-writing instruction set, mainly the `no-ai-slop` output style, and splits it into a persona (left to the model), constraints (enforced by Vale), and structural guardrails. Vale's markup parsing keeps the rules on prose. — [Vale blog](https://vale.sh/blog/voices)
- Rule types:
  - Existence: token lists such as `Direct.Hedging` and `Voices.Banned`.
  - Substitution.
  - Length: `Direct.Length`, with configurable limits.
  - A Hunspell vocabulary of Basic English's 850 words, for the "Simple" voice.
  - Inheritance from Std, with `+` and `-` keys to edit lists.
  
  — [Vale blog](https://vale.sh/blog/voices)
- Its argument, with measurements:
  - A skill prompt cost 3,777 tokens per request and a brief 1,535, while rules cost nothing on a clean draft. Prompt caching narrows the gap.
  - Vale took about 50 ms against 23.5 s for a model review.
  - Exit codes let CI gate on the rules.
  - Each rule ships with a test.
  
  — [Vale blog](https://vale.sh/blog/voices)
- Its stated limits:
  - Rules catch patterns and counts but not judgment. The model's review caught vague wording that no rule could.
  - A repair can introduce new violations. One example took four passes to go from 25 to 0.
  
  — [Vale blog](https://vale.sh/blog/voices)

**Linters outside Vale**
- slop-guard, https://github.com/eric-tramel/slop-guard. By Eric W. Tramel of NVIDIA, 2026-02-17; on PyPI, latest 0.4.1, with a Rust crate of the same name on docs.rs.
  - It is pure regex. It runs as an MCP server (`check_slop`, `check_slop_file`) and a CLI (`sg`).
  - Rules:
    - About 80 vocabulary words at 2 points each.
    - About 50 phrases at 3 points each.
    - Structure: bold header then explanation, repeated; runs of 6 or more bullets; triads.
    - Tone: "Certainly!", "studies show".
    - Rhythm: flags sentence-length variation below 0.3 over 5 or more sentences.
    - Em dashes above 1 per 150 words.
    - "X, not Y" pairs and pithy fragments.
  - The score is `100·exp(−0.04·density)` per 1,000 words, with a multiplier for repeats of the same pattern. Bands: ship above 80, review at 60–79, revise below 60.
  
  — [Tramel blog](https://eric-tramel.github.io/blog/2026-02-18-slop-guard); [PyPI](https://pypi.org/project/slop-guard/)
- unsloppify (petems): a Python CLI for Markdown, shipped with a Claude Code skill. Regex handles the hard rules (em dashes, banned phrases, structural clichés) and the skill handles judgment calls. It emits GitHub Actions annotations. — [GitHub](https://github.com/petems/unsloppify); [PyPI](https://pypi.org/project/unsloppify/)
- anti-slop (SkeinLLC): a VS Code extension and npm CLI that flags banned vocabulary, em dashes, and rule-of-three in Markdown and plain text, and "does not rewrite your prose". — [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=SkeinLLC.anti-slop)
- slopless (berelevant-ai / seochecks-ai): a deterministic Node/TypeScript linter. Its wiki has a comparison page setting it against proselint and write-good. — [slopless wiki](https://github.com/berelevant-ai/slopless/wiki/Comparison)
- textlint-rule-preset-ai-writing (textlint-ja) is for Japanese AI prose only. No English textlint, proselint, write-good, or alex rule for AI tells was found. — [sourcepulse](https://www.sourcepulse.org/projects/7274325); [search summary](https://best-of-web.builder.io/library/amperser/proselint)
- Model-side, for fiction: Sam Paech's Antislop. Its sampler backtracks to suppress banned strings, words, phrases, or regexes. FTPO fine-tuning suppresses more than 8,000 patterns with a reported 90% cut in slop, while DPO degraded writing quality. MIT licence, at github.com/sam-paech/auto-antislop. EQ-Bench's slop score reports "delves" at about 25 times its pre-2023 frequency. — [arXiv 2510.15061](https://arxiv.org/abs/2510.15061); [Tramel blog](https://eric-tramel.github.io/blog/2026-02-18-slop-guard)

### Inferences
- Vale existence and substitution rules handle the word and phrase layer well. The structural tells need more:
  - Contrast pairs, mic-drop closers, and staged run-ups need sequence or part-of-speech rules (tbhb).
  - Rhythm and density tells, such as uniform sentence length and em dashes per word, need a metric or a custom scorer (slop-guard), which plain Vale existence rules can't express.
- Severity tiers based on confidence (ammil, avoid-ai-writing) and "off by default" (Deslop) are the main defences against false positives. tbhb defaults everything to `error`, which makes sense only once a project has trimmed the set.

### Gaps
- The exact Vale `extends:` type of each tbhb rule wasn't captured; the YAML files would answer it.
- No LanguageTool rule set for AI tells was found.
- Deslop's repository URL and contents couldn't be verified.

## Which tools target technical documentation or developer-tool copy specifically?

### Takeaway
Few do. tbhb/vale-ai-tells is the only one that says it is for technical documentation, and it adds a commit-message style. avoid-ai-writing has a "technical" profile and carve-outs for changelogs and parameter docs. humanizer and unsloppify handle Markdown and docs-as-code files safely. Write the Docs hasn't published a list of tells; its discussion is about accuracy and missing user context, not phrasing.

### Cited Findings
- tbhb/vale-ai-tells targets technical documentation, has 15 commit-message rules, and notes which rules fire on spec lists and ordinary technical prose. — [GitHub](https://github.com/tbhb/vale-ai-tells)
- avoid-ai-writing's default detector context is "technical". Its carve-outs keep bare noun phrases in changelogs and parameter docs and keep compound terms like "real-time analytics". Its Edit mode refuses source code, config, and generated data. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
- humanizer keeps "technical and reference prose neutral and plain" and leaves code, data, frontmatter, and link targets untouched. — [GitHub](https://github.com/blader/humanizer)
- unsloppify is built for Markdown and CI annotations. — [GitHub](https://github.com/petems/unsloppify)
- Write the Docs: no published list of tells was found. Blog coverage says the WTD Slack's loose consensus is that readers often can't tell AI docs from human ones. The failures people name are factual errors in clean prose, docs that describe the code rather than where users get stuck, and the "README style". — [engineered.at / passo.uno](https://passo.uno/reconsider); [I'd Rather Be Writing](https://idratherbewriting.com/blog/whats-wrong-ai-webinar-recording); [engineered.at](https://engineered.at/articles/what-s-wrong-with-ai-generated-docs)

### Inferences
- For docs, the patterns that matter most are structural, and few skills cover them well:
  - Announcement and explainer headings.
  - "This section covers…" metacommentary.
  - Bold-label bullet runs.
  - Puffery in feature descriptions.
  - Restating the heading in the first sentence.
- tbhb's heading rules and humanizer's category E come closest.
- None of the tools checks accuracy against the code. The docs community counts that as the main failure, and it lies outside what pattern tools can reach.

### Gaps
- No tool aimed at developer-tool *marketing* copy was found. avoid-ai-writing's "sales language" and "promotional" patterns are the nearest.

## What do the better tools do that the weaker ones don't, and what are the criticisms?

### Takeaway
The better tools treat tells as weighted signals, not bans:
- Weak-alone and cluster thresholds.
- Lower weight for plain wordiness.
- Domain carve-outs.
- A detect-only mode.
- Keeping the meaning, and asking rather than inventing.
- A critique pass after the rewrite.

The weaker ones are flat lists of banned words and absolute rules ("no adverbs", "no em dashes") that strip real meaning, go stale, and create a new uniform style. Every tool agrees that none of them proves authorship or defeats detectors.

### Cited Findings
- Things the better tools do:
  - Humanizer marks dashes, qualifiers, hyphen pairs, passive voice, and curly quotes as "weak alone", counted only together, because careful writers use them on purpose. — [GitHub](https://github.com/blader/humanizer)
  - avoid-ai-writing weights clarity edits ("utilize") below frequency markers ("delve") so wordiness alone doesn't make a document look AI-written. It separates clear problems from judgment calls and has a validator that checks code, links, and structure survive. — [GitHub](https://github.com/conorbronsdon/avoid-ai-writing)
  - Humanizer's guidance says some AI patterns serve clarity and should be kept. — [Capital & Compute comparison](https://capitalandcompute.net/blog/stop-slop-vs-humanizer/)
  - One skill says not to swap a banned word for a synonym but to restructure the sentence to say what it means, since the real problem is a sentence that carries no information. — [Capital & Compute](https://capitalandcompute.net/blog/stop-slop-vs-humanizer/)
- Criticisms of over-correction:
  - stop-slop's "kill all adverbs, no -ly words, no softeners, no hedges", naming "really", "just", and "actually", removes words that express real uncertainty.
  - stop-slop's changelog only adds rules and never retires one.
  - Lists of banned words age badly as vendors patch the famous ones.
  
  — [Capital & Compute](https://capitalandcompute.net/blog/stop-slop-vs-humanizer/)
- The size of a skill costs context: humanizer is about 2.3 times all of stop-slop's files and 12.7 times its core file, and an agent reads all of it each time the skill runs. — [Capital & Compute](https://capitalandcompute.net/blog/stop-slop-vs-humanizer/)
- A new tell: if every chatbot avoids the same words and structures, the avoidance itself becomes a tell. People are also adopting AI phrasing in their own writing. — [Tech Brew](https://www.techbrew.com/stories/2026/01/21/ai-writing-claude-code-plugin)
- Weak results in testing:
  - Tech Brew's test of Humanizer found the rewrite still unnatural. It said it had replaced em dashes and hadn't, and GPTZero still rated the text 100% AI.
  - Another test found it kept formulaic comma lists that the Wikipedia guide itself flags.
  
  — [Tech Brew](https://www.techbrew.com/stories/2026/01/21/ai-writing-claude-code-plugin); [Feedbagel](https://feedbagel.com/post/new-humanizer-tool-uses-wikipedias-ai-detection-guide-to-improve-ai-writing-qual)
- Regex scorers are gamed: a model can learn to avoid the specific regexes and produce new formulaic patterns. slop-guard was calibrated on only seven essays from before 2020 and penalises long texts; Hamming's 14,000-word lecture scored 39. — [Tramel blog](https://eric-tramel.github.io/blog/2026-02-18-slop-guard)
- Density matters more than single errors: AI prose is usually competent but becomes monotonous through repetition. Slop-guard's first version gave a 100 to a post a human reader had flagged for repeated contrast pairs and colon elaborations. — [Tramel blog](https://eric-tramel.github.io/blog/2026-02-18-slop-guard)
- LLM-as-judge is criticised as slow, costly, non-deterministic, and "circular". Vale's counterpoint is that rules can't judge vagueness, so a model pass is still needed. — [Tramel blog](https://eric-tramel.github.io/blog/2026-02-18-slop-guard); [Vale blog](https://vale.sh/blog/voices)
- Fixing surface features hides the signal rather than fixing the problem. — [Wikipedia:Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- The em dash is losing value as a signal: models have been trained away from it, and one source says overuse is now mainly a Claude Opus 4.5 trait. — [pasqualepillitteri.it](https://pasqualepillitteri.it/en/news/11195/new-em-dash-9-ai-writing-tells); [search summary](https://willfrancis-com.pages.dev/how-to-stop-claude-writing-like-an-ai/)

### Inferences
- The strongest design pairs three things:
  - A deterministic checker for countable tells (vocabulary, phrases, punctuation density, heading shapes), with confidence tiers and domain carve-outs.
  - A short model-side guide for judgment tells: staging, inflation, metacommentary, re-explaining.
  - A rule that the meaning, code, and links stay intact.
  
  Voices, unsloppify, and avoid-ai-writing each take a version of this approach.
- For technical docs, absolute bans do the most harm:
  - Passive voice is legitimate when the actor is unknown or irrelevant.
  - Hedges carry real conditions ("usually", "may").
  - Bare noun phrases are standard in reference tables and changelogs.
  - Domain words like "robust", "leverage", and "surface" have technical meanings.
  
  The tools that work for docs either carve these out or mark them weak alone.

### Gaps
- No independent, measured study of whether any skill improves clarity or reader trust was found. The evidence is author claims, one journalist's test, and blog comparisons.
- The Hacker News threads on humanizer and stop-slop weren't located directly; only a summary of one was.
