# Catalogs of AI-writing tells, sorted by whether they apply to technical documentation

Notes gathered 2026-11-03. Marked throughout: **[catalog]** means at least one documented catalog lists the pattern, and **[inference]** means the researcher's own judgment, mostly about which kind of writing a pattern applies to. No source measured harm to readers of technical docs, so every "harmful in docs" judgment below is inference unless a source is cited for it.

Sources used, with dates:

- **WP-now**: Wikipedia, "Wikipedia:Signs of AI writing" (WikiProject AI Cleanup advice page), current revision read 2026-11-03. The page says "As of October 2026". https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing
- **WP-2025**: the same page, revision of 2025-10-31T18:18:16Z, read through the MediaWiki API. Several sections were later cut, merged, or moved to "Historical indicators", and this revision still has them (false ranges, outlines of negatives, subject lines, elegant variation in the main list). https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history
- **WP-Style**: the subpage "Wikipedia:Signs of AI writing/Style", read 2026-11-03. https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style
- **tropes.fyi**: "AI Writing Tropes to Avoid" by Ossama Chaib (ossama.is), a gist last active 2026-10-23, plus the tropes.fyi directory. https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1 and https://tropes.fyi/
- **Reynolds**: Alex Reynolds, "AI writing: avoid the slop", updated October 2026, adapted from tropes.fyi. https://www.alexreynolds.com/tools/guides/anti-slop
- **Bouchard**: Louis-François Bouchard, "The Anti-Slop Writing Guide", October 2026 edition (GitHub). https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md
- **Economist**: "How to spot AI writing", The Economist, 2026-07-30. Paywalled. Known here through Wikipedia's citation and a secondary summary at TheOutpost.ai (around 2026-08-04). https://www.economist.com/culture/2026/07/30/how-to-spot-ai-writing and https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/
- **WaPo**: Merrill, Chen and Kumer, "What are the clues that ChatGPT wrote something? We analyzed its style.", Washington Post, 2025-11-13, syndicated by the Boston Globe the same day. Not read directly; known through Wikipedia's citations and the Globe's search snippet. https://www.washingtonpost.com/technology/interactive/2025/how-detect-chatgpt-em-dash/ and https://www.bostonglobe.com/2025/11/13/business/chatgpt-writing-style-clues/
- **Kriss/NYT**: Sam Kriss, "Why Does A.I. Write Like … That?", New York Times Magazine, 2025-12-03. Paywalled; known only through Wikipedia's citations of it for vocabulary, the rule of three, and em dashes. https://www.nytimes.com/2025/12/03/magazine/chatbot-writing-style.html
- **Belcher/Chronicle**: Wendy Belcher, "10 Ways AI Is Ruining Your Students' Writing", Chronicle of Higher Education, 2025-09-16. Known through Wikipedia's citation for elegant variation. https://www.chronicle.com/article/10-ways-ai-is-ruining-your-students-writing
- **Russell et al.**: "People who frequently use ChatGPT for writing tasks are accurate and robust detectors of AI-generated text", ACL 2025. Wikipedia cites it for title case, negative parallelism, the rule of three, and AI vocabulary. https://aclanthology.org/2025.acl-long.267/
- **Max Read**: "Who is Elara Voss?" (Substack), cited by Wikipedia as the source for fiction-specific tells. https://maxread.substack.com/p/who-is-elara-voss

---

## 1. What Wikipedia's "Signs of AI writing" lists, by category

### Takeaway
The page is the most thorough and best-sourced catalog. It splits the signs into Content, Language and grammar, Style, Communication intended for the user, Markup, Citations, Edit summaries, and Miscellaneous. It separately lists "Ineffective indicators", and since 2026 it moves dated tells (promotional tone, section summaries, em dashes, elegant variation) to "Historical indicators". It calls itself descriptive, a list of signs of a deeper problem, and says it is "less useful for texts which are not informational writing".

### Cited findings

