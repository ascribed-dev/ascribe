# Zod schema check

`@ascribed/zod-check`, a private workspace package. It checks the Zod schemas the site output generates (`_ascribe/schema.ts`, written by `crates/ascribe-emit/src/zod/`) as a user's Astro site would use them:

- `pnpm --filter @ascribed/zod-check typecheck` compiles the generated schemas in `generated/` under the workspace's strict TypeScript settings.
- `pnpm --filter @ascribed/zod-check test` (`schema.test.ts`) validates the Quill project's frontmatter with them, and checks that they reject what the content model doesn't allow.

`generated/` is written by `crates/ascribe-emit/tests/all/zod.rs`, which fails when it's stale; rewrite it with `ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all zod`. `astro/zod` is aliased to `zod/v4`, which is what Astro 7.3 uses.
