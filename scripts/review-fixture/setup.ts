// Builds the review fixture: a small repository with one open pull request
// whose changes and review threads cover the cases review has to handle, for
// trying review by hand (project-docs/review/phase-8-docs.md).
//
//   node scripts/review-fixture/setup.ts <owner>/<name> [--dir <dir>] [--force]
//   node scripts/review-fixture/setup.ts --local [--dir <dir>]
//
// The repository must exist on GitHub already, empty or built by this script
// before. Each run rebuilds it from nothing: it closes the fixture's open pull
// request, force-pushes both branches, opens a new pull request, and seeds its
// threads. So the comments a pass leaves behind are gone on the next run. It
// refuses a repository whose README it didn't write, unless --force.
//
// --local makes only the checkout, with a bare repository beside it as its
// remote, and doesn't touch GitHub: `ascribe diff` and the page preview's
// changes work there, without threads.
//
// It needs `git`, and for GitHub the GitHub CLI (`gh`), signed in with access
// to the repository, and a built @ascribed/review
// (`pnpm --filter @ascribed/review build`), which writes the comments GitHub
// can't anchor exactly as review does.
//
// The checkout stays in --dir (a new temporary directory by default), ready
// to open: the script prints how.
import { execFileSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import process from "node:process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

const root = fileURLToPath(new URL("../..", import.meta.url));
const BRANCH = "scheduled-rollouts";
/** The first line of the fixture's README: how a later run knows the repository is its own. */
const MARK = "<!-- ascribe review fixture: rebuilt by scripts/review-fixture/setup.ts -->";

const { values: options, positionals } = parseArgs({
  allowPositionals: true,
  options: {
    dir: { type: "string" },
    local: { type: "boolean", default: false },
    force: { type: "boolean", default: false },
  },
});
const [repository] = positionals;
if (options.local ? repository !== undefined : !/^[\w.-]+\/[\w.-]+$/.test(repository ?? "")) {
  fail(
    "usage: setup.ts <owner>/<name> [--dir <dir>] [--force]\n       setup.ts --local [--dir <dir>]",
  );
}

const work = options.dir ?? mkdtempSync(join(tmpdir(), "ascribe-review-fixture-"));
const checkout = join(work, "fixture");
const docs = join(checkout, "docs");
if (existsSync(checkout)) fail(`${checkout} exists already; choose another --dir, or remove it.`);

if (repository !== undefined) checkRepository(repository);

// --- The base: the Lantern docs, and an Astro site around them ---

mkdirSync(checkout, { recursive: true });
git("init", "-q", "-b", "main");
cpSync(join(root, "examples", "monorepo", "docs"), docs, { recursive: true });
cpSync(join(root, "examples", "astro-site", "src"), join(docs, "src"), { recursive: true });
write(
  "README.md",
  [
    MARK,
    "",
    "# Ascribe review fixture",
    "",
    "A made-up docs project (Lantern, from Ascribe's `examples/monorepo`) with one open pull request, for trying Ascribe's review by hand. Don't edit it: `scripts/review-fixture/setup.ts` in the Ascribe repository rebuilds it from nothing, comments included.",
    "",
  ].join("\n"),
);
write(
  "docs/astro.config.mjs",
  [
    'import { defineConfig } from "astro/config";',
    'import ascribe from "@ascribed/astro";',
    "",
    "// `site` repeats ascribe.toml's [consumer]. The `ascribe` binary comes from",
    "// ASCRIBE_BIN, or from @ascribed/cli.",
    "export default defineConfig({",
    '  site: "https://docs.lantern.example",',
    '  integrations: [ascribe({ build: "site" })],',
    "});",
    "",
  ].join("\n"),
);
const example = readJson(join(root, "examples", "astro-site", "package.json"));
const version = String(readJson(join(root, "packages", "astro", "package.json"))["version"]);
write(
  "docs/package.json",
  `${JSON.stringify(
    {
      name: "ascribe-review-fixture",
      private: true,
      type: "module",
      scripts: { dev: "astro dev", build: "astro build" },
      dependencies: {
        "@ascribed/astro": version,
        "@ascribed/cli": version,
        "@ascribed/elements": version,
        astro: dependency(example, "astro"),
        sharp: dependency(example, "sharp"),
      },
    },
    null,
    2,
  )}\n`,
);
write("docs/tsconfig.json", '{ "extends": "astro/tsconfigs/base" }\n');
write("docs/.gitignore", ".ascribe/\n.astro/\ndist/\nnode_modules\n");
// The fragment on a second page, so a change to it reaches two.
edit("docs/content/guides/create-flags.md", (text) =>
  replace(
    text,
    "\n## Flag types\n",
    "\n@include: ../_fragments/prerequisites.md\n\n## Flag types\n",
  ),
);
commit("Lantern docs, from Ascribe's examples/monorepo");
check("the base");

// --- The pull request's first commit ---

git("checkout", "-q", "-b", BRANCH);
const ROLLOUTS = "docs/content/guides/rollouts.md";
const PREREQUISITES = "docs/content/_fragments/prerequisites.md";
const OPENING =
  "A rollout turns a flag on for more users over time, so a problem reaches a few users before it reaches all of them.";
const ADDED = "Start at 1% for a change that touches payments.";
const NOTE =
  "Segments match on the attributes you pass to `lantern.enabled()`. An attribute your app doesn't send never matches.";
const RESOLVED = "Does everything need Node.js 22, or only the scheduler? (Resolved.)";
/** Where the note was, in the base: a thread goes on it. */
const noteLine = lineOf(ROLLOUTS, NOTE);
edit(ROLLOUTS, (text) => {
  // A page's own file: a paragraph reworded, one added, and a note removed.
  let next = replace(
    text,
    "a few users instead of all of them.",
    "a few users before it reaches all of them.",
  );
  next = replace(
    next,
    "A common plan is 1%, 10%, 50%, then 100%.\n",
    `A common plan is 1%, 10%, 50%, then 100%.\n\n${ADDED}\n`,
  );
  return replace(next, `@note\n${NOTE}\n\n`, "");
});
// A fragment two pages include.
edit(
  PREREQUISITES,
  (text) =>
    `${replace(text, "Node.js 20", "Node.js 22").trimEnd()}\n- The **Release manager** role.\n`,
);
// A phrase: pages change whose files don't.
edit("docs/ascribe.toml", (text) => replace(text, 'version = "2.5.0"', 'version = "2.6.0"'));
// A variant arm.
edit("docs/content/getting-started.md", (text) =>
  replace(text, "https://get.lantern.example | sh", "https://get.lantern.example/install.sh | sh"),
);
commit("Document rollouts for 2.6");

/** The threads made on the first commit, before the second moves and rewords lines under them. */
const threads = [
  {
    path: ROLLOUTS,
    line: lineOf(ROLLOUTS, OPENING),
    side: "RIGHT",
    body: 'Is "before it reaches" clearer than "instead of"? I think so.',
  },
  {
    path: ROLLOUTS,
    line: lineOf(ROLLOUTS, OPENING),
    side: "RIGHT",
    body: "Could this say what kind of problem? A second thread on the same block.",
  },
  {
    path: ROLLOUTS,
    line: lineOf(ROLLOUTS, ADDED),
    side: "RIGHT",
    body: "Good addition. (The next commit rewords this line, so this thread is outdated.)",
  },
  {
    path: ROLLOUTS,
    line: noteLine,
    side: "LEFT",
    body: "Why remove this note? It still looks true. (A thread on removed text.)",
  },
  {
    path: PREREQUISITES,
    start_line: 3,
    start_side: "RIGHT",
    line: 5,
    side: "RIGHT",
    body: "A thread on three lines of a fragment that two pages include.",
  },
  { path: PREREQUISITES, line: 3, side: "RIGHT", body: RESOLVED },
];

let pullRequest: { number: number; url: string } | undefined;
if (repository !== undefined) {
  git("remote", "add", "origin", `https://github.com/${repository}.git`);
  for (const open of ghJson<{ number: number }[]>(
    "pr",
    "list",
    "--repo",
    repository,
    "--head",
    BRANCH,
    "--state",
    "open",
    "--json",
    "number",
  )) {
    gh(
      "pr",
      "close",
      String(open.number),
      "--repo",
      repository,
      "--comment",
      "Closed by the fixture's setup script, which opens a new one.",
    );
  }
  git("push", "-q", "--force", "origin", "main", BRANCH);
  const url = gh(
    "pr",
    "create",
    "--repo",
    repository,
    "--base",
    "main",
    "--head",
    BRANCH,
    "--title",
    "Document rollouts for 2.6",
    "--body",
    "The review fixture's pull request. Its commits and its first comments are made by `scripts/review-fixture/setup.ts`.",
  ).trim();
  pullRequest = { number: Number(url.slice(url.lastIndexOf("/") + 1)), url };
  // One review holds every thread.
  gh(
    "api",
    `repos/${repository}/pulls/${pullRequest.number}/reviews`,
    "--method",
    "POST",
    "--input",
    "-",
    {
      input: JSON.stringify({
        commit_id: git("rev-parse", "HEAD"),
        event: "COMMENT",
        body: "",
        comments: threads,
      }),
    },
  );
  resolve(repository, pullRequest.number);
} else {
  const remote = join(work, "remote.git");
  execFileSync("git", ["init", "-q", "--bare", remote]);
  git("remote", "add", "origin", remote);
  git("push", "-q", "origin", "main", BRANCH);
}

// --- The second commit: lines move, and a commented line is reworded ---

edit(ROLLOUTS, (text) => {
  const next = replace(
    text,
    `${OPENING}\n`,
    `Rollouts are how most teams ship risky changes.\n\n${OPENING}\n`,
  );
  return replace(
    next,
    ADDED,
    "Start at 1% or lower for a change that touches payments or sign-in.",
  );
});
commit("Add an opening line, and widen the advice");
check("the pull request's head");
git("push", "-q", "origin", BRANCH);
git("branch", "-q", `--set-upstream-to=origin/${BRANCH}`);

// --- The comments GitHub can't anchor, made as review makes them ---

if (repository !== undefined) await summaryComments();

// The example site's dependencies, so `astro dev` runs here against this checkout's packages.
const modules = join(root, "examples", "astro-site", "node_modules");
if (existsSync(modules)) symlinkSync(modules, join(docs, "node_modules"), "junction");

console.log(`
The fixture is ready.

  checkout      ${checkout}  (branch ${BRANCH}, at the pull request's head)
  pull request  ${pullRequest?.url ?? "none (--local): changes only"}

What review should show:
  guides/rollouts.md      changed: a paragraph added by the second commit, one reworded,
                          one added, a note removed${
                            pullRequest
                              ? `
                          two threads on the reworded paragraph, an outdated thread on
                          "Start at 1% or lower…", a thread on the removed note, and a
                          comment in the review summary on the last paragraph`
                              : ""
                          }
  getting-started.md      changed: the Linux install command, the fragment, and {version}${
    pullRequest
      ? `
                          the fragment's two threads (one resolved)`
      : ""
  }
  guides/create-flags.md  changed only through _fragments/prerequisites.md${
    pullRequest
      ? `
                          the fragment's two threads again`
      : ""
  }${
    pullRequest
      ? `
  guides/self-hosting.md  its file isn't in the pull request, with a comment in the review summary`
      : ""
  }
  other pages             any that write {version} change only through ascribe.toml

To try it:
  page preview   open ${checkout} in VS Code, open a page, Ascribe: Start Review
  site preview   cd ${docs}
                 ASCRIBE_BIN=${join(root, "target", "debug", "ascribe")} ./node_modules/.bin/astro dev${
                   existsSync(modules)
                     ? ""
                     : `
                 (first: pnpm install in ${root}, then link ${modules} as ${join(docs, "node_modules")})`
                 }
  the report     cd ${docs} && ascribe diff --format html > review.html
`);

/** Refuses a repository this script didn't build, so a typo can't force-push over real work. */
function checkRepository(name: string): void {
  let readme: string;
  try {
    readme = gh("api", `repos/${name}/readme`, "--jq", ".content");
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    // An empty repository, or one with no README yet.
    if (/empty|404|Not Found/i.test(message)) {
      try {
        gh("api", `repos/${name}`, "--jq", ".full_name");
      } catch {
        fail(
          `${name} doesn't exist on GitHub, or the GitHub CLI can't see it. Create it first (private), then run this again.`,
        );
      }
      if (/empty/i.test(message) || options.force) return;
      fail(
        `${name} has files but no README from this script. Use an empty repository, or --force to overwrite it.`,
      );
    }
    throw error;
  }
  const text = Buffer.from(readme, "base64").toString("utf8");
  if (!text.startsWith(MARK) && !options.force) {
    fail(
      `${name} isn't a review fixture (its README isn't this script's). Use an empty repository, or --force to overwrite it.`,
    );
  }
}