**Framing and caveats**
- The page is "a field guide to help detect undisclosed AI-generated content *on Wikipedia*: while some of the signs may be broadly applicable, some may not apply in a non-Wikipedia context". It is "descriptive, not prescriptive". The signs are "only potential signs of a problem, not the problem itself", and fixing the surface while leaving the deeper problem can make detection harder. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Fiction is out of scope: "the many tells specific to fiction (whispering woods, Elara Voss, etc.) are less relevant in Wikipedia and are not listed here." — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), pointing to [Max Read](https://maxread.substack.com/p/who-is-elara-voss)
- The root cause it gives: LLMs "regress to the mean", replacing specific, unusual facts with generic, positive, important-sounding statements. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Its warning about human detection: people are "notoriously bad" at telling human from LLM text, and human writing is converging on LLM style. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- The most common signs of newly generated text as of October 2026 are Markdown, AI edit summaries, hedging disclaimers about sources and usage, vague expressions of connection, and undue emphasis (on significance or identification). — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Content**
- *Undue emphasis on significance, legacy, and broader trends.* Words to watch: "stands/serves as", "is a testament/reminder", "a crucial/pivotal/vital/significant/key role/moment", "underscores/highlights its importance", "reflects broader", "symbolizing its ongoing/enduring/lasting", "setting the stage for", "marking/shaping the", "represents/marks a shift", "key turning point", "evolving landscape", "focal point", "indelible mark", "deeply rooted". Example: an institute's founding "marking a pivotal moment in the evolution of regional statistics". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Canned emphasis on notability, attribution, and media coverage.* "independent coverage", "featured/profiled in", "active social media presence"; listing kinds of sources to prove notability. More common from 2025. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Undue emphasis on identification by primary or tertiary sources* (from 2026): "Databases/records/listings identify [subject] as…", "is listed in". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Superficial analyses*: trailing "-ing" clauses. Words: "highlighting/underscoring/emphasizing…", "ensuring…", "reflecting/symbolizing…", "contributing to…", "cultivating/fostering…", "encompassing…", "enhancing…", "valuable insights", "align/resonate with". Example: "…creating a lively community within its borders." — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Vague expression of connection or association*: "in connection with", "connected to", "in association with", "associated with", used instead of stating the relationship. Example: "That literature — associated principally with Malcolm Comeaux…". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Vague attributions and overgeneralization* (weasel wording): "Industry reports", "Observers have cited", "Experts argue", "Some critics argue", "several sources" when few are cited, and "such as" before an exhaustive list. Example: a river "is of interest to researchers and conservationists". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history)
- *Outline-like conclusions about challenges and future prospects*: "Despite its… faces several challenges…", "Despite these challenges", "Challenges and Legacy", "Future Outlook". The sign is the formula, not the mention of challenges. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *An "Awards and recognition" section*, and "X and Y" section titles generally. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Language and grammar**
- *High density of "AI vocabulary".* The full box: Additionally (especially at the start of a sentence), align with, boasts (meaning "has"), bolstered, crucial, deep dive, delve, emphasizing, enduring, enhance, fostering, highlight (verb), landscape (abstract noun), robust, tapestry (abstract noun), and more. The page dates the words by model era:
  - 2023 to mid-2024 (GPT-4): Additionally, boasts, bolstered, crucial, delve, emphasizing, enduring, garner, intricate/intricacies, interplay, key, landscape, meticulously, pivotal, underscore, tapestry, testament, valuable, vibrant.
  - Mid-2024 to mid-2025 (GPT-4o): align with, bolstered, crucial, emphasizing, enhance, enduring, fostering, highlighting, pivotal, showcasing, underscore, vibrant.
  - Mid-2025 to mid-2026 (GPT-5): emphasizing, enhance, highlighting, showcasing.
  - Mid-2026 on (GPT-6): challenge, clearer, dependable, echoed, foster, leverage, matters, multifaceted, practical, prioritize, quietly, steady, universally, with "much more period and comma usage" and sentences of about 8–20 words. This entry cites a single review site.
  - Grok: causal, empirical, correlate, and still underscore.
  - "delve" dropped off sharply in 2025.
  - The page says to take the list literally: a word's synonyms aren't therefore overused, and context matters (a literal "underscore").
  - Example: "Somali cuisine is an intricate and diverse fusion… drawing from the rich tapestry of… flavours." — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), citing [Kriss/NYT](https://www.nytimes.com/2025/12/03/magazine/chatbot-writing-style.html), [WaPo](https://www.washingtonpost.com/technology/interactive/2025/how-detect-chatgpt-em-dash/), [Russell et al.](https://aclanthology.org/2025.acl-long.267/), and Juzek & Ward (ACL 2025)
- *Avoidance of basic copulatives*: "serves as / stands as / marks / functions as / operates as / represents [a]", "boasts / features / maintains / offers [a]", "refers to" in place of "is" or "has". Also common in AI copyedits. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Negative parallelisms*, in three forms:
  - "Not just X, but also Y": "Not only… but…", "It is not just…, it's…". Example: "It's not just about the beat riding under the vocals; it's part of the aggression and atmosphere."
  - "Not X, but Y": "It's not…, it's…", and "no…, no…, just…".
  - "Y rather than X": reversed, and "particularly common in recent output".
  - The 2025 revision also had *outlines of negatives*: "There are no long-form profiles. No editorial insights. … Just TikTok recaps", and "No redirect. No merge. Just delete." — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history)
- *Rule of three*: "adjective, adjective, adjective" or "short phrase, short phrase, and short phrase", often used to make a superficial analysis look comprehensive. It is a stronger sign where nobody would bother with the flourish, such as an edit summary. Example: "mythological image, magical weapons, and spirit of resistance… contemporary values, modern aesthetics, and digital animation technology". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), citing Russell et al., Kriss/NYT, and the Economist
- *Leads that treat list titles as proper nouns*: "The '[Title]' is a curated compilation of…". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *False ranges* (in the 2025 revision, absent from the current one): "from X to Y" where no meaningful scale joins the endpoints. The test is whether a middle can be named without changing scale. Example: "From problem-solving and tool-making to scientific discovery, artistic expression, and technological innovation…". — [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history)

**Style**
- *Title case* in section headings: "AI chatbots strongly tend to capitalize all main words". Example: "Impact of Technology and Digitalization". — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style), citing Russell et al.
- *Overused boldface*, emphasizing "every instance of a chosen word or phrase, often in a 'key takeaways' fashion". The page traces it to READMEs, fan wikis, how-tos, sales pitches, slide decks, and listicles. — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style)
- *Inline-header vertical lists*: a list marker, then a bold inline header, a colon, and the text. Example: "**Heavy-Duty Rotary Saws**: Designed for tougher materials such as tiles, metals, and plastics." — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style)
- *Heading errors*: a level-1 heading repeating the title, many level-1 headings, skipped levels, and "headings only containing other headings". — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style)
- *Emoji as formatting* at the start of headings or list items. Example: "🧑‍💻 About Me", "💻 **Computing & Open Source Technology**". — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style)
- *Unusual use of tables*: small tables that would be better as prose. — [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style)
- *Curly quotation marks and apostrophes* in place of straight ones. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Communication intended for the user** (chatbot artifacts)
- *Collaborative communication*: "I hope this helps", "Of course!", "Certainly!", "You're absolutely right!", "Would you like…", "let me know", "more detailed breakdown", "here is a…", "[check/verify] … before publishing". One example is a meta-commentary opener: "In this section, we will discuss the background information related to the topic…". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Hedging disclaimers about sources and recommended usage*, in three dated forms:
  - 2023–24, didactic: "it's important to note", "It is crucial to differentiate…", "It's important to remember…".
  - 2024–25: "not widely documented", often with speculation about why.
  - Mid-2026 on: "[claim] should be treated as… rather than…", "does not by itself establish", and "should not be confused" disambiguation, nearly absent before about July 2026. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Placeholders*: unfilled templates and bracketed instructions, dates such as "2026-xx-xx", URLs such as "INSERT_SOURCE_URL" or "PASTE_…_HERE", and example.com subpages. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Subject lines* left in pasted email text, such as "Subject: Request for Review…" (2025 revision). — [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history)

