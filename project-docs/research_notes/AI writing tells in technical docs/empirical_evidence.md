# Empirical evidence behind claimed AI-writing tells

Scope: which "AI tells" are measured, by how much, and how well the evidence carries over to technical documentation. Studies are 2023–2026 preprints or peer-reviewed papers unless marked. Numbers were read from the primary paper (arXiv HTML or abstract) unless a line says otherwise.

## 1. Measured LLM-favored vocabulary in human-published text

### Takeaway
Word-level excess vocabulary has the strongest measurement behind it. Across 15 million PubMed abstracts, arXiv abstracts, and peer reviews, a small set of mostly style words (verbs and adjectives such as *delves*, *underscores*, *showcasing*, *meticulous*, *commendable*, *intricate*, *crucial*, *notably*) rose 10 to 35 times above baseline after ChatGPT's release. The measurements are corpus-level, though. They estimate shares of a population and are explicitly not detectors for a single text.

### Cited Findings
- **Kobak et al. (Science Advances 2025; arXiv 2406.07016):** 15,103,888 PubMed abstracts from 2010 to 2024. Each word's expected 2024 frequency is extrapolated linearly from 2021–2022, then the excess gap δ = p − q and the ratio r = p/q are measured. No labeled AI text is used. — [arXiv 2406.07016](https://arxiv.org/html/2406.07016)
  - Ratios: *delves* r = 28.0, *underscores* r = 13.8, *showcasing* r = 10.7. Gaps (common words): *potential* δ = 0.052, *findings* δ = 0.041, *crucial* δ = 0.037. — [same](https://arxiv.org/html/2406.07016)
  - At least **13.5%** of 2024 abstracts were LLM-processed. Two independent word sets agree: 291 rare style words give 0.136, and 10 common words (across, additionally, comprehensive, crucial, enhancing, exhibited, insights, notably, particularly, within) give 0.134. — [same](https://arxiv.org/html/2406.07016)
  - Excess words: 454 in 2024, against 190 in 2021, the COVID peak. COVID-era excess words were almost all content nouns (covid, pandemic). The 2024 ones were mostly style words. Of the 379 excess style words in 2024, about 66% were verbs and 14% adjectives. — [same](https://arxiv.org/html/2406.07016)
  - Subgroup lower bounds ranged from under 5% to over 40%: UK/Australia ≈ 0.05; China, South Korea, Taiwan ≈ 0.20; computation and bioinformatics ≈ 0.20; MDPI 0.21, Sensors 0.25; Nature/Science/Cell 0.07; Chinese computation papers 0.41. — [same](https://arxiv.org/html/2406.07016)
  - Caveats: the estimate is a lower bound, since abstracts with no marker word go uncounted. It cannot separate LLM use from humans adopting LLM words. Subgroup gaps may partly reflect native speakers editing out the marker words. It cannot tell one LLM from another. — [same](https://arxiv.org/html/2406.07016)
- **Liang et al., "Monitoring AI-Modified Content at Scale" (ICML 2024; arXiv 2403.07183):** a maximum-likelihood mixture model fitted on human and AI reference texts, applied to peer reviews. Estimated LLM-modified sentence share: ICLR 2024 10.6% (from 1.6% before ChatGPT), NeurIPS 2023 9.1% (from 1.9%), CoRL 2023 6.5% (from 2.4%), EMNLP 2023 ≈ 16.9%. Nature-portfolio reviews showed no significant increase. — [arXiv 2403.07183](https://arxiv.org/html/2403.07183)
  - In ICLR 2024 reviews, the per-sentence probability of *commendable*, *meticulous*, and *intricate* rose **9.8×, 34.7× and 11.2×**. — [same](https://arxiv.org/html/2403.07183)
  - Correlates: the estimated share was higher in reviews submitted near the deadline, in low-confidence reviews, in reviews without "et al." (fewer citations), and among reviewers less likely to answer rebuttals. Reviews closest to the centroid of a paper's reviews scored higher, a sign of homogenization. — [arXiv 2403.07183 abstract](https://arxiv.org/abs/2403.07183); [full text](https://arxiv.org/html/2403.07183)
- **Liang et al., society-wide adoption (arXiv 2502.09747):** the same population-level framework, January 2022 to September 2024. About 18% of financial consumer-complaint text, up to 24% of corporate press-release text, just under 10% of job postings at small firms, and about 14% of UN press releases were LLM-assisted. Growth surged after November 2022 and leveled off in 2024. The authors say the plateau may be saturation, or models getting harder to detect. — [arXiv 2502.09747](https://arxiv.org/abs/2502.09747)
- **Reinhart et al. (PNAS 2025; arXiv 2410.16107):** parallel corpora from shared prompts compare human text with GPT-4o, GPT-4o Mini, and Llama 3 variants. Rates relative to humans: *camaraderie* 162× (GPT-4o), *tapestry* 155×, *intricate* 119×, *palpable* 145× (4o Mini). *tapestry* appears in 23% of GPT-4o outputs, *amidst* in 27%. — [arXiv 2410.16107](https://arxiv.org/html/2410.16107)
- **Juzek & Ward (COLING 2025):** 21 focal words (including *delve*, *intricate*, *underscore*) were flagged from the rise in scientific abstracts. The authors found no evidence that architecture, algorithm choices, or training data cause the overuse. Model tests are "consistent with RLHF playing a role," which is suggestive rather than conclusive. — [arXiv 2412.11385](https://arxiv.org/abs/2412.11385)
- **Russell, Karpinska & Iyyer (ACL 2025; arXiv 2501.15654):** people who often use LLMs for writing judged 300 non-fiction articles from GPT-4o, Claude, and o1. "AI vocabulary" was the most-cited clue, in 53.1% of explanations. Words named: *vibrant*, *crucial*, *significantly*, *testament*, *delve*, plus *notes*/*explains* where a human would write *says*. — [arXiv 2501.15654v2](https://arxiv.org/html/2501.15654v2)

### Inferences
- Vocabulary is the best-grounded tell, with ratios in the tens and replication across PubMed, arXiv, peer reviews, and press releases. The evidence describes populations, though. Since Kobak's whole method rests on excess *over a human baseline*, any single word on these lists is also a normal human word.
- The excess words are mostly evaluative or "connective-showy" verbs and adjectives (*underscores*, *showcasing*, *meticulous*, *commendable*, *crucial*, *notably*, *comprehensive*, *enhancing*). These rarely carry technical content, so a style rule can target the class (evaluative filler) rather than a frozen word list.
- Instruction tuning appears to cause the bias (Reinhart: base models' word choice is closest to human; Juzek & Ward: consistent with RLHF). Each new model therefore brings its own preferred words, and a static list goes stale.

### Gaps
- No per-model vocabulary list for current models (Claude 4.x, GPT-5.x, Gemini 2.5+) in technical prose turned up. Reinhart covers only GPT-4o and Llama 3.
- Liang et al. 2024 (arXiv 2404.01268, CS papers up to ~17.5%) and the Nature news coverage were not opened, so their figures are unverified here.
- The full taxonomy of Russell et al.'s Table 12 (expert-cited AI words) was not read.

## 2. Measured structural and stylistic features

### Takeaway
Several structural tells are measured, at very different strengths:
- **Strong (controlled, large effects, replicated):** present-participial clauses (*", highlighting…"*, *", ensuring…"*), at 2–5× the human rate in instruction-tuned models. Nominalizations, at 1.5–2×.
- **Real but model-dependent:** em dashes, which range from 0 to about 10 per 1,000 words depending on the model and whether it is told to avoid markdown.
- **Moderately supported, older models or small studies:** lower variance in sentence and paragraph length. Groups of three.
- **Contradictory or thin:** lexical diversity, which studies find going both ways. Hedging, where ChatGPT shows *fewer* epistemic markers, contrary to folklore.
- **No direct measurement found:** negative parallelism ("not X but Y").

### Cited Findings
**Grammar (Biber features)**
- **Reinhart et al. (PNAS 2025):** GPT-4o against human text written from the same prompts:
  - present-participial clauses 5.3× (paired Cohen's d = 1.38), with instruction-tuned models overall at 2–5×;
  - nominalizations 2.1× (d = 1.23), with instruction-tuned models overall at 1.5–2×;
  - "that"-clauses as subject 2.6× (d = 0.77);
  - phrasal coordination 1.9× (d = 0.81);
  - agentless passives about half the human rate;
  - downtoners: GPT-4o uses more, Llama 3 fewer;
  - clausal coordination: GPT-4o avoids it, Llama uses more.

  Base models sit closer to human rates than instruction-tuned ones, and larger models are not more human-like. A random forest picks the source among human and six LLMs with 66% accuracy (chance 14%). Single human-vs-one-LLM classifiers reach 93–98%. — [arXiv 2410.16107](https://arxiv.org/html/2410.16107)
- **Herbold et al. (Scientific Reports 2023; arXiv 2304.14276):** argumentative essays rated by teachers. ChatGPT essays scored higher on quality, and showed more nominalizations, greater lexical diversity, and *fewer* discourse and epistemic markers. — [arXiv 2304.14276](https://arxiv.org/abs/2304.14276)

**Punctuation (em dashes)**
- **Freeburg, "The Last Fingerprint" (arXiv 2603.27006, 2026, single-author preprint):** 12 instruction-tuned models, about 240k words of 1,000-word essays on 10 topics. Human baseline: 8 published essays, 57,232 words, 3.23 em dashes per 1,000 words (median 3.83, range 0.33–17.12). — [arXiv 2603.27006](https://arxiv.org/html/2603.27006v1)

  | Model | Unconstrained | Told "no markdown" |
  |---|---|---|
  | GPT-4.1 | 10.62 | 9.10 |
  | Claude Opus 4.6 | 9.09 | 0.19 |
  | Claude Sonnet 4 | 8.29 | 1.31 |
  | Claude Haiku 3.5 | 7.51 | 0.18 |
  | DeepSeek V3 | 6.95 | 5.41 |
  | GPT-4o | 4.12 | 2.68 |
  | Gemini 2.5 Pro | 3.53 | 0.00 |
  | GPT-5.4 | 1.43 | 0.29 |
  | Gemini 2.5 Flash | 1.28 | 1.48 |
  | Llama 3.1 8B and 3.3 70B | 0.00 | 0.00 |

  - In 5,000-word essays: GPT-4.1 at 14.03 unconstrained and 6.97 even when em dashes are explicitly prohibited. Claude Opus 4.6 at 12.91 unconstrained and 0 under either suppression. — [same](https://arxiv.org/html/2603.27006v1)
  - Hypothesis: the em dash is "markdown leaking into prose." Training data installs the tendency, and post-training amplifies or suppresses it. The paper cites Sam Altman's public statement that ChatGPT's em-dash frequency was tuned to user preference. — [same](https://arxiv.org/html/2603.27006v1)
  - Caveats the paper's own tables raise: unconstrained figures disagree between Table 1 and Table 2 for the same models (for example, GPT-5.4 at 1.43 and 0.75) without explanation. Human rates vary about 50-fold. The genealogy is hard to falsify. — [same](https://arxiv.org/html/2603.27006v1)
- **Czuma, medRxiv em-dash study (arXiv 2606.29540, 2026 preprint, pre-registered):** 69,632 medRxiv preprints, 2020–2025. The share of Discussion sections with at least one em dash rose from 4.23% to 11.58% (+7.35 pp, 95% CI 6.94–7.77; OR 2.96). By year: about 4% through 2023, 8.0% in 2024, 20.3% in 2025. A placebo split inside the pre-LLM era showed +0.13 pp (no change), and boilerplate sections showed no effect. The author calls it "a population-level indicator, not a per-paper detector," and the design does not establish causality. — [arXiv 2606.29540](https://arxiv.org/abs/2606.29540)
- **Russell et al. 2025:** expert annotators said AI text is grammatically clean and *avoids* dashes and ellipses, while human text mixes in dashes, brackets, and quotes. This holds for their GPT-4o/Claude 3.5/o1 set. Clue categories: vocabulary 53.1%, sentence structure 35.9%, grammar and punctuation 24.8%, originality 23.7%, quotes 22.3%, clarity 19.5%. — [arXiv 2501.15654v2](https://arxiv.org/html/2501.15654v2)

**Uniformity ("burstiness") and length**
- **Desaire et al. (Cell Reports Physical Science 2023):** a classifier separated scientists' perspective articles from ChatGPT (GPT-3.5) versions, with 100% accuracy per document and 92% per paragraph. Training data: 64 human perspectives and 128 ChatGPT articles. Distinguishing features:
  - ChatGPT had more uniform sentence lengths and a steadier word count per paragraph;
  - humans wrote longer paragraphs and used more contrastives (*however*, *but*, *although*);
  - ChatGPT used *others* and *researchers* more.

  The authors say it was not designed for other genres. (Figures from press coverage of the paper.) — [ScienceDaily](https://www.sciencedaily.com/releases/2023/06/230607124132.htm); [arXiv 2303.16352](https://arxiv.org/abs/2303.16352v1)
- **Muñoz-Ortiz, Gómez-Rodríguez & Vilares (Artificial Intelligence Review 2024):** New York Times news against six open LLMs (LLaMA family, Falcon, Mistral). Human text has more scattered sentence-length distributions, more varied vocabulary, shorter constituents, and more optimized dependency distances. LLMs use more numbers, symbols, auxiliaries, and pronouns. Human–LLM differences are larger than differences between LLMs. — [arXiv 2308.09067](https://arxiv.org/pdf/2308.09067)

**Templates and triplets**
- **Shaib et al. (EMNLP 2024):** models reuse part-of-speech templates more than human references. One reported example, taken from a third-party summary and not verified in the paper: 95% of model outputs on Rotten Tomatoes contained length-6 templates, against 38% of human references. The authors make no claim about detection. — [arXiv 2407.00211](https://arxiv.org/abs/2407.00211)
- **Groups of three:** Russell et al.'s annotators noted that LLMs group examples in threes ("listings of specifically three ideas"). This is a qualitative annotator observation, not a measured rate. — [arXiv 2501.15654v2](https://arxiv.org/html/2501.15654v2)

**Lexical diversity (conflicting)**
- Herbold found *higher* lexical diversity in ChatGPT essays than in student essays. — [arXiv 2304.14276](https://arxiv.org/abs/2304.14276)
- Muñoz-Ortiz found *more varied* vocabulary in human news. — [arXiv 2308.09067](https://arxiv.org/pdf/2308.09067)
- A search summary found studies going both ways depending on the comparison group. — [T&F stylometric comparison ChatGPT/DeepSeek](https://www.tandfonline.com/doi/full/10.1080/23311983.2025.2553162); [Springer short-story comparison](https://link.springer.com/article/10.1186/s40561-025-00388-z)

### Inferences
| Claimed tell | Evidence strength | Direction and size |
|---|---|---|
| LLM-favored evaluative words (*delve*, *underscore*, *showcase*, *meticulous*, *crucial*) | Strong: millions of documents, several corpora | 10–35× ratios for rare words. Population-level only |
| Present-participial tack-ons (*", ensuring…"*) | Strong: controlled, large d | 2–5× human rate |
| Nominalizations | Strong: two independent studies | 1.5–2× |
| Em dashes | Moderate: real but model-specific | 0 to ~10/1,000 words against human ~3. Recent OpenAI models and Llama are *below* human. Claude and GPT-4.1 are high |
| Uniform sentence and paragraph length | Moderate: older models, small or genre-specific samples | Direction consistent, magnitudes not reported |
| Triplets | Weak: qualitative annotator reports | No rate measured |
| Hedging overuse | Contradicted: ChatGPT showed *fewer* epistemic markers (Herbold) | Folk claim not supported for essays |
| Lexical diversity | Conflicting | Depends on the comparison group |
| Negative parallelism ("not X, but Y") | Not measured in any source found | Folklore so far |
| Bullet and list density | Freeburg counts markdown features only in aggregate | No per-feature rates |

- Participial tack-ons and nominalizations are the strongest *grammatical* tells. Both also show up in style guides as habits that make prose worse, so they are the safest rules to adopt.
- The em dash is a poor universal tell. Human rates span 50×, some current models use fewer em dashes than people do, and one instruction ("no markdown") removes almost all of them in Claude models.

### Gaps
- No primary measurement found for negative parallelism, rhetorical "It's not just X — it's Y", or colon-led lists.
- No per-feature measurement of bullets, headings, or bold in LLM prose against human prose. Freeburg reports only an aggregate count.
- The magnitude of sentence-length variance ("burstiness") is not given in any primary source read. GPTZero's perplexity and burstiness framing is a vendor claim; I found no peer-reviewed effect size.
- Shaib et al. 2025 ("Measuring AI slop", arXiv 2509.19163) builds a taxonomy (repetition, verbosity, density, vagueness, per search summaries). Only the abstract was read: slop judgments "correlate with latent dimensions such as coherence and relevance." Its numbers are unverified.

## 3. Differences between models, change over time, and generalization

### Takeaway
The tells are moving targets. Flagged words fell in arXiv abstracts from April 2024 once they were publicized, while less-noticed ones (*significant*, *additionally*) kept rising. Em-dash rates differ by an order of magnitude between model families and versions and respond to tuning. Almost all of the evidence is English and academic (abstracts, reviews, essays) or news. No measurement on technical documentation was found.

### Cited Findings
- **Geng & Trotta, "Human-LLM Coevolution" (Findings of ACL 2025; arXiv 2502.09606):** 1,294,653 arXiv abstracts, January 2018 to December 2024, measured monthly. Flagged words (*delve*, *showcasing*, *pivotal*, *intricate*, *realm*, *commendable*, *meticulous*…) "began to decrease" from April 2024, with *delve* and *showcasing* peaking in January–March 2024. *significant*, *additionally*, *crucial*, *effectively*, *comprehensive*, *enhance*, *capabilities*, and *valuable* kept rising. The paper gives no percentages, only charts. The authors argue that per-text detection may be impossible and that corpus-level measurement is the practical option. — [arXiv 2502.09606v2](https://arxiv.org/html/2502.09606v2); [ACL Anthology](https://preview.aclanthology.org/setup/2025.findings-acl.657)
- **Model differences in em dashes:** GPT-4.1 10.62, GPT-4o 4.12, and GPT-5.4 1.43 per 1,000 words show a steep decline within OpenAI. Claude models run 7.5–9.1 unconstrained but drop to about 0.2 under a no-markdown instruction. Llama is 0. — [arXiv 2603.27006](https://arxiv.org/html/2603.27006v1)
- **Model differences in grammar:** GPT-4o uses more downtoners and avoids clausal coordination; Llama 3 does the opposite. Seven sources (human plus six LLMs) can be told apart at 66%, which suggests each model family has its own fingerprint. — [arXiv 2410.16107](https://arxiv.org/html/2410.16107)
- **Instruction tuning drives the difference:** Llama base models are closer to human feature rates than instruct models (Reinhart). In Freeburg's base-versus-instruct test, the Llama base model produced 0.49 em dashes per 1,000 words and the instruct model 0.00, so post-training can suppress as well as amplify. — [arXiv 2410.16107](https://arxiv.org/html/2410.16107); [arXiv 2603.27006](https://arxiv.org/html/2603.27006v1)
- **Liang et al. 2025** note that the 2024 plateau in measured LLM share may reflect "increasing subtlety of more advanced models." — [arXiv 2502.09747](https://arxiv.org/abs/2502.09747)
- **Genre dependence:** Reinhart et al. note that *camaraderie* and *palpable* are conspicuous only outside fiction. Desaire's classifier was built for science perspectives and is not claimed to transfer. — [arXiv 2410.16107](https://arxiv.org/html/2410.16107); [ScienceDaily](https://www.sciencedaily.com/releases/2023/06/230607124132.htm)
- **Corpora used:** PubMed (Kobak), arXiv (Geng & Trotta), ML peer reviews (Liang 2024), medRxiv (Czuma), essays (Freeburg, Herbold, Reinhart), NYT news (Muñoz-Ortiz), and consumer complaints, press releases, and job posts (Liang 2025). All are English. — sources above

### Inferences
- Word lists decay: a list fitted to 2024 GPT-4 output (*delve*) now catches mostly humans who picked up the word, plus older models. Rules should describe *function* (evaluative, promotional, or empty-connective words), not a fixed lexicon.
- Generalizing to technical docs is plausible for the grammatical tells (participial tack-ons, nominalizations), which appear across essays, abstracts, and news. It is weaker for vocabulary, since technical docs have their own register. For em dashes, the human baseline in docs is unknown.
- Structural markdown (bullets, headings, bold) is *expected* in docs. Freeburg's finding that LLMs default to markdown in prose doesn't make a bulleted docs page suspect. The tell is markdown structure where prose would serve better, which no study measures.

### Gaps
- No longitudinal measurement of Claude-family tells was found beyond Freeburg's single snapshot.
- A claimed Economist-commissioned test (four models rewriting Economist articles; only Claude above human em-dash rate, ChatGPT below) appears in a secondary blog only. Unverified. — [SilentRoom (secondary)](https://silentroom.media/the-craft/history-of-the-em-dash)
- A claim that a "GPT-6 Astra" uses em dashes at one-eighth of the human rate came up in search and could not be traced to a primary source. Treat it as unverified.
- No non-English or technical-documentation corpus study of any tell was found.

## 4. False positives, non-native and technical writers, and whether avoiding tells improves text

### Takeaway
Perplexity-based detectors flag non-native English writing heavily: 61% of TOEFL essays on average, 98% by at least one detector. Simplifying native writing makes it look like AI. Untrained readers are near chance, while frequent LLM users are very accurate. Whether stripping the tells makes text *better*, rather than just different, is largely unmeasured. The best-grounded tells (participial tack-ons, nominalizations, evaluative filler) overlap with long-standing plain-language advice, which is the main reason to avoid them.

### Cited Findings
- **Liang et al., "GPT detectors are biased against non-native English writers" (Patterns 2023; arXiv 2304.02819):** 7 detectors (Originality.AI, Quill.org, Sapling, OpenAI, Crossplag, GPTZero, ZeroGPT). — [arXiv 2304.02819](https://arxiv.org/html/2304.02819)
  - 91 TOEFL essays: 61.22% average false-positive rate; 89 of 91 (97.80%) flagged by at least one detector; 18 of 91 (19.78%) flagged by all seven.
  - 88 US 8th-grade essays: 5.19% average misclassification.
  - Having an LLM enrich the TOEFL essays' vocabulary cut false positives to 11.77% (−49.45 pp). Simplifying the 8th-grade essays raised misclassification to 56.65%.
  - Mechanism: low perplexity. Unanimously flagged essays had significantly lower perplexity (P = 9.74E-05).
- **Russell, Karpinska & Iyyer 2025:** — [arXiv 2501.15654v2](https://arxiv.org/html/2501.15654v2)
  - Five frequent-LLM-user experts by majority vote: TPR 99.3%, FPR 0%, 1 error in 300 articles. Individual experts: average TPR 92.7%, FPR about 3–4%, with one expert at 59.3% TPR.
  - Nonexperts: TPR 56.7%, FPR about 52%, near chance yet confident.
  - Humanized o1-Pro text: majority vote still at 100% TPR, but one expert fell from 16.7% to 0%.
  - Detectors for comparison: Pangram 98.0%/2%, GPTZero 85.3%/0.7%, Fast-DetectGPT 80.0%/7.2%, Binoculars 66.7%/1.3%, RADAR 15.3%/2% (TPR/FPR).
- **Herbold et al. 2023:** teachers rated ChatGPT essays *higher* in quality than student essays. "Sounds AI" and "is worse" are separate judgments. — [arXiv 2304.14276](https://arxiv.org/abs/2304.14276)
- **Kobak et al.:** lower measured LLM shares in UK and Australian papers may partly reflect native speakers editing out the marker words, so tell-avoidance is already a confound in the measurements. — [arXiv 2406.07016](https://arxiv.org/html/2406.07016)
- **Geng & Trotta:** humans and LLMs co-evolve, which makes individual texts hard to classify. — [arXiv 2502.09606v2](https://arxiv.org/html/2502.09606v2)
- **Shaib et al. 2025 ("slop"):** human slop judgments "correlate with latent dimensions such as coherence and relevance". This links the *quality* complaint to coherence and relevance rather than to AI origin per se (abstract only). — [arXiv 2509.19163](https://arxiv.org/abs/2509.19163)
- **Freeburg:** the human em-dash baseline ranges 0.33–17.12 per 1,000 words, so a heavy em-dash human writer sits above every model measured. — [arXiv 2603.27006](https://arxiv.org/html/2603.27006v1)

### Inferences
- Technical writers are at elevated false-positive risk by the same mechanism as non-native writers: controlled vocabulary, short declarative sentences, and consistent terminology all lower perplexity. No study tests this directly (see Gaps).
- A style skill should frame tells as *quality* problems, such as empty evaluation, tacked-on participles, nominalized verbs, or formulaic triplets, not as "sounds like AI." The quality case holds even if the AI association fades, and it avoids penalizing plain, consistent technical prose.
- Since expert human readers rely mostly on vocabulary (53%) and sentence structure (36%), those are the tells readers actually notice. Removing them changes perception the most.

### Gaps
- No study found that measures whether removing tells improves comprehension, task success, or trust for readers. The evidence that such edits "help" is only the overlap with plain-language guidance.
- No study found on detector false-positive rates for technical or developer documentation, or for professional technical writers.

## 5. Studies on AI-generated technical documentation specifically

### Takeaway
No peer-reviewed or preprint study was found that characterizes the linguistic features of AI-generated developer docs, READMEs, or API reference, or that measures detection on them. Work in this area evaluates *usefulness* or *similarity* (BLEU, ROUGE, BERTScore), not stylistic fingerprints. Anything said about "AI tells in docs" is extrapolated from academic and essay corpora, or is anecdotal.

### Cited Findings
- **"Can Developers Prompt?" (arXiv 2408.00686):** a controlled experiment on code-documentation generation. Developers rated docs from predefined prompts significantly higher for readability, conciseness, usefulness, and helpfulness than docs from their own ad-hoc prompts. Most evaluations of AI-generated docs rely on BLEU, ROUGE, or METEOR. — [arXiv 2408.00686](https://arxiv.org/pdf/2408.00686)
- **ReadmeReady (LLM README generator):** BLEU 15–30 (mean about 20) against original READMEs, and BERT-based similarity F1 about 85%. Read as understandable but needing substantial editing. It measures similarity, not style. — [ReadmeReady](https://github.com/souradipp76/ReadMeReady)
- **README classification (arXiv 2507.21899):** fine-tuned encoders label README sections (F1 0.98) in human-written READMEs. Not about AI authorship. — [arXiv 2507.21899](https://arxiv.org/html/2507.21899)
- **Anecdotal:** practitioner write-ups describe AI-generated docs as formal and padded with constructions like "it is important to note that." Unmeasured. — [coffeecup.tech (anecdotal)](https://www.coffeecup.tech/docs/tw_withAI/intro_ai/)

### Inferences
- The tells most likely to transfer to docs are grammatical (participial tack-ons, nominalizations) and evaluative vocabulary (*seamless*, *robust*, *comprehensive*, *crucial*). They are register-independent in the studies above, and they conflict with docs norms (imperative mood, concrete verbs).
- Tells that likely *don't* transfer cleanly: markdown structure (lists, headings, and tables are native to docs), sentence-length uniformity (procedures are uniform by design), and low lexical diversity (consistent terminology is a docs virtue).

### Gaps
- No corpus study of AI-written versus human-written READMEs, API references, or docs sites. A mining-software-repositories study (for example, READMEs before and after 2023 on GitHub, à la Kobak) would fill this, but none was found.
- No human-baseline rates for em dashes, participial clauses, or the excess-vocabulary words in technical documentation.
