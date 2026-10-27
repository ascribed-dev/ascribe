// Every generated file, rewritten in one command: `pnpm bless`.
//
// The rewrites are the rows of the tables under ARCHITECTURE.md's
// `ASCRIBE_BLESS=1`, each named as its row names it, so scripts/bless.test.ts
// fails when the two drift apart. A test's command is the one the test says
// rewrites it, which bless.test.ts compares too. A row that doesn't run here
// says why. They run in an order where each sees what the earlier ones wrote:
// the names, the JSON's types, and the stylesheets before what's built from
// them, the docs' fragments before the surface map, and the command output the
// docs show before `ascribe observe`. First it builds what the rewrites run:
// `@ascribed/cli`, which the Astro starter's test imports, and the debug
// binary, which the templates' conversion runs. It stops at the first
// failure, and ends with `git diff --stat`: read the diff before committing it.
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

export interface Rewrite {
  /** The test or command, as its row's first cell names it. */
  writer: string;
  /** What runs, from the root: leading `NAME=value` words are its environment. */
  run?: string;
  /** Why it doesn't run here. */
  skip?: string;
}

/** What the rewrites need built, run before them. */
const prepare: string[] = [
  "pnpm --filter @ascribed/cli build",
  "cargo build -p ascribe-cli --locked",
];

export const rewrites: Rewrite[] = [
  {
    writer: "crates/ascribe-core/tests/names.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-core --test names",
  },
  {
    writer: "crates/ascribe-cli/src/shapes.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli shapes",
  },
  {
    writer: "scripts/design/tokens.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/tokens.test.ts",
  },
  // After the names, the JSON's types, and the stylesheets it bundles.
  { writer: "pnpm --filter @ascribed/review embed", run: "pnpm --filter @ascribed/review embed" },
  {
    writer: "tests/conformance/tests/docs.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-conformance --test docs",
  },
  {
    writer: "crates/ascribe-cli/src/docs.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli docs",
  },
  {
    writer: "packages/vscode/test/unit/docs.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm --filter ascribe-vscode exec vitest run test/unit/docs.test.ts",
  },
  {
    writer: "scripts/docs-site/elements.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/docs-site/elements.test.ts",
  },
  {
    writer: "crates/ascribe-cli/src/agents/skill.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli skill",
  },
  {
    writer: "crates/ascribe-cli/src/agents/plugin.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli plugin",
  },
  // Before the command output, which shows the set the binary compiles in. It
  // skips, saying so, without The Good Docs Project's repository.
  {
    writer: "scripts/templates/convert.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/templates/convert.test.ts",
  },
  {
    writer: "crates/ascribe-cli/tests/all/output.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test all output::",
  },
  {
    writer: "crates/ascribe-cli/tests/all/agents.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-cli --test all agents::",
  },
  {
    writer: "crates/ascribe-emit/tests/all/site_anchors.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all site_anchors::",
  },
  {
    writer: "crates/ascribe-emit/tests/all/site_sidebar.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all site_sidebar",
  },
  {
    writer: "crates/ascribe-emit/tests/all/zod.rs",
    run: "ASCRIBE_BLESS=1 cargo test -p ascribe-emit --test all zod",
  },
  {
    writer: "packages/astro/test/init.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm --filter @ascribed/astro exec vitest run test/init.test.ts",
  },
  {
    writer: "scripts/design/specimen.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/design/specimen.test.ts",
  },
  { writer: "node scripts/design/assets.ts", run: "node scripts/design/assets.ts" },
  // After every docs fragment, since it reads them.
  {
    writer: "scripts/surface/inventory.test.ts",
    run: "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/surface/inventory.test.ts",
  },
  // After the command output the docs show. At a terminal it lists the
  // commands it would run and asks first.
  { writer: "ascribe observe", run: "cargo run -p ascribe-cli -- observe" },
  {
    writer: "cargo insta review",
    skip: "it asks about each snapshot; run it yourself, and never accept one blindly",
  },
  {
    writer: "ASCRIBE_CORPORA_BLESS=1",
    skip: "it needs the corpora downloaded, and rewrites their counts only after you've reviewed a change (tests/corpora/README.md)",
  },
];

/** A rewrite's command and its environment, from its `run`. */
export function command(run: string): { env: Record<string, string>; argv: string[] } {
  const words = run.split(" ");
  const env: Record<string, string> = {};
  while (/^[A-Z_]+=/.test(words[0] ?? "")) {
    const [name = "", ...value] = (words.shift() ?? "").split("=");
    env[name] = value.join("=");
  }
  return { env, argv: words };
}

function main(): void {
  const runs: Rewrite[] = [...prepare.map((run) => ({ writer: run, run })), ...rewrites];
  for (const rewrite of runs) {
    if (rewrite.run === undefined) {
      process.stdout.write(`\n# skipped: ${rewrite.writer}: ${rewrite.skip ?? ""}\n`);
      continue;
    }
    process.stdout.write(`\n$ ${rewrite.run}\n`);
    const { env, argv } = command(rewrite.run);
    const [program = "", ...args] = argv;
    // pnpm is a .cmd script on Windows, which only a shell can start.
    const result = spawnSync(program, args, {
      cwd: root,
      env: { ...process.env, ...env },
      stdio: "inherit",
      shell: process.platform === "win32",
    });
    if (result.status !== 0) {
      process.stderr.write(`\nFailed: ${rewrite.run}. The rewrites after it didn't run.\n`);
      process.exit(result.status ?? 1);
    }
  }
  process.stdout.write("\n$ git diff --stat\n");
  spawnSync("git", ["diff", "--stat"], { cwd: root, stdio: "inherit" });
  process.stdout.write("\nRead the diff before committing it.\n");
}

if (import.meta.main) main();