**Markup**
- Markdown in wikitext: `**` for bold, `#` headings, `---` dividers, `[text](url)` links, and, least ambiguous of all, triple-backtick code blocks. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Exposed citation data such as `contentReference[oaicite:1]` ("unambiguous"); nonexistent categories and templates. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Citations**
- Broken links absent from archives, invalid ISBNs and DOIs, DOIs that resolve to unrelated papers, book citations without page numbers, misused reference syntax, footnote marks such as "↩", and named references that are never used. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Tracking parameters `utm_source=openai`, `utm_source=chatgpt.com`, `utm_source=copilot.com`, and `referrer=grok.com`. These identify the chatbot but don't prove it wrote the prose. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Edit summaries and miscellaneous** (mostly specific to Wikipedia)
- Formulaic summaries, canned assurances of policy compliance, "preserved/avoided" statements, overemphasis on sourcing and template parameters, and AfC submission statements. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- A sudden shift in style. Model differences: "ChatGPT and Grok tend toward broader context; Gemini and Claude tend to be more concise." — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Historical indicators** (common in older models, rarer now)
- *Promotional, ad-like language* (2023–2025): "boasts a", "vibrant", "rich", "profound", "enhancing", "showcasing", "exemplifies", "commitment to", "natural beauty", "nestled", "in the heart of", "groundbreaking", "renowned", "featuring", "diverse array". Example: "TTDC acts as the gateway to Tamil Nadu's diverse attractions, seamlessly connecting… It offers dependable, value-driven experiences…". It became less common from about 2024. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), citing Sussman & Carter (2025) and Walker Rettberg (jilltxt)
- *Section summaries*: "In summary", "In conclusion", "Overall", Conclusion sections, and paragraphs that end by restating their point. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Knowledge-cutoff disclaimers* (2022–23), *prompt refusals* ("As an AI language model…"), *abrupt cut-offs*, and *outdated access-dates*. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Em dashes* (2022 to September 2026): used more than in nonprofessional human text, where people would use commas, parentheses, or colons; "formulaic, pat… mimicking 'punched up' sales-like writing"; usually set with spaces. Since about September 2026 the newest models are often hard-coded to avoid them, so sparse, comma-and-period-only punctuation "has become its own sign" (that sentence is tagged *citation needed*). — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- *Elegant variation*: repetition penalties make models cycle synonyms for the same referent. Example: "the constraints of socialist realism" / "Soviet artistic constraints" / "state-imposed artistic norms". Non-native writers taught to avoid repetition (Italian schools are the example given) do the same. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), citing [Belcher/Chronicle](https://www.chronicle.com/article/10-ways-ai-is-ruining-your-students-writing)

**Ineffective indicators** (the page says these don't work)
- Perfect grammar; mixed casual and formal registers (the page says this "may indicate the casual writing of a person in a technical field, such as computer science"); "bland/robotic" prose; "fancy/academic" prose in general; transition words such as "Additionally" or "Notably" on their own; unsourced content; odd wikitext; correct wikitext. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**Signs of human writing**
- Simple "is/has"; plain verbs ("wrote", not "authored"; "used", not "utilized"; "tried", not "attempted"); superlatives ("the only", "was the first"); qualifiers ("very", "perhaps", "tends to"); wordy phrases ("in order to", "the fact that"). — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), citing Reinhart et al., PNAS

### Inferences
- [inference] Wikipedia's catalog is tuned to encyclopedic biography and place articles. Its strongest signals (significance and legacy, notability, awards, challenges and outlook) are about *content* inflation in that genre, and have only weak counterparts in reference docs.
- [inference] Its catalog keeps moving. False ranges and outlines of negatives left the main list between October 2025 and November 2026, and em dashes, elegant variation, and promotional tone became "historical". A checker built on a list of words or punctuation will go stale; the structural patterns (copula avoidance, superficial -ing clauses, vague connection, negative parallelism) have lasted longer.

### Gaps
- The talk page wasn't read, so the disputes over which signs to include are not captured here.
- The Economist, NYT, WaPo, and Chronicle pieces are paywalled. Their findings are reported here only as Wikipedia or secondary summaries give them.

---

## 2. Other catalogs and what they add

### Takeaway
After Wikipedia, the main catalog is tropes.fyi (Ossama Chaib), with derivatives by Alex Reynolds and Louis-François Bouchard and many agent "deslop/humanizer" skills. They add rhetorical and essay tics that Wikipedia doesn't name: staged reveals, self-answered questions, punchy fragments, pedagogical hand-holding, invented concept labels, fractal summaries, and hollow fluency. Journalism's quantitative work (WaPo 2025, the Economist 2026) adds punctuation and rhythm measures, and finds em dashes are now a weak signal. The technical-writing blogs searched for (I'd Rather Be Writing, passo.uno, Write the Docs) turned up no dedicated catalog of tells.

### Cited findings

**tropes.fyi** (gist active 2026-10-23), grouped as the gist groups them:
- *Word choice*: magic adverbs ("quietly", "deeply", "arguably"); "delve" and friends (delve, certainly, utilize, robust, leverage); "tapestry" and "landscape"; the "serves as" dodge. — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1)
- *Sentence structure*:
  - Negative parallelism ("It's not bold. It's backwards."), which the directory calls the single most commonly identified tell.
  - "Not X. Not Y. Just Z." ("Not a bug. Not a feature. A fundamental design flaw.")
  - "The X? A Y." self-answered questions ("The result? Devastating.")
  - Anaphora abuse; tricolon abuse.
  - "It's worth noting" filler transitions (importantly, interestingly, notably).
  - Superficial -ing analyses.
  - False ranges ("From innovation to implementation to cultural transformation"). — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [tropes.fyi directory](https://tropes.fyi/)
- *Paragraph structure*: short punchy fragments ("He published this. Openly. In a book. As a priest."); the "listicle in a trench coat" (numbered points dressed as prose). — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1)
- *Tone*:
  - "Here's the kicker" / "Here's where it gets interesting".
  - "Think of it as…" patronizing analogies ("Think of it like a highway system for data").
  - "Imagine a world where…".
  - False vulnerability ("This is not a rant; it's a diagnosis").
  - "The truth is simple".
  - Grandiose stakes inflation ("will define the next era of computing").
  - "Let's break this down" / "Let's unpack…".
  - Vague attributions ("Industry reports suggest…").
  - Invented concept labels ("the supervision paradox", "workload creep"). — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1)