/** Resolves the thread meant to be resolved. */
function resolve(name: string, number: number): void {
  const [owner, repo] = name.split("/");
  const found = ghJson<{
    data: {
      repository: {
        pullRequest: {
          reviewThreads: { nodes: { id: string; comments: { nodes: { body: string }[] } }[] };
        };
      };
    };
  }>(
    "api",
    "graphql",
    "-f",
    "query=query($owner: String!, $repo: String!, $number: Int!) { repository(owner: $owner, name: $repo) { pullRequest(number: $number) { reviewThreads(first: 50) { nodes { id comments(first: 1) { nodes { body } } } } } } }",
    "-f",
    `owner=${owner}`,
    "-f",
    `repo=${repo}`,
    "-F",
    `number=${number}`,
  );
  const thread = found.data.repository.pullRequest.reviewThreads.nodes.find(
    (t) => t.comments.nodes[0]?.body === RESOLVED,
  );
  if (!thread) fail("The thread to resolve wasn't made.");
  gh(
    "api",
    "graphql",
    "-f",
    "query=mutation($id: ID!) { resolveReviewThread(input: { threadId: $id }) { thread { id } } }",
    "-f",
    `id=${thread.id}`,
  );
}

/** Two comments held in a review's summary: on a line away from the diff, and on a page the pull request doesn't touch. */
async function summaryComments(): Promise<void> {
  const built = join(root, "packages", "review", "dist", "github", "index.js");
  if (!existsSync(built)) {
    fail(
      "@ascribed/review isn't built, so the summary comments weren't made. Run `pnpm --filter @ascribed/review build`, then this script again.",
    );
  }
  interface Anchor {
    source: string;
    via: string[];
  }
  interface Session {
    comment(
      anchor: Anchor,
      body: string,
      page: { build: string; path: string; anchors: Anchor[]; removed: Anchor[] },
    ): Promise<{ kind: string }>;
    submit(event: "COMMENT"): Promise<void>;
  }
  const review = (await import(pathToFileURL(built).href)) as {
    openReview(options: { projectDir: string }): Promise<Session | undefined>;
  };
  const session = await review.openReview({ projectDir: docs });
  if (!session) fail("The pull request wasn't found from the checkout.");
  const held: [string, string, string][] = [
    [
      "guides/rollouts.md",
      "The change reaches every SDK within a few seconds.",
      "Is \"a few seconds\" still right for 2.6? (On a line GitHub can't anchor: it's away from the changes.)",
    ],
    [
      "guides/self-hosting.md",
      firstParagraph("docs/content/guides/self-hosting.md"),
      "Should this page mention the Release manager role too? (On a page the pull request doesn't touch.)",
    ],
  ];
  for (const [page, text, body] of held) {
    const line = lineOf(`docs/content/${page}`, text);
    const anchor = { source: `${page}:${line}-${line}`, via: [] };
    const made = await session.comment(anchor, body, {
      build: "site",
      path: page,
      anchors: [anchor],
      removed: [],
    });
    if (made.kind !== "conversation")
      fail(
        `The comment on ${page}:${line} was anchored by GitHub; the fixture expects it in the summary.`,
      );
  }
  await session.submit("COMMENT");
}

