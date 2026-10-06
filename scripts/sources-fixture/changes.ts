// The sources fixture's code repository, and the changes that can be made to
// it: an example changed, a change no page shows, a region renamed, a file
// moved, and everything put back. scripts/sources-fixture/setup.ts builds the
// repository from CODE and copies this file into it, as `.github/change.ts`,
// where a scheduled workflow runs it:
//
//   node change.ts <example|unrelated|rename|move|restore|next> [--dir <checkout>]
//
// It changes the checkout (the current directory by default) and commits, but
// doesn't push. `next` makes the change after the last one, in CYCLE's order.
// It needs only Node and `git`, so the code repository runs it as it is.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import process from "node:process";
import { pathToFileURL } from "node:url";
import { parseArgs } from "node:util";

/** The code repository as it's built: a made-up client library for Lantern. */
export const CODE: Record<string, string> = {
  "src/client.ts": `// The Lantern client: one for each process.
import { login, type Session } from "./auth.ts";
import { withRetry } from "./retry.ts";

export interface Options {
  host: string;
  token: string;
}

export class Client {
  #options: Options;
  #session: Session | undefined;

  constructor(options: Options) {
    this.#options = options;
  }

  // :snippet-start: connect
  async connect(): Promise<void> {
    const { host, token } = this.#options;
    this.#session = await withRetry(() => login(host, token));
  }
  // :snippet-end:

  enabled(flag: string): boolean {
    return this.#session?.flags.includes(flag) ?? false;
  }
}
`,
  "src/auth.ts": `export interface Session {
  flags: string[];
}

// :snippet-start: login
export async function login(host: string, token: string): Promise<Session> {
  const response = await fetch(\`https://\${host}/v1/session\`, {
    headers: { authorization: \`Bearer \${token}\` },
  });
  return (await response.json()) as Session;
}
// :snippet-end:
`,
  "src/retry.ts": `// Calls \`call\` again when it fails, up to 3 times in all.
export async function withRetry<T>(call: () => Promise<T>, attempts = 3): Promise<T> {
  for (let attempt = 1; ; attempt++) {
    try {
      return await call();
    } catch (error) {
      if (attempt >= attempts) throw error;
    }
  }
}
`,
  "examples/quickstart.ts": `import { Client } from "lantern-sdk";

const client = new Client({ host: "flags.example.com", token: process.env.LANTERN_TOKEN ?? "" });
await client.connect();
console.log(client.enabled("new-checkout"));
`,
};

export type Change = "example" | "unrelated" | "rename" | "move" | "restore";

/** What each change is, as its commit's subject says. */
export const SUBJECTS: Record<Change, string> = {
  example: "Change the login example",
  unrelated: "Change the retry count",
  rename: "Rename the connect region",
  move: "Move the quickstart",
  restore: "Put the code back as it was built",
};

/** The order `next` follows: each break is mended before the next, so the docs can catch up. */
export const CYCLE: Change[] = ["example", "unrelated", "rename", "restore", "move", "restore"];

/** Every commit this file makes starts with this. */
export const PREFIX = "Scripted change: ";

const MOVED = "examples/start/quickstart.ts";

/** Makes a change in the checkout at `dir` and commits it; returns the change. */
export function makeChange(dir: string, change: Change | "next"): Change {
  const made = change === "next" ? nextChange(dir) : change;
  const file = (path: string) => join(dir, ...path.split("/"));
  const read = (path: string) => readFileSync(file(path), "utf8").replace(/\r\n/g, "\n");
  const write = (path: string, text: string) => {
    mkdirSync(dirname(file(path)), { recursive: true });
    writeFileSync(file(path), text);
  };
  /** Swaps `a` for `b` in a file, or back again: each change can be made twice in a row. */
  const toggle = (path: string, a: string, b: string) => {
    const text = read(path);
    write(path, text.includes(a) ? text.replace(a, b) : text.replace(b, a));
  };
  switch (made) {
    case "example":
      toggle("src/auth.ts", "/v1/session", "/v2/session");
      break;
    case "unrelated":
      toggle("src/retry.ts", "up to 3 times in all.\n", "up to 4 times in all.\n");
      toggle("src/retry.ts", "attempts = 3", "attempts = 4");
      break;
    case "rename":
      toggle("src/client.ts", ":snippet-start: connect\n", ":snippet-start: open\n");
      break;
    case "move": {
      const [from, to] = existsSync(file(MOVED))
        ? [MOVED, "examples/quickstart.ts"]
        : ["examples/quickstart.ts", MOVED];
      write(to, read(from));
      rmSync(file(from));
      break;
    }
    case "restore":
      rmSync(file("src"), { recursive: true, force: true });
      rmSync(file("examples"), { recursive: true, force: true });
      for (const [path, text] of Object.entries(CODE)) write(path, text);
      break;
  }
  git(dir, "add", "-A", "src", "examples");
  git(dir, "commit", "-q", "--allow-empty", "-m", `${PREFIX}${SUBJECTS[made]}`);
  return made;
}

/** The change after the last one this file made, in CYCLE's order. */
function nextChange(dir: string): Change {
  const made = git(dir, "log", "--format=%s", `--grep=^${PREFIX}`)
    .split("\n")
    .filter((line) => line.startsWith(PREFIX)).length;
  return CYCLE[made % CYCLE.length] ?? "example";
}

/** Runs `git` in `dir`, as a fixed author, without signing or line-ending conversion. */
export function git(dir: string, ...args: string[]): string {
  return execFileSync(
    "git",
    [
      "-c",
      "user.name=Sources Fixture",
      "-c",
      "user.email=fixture@ascribe.invalid",
      "-c",
      "commit.gpgsign=false",
      "-c",
      "core.autocrlf=false",
      ...args,
    ],
    { cwd: dir, encoding: "utf8" },
  ).trim();
}

if (process.argv[1] !== undefined && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { values, positionals } = parseArgs({
    allowPositionals: true,
    options: { dir: { type: "string" } },
  });
  const [change] = positionals;
  const kinds = [...Object.keys(SUBJECTS), "next"];
  if (change === undefined || !kinds.includes(change)) {
    console.error(`usage: change.ts <${kinds.join("|")}> [--dir <checkout>]`);
    process.exit(1);
  }
  const made = makeChange(values.dir ?? process.cwd(), change as Change | "next");
  console.log(`${PREFIX}${SUBJECTS[made]}`);
}