- *Formatting*: em-dash addiction; bold-first bullets ("**Security**: Environment-based configuration with…"); Unicode decoration ("Input → Processing → Output", curly quotes). — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1)
- *Composition*:
  - Fractal summaries ("And so we return to where we began").
  - The dead metaphor.
  - Historical analogy stacking ("Apple didn't build Uber…").
  - One-point dilution; content duplication.
  - The signposted conclusion ("In conclusion…").
  - "Despite its challenges…".
  - From the comments, not the main list: false exclusivity ("What most people don't know"), clichéd idioms ("move the needle", "game changer"), numbered phase labels ("Phase 1:"), gravitas words ("fundamental", "crucial"), the compliment sandwich, and fake casual quotes. — [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1)

**Alex Reynolds** (October 2026; adapted from tropes.fyi). What it adds:
- *Hollow fluency*: "grammatical sentences that teach the reader nothing", with a test: after each paragraph, can the reader say what they learned? — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)
- Hedged symmetry (every point balanced by a counterpoint so no stance is taken); the escalation cascade; the unearned gravitas tag (", and that matters"); two-beat verdict headlines; ranked praise; honesty theater ("to be completely honest"). — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)
- Formatting: horizontal rules in prose, predictable sentence rhythm, Title Case headings. — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)
- "Claude-specific tells":
  - Sycophantic openers; validate-then-decorate.
  - Reflexive hedging ("stacked qualifiers on every claim").
  - False-depth phrases ("At its core", "what really matters").
  - Assistant artifacts ("I hope this helps"); stock offer closers ("don't hesitate to reach out"); false-reassurance fillers.
  - Engineering words as prose metaphors ("gated on"); industry shorthand.
  - Wordy filler ("in order to"). — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)
- Drama words (game-changer, revolutionary, unlock, cutting-edge) and banned vocabulary (delve, utilize, leverage as a verb, robust, tapestry, synergy, paradigm, multifaceted). — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)

**Louis-François Bouchard** (October 2026):
- Fourteen families: contrast formulas, teases and staged reveals, emphasis commands ("Let that sink in"), self-answered questions, signposting ("Let's break it down"), stock openers and closers ("In today's fast-paced world", "In conclusion"), staging and filler ("It's worth noting"), rhythm tics (lists of three, runs of fragments, a punchline ending every paragraph), decorative vocabulary, frontier-model tics ("genuinely", "quietly", "the gap"), consultant speak ("under the hood", "table stakes"), empty voice ("This is impressive."), punctuation and formatting (em dashes, scattered bold, emoji bullets, "headings on every paragraph"), and chatbot leftovers. — [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md)