/** The first paragraph after a page's frontmatter. */
function firstParagraph(file: string): string {
  const lines = read(file).split("\n");
  const end = lines.indexOf("---", 1);
  const found = lines
    .slice(end + 1)
    .find((line) => line.trim() !== "" && !line.startsWith("@") && !line.startsWith("#"));
  if (found === undefined) fail(`${file} has no paragraph.`);
  return found;
}

/** The line (from 1) that starts with `text`, in the checkout's file as it is now. */
function lineOf(file: string, text: string): number {
  const index = read(file)
    .split("\n")
    .findIndex((line) => line.startsWith(text));
  if (index < 0)
    fail(
      `${file} has no line starting "${text}": examples/monorepo changed, and this script needs updating.`,
    );
  return index + 1;
}

function replace(text: string, from: string, to: string): string {
  if (!text.includes(from))
    fail(
      `Expected to find "${from.trim()}": examples/monorepo changed, and this script needs updating.`,
    );
  return text.replace(from, to);
}

function read(file: string): string {
  return readFileSync(join(checkout, file), "utf8").replace(/\r\n/g, "\n");
}

function write(file: string, text: string): void {
  writeFileSync(join(checkout, file), text);
}

function edit(file: string, change: (text: string) => string): void {
  write(file, change(read(file)));
}

