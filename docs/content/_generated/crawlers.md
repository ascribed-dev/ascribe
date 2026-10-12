<!-- Generated from crates/ascribe-model/data/crawlers.toml by tests/conformance/tests/docs.rs. Edit the data, then run `ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs`. -->

| Purpose | Token | Operator | What the operator says of `robots.txt` | Read |
|---|---|---|---|---|
| `training` | `GPTBot` | OpenAI | Follows it. | [2026-10-12](https://developers.openai.com/api/docs/bots) |
| `training` | `ClaudeBot` | Anthropic | Follows it. | [2026-10-12](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) |
| `training` | `Google-Extended` | Google | Follows it. A token, not a crawler: Googlebot fetches, and this says what Google may do with it. It covers training Gemini models and grounding answers in Gemini Apps, and doesn't affect Google Search. | [2026-10-12](https://developers.google.com/search/docs/crawling-indexing/google-common-crawlers) |
| `training` | `Applebot-Extended` | Apple | Follows it. A token, not a crawler: Applebot fetches, and this says whether Apple may train its models on it. Pages it disallows can still appear in Apple's search. | [2026-10-12](https://support.apple.com/en-us/119829) |
| `training` | `MistralAI-Training` | Mistral AI | Follows it. | [2026-10-12](https://docs.mistral.ai/robots) |
| `search` | `OAI-SearchBot` | OpenAI | Follows it. | [2026-10-12](https://developers.openai.com/api/docs/bots) |
| `search` | `Claude-SearchBot` | Anthropic | Follows it. | [2026-10-12](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) |
| `search` | `PerplexityBot` | Perplexity | Follows it. | [2026-10-12](https://docs.perplexity.ai/guides/bots) |
| `search` | `Amzn-SearchBot` | Amazon | Follows it. When `robots.txt` doesn't name it, it follows the rules given to other search bots. | [2026-10-12](https://developer.amazon.com/amazonbot) |
| `search` | `MistralAI-Index` | Mistral AI | Follows it. | [2026-10-12](https://docs.mistral.ai/robots) |
| `on-request` | `ChatGPT-User` | OpenAI | May not follow it. | [2026-10-12](https://developers.openai.com/api/docs/bots) |
| `on-request` | `Claude-User` | Anthropic | Follows it. | [2026-10-12](https://support.claude.com/en/articles/8896518-does-anthropic-crawl-data-from-the-web-and-how-can-site-owners-block-the-crawler) |
| `on-request` | `Perplexity-User` | Perplexity | May not follow it. | [2026-10-12](https://docs.perplexity.ai/guides/bots) |
| `on-request` | `Amzn-User` | Amazon | May not follow it. | [2026-10-12](https://developer.amazon.com/amazonbot) |
| `on-request` | `MistralAI-User` | Mistral AI | Follows it. | [2026-10-12](https://docs.mistral.ai/robots) |
| `on-request` | `Meta-ExternalFetcher` | Meta | May not follow it. | [2026-10-12](https://developers.facebook.com/docs/sharing/webmasters/web-crawlers/) |