**Quantitative journalism**
- WaPo analyzed 37,929 ChatGPT conversations (May 2024 to July 2025). Fewer than 1 in 10 responses used an em dash a year earlier; by summer 2025 more than half did. — [Boston Globe syndication of WaPo, 2025-11-13](https://www.bostonglobe.com/2025/11/13/business/chatgpt-writing-style-clues/) (search snippet; the article itself wasn't read)
- The Economist (2026-07-30) compared 55,940 sentences (1.2M words) from ChatGPT, Claude, Gemini, and Grok with NYT, WaPo, and novels from 1950–2022. Its findings:
  - Em dashes are "no longer reliable markers of AI text". Only Claude uses them more than human writers, and ChatGPT uses them least.
  - Models use *fewer* commas, semicolons, and parentheses than people do.
  - Models write longer sentences and overuse "and".
  - Models favor rarer, scientific, polysyllabic words and nominalizations.
  - Models overuse "it's not X, it's Y" and the rule of three.
  - Models write blocky paragraphs with uniform sentence length. — [TheOutpost summary of the Economist](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/); the em-dash finding is also cited by [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- OpenAI's GPT-5.1 was tuned to stop overusing em dashes (Ars Technica, 2025-11-14, via Wikipedia). — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

**On technical docs specifically**
- A GitBook guide says AI drafts can look polished while containing generic filler, unsupported claims, missing product details, inconsistent terminology, and technical errors. — [GitBook blog](https://www.gitbook.com/blog/fix-ai-generated-documentation) (search-result summary only; the page wasn't read)
- Agent "deslop" skills: one says that for technical documentation, "plain and specific is a complete voice"; another flags "leverage", "landscape", and "ecosystem" leaking into technical prose; another warns that the checks also fire on second-language writers and "technical genres that compress vocabulary by design". — [skills.sh deslop (stephenturner)](https://skills.sh/stephenturner/skill-deslop/deslop) and others in the same search (search-result summaries; not read individually)

### Inferences
- [inference] The tropes.fyi family is aimed at essays, newsletters, LinkedIn posts, and marketing. Most of what it adds (staged reveals, punchy fragments, stakes inflation, analogy stacking) is rhetoric for persuasion, which rarely appears in reference docs but does appear in developer-product landing pages, launch posts, and READMEs.
- [inference] The catalogs disagree on "in order to": Wikipedia lists it as a sign of *human* writing, and Reynolds lists it as a Claude tell. Hedging splits the same way: plain qualifiers ("perhaps", "tends to") read as human per Wikipedia and Reinhart et al., while templated disclaimers ("it's important to note", "does not by itself establish") and stacked qualifiers read as AI.
- [inference] The punctuation findings conflict. Wikipedia says the newest models use more periods and commas (one review cited, plus a *citation needed*); the Economist says models use fewer commas than people. Punctuation counts aren't a usable signal either way.

### Gaps
- I found no post from I'd Rather Be Writing (Tom Johnson), passo.uno (Fabrizio Ferri Benedetti), or Write the Docs that catalogs AI tells. Searches turned up those sites' general posts on AI and docs only.
- No journalism or publishing style memo on AI text (an AP or newsroom memo, for instance) turned up in these searches.
- Max Read's fiction piece wasn't read. The fiction-specific tells (stock names such as "Elara Voss", "whispering woods") are known here only from Wikipedia's footnote.

---

## 3. The consolidated pattern list, sorted by the kind of writing it applies to

Each entry gives a definition, examples, the catalogs that list it, and a judgment. **Applies to** is always [inference] unless a source is cited for it. "Docs" means technical documentation and developer-product copy (READMEs, landing pages, CLI help); "essays/marketing" covers blogs, newsletters, social posts, and press copy.

### Takeaway
Most cataloged patterns appear in developer-product copy. A smaller set does specific damage in reference and task docs: copula avoidance, superficial -ing clauses, vague connection, puffery adjectives that stand in for specs, signposting and meta-commentary, didactic disclaimers, elegant variation, bold-first bullets and heading inflation, summary closers, placeholders, and curly quotes or Unicode in places a reader copies. The rhetoric (staged reveals, fragments, aphorisms, "Here's the thing") is mainly an essay and marketing problem, and the fiction tells don't apply to docs at all.

### Cited findings and judgments

#### A. Apply to technical docs and developer-product copy

1. **AI vocabulary** (delve, tapestry, testament, crucial, pivotal, seamless, robust, leverage, landscape, realm, ensure, enhance, foster, showcase, streamline, multifaceted, utilize).
   - *Definition*: words models overuse relative to human text.
   - *Examples*: "a robust, seamless integration"; "leverage the API to enhance your workflow".
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (delve, tapestry, testament, crucial, pivotal, robust, landscape, leverage, enhance, foster, showcasing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) (delve, utilize, robust, leverage, tapestry, landscape); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) (adds synergy, paradigm, multifaceted); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) (delve, tapestry, robust); [Economist summary](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/) (polysyllabic words and nominalizations).
   - *Not in any catalog read*: "seamless", "realm", and "ensure" as standalone words. "seamlessly" appears in a Wikipedia promotional example, and "ensuring…" is listed only as a trailing -ing clause.
   - *Applies to*: docs and product copy [inference]. "robust", "seamless", and "leverage" are endemic to developer marketing. In reference docs, "ensure" is often the precise verb ("ensure the port is free"). Bouchard's rule fits: "Keep a listed word only if it is the precise term" (his example is "critical section").
   - *Harm in docs* [inference]: an evaluative adjective ("robust", "powerful") takes the place of the spec that would tell the reader something (what failures it survives, what it supports).

2. **Copula avoidance** ("serves as", "functions as", "acts as", "stands as", "represents", "offers", "features", "boasts", "provides" instead of "is" or "has").
   - *Example*: "The `config` object serves as the central hub for…".
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
   - *Applies to*: docs, strongly [inference].
   - *Harm*: a definition stops being a definition; "X serves as Y" leaves the reader unsure whether X *is* Y.

3. **Superficial "-ing" analysis** (", ensuring…", ", enabling…", ", highlighting the importance of…", ", making it easy to…").
   - *Examples*: "Caching is applied per request, ensuring optimal performance"; "…underscoring its role as a dynamic hub".
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
   - *Applies to*: all genres; docs especially [inference].
   - *Harm*: the clause claims a benefit or consequence without saying how or under what conditions.

4. **Vague connection** ("associated with", "in connection with", "related to", "tied to").
   - *Example*: "the tokens associated with your account" (owned by? created by? scoped to?).
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing). Wikipedia names it one of the most common current signs.
   - *Applies to*: docs, strongly [inference].
   - *Harm*: the exact relationship between two things is the fact a reference reader came for.

5. **Promotional or puffed-up tone** (powerful, seamless, effortless, cutting-edge, game-changer, revolutionary, unlock, groundbreaking, renowned, vibrant, "boasts a", "diverse array").
   - *Example*: "unlock a seamless developer experience".
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (historical, 2023–2025: "no longer common in newer LLM output unless a user specifically asks"); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) (drama inflation, puffery); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) (stakes inflation, clichéd idioms).
   - *Applies to*: developer-product copy and marketing; less so reference docs [inference].
   - *Harm*: credibility. A technical reader discounts the claims around it.