function readJson(file: string): Record<string, unknown> {
  return JSON.parse(readFileSync(file, "utf8")) as Record<string, unknown>;
}

function dependency(manifest: Record<string, unknown>, name: string): string {
  return String((manifest["dependencies"] as Record<string, unknown>)[name]);
}

function commit(message: string): void {
  git("add", "-A");
  git("commit", "-q", "-m", message);
}

/** Checks the project as it is now, when there's an `ascribe` to check it with: the fixture must have no problems of its own. */
function check(what: string): void {
  const built = join(
    root,
    "target",
    "debug",
    process.platform === "win32" ? "ascribe.exe" : "ascribe",
  );
  const binary = process.env["ASCRIBE_BIN"] ?? (existsSync(built) ? built : undefined);
  if (binary === undefined) return;
  try {
    execFileSync(binary, ["check", "--deny-warnings"], {
      cwd: docs,
      encoding: "utf8",
      stdio: "pipe",
    });
  } catch (error) {
    fail(
      `${what} doesn't pass \`ascribe check\`: examples/monorepo changed, and this script needs updating.\n${(error as { stdout?: string }).stdout ?? ""}`,
    );
  }
}

/** Runs `git` in the checkout, as a fixed author, without signing or line-ending conversion. */
function git(...args: string[]): string {
  return execFileSync(
    "git",
    [
      "-c",
      "user.name=Review Fixture",
      "-c",
      "user.email=fixture@ascribe.invalid",
      "-c",
      "commit.gpgsign=false",
      "-c",
      "core.autocrlf=false",
      ...args,
    ],
    { cwd: checkout, encoding: "utf8" },
  ).trim();
}

/** Runs the GitHub CLI; the last argument may give its standard input. */
function gh(...args: (string | { input: string })[]): string {
  const last = args.at(-1);
  const input = typeof last === "object" ? last.input : undefined;
  const list = args.filter((arg) => typeof arg === "string");
  try {
    return execFileSync("gh", list, { encoding: "utf8", input, stdio: ["pipe", "pipe", "pipe"] });
  } catch (error) {
    const stderr = (error as { stderr?: string }).stderr ?? "";
    throw new Error(`gh ${list.slice(0, 3).join(" ")} failed: ${stderr.trim() || String(error)}`, {
      cause: error,
    });
  }
}

function ghJson<T>(...args: string[]): T {
  return JSON.parse(gh(...args)) as T;
}

function fail(message: string): never {
  console.error(message);
  process.exit(1);
}
