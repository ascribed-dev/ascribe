# What docs tools have shipped for agents that write docs (state as of October 2026)

Scope note: this survey covers features aimed at agents *authoring and maintaining* docs. Read-side features (llms.txt, docs MCP search servers) are mentioned only where the same tool also ships a write-side feature. Items confirmed only for 2025 are marked "(2025)". Research date: 2026-10-04.

## Key question 1: What have the named docs tools shipped for agents that write docs?

### Takeaway
The hosted platforms (Mintlify, Fern, GitBook, ReadMe, Redocly) have all shipped a "docs agent" with a PR-opening workflow, an admin/write MCP server, and an installable agent skill; the open-source generators (Docusaurus, Starlight, Fumadocs, Nextra, MkDocs Material, Antora, Sphinx, Hugo) have shipped almost nothing official for writing agents beyond AGENTS.md files and lint commands, and the gap is filled by community skills. Vale is the only linter with an official, first-party agent toolkit (skills + edit hook + MCP).

### Cited Findings

**Mintlify**
- Mintlify's agent runs a five-step loop: research existing docs and connected repos, plan, write/update content, "Runs Mintlify CLI checks to ensure documentation builds correctly", then opens a PR or commits to the deployment branch; branch protection forces PR mode; requires Pro or Enterprise plan — [Mintlify docs: Agent](https://www.mintlify.com/docs/agent)
- Automations have five trigger types: content update (push to docs repo), code change (PR merge in connected source repos), custom schedule (queued within 10 minutes), integration (Slack/connected services, Enterprise only), webhook (authenticated endpoint). Each automation has exactly one trigger. Output is a PR; related changes are grouped into a single PR except "Draft changelog" and "Improve docs from user feedback" which open one PR per run. Each run that updates docs costs 250 credits; runs that find nothing cost 0; limit 500 runs/automation/day. On GitHub, Mintlify auto-assigns reviewers from the source-code PR authors and adds PR comments explaining the changes and which source PRs were used as context — [Mintlify docs: Agent workflows](https://www.mintlify.com/docs/agent/workflows)
- Self-updating docs announcement (automations run "on a schedule or on a push to a repository", agent clones repos and follows a configured prompt, suggestions appear in "the agent panel") — [Mintlify blog, Dec 8 2025 (2025)](https://www.mintlify.com/blog/autopilot)
- Agent suggestions from assistant questions: dashboard surfaces "focused recommendations such as clarifying a concept, adding an example, or restructuring a section" based on patterns in docs-assistant questions — [Mintlify blog, Jan 7 2026](https://www.mintlify.com/blog/agent-suggestions-assistant)
- Admin MCP server at `https://mcp.mintlify.com` gives write access: create/update/move/delete pages and navigation nodes, edit `docs.json`, upload images, open PRs, "create workflows", manage settings/members/analytics via a "code mode"; OAuth login; install with `claude mcp add --transport http mintlify https://mcp.mintlify.com`, Cursor `mcp.json`, Codex `~/.codex/config.toml`, ChatGPT connector. Changes via the MCP "apply immediately to the live project without pull requests". Only validation mentioned: uploaded file contents must match extension — [Mintlify docs: Admin MCP](https://www.mintlify.com/docs/ai/mintlify-mcp)
- Official Claude Code guide recommends a `CLAUDE.md` with frontmatter requirements (title, description), writing standards, and a validation loop of `mint dev --no-open`, `mint validate`, `mint broken-links`; installs the Mintlify skill with `npx skills add https://mintlify.com/docs`; MCP for docs search at `https://mintlify.com/docs/mcp` and admin at `https://mcp.mintlify.com` — [Mintlify docs: Write documentation with Claude Code](https://www.mintlify.com/docs/guides/claude-code)
- CLI checker commands: `mint validate` "validates the documentation build in strict mode and exits with a non-zero code on any warning or error"; `mint broken-links` with flags `--files`, `--check-anchors`, `--check-external`, `--check-redirects`, `--check-snippets` (links inside `<Snippet>` components) — [Mintlify CLI command reference](https://www.mintlify.com/docs/cli/commands)
- Mintlify maintains its agent skill in-repo at `agent-context/context/skills/mintlify/` (SKILL.md + `reference/cli.md`) with tests; a `mintlify[bot]` automation opened PR #7471 on Sep 23 2026 titled "Agent skill sync with docs" to remove deprecated `mint automations create|list|delete` guidance from the skill; after merge a build workflow "distributes the updated skill to the Codex, Cursor, and Claude plugin repositories" — [mintlify/docs PR #7471](https://github.com/mintlify/docs/pull/7471)
- The Mintlify bot opens "Fix broken links" PRs in customer docs repos (examples: superradcompany/microsandbox #1530, lightdash/mintlify-docs #1191, mintlify/docs #7711) — [microsandbox PR #1530](https://github.com/superradcompany/microsandbox/pull/1530); [lightdash PR #1191](https://github.com/lightdash/mintlify-docs/pull/1191)
- A `doc-author` skill in mintlify/docs' `.claude/skills` is described as "write and maintain documentation autonomously... when you are assigned to create, update, or improve documentation without direct human oversight. It always opens PRs for review" — [skills.lc listing of mintlify/docs doc-author SKILL.md](https://skills.lc/mintlify/docs/mintlify-docs-claude-skills-doc-author-skill-md) (third-party index of the repo file)
- Mintlify plugin listed in the Claude marketplace — [claude.com marketplace: Mintlify](https://claude.com/marketplace/plugins/mintlify)
- A community pre-commit hook wraps Mintlify validation — [CoderJoshDK/precommit-mintlify-validate](https://github.com/CoderJoshDK/precommit-mintlify-validate)

**Fern**
- Fern Agent is available in the Fern Dashboard chat panel, Slack (`@Fern`), and to coding agents via MCP. It reads the source repo, understands `docs.yml`, navigation, product switchers, versions, redirects, component library, RBAC; writes config in Fern's format (e.g., auto-generates redirects for moved pages); delivers changes as GitHub PRs only (no GitLab); receives analytics (pageviews, 404s, search queries), Ask Fern conversations, deployment history, and image attachments as context. No mention of `fern check` or link validation — [Fern docs: Fern Agent](https://buildwithfern.com/learn/docs/ai-features/fern-agent)
- Fern Agent launch post says edits are "restrained by default, modifying only the lines that need changing rather than regenerate the page" — [Fern blog: One agent for your documentation](https://buildwithfern.com/post/fern-agent)
- Setup: MCP URL `https://fai.buildwithfern.com/organizations/YOUR_ORGANIZATION/mcp` (org from `fern/fern.config.json`); Claude Code/Cursor/Codex: install Fern CLI, `fern login`, `npx skills add fern-api/skills --skill fern-docs -a [agent]`, `fern mcp install --client [client]`; Copilot gets skill only (no MCP step); `-g` writes to `.agents/skills/` — [Fern docs: agent setup](https://buildwithfern.com/learn/docs/ai-features/agent-setup.md)
- `fern-api/skills` repo ships one skill, `fern-docs`: "Building Fern docs sites: docs.yml, navigation, pages, custom MDX, landing pages, changelog entries, and access control"; install `npx skills add fern-api/skills`; targets Claude Code, Cursor, Codex, Copilot — [fern-api/skills](https://github.com/fern-api/skills)
- Fern published "AI Agents for Technical Writing (January 2026)" comparing Devin, Cursor, Claude Code for docs work — [Fern blog](https://buildwithfern.com/post/technical-writing-ai-agents-devin-cursor-claude-code) (page content could not be fetched in full; only the title and summary snippet were confirmed)

**GitBook**
- GitBook Agent (beta) launched Dec 10 2025: update/rewrite pages from prompts, open change requests, "Apply your style guide automatically", review change requests with feedback, respond to `@GitBook` mentions; roadmap: proactive gap-spotting, early access via Organization Settings → Docs Agent — [GitbookIO discussion #1115 (2025)](https://github.com/orgs/GitbookIO/discussions/1115)
- Current docs: invoke via sidebar chat, `@gitbook` in comments, or change-request review; reviews "flag style guide issues, and suggest or fix errors" acting as a linter; configure under Site Settings > Agents with style guides "treated as source of truth" and site-specific instructions (example: link conventions, block type restrictions); free in beta, 10 messages/week on non-Pro plans; proactive "content gaps" suggestions in beta; data not used for training, OpenAI is the model provider — [GitBook docs: Agent overview](https://gitbook.com/docs/gitbook-agent/overview.md)
- GitBook describes agent as "reactive, not proactive" as of its 2026 review coverage — [Ferndesk GitBook review 2026](https://ferndesk.com/blog/gitbook-review) (secondary source)

**ReadMe**
- GitHub AI Writer: watches PRs when opened, "reads the diff and asks one question: does any existing documentation need to change because of this?", drafts on a review branch in ReadMe (not a GitHub PR), posts a PR comment with preview link; writers edit inside ReadMe; Pro, Enterprise, legacy Business plans — [ReadMe blog: Introducing GitHub AI Writer](https://readme.com/blog/ai-writer)
- `@readme/cli` (`rdme`) can "lint your docs, sync OpenAPI specs, preview locally, and set up CI in one command" and is explicitly "built for" Claude Code and Codex so agents can "draft pages, iterate against lint warnings, and ship clean docs without manual oversight"; changelog fixed the `llms.txt` linter wrongly flagging `doc:` links with anchors; models updated to Claude Opus 4.7 and GPT-5.5 (dates this to 2026) — [ReadMe changelog: May launch](https://docs.readme.com/main/changelog/may-launch)
- "Agent Owlbert handles audits and linting, helping teams identify gaps in their docs and ensure updates match their style guide" — [Mintlify library: Best AI documentation tools](https://www.mintlify.com/library/best-ai-documentation-tools) (competitor-written, secondary)

**Redocly (Markdoc-based)**
- Aug 14 2026: shipped built-in Docs MCP at `/mcp` (protocol 2026-07-28), MCP server cards and A2A agent cards, "Agent skills: Drop task-focused skill files into an `@skills` folder", custom Markdoc tags with `renderForLlms` (locale-aware), and a "Reviewer" that checks API descriptions, clarity, OWASP Top 10 risks, and cross-document consistency. Planned: "Writer: Autonomous agent identifying documentation gaps via search queries and analytics", Gateway MCP, "code mode" (86% fewer input tokens) — [Redocly blog](https://redocly.com/blog/ai-features-summer-2026)

**Docusaurus**
- Official `AGENTS.md` requires "(AI-assisted)" labels in commits/PRs, forbids non-maintainer agents from opening issues/PRs, and lists lint commands `pnpm format` (oxfmt), `pnpm lint:js`, `pnpm lint:style`, `pnpm lint:spelling` (CSpell), `pnpm lint:knip`; no MDX or frontmatter conventions — [facebook/docusaurus AGENTS.md](https://github.com/facebook/docusaurus/blob/main/AGENTS.md)
- Community: `rio225/docusaurus-skill` (Diátaxis-based guidance), `DevRico003/docusaurus-skills` (setup + llms.txt + GitHub Actions CI/CD for Claude Code and Codex) — [docusaurus-skill](https://github.com/rio225/docusaurus-skill); [docusaurus-skills](https://github.com/DevRico003/docusaurus-skills)

**Astro Starlight**
- No official Starlight agent feature found; several community Claude Code skills exist (e.g., "Astro Starlight Documentation", "astro-coding" with tiered loading) — [mcpmarket Astro Starlight skill](https://mcpmarket.com/tools/skills/astro-starlight-documentation); [astro-coding skill](https://www.claudepluginhub.com/skills/superbenefit-astro-dev-astro-dev/skills/astro-coding)

**Fumadocs**
- `fumadocs-core/mcp` exposes read-only tools `list_pages`, `get_page`, `search` and an `/api/mcp` route — read side only — [Fumadocs MCP](https://www.fumadocs.dev/docs/headless/utils/mcp)
- Inkeep's pipeline turns Fumadocs sites into Agent Skills using `meta.json` for structure — [Inkeep blog: Docs to agent skills](https://inkeep.com/blog/docs-to-agent-skills)
- Community "Fumadocs Patterns" skill for MDX management and navigation syncing — [mcpmarket](https://mcpmarket.com/tools/skills/fumadocs-documentation-patterns)

**Nextra**: only a community Claude Code skill found — [mcpmarket Nextra skill](https://mcpmarket.com/tools/skills/nextra-documentation-framework-1)

**MkDocs Material**
- Discussion #8433 (Sep 2 2025): maintainer Martin Donath said AI features would be considered "once our current foundational work reaches stability"; views MCP as more promising than RAG chatbots; "No AI assistant integration has been shipped" — [squidfunk/mkdocs-material discussion #8433 (2025)](https://github.com/squidfunk/mkdocs-material/discussions/8433)
- Community skills: terminalskills.io mkdocs skill (Claude Code, Codex, Gemini CLI, Cursor), "MkDocs Documentation Manager" (mkdocs.yml schema, CLI reference) — [terminalskills mkdocs](https://terminalskills.io/skills/mkdocs); [MkDocs Documentation Manager](https://mcpmarket.com/tools/skills/mkdocs-documentation-manager)

**Sphinx**
- Sphinx has an official AI policy page — [Sphinx AI Policy](https://www.sphinx-doc.org/en/master/internals/ai-policy.html) (not fetched; content unconfirmed)
- Community: `fix-sphinx-docs` skill runs the build and fixes warnings "within a 60-second timeframe"; `sphinx-dev-agent` includes a "verifying builds" skill; `sphinxdocs_mcp` indexes Sphinx text output into SQLite FTS5 (read side) — [fix-sphinx-docs](https://lobehub.com/skills/tradingstrategy-ai-web3-ethereum-defi-fix-sphinx-docs); [sphinx-dev-agent](https://github.com/jahn-junior/sphinx-dev-agent); [sphinxdocs_mcp](https://github.com/AUrbanec/sphinxdocs_mcp)

**Writerside**: JetBrains AI Assistant integration offers Rephrase, Review, Translate to English, Generate TLDR — writer-facing editor actions, not agent tooling; 2026 releases include AI Assistant stability fixes — [Writerside AI Assistant help](https://www.jetbrains.com/help/writerside/ai-assistant.html); [Writerside 2026.09.0357](https://plugins.jetbrains.com/plugin/20158-writerside/versions/stable/1179837)

**Antora**: only a community "Antora Structure" Claude Code skill (validates project layout, modules, family directories) — [mcpmarket Antora Structure](https://mcpmarket.com/tools/skills/antora-structure-guide)

**Hugo**: Hugo FixIt theme ships an official skills collection; Hextra theme has built-in Markdown output format and llms.txt templates (read side) — [hugo-fixit/skills](https://github.com/hugo-fixit/skills); [Dachary Carey: Make your Hugo site agent friendly (Mar 1 2026)](https://dacharycarey.com/2026/03/01/make-hugo-site-agent-friendly/)

**Vale**
- Official agent toolkit at `vale-cli/agent-tools` (MIT): five skills `/vale:setup`, `/vale:fix` (one PR per file, error-level only), `/vale:triage` (per-rule fix/downgrade/disable), `/vale:vocab`, `/vale:ci`; an edit hook that runs `vale --no-exit --output=line <file>` when the agent writes a file and returns only error-level alerts (configurable via `VALE_HOOK_LEVEL`: error/warning/suggestion); a Vale CMS MCP server with `scaffold_rule`, `diagnose_rule`, `test_rule`, `stress_rule`, `diff_rule`, `audit_style` (requires paid Vale CMS); install in Claude Code with `/plugin marketplace add vale-cli/agent-tools` then `/plugin install vale@agent-tools`; Cursor by copying `skills/` to `.cursor/skills/`; other clients via `https://vale.sh/AGENTS.md` — [vale-cli/agent-tools](https://github.com/vale-cli/agent-tools); [vale.sh/skills](https://vale.sh/skills)
- `vale.sh/AGENTS.md` instructs agents to use `--output=JSON` ("default terminal-oriented output is unstable for scraping"), notes "Only `error` sets a non-zero exit code", tells agents to fix prose or add vocab rather than disable rules ("disabling `Vale.Spelling` because a product name trips it is the wrong fix") — [vale.sh/AGENTS.md](https://vale.sh/AGENTS.md)
- Third-party Vale MCP servers predate the official one: ChrisChinchilla/Vale-MCP and theletterf/vale-mcp-server (Fabrizio Ferri Benedetti) — [Vale-MCP](https://github.com/ChrisChinchilla/Vale-MCP); [PulseMCP listing](https://www.pulsemcp.com/servers/theletterf-vale)

**Speakeasy**: docs generation is OpenAPI-driven (`docs-md` compiler, Docs MCP for reading); the CLI has `speakeasy agent context` for "structured documentation and project guidance" to agents — no docs-writing agent found — [speakeasy-api/docs-md](https://github.com/speakeasy-api/docs-md); [Speakeasy CLI reference](https://www.speakeasy.com/docs/speakeasy-reference/cli/)

### Inferences
- The delivery pattern that has converged across Mintlify, Fern, and Vale is a triple: an installable skill (via `npx skills add` or Claude plugin marketplace), a hosted MCP server for write access, and a CLI checker the skill tells the agent to run. Mintlify additionally treats its skill as a build artifact synced from docs by a bot and distributed to Codex/Cursor/Claude plugin repos.
- Only Mintlify explicitly documents its agent running the project's own CLI checks before opening a PR; Fern and GitBook document no checker loop.
- Open-source generators are relying on AGENTS.md plus existing lint commands; none ship an agent-specific checker or hook comparable to Vale's.

### Gaps
- Could not confirm the exact MCP tool names exposed by Fern's or Mintlify's write servers (docs describe capabilities, not tool schemas).
- Fern's Jan 2026 technical-writing-agents post and "agent-friendly docs" post could not be fetched (truncated); their specific claims are unconfirmed.
- No official Starlight, Nextra, Antora, or MkDocs Material agent features were found; absence is inferred from searches, not confirmed by maintainers (except MkDocs Material).

## Key question 2: Which docs agents loop on a checker, and how do they surface problems to an agent?

### Takeaway
Few products close the loop in the product itself: Mintlify's agent runs CLI checks before PRs, ReadMe designed `rdme` lint output for agents to iterate against, Vale returns error-level alerts into the agent's turn via a hook, and DocuGardener uses a generator/verifier pair; GitBook, Fern, Kapa, and Inkeep surface issues to humans (dashboards, PR comments, change requests) rather than to a checker loop.

### Cited Findings
- Mintlify agent "Runs Mintlify CLI checks to ensure documentation builds correctly" as step 4 of 5 before opening a PR — [Mintlify docs: Agent](https://www.mintlify.com/docs/agent)
- ReadMe `rdme` adapts output so agents can "iterate against lint warnings" — [ReadMe changelog](https://docs.readme.com/main/changelog/may-launch)
- Vale hook: "Lints each prose file the moment your assistant writes it and hands back only the error-level alerts, so mistakes get fixed in the same turn they were made" — [vale.sh/skills](https://vale.sh/skills)
- DocuGardener uses "a two-stage LLM pipeline: a Generator creates draft fixes, then a temperature-0 Verifier evaluates accuracy to prevent hallucinations"; surfaces as GitHub check run (red/green/neutral by drift severity), inline PR comments with semantic diffs, dashboard inbox, Slack/Jira — [docugardener/docugardener](https://github.com/docugardener/docugardener)
- Entropic Drift's ripgrep pipeline (Claude Agent SDK, Map/Reduce phases) validates with `mkdocs build --strict`, cross-page consistency checks, and `mermaid-sonar` complexity analysis; lesson: "Syntax validation alone isn't enough for AI-generated diagrams" — [Entropic Drift: Transforming ripgrep's documentation](https://entropicdrift.com/blog/mkdocs-drift-automation/)
- Passo Uno's approach: deterministic tools (Vale, Codespell, Lychee, git log) detect first, "the agent receives their outputs as context. The agent's job is to filter rather than find things deterministic tools can detect"; seven sweeps cover frontmatter validation, applicability metadata, opening paragraphs, style consistency, typos, staleness, coherence — [Passo Uno, May 11 2026](https://passo.uno/agentic-workflows-for-docs/)
- GitBook Agent reviews change requests and can "flag style guide issues, and suggest or fix errors" (surfaces to humans in the change request) — [GitBook docs](https://gitbook.com/docs/gitbook-agent/overview.md)
- Kapa: "Coverage Gaps" view clusters unanswered questions into findings plus an AI recommendation; the recommended way to act is the "Analyze Coverage Gaps" skill run by the user's coding agent inside the docs repo, after exporting gaps from the dashboard — [Kapa docs: Coverage Gaps](https://docs.kapa.ai/analytics/coverage-gaps); [kapa-ai/kapa-skills](https://github.com/kapa-ai/kapa-skills)
- kapa-skills: five skills (Analyze Coverage Gaps, Analyze Top Questions, Analyze Source Analytics, Answer RFP, Agent SDK Integration), installed by copying folders into `.claude/skills/`, `.cursor/skills/`, `.agents/skills/`; "All product capability claims are retrieved live from your kapa MCP; nothing is invented or assumed" — [kapa-ai/kapa-skills](https://github.com/kapa-ai/kapa-skills)
- Inkeep Content Writer: background agent turning resolved tickets, merged PRs, and Slack threads into GitHub PRs or Zendesk drafts via Router Agent / KB Article Agent / Page Drafter sub-agents, with a human review gate; Enterprise only, provisioned by Inkeep — [Inkeep docs: Content Writer](https://docs.inkeep.com/guides/agents/content-writer/overview)
- Redocly's "Reviewer" checks clarity, security, and cross-document consistency; its gap-finding "Writer" is still planned — [Redocly blog](https://redocly.com/blog/ai-features-summer-2026)

### Inferences
- The clearest "problem surface" for agents in production is a linter's structured output fed back in the same turn (Vale hook) or a CLI with non-zero exit on warnings (`mint validate`). Platform agents mostly surface to humans via PR comments and dashboards, and let the human prompt the agent again.
- Analytics-driven gap tools (Kapa, Mintlify suggestions, Redocly's planned Writer) are converging on a "export finding → run skill in repo" handoff rather than directly driving an agent.

### Gaps
- No public details on whether Fern Agent or GitBook Agent validate output (build, links) before proposing changes.
- Inkeep's `inkeep-agents-action` GitHub Action exists but its docs-writing role was not confirmed — [inkeep/inkeep-agents-action](https://github.com/inkeep/inkeep-agents-action)

## Key question 3: How do teams use Vale, markdownlint, textlint, alex, write-good, lychee, markdown-link-check with agents?

### Takeaway
The dominant pattern is deterministic linters first (pre-commit or CI), results posted via reviewdog or returned to the agent as structured output, with the agent fixing rather than finding; Copilot Autofix does not apply to docs linters, so teams build their own feedback paths (Vale's edit hook, gh-aw workflows, custom Claude Code actions).

### Cited Findings
- Vale edit hook: `vale --no-exit --output=line` on agent-written files, errors only by default; `/vale:ci` skill configures GitHub Actions and pre-commit "with pinned versions, distinguishing between advisory and blocking execution" — [vale-cli/agent-tools](https://github.com/vale-cli/agent-tools); [vale.sh/skills](https://vale.sh/skills)
- Vale's agent guidance prefers `--output=JSON` for parsing and warns warnings/suggestions exit 0 — [vale.sh/AGENTS.md](https://vale.sh/AGENTS.md)
- reviewdog "posts them as a comment if findings are in the diff of patches"; `reviewdog/action-markdownlint` exists — [reviewdog](https://github.com/reviewdog/reviewdog); [reviewdog/action-markdownlint](https://github.com/reviewdog/action-markdownlint)
- `leinardi/gha-pre-commit-markdownlint-cli2-reviewdog` runs markdownlint-cli2 via pre-commit and reports "suggested fixes (as a diff review) and diagnostics (inline comments on the PR)" — [repo](https://github.com/leinardi/gha-pre-commit-markdownlint-cli2-reviewdog)
- Copilot Autofix "is available for CodeQL analysis" (code scanning alerts), not for prose/markdown linters — [GitHub docs: Copilot Autofix](https://github.com/github/docs/blob/main/content/code-security/concepts/code-scanning/copilot-autofix-for-code-scanning.md)
- Passo Uno (Elastic) runs Vale, Lychee, Codespell inside GitHub Agentic Workflows and passes their output to the agent as context — [Passo Uno](https://passo.uno/agentic-workflows-for-docs/)
- Larah Vasquez (Tailscale) has ~200 skills including "automated linting with reasoning-based fixes" and drift detection; a proposed skill would "generate Vale rules automatically from existing style guide sites" — [I'd Rather Be Writing podcast, Apr 12 2026](https://idratherbewriting.com/blog/ai-skills-agentic-workflows-larah-fabrizio)
- Example of agent-driven Vale adoption: an issue to "clean up 1769 pre-existing error-level alerts" in a repo's docs — [pvliesdonk/markdown-vault-mcp #686](https://github.com/pvliesdonk/markdown-vault-mcp/issues/686)
- `yzhao062/agent-style`: "21 writing rules for AI coding and writing agents. Drop-in for Claude Code, Codex, Copilot, Cursor, and Aider" — an agent-facing style guide delivered as instruction files — [agent-style](https://github.com/yzhao062/agent-style)

### Inferences
- Because no SARIF/autofix path exists for prose linters, the Vale hook (agent-side, same-turn) and reviewdog (PR-side, human-visible) are the two practical feedback channels; a checker that emits both a machine format for the agent and reviewdog/rdjson for PRs covers both.
- `/vale:triage` is a notable design: on a pre-existing corpus, let the agent decide per rule whether to fix, downgrade, or disable, instead of producing thousands of alerts.

### Gaps
- No sources found specifically about textlint, alex, write-good, or markdown-link-check used with agents in 2025–2026; lychee appears only in Passo Uno's stack.
- No evidence found of SARIF uploads for docs linters being consumed by agents.

## Key question 4: Docs drift tools that detect code changes and prompt an agent

### Takeaway
Drift tooling in 2026 comes in three shapes: platform-native (Mintlify "code change" automations, ReadMe GitHub AI Writer, Fern Agent, Inkeep Content Writer), open-source GitHub Apps (DocuGardener), and DIY GitHub Actions (claude-code-action, gh-aw docs-updater, Copilot `assign-to-agent`); what they feed the agent is almost always the PR diff plus a docs directory, sometimes with the PR metadata and a prompt that tells the agent when *not* to change docs.

### Cited Findings
- Mintlify "code change" trigger fires when PRs merge in connected source repos; agent reads project content and repos and follows the prompt — [Mintlify docs: workflows](https://www.mintlify.com/docs/agent/workflows)
- ReadMe AI Writer triggers when PRs are *opened*, reads the diff, drafts in ReadMe, comments on the PR with a preview link — [ReadMe blog](https://readme.com/blog/ai-writer)
- DocuGardener: parses diffs with tree-sitter, compares against Markdown/RST docs, blocks merges via check run, "AI Author Mode" detects AI-authored PRs (Copilot, Cursor, Devin, Claude) and auto-drafts and auto-merges doc fixes; overrides `!dgignore`, triage inbox, admin force-merge logged as debt; AGPL-3.0, Docker Compose or Helm, LLM providers Gemini/OpenAI/Anthropic/Ollama; free tier 1 public repo / 50 analyses per month; solo project since April 2026 — [docugardener/docugardener](https://github.com/docugardener/docugardener)
- Dosu's recipe (Mar 6 2026): `anthropics/claude-code-action@v1` on `pull_request: types: [closed], branches: [main]` with path filters (`src/`, `lib/`, `api/`, `packages/`); prompt tells Claude to read changed files, review `docs/` and `README.md`, skip internal-only changes, create branch `docs/update-from-pr-[number]`, open a follow-up PR; passes PR title/body/number/author and changed-file list wrapped in XML delimiters as untrusted input; `--max-turns 15`; author-association gating, bot exclusion, `skip-docs-check` label — [Dosu blog](https://dosu.dev/blog/how-to-catch-documentation-drift-claude-code-github-actions)
- GitHub Agentic Workflows: Markdown + YAML frontmatter compiled by `gh aw compile` to `.lock.yml`; gallery "Documentation Updater" runs weekly, finds "outdated setup steps, missing option descriptions, and examples that no longer match current behavior", uses `create-pull-request` safe output with `[docs]` prefix and `draft: true`; install `gh aw add-wizard githubnext/agentics/docs-updater` — [gh-aw docs automation gallery](https://github.github.com/gh-aw/gallery/docs-automation/); [github/gh-aw](https://github.com/github/gh-aw)
- gh-aw `assign-to-agent` safe output hands an existing issue/PR to Copilot coding agent, which "implement[s] the described task and open[s] a pull request"; requires a fine-grained PAT (App tokens rejected); default max 1 assignment — [gh-aw Copilot cloud agent reference](https://github.github.com/gh-aw/reference/copilot-cloud-agent/)
- Copilot coding agent can be assigned issues on github.com, Mobile, or CLI, and GitHub lists "update documentation" among its tasks — [GitHub blog: coding agent 101](https://github.blog/ai-and-ml/github-copilot/github-copilot-coding-agent-101-getting-started-with-agentic-workflows-on-github/)
- Docusaurus-oriented community automation: agent "analyzes your Git staged changes, identifies what needs documentation updates, and modifies the appropriate Markdown files" — [deeptoai guide](https://cc.deeptoai.com/docs/en/tools/automated-documentation-claude-code-docusaurus)
- Focused Labs argues drift breaks agents that consume docs, giving a second motivation for drift tooling — [dev.to: Documentation Drift Breaks Coding Agents](https://dev.to/focused_dot_io/documentation-drift-breaks-coding-agents-focused-labs-3bkp)
- Agent pattern writeup: schedule- or push-triggered audits that "open reviewable PRs to realign docs" — [AgentPatterns: Continuous Documentation](https://www.agentpatterns.ai/workflows/continuous-documentation/)

### Inferences
- The inputs that recur: PR diff (or weekly git log), docs tree, PR metadata, and negative instructions ("skip for bug fixes/perf/internal"). The Dosu and gh-aw recipes also encode safety: draft PRs, bot-author exclusion, turn limits, untrusted-input delimiters.
- DocuGardener's "AI Author Mode" is the first tool observed to treat AI-authored code PRs as a distinct class deserving automatic doc fixes.

### Gaps
- No evidence found of OpenAI Codex cloud tasks being triggered from CI for docs updates.
- No published accuracy numbers for any drift detector beyond DocuGardener's self-reported test count.

## Key question 5: Frontmatter and schema enforcement patterns agents handle well

### Takeaway
Schema validation at build time (Astro/Zod, `mint validate`) reliably *catches* agent frontmatter mistakes, and real PRs show agents commit invalid enums and missing required fields anyway; the tools that aim to prevent the error put the frontmatter contract into a skill or CLAUDE.md, and the most agent-friendly checkers emit human-readable or JSON errors rather than raw validator dumps. No controlled comparison of scaffold commands versus schema-in-instructions was found.

### Cited Findings
- Astro content collections: "Schemas enforce consistent frontmatter or entry data within a collection through Zod validation" — [Astro docs](https://docs.astro.build/en/guides/content-collections/)
- Real agent failure: five agent-generated stubs broke `astro build` with `format: "guide"` (invalid enum), missing required `dek` and `heroImage`, and `status: published` on placeholder content; a Copilot PR (Aug 1 2026) fixed them by choosing valid enums (`hub-guide`, `peninsula-notes`), adding fields, and setting `status: draft`; no new guardrails were added — [richmondjw/peninsula-insider PR #301](https://github.com/richmondjw/peninsula-insider/pull/301)
- Astro v6 regressed validation errors to "raw serialized zod error" output (`"**: [ { "expected": "object", "code": "invalid_type"...`); issue Mar 18 2026, closed by PR #15981 — [withastro/astro #15976](https://github.com/withastro/astro/issues/15976)
- Mintlify's skill is marketed for "adding a new docs page in the right place with the right frontmatter" and "inserting the right component without looking up syntax" — [enterprisedna skills listing of Mintlify skill](https://enterprisedna.co/directories/skills/mintlify-skill/) (third-party listing); Mintlify's CLAUDE.md template requires `title` and `description` frontmatter — [Mintlify Claude Code guide](https://www.mintlify.com/docs/guides/claude-code)
- Passo Uno's sweeps include "frontmatter validation" and "applicability metadata" as deterministic checks fed to the agent — [Passo Uno](https://passo.uno/agentic-workflows-for-docs/)
- gh-aw itself uses the pattern of Markdown with YAML frontmatter validated by `gh aw compile` before anything runs — [GitHub docs: About Agentic Workflows](https://docs.github.com/en/copilot/concepts/agents/about-github-agentic-workflows)
- Skills themselves rely on frontmatter ("structured metadata (frontmatter) containing descriptions and specifications, enabling efficient discovery without loading full content") — [I'd Rather Be Writing podcast](https://idratherbewriting.com/blog/ai-skills-agentic-workflows-larah-fabrizio)
- Vale CMS MCP `scaffold_rule` generates valid YAML rule templates for agents rather than having them write rules from a schema — [vale-cli/agent-tools](https://github.com/vale-cli/agent-tools)

### Inferences
- Evidence from PR #301 suggests schema-only enforcement is a late signal: the agent that wrote the stubs never ran the build. Enforcement that runs at write time (a hook like Vale's, or an editor-side check) would have caught it earlier.
- Vale's choice to ship `scaffold_rule` as an MCP tool is the one data point of a vendor preferring a scaffold over schema-in-instructions for structured YAML authored by agents.
- Readable validator output matters for agents too: the Astro v6 regression was reported as a human-readability bug, but the same raw Zod dump is what an agent would receive.

### Gaps
- No study or writeup directly comparing `new page` scaffold commands against schema descriptions in instructions for agent accuracy.
- No evidence of JSON Schema for frontmatter being delivered to agents as a first-class artifact by any of the named docs tools.

## Key question 6: Variants, reusable snippets, and common agent mistakes with MDX/components/includes; evidence on AI-written docs quality

### Takeaway
Writeups agree that MDX components, imports, and includes are where agents stumble, and most tooling responses are either to teach the component vocabulary via a skill (Mintlify, Fern) or to validate component-level links and builds (`mint broken-links --check-snippets`, `mint validate`, `mkdocs build --strict`); survey data shows AI drafting is mainstream (76%) while hallucination remains the top concern and under half of teams have guidelines.

### Cited Findings
- Mintlify's checker has a dedicated flag `--check-snippets` to "check links inside `<Snippet>` components" and `--check-anchors` for heading slugs — [Mintlify CLI reference](https://www.mintlify.com/docs/cli/commands)
- Mintlify's skill ships "page conventions, navigation config, component patterns, OpenAPI integration" so the agent can insert "the right component without looking up syntax" — [enterprisedna listing](https://enterprisedna.co/directories/skills/mintlify-skill/)
- Fern's `fern-docs` skill covers "custom MDX, landing pages, changelog entries, and access control" — [fern-api/skills](https://github.com/fern-api/skills)
- IMG.LY: MDX files "are incomplete at rest, containing import statements that reference external modules, JSX expressions that require a runtime, and component calls that depend on a framework"; they compile MDX to resolved Markdown and report 7x faster ingestion — [IMG.LY blog](https://img.ly/blog/making-docs-machine-readable-why-we-native-compile-markdown-for-ai-agents/)
- Andrei Nita: "MDX earns its place only if the document needs reusable components or interactive examples"; otherwise Markdown is safer for agent repos — [MD vs MDX](https://andreinita.co/blog/md-vs-mdx-ai-agent-repos/)
- Entropic Drift found AI over-engineers Mermaid diagrams and built `mermaid-sonar` to measure node density and branching; "Mechanical rule-based enhancement either over-applies or under-applies" tabs/admonitions, so per-page agent judgement was used; result 55 pages, 87 diagrams, 358 admonitions — [Entropic Drift](https://entropicdrift.com/blog/mkdocs-drift-automation/)
- Fern Agent is designed to make small diffs instead of regenerating pages — [Fern blog](https://buildwithfern.com/post/fern-agent)
- State of Docs 2026 (published Aug 28 2026): 76% use AI regularly for documentation creation (60% in 2025); top tasks drafting 62%, proofreading 58%, style guide matching 50%; only 25% use AI to write complete documents; hallucinations are the leading concern (62%); only 44% have AI guidelines; documentation-specific AI tools used by 26% vs general LLMs 73% — [State of Docs 2026](https://www.stateofdocs.com/2026/ai-and-documentation-creation)
- Fabrizio Ferri-Benedetti: "how much of the work I'm doing is not so much about AI as it's about building elaborate pipes and fences around it" — [Passo Uno](https://passo.uno/agentic-workflows-for-docs/)
- Podcast finding: "thin skills that call out to an MCP server for doc content tend to outperform...skills with everything inlined" — [I'd Rather Be Writing](https://idratherbewriting.com/blog/ai-skills-agentic-workflows-larah-fabrizio)
- Write the Docs Berlin 2026: Anastasia Suboch presented a docs health framework measuring coverage, readability, style compliance, freshness, discoverability — [WTD Berlin 2026 speakers](https://www.writethedocs.org/conf/berlin/2026/speakers/)
- Docusaurus maintainers explicitly reject "unedited AI output" from non-maintainers — [Docusaurus AGENTS.md](https://github.com/facebook/docusaurus/blob/main/AGENTS.md)

### Inferences
- Vendors are handling variants and snippets for agents by vocabulary (skills that enumerate components) plus checkers that understand components (snippet links, anchors), not by new authoring syntax. No tool was found that offers an agent-specific way to author platform tabs or editions.
- The MDX critique in agent-reading writeups also applies to agent writing: unresolved imports and runtime-dependent components are what agents get wrong, which argues for a content model where reusable phrases/fragments are referenced by plain Markdown-compatible syntax that a checker can resolve.

### Gaps
- No docs-team writeup found that quantifies agent error rates on MDX components, tabs, or includes.
- No source describes how any tool helps agents author variants (platform tabs, product editions) specifically.

## Key question 7: Review of AI-written docs and connecting comments back to an agent

### Takeaway
Every hosted platform renders a docs PR as a preview (Mintlify preview deployments, ReadMe review branch preview, GitBook change requests), and GitBook and Fern let reviewers address the agent inside the review surface (`@gitbook` in comments, Slack thread replies to Fern); on GitHub, DocuGardener and CodeRabbit post inline comments or follow-up PRs, but no tool was found that automatically turns a human review comment on a docs PR into a new agent task.

### Cited Findings
- Mintlify preview deployments at `<subdomain>-<branch-name>.mintlify.site` for every PR when the GitHub app is installed; the editor shows PR status (Draft, Review required, Changes requested, Approved), diff view, and an Approve PR button; "if your branch has open comment threads, Mintlify adds a summary of them to the pull request description" — [Mintlify docs: Review changes](https://www.mintlify.com/docs/editor/review); [Mintlify docs: Preview deployments](https://www.mintlify.com/docs/deploy/preview-deployments)
- Mintlify automation PRs carry comments explaining changes and listing the source PRs used as context, and auto-assign source-code authors as reviewers — [Mintlify docs: workflows](https://www.mintlify.com/docs/agent/workflows)
- GitBook: agent participates via `@gitbook` mentions in comments and reviews change requests, flagging style issues and fixing errors — [GitBook docs](https://gitbook.com/docs/gitbook-agent/overview.md)
- Fern: "Slack users can request changes by replying in threads"; sessions continue in the Dashboard — [Fern docs: Fern Agent](https://buildwithfern.com/learn/docs/ai-features/fern-agent)
- ReadMe: drafts land on a review branch inside ReadMe with a PR comment and preview links so writers review "in the same place you already write docs" rather than reading Markdown diffs — [ReadMe blog](https://readme.com/blog/ai-writer)
- DocuGardener posts "inline PR comments with semantic diffs" and a check run — [docugardener](https://github.com/docugardener/docugardener)
- CodeRabbit: `@coderabbitai generate docstrings` scans with ast-grep, matches existing docstring style, then "either opens a follow-up PR with the generated docstrings or posts them back as a PR comment"; configurable docs review in YAML — [CodeRabbit docs: Docstrings](https://docs.coderabbit.ai/finishing-touches/docstrings)
- Redocly "Reviewer" checks clarity and cross-document consistency of API docs — [Redocly blog](https://redocly.com/blog/ai-features-summer-2026)
- leinardi's action posts markdownlint fixes as reviewdog suggested-change diffs that reviewers can apply with one click — [gha-pre-commit-markdownlint-cli2-reviewdog](https://github.com/leinardi/gha-pre-commit-markdownlint-cli2-reviewdog)

### Inferences
- The closest things to "comment → agent task" are platform-internal: `@gitbook` in a change-request comment, `@Fern` thread replies, and Mintlify's admin MCP plus comment-thread summaries in PR descriptions. On plain GitHub, the handoff is manual or via generic mechanisms (Copilot `assign-to-agent`, claude-code-action on `@claude` mentions), not docs-specific.
- Rendering the PR as a real preview and summarizing open comment threads into the PR body (Mintlify) is a low-cost pattern that also makes the comments available to any agent that reads the PR.

### Gaps
- No tool was found that packages review comments on a docs PR into an agent prompt automatically (the audience's planned "buttons that build prompts" has no direct precedent in the surveyed tools).
- GitBook's Git Sync interaction with the agent's change requests is undocumented in the sources fetched.