6. **Undue significance** ("plays a crucial/pivotal role", "is a testament to", "marks a shift").
   - *Example*: "Middleware plays a pivotal role in the request lifecycle."
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) (grandiose stakes, gravitas words).
   - *Applies to*: essays and encyclopedic writing first; concept pages in docs second [inference].
   - *Harm in docs*: it says something is important instead of saying what it does.

7. **Signposting and meta-commentary** ("In this section, we'll discuss…", "Let's dive in", "Let's break this down", "Here's what you need to know", "In today's fast-paced world").
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (the example "In this section, we will discuss the background information…" under collaborative communication); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) ("Let's break this down"); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) (structural crutches); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) (signposting, stock openers).
   - *Applies to*: docs (tutorials and guides especially) and essays [inference].
   - *Harm*: it delays the content and costs the scanning reader a line. An Artem Andreev post on agent-written docs reportedly says the vague openers can be deleted without losing content (search snippet; LinkedIn source not read).

8. **"It's important to note" / didactic disclaimers / "worth noting"**.
   - *Examples*: "It's important to note that results may vary"; "should be treated as… rather than…".
   - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (hedging disclaimers, three dated forms); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md).
   - *Applies to*: docs, strongly [inference].
   - *Harm*: an unspecific caveat reads like a warning but says neither what goes wrong nor when.

9. **Hedging** (stacked qualifiers on every claim, "may", "typically", "in most cases", hedged symmetry).
   - *Sources*: [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) (reflexive hedging, hedged symmetry); [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (the disclaimer form). Contrary evidence: Wikipedia lists plain qualifiers ("perhaps", "tends to") as signs of *human* writing.
   - *Applies to*: docs and essays [inference].
   - *Harm in docs*: "may" over deterministic behavior hides whether the behavior is guaranteed. Hedging is a sign only when it's templated or stacked.

10. **Elegant variation** (cycling synonyms for one referent).
    - *Example*: "workspace… project… environment…" for the same thing.
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (historical; caveat about non-native writers); [Belcher/Chronicle](https://www.chronicle.com/article/10-ways-ai-is-ruining-your-students-writing) (via Wikipedia); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) lists the opposite failure, the repeated "dead metaphor".
    - *Applies to*: docs, strongly [inference].
    - *Harm*: in technical writing a new term implies a new thing. GitBook's "inconsistent terminology" complaint is the same failure ([GitBook](https://www.gitbook.com/blog/fix-ai-generated-documentation), snippet only).

11. **Bold inline headers in lists** (bold-first bullets: "**Security**: …").
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: docs, with a split verdict. Reynolds says "Code comments and technical documentation get a pass where some patterns" are conventional, and names bold-first bullets in API docs. Wikipedia traces the habit to READMEs and how-tos.
    - *Harm* [inference]: legitimate for a definition list of parameters or options; harmful when it turns an argument or a sequence into disconnected labeled fragments.

12. **Overused boldface in prose**.
    - *Example*: bolding every key term in a paragraph.
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style) ("key takeaways" fashion); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) (scattered bold).
    - *Applies to*: docs and READMEs [inference].
    - *Harm*: when everything is emphasized, nothing is. One developer's self-audit, from a search snippet only, says "when every emphasised word isn't, none is".

13. **Excessive bullets, headings, and tables** (a heading on every paragraph, headings holding only headings, tiny tables, "Phase 1:" labels).
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style) (heading errors, unusual tables); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) ("headings on every paragraph"); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) (numbered phase labels, from a commenter).
    - *Applies to*: docs [inference].
    - *Harm*: it breaks reasoning that needs connecting words ("because", "unless") into fragments, and inflates the table of contents a scanner relies on.

14. **Title case in headings**.
    - *Example*: "Impact of Technology and Digitalization".
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style), citing [Russell et al.](https://aclanthology.org/2025.acl-long.267/); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: docs [inference]. Many developer style guides (Google, Microsoft) call for sentence case, but that wasn't verified here.
    - *Harm*: low. Mostly a style-consistency sign.

15. **Summary closers and fractal summaries** ("In conclusion", "In summary", "Overall", recap sections, "And so we return to…").
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (historical; "Section summaries"); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) (fractal summaries, signposted conclusion); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md).
    - *Applies to*: docs and essays [inference].
    - *Harm in docs*: it repeats what the reader just read where a next step or a link belongs.

16. **Pedagogical voice and "Think of it as…" analogies**.
    - *Examples*: "Think of it like a highway system for data"; "Let's unpack what this really means".
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) ("Pedagogical voice: teacher-student tone aimed at expert readers").
    - *Applies to*: tutorials and conceptual docs [inference].
    - *Harm*: it condescends to experts, and an analogy can stand in for the precise mechanism.

17. **Hollow fluency**: grammatical sentences that teach nothing.
    - *Sources*: [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) ("empty voice"). Wikipedia's root-cause note, generic in place of specific, describes the same thing.
    - *Applies to*: docs, strongly [inference]. This is the overall failure that most of the other patterns are instances of.

18. **Vague attribution** ("experts say", "studies show", "teams report", "developers love", "industry reports suggest").
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: product copy and essays; rare in reference docs [inference].

19. **Emoji** at the start of headings or bullets.
    - *Example*: "🚀 Getting Started".
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) (emoji bullets). One deslop skill (search snippet) says not to add emoji to make technical prose feel friendly.
    - *Applies to*: READMEs and developer-product copy, especially [inference].

20. **Curly quotes, Unicode arrows, and formatting artifacts**.
    - *Examples*: “” and ’ in place of " and '; "Input → Processing → Output".
    - *Sources*: [WP-Style](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing/Style); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: docs, with a functional harm [inference]. Curly quotes or Unicode dashes inside commands, flags, or code break copy-paste. In prose they are only a weak sign, since many publishing tools curl quotes automatically.

21. **Chatbot artifacts and placeholders** ("Certainly!", "I hope this helps", "Let me know if…", "Here is a…", bracketed fill-ins, "INSERT_URL", "2026-xx-xx", `utm_source=chatgpt.com`, `oaicite`, Markdown leaking into another markup).
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: all genres, docs included [inference].
    - *Caveat*: example.com links are standard practice in technical docs, so that particular Wikipedia sign doesn't transfer.

22. **Negative parallelism** ("not X, but Y", "it's not just X, it's Y", "Y rather than X", "Not X. Not Y. Just Z.").
    - *Example*: "Ascribe isn't just a linter — it's a…".
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history), [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1) ("the single most commonly identified AI writing tell"), [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop), [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md), and [Economist summary](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/).
    - *Applies to*: product copy and essays mainly [inference]. In reference docs a contrast is often needed ("pass the path, not the file's contents"; "use `--yes`, not a pipe"). The sign is a reframe that knocks down a claim nobody made.

23. **Rule of three / stacked triplets**.
    - *Example*: "fast, reliable, and scalable".
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1), [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop), [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md), [Economist summary](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/), and [Russell et al.](https://aclanthology.org/2025.acl-long.267/) (via Wikipedia).
    - *Applies to*: all genres; product copy most [inference].
    - *Harm in docs*: padding a list to three implies there are exactly three, which misleads when the real list is longer or shorter.

24. **False ranges** ("from X to Y" with no scale between).
    - *Examples*: "from prototypes to production to enterprise scale"; "from innovation to implementation to cultural transformation".
    - *Sources*: [WP-2025](https://en.wikipedia.org/w/index.php?title=Wikipedia:Signs_of_AI_writing&action=history) (since dropped from the page); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Applies to*: product copy and essays [inference]. Docs use real ranges ("from 1 to 65535"), which aren't the sign.

25. **Em dashes**.
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (historical, 2022 to September 2026; spaced; formulaic); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) ("A few per piece reads as human"); [WaPo via Globe](https://www.bostonglobe.com/2025/11/13/business/chatgpt-writing-style-clues/) (from under 10% to over 50% of ChatGPT responses during 2025). Contrary: [Economist summary](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/) says "no longer reliable", except that Claude still exceeds human rates.
    - *Applies to*: all genres; a weak signal now [inference]. Relevant to a Claude-written docs project because Claude is the one model that still over-uses them, according to the Economist summary.

26. **Uniform rhythm and blocky paragraphs** (same-length sentences, sparse punctuation, "and" chains).
    - *Sources*: [Economist summary](https://theoutpost.ai/news-story/ai-writing-s-biggest-giveaway-has-changed-the-economist-report-reveals-new-detection-patterns-29372/); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) (8–20-word sentences in mid-2026 models).
    - *Applies to*: all genres; hard to act on in docs [inference].

#### B. Mainly essays, newsletters, social posts, and marketing

27. **Punchy one-line closers, fragments, and aphorisms** ("Platforms do."; "He published this. Openly."; ", and that matters"; "Let that sink in").
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop) (unearned gravitas tag, escalation cascade); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md) (punchline endings, emphasis commands).
    - *Docs relevance* [inference]: launch posts, landing pages, and the closing lines of a README pitch.

28. **"Here's the thing" / "Here's the kicker" / staged reveals** ("Here's where it gets interesting", "What most people don't know").
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md). Not found in Wikipedia.

29. **Rhetorical / self-answered questions** ("The result? Devastating.", "Why does this matter?").
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md).
    - *Docs* [inference]: FAQ headings and troubleshooting questions are legitimate. The tell is the question answered in the next fragment.

30. **"Imagine a world where…", stakes inflation, historical analogy stacking, invented concept labels, false vulnerability, honesty theater, "The truth is simple", clichés and consultant speak** ("move the needle", "table stakes", "under the hood").
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md).
    - *Docs* [inference]: "under the hood" appears in docs, and invented labels show up in concept pages. Otherwise this is essay and marketing rhetoric.

31. **Anaphora abuse, the listicle in a trench coat, one-point dilution, content duplication, the dead metaphor**.
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop).
    - *Docs* [inference]: duplicated content across pages is a docs problem in its own right.

32. **Magic adverbs and false-depth phrases** ("quietly", "deeply", "genuinely", "fundamentally", "At its core", "what really matters").
    - *Sources*: [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1); [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop); [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md); [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) ("quietly" among mid-2026 words).
    - *Docs* [inference]: "At its core, X is…" is a common opener on AI-written concept pages.

33. **"Despite its challenges" and Challenges / Future outlook sections**.
    - *Sources*: [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing); [tropes.fyi](https://gist.github.com/ossa-ma/f3baa9d25154c33095e22272c631f5a1).
    - *Applies to*: reports, essays, and encyclopedic writing [inference].

#### C. Specific to Wikipedia or encyclopedic writing (little bearing on docs)
- Canned notability claims, "identified by databases", Awards and recognition sections, leads that treat a list title as a proper noun, edit-summary formulas, AfC submission statements, invalid ISBNs and DOIs, and named references left unused. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

#### D. Mainly fiction
- Stock character names ("Elara Voss") and stock settings ("whispering woods"). — [WP-now footnote](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing), [Max Read](https://maxread.substack.com/p/who-is-elara-voss)
- [inference] The dead metaphor and sensory clichés overlap with fiction too. No fiction-specific catalog was read in depth.

### Inferences
- [inference] Developer-product copy (landing pages, README intros, launch posts) is where the essay and marketing patterns (B) and the docs patterns (A) meet. Puffery, false ranges, negative parallelism, triplets, and emoji headers cluster there, while reference pages mostly show patterns 2–4, 7–13, and 20–21.
- [inference] Several cataloged tells have a legitimate technical use that a checker must not flag: "ensure" as a precise verb, a needed "X, not Y" contrast, numeric "from X to Y" ranges, bold-first definition lists for parameters, FAQ questions, example.com, and "critical"/"robust" as terms of art (Bouchard's "keep a listed word only if it is the precise term").

### Gaps
- No catalog gives frequencies by genre, so where each pattern appears is judgment, not measurement.
- Developer style guides (Google, Microsoft) on title case, "please", and "simply/easy" weren't fetched, though they would support several judgments here.

---

## 4. Which patterns are called harmful in docs, as opposed to merely sounding like AI

### Takeaway
No catalog read takes up docs specifically. Wikipedia is the only catalog that explains *why* the signs matter: they "can point to less outwardly visible problems", generic and positive claims in place of specific facts, and vague attributions in place of sources. Reynolds and Bouchard state functional tests: what a paragraph teaches, and whether a listed word is the precise term. The case that particular patterns hide meaning, slow reading, or break scanning in docs is mostly inference, supported in part by a few sources.

### Cited findings
- Wikipedia: the signs "are also only potential signs of a problem, not the problem itself… easy to fix… they can point to less outwardly visible problems". LLMs "regress to the mean", omitting "specific, unusual, nuanced" facts in favor of generic, important-sounding statements. — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Wikipedia on the rule of three: used "to make superficial analyses appear more comprehensive". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Wikipedia on vague connection: models link subjects "instead of stating the relationship directly". — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
- Reynolds' hollow-fluency test ("after each paragraph, can the reader say what they learned?"), and the exemption: "Code comments and technical documentation get a pass where some patterns" are conventional, such as bold-first bullets in API docs. — [Reynolds](https://www.alexreynolds.com/tools/guides/anti-slop)
- Bouchard: "Keep a listed word only if it is the precise term" ("critical section"); and for reports and outlines, "Dates, names, numbers, links, and uncertainties preserved". — [Bouchard](https://github.com/louisfb01/ai-engineering-cheatsheets/blob/main/Anti_Slop_AI_Writing_Guide.md)
- GitBook's complaints about AI drafts of docs: generic filler, unsupported claims, missing product details, inconsistent terminology, technical errors under a polished surface. — [GitBook](https://www.gitbook.com/blog/fix-ai-generated-documentation) (search-result summary; not read)
- The deslop skill on technical docs: "plain and specific is a complete voice", with a warning against overediting specific prose into generic prose. — [skills.sh deslop](https://skills.sh/stephenturner/skill-deslop/deslop) (search-result summary)
- Wikipedia's ineffective-indicators list warns that mixed registers can signal "a person in a technical field". Technical writing's terse, compressed vocabulary trips some checkers by design (deslop skill, search summary). — [WP-now](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)

### Inferences
- [inference] **They hide meaning**: copula avoidance (2), superficial -ing clauses (3), vague connection (4), evaluative adjectives standing in for specs (1, 5), unspecific disclaimers (8), templated hedging over deterministic behavior (9), elegant variation (10, a new word reads as a new thing), and padded triplets (23, an implied count). These change what a reader believes about how the software behaves, so they are correctness problems, not style problems.
- [inference] **They slow the reader**: signposting and meta-commentary (7), summary closers and fractal summaries (15), "It's important to note" preambles (8), pedagogical lead-ins (16), and hollow fluency (17).
- [inference] **They break scanning**: bold scattered through prose (12, which flattens the emphasis a scanner relies on); bold-first bullets used for reasoning rather than definitions (11); headings on every paragraph or headings holding only headings (13, which inflate the table of contents); emoji in headings (19, which hurt alphabetic scanning and screen-reader output); and title case mixed with sentence case (14, low harm).
- [inference] **They break things**: curly quotes and Unicode dashes in commands or code (20), leftover placeholders and Markdown (21), and chatbot tracking parameters in links (21).
- [inference] **Mostly a matter of voice in docs**: em dashes (25), negative parallelism (22), rhetorical questions, "Here's the thing", and aphorisms (27–29). Readers dislike them because they sound machine-made, and in product copy they cost credibility, but they seldom change what a reference page says.

### Gaps
- No usability or eye-tracking evidence was found that ties any of these patterns to slower reading or failed scanning in docs. Every "breaks scanning" claim above is judgment.
- No Write the Docs talk, I'd Rather Be Writing post, or passo.uno post on these patterns' effect on docs readers turned up. A targeted search of those sites' archives (passo.uno/sitemap.md, the writethedocs.org videos) might still find one.
