// How requests reach GitHub's GraphQL API: through the GitHub CLI, or with a
// token the host supplies. Neither stores a token or puts one in an error.
import { spawn } from "node:child_process";
import { ReviewError, scrub } from "../shared/errors.js";

/** Sends one GraphQL request and resolves with its `data`. */
export interface GitHubTransport {
  graphql<T>(query: string, variables: Record<string, unknown>): Promise<T>;
}

/** The oldest `gh` this package runs. */
export const MIN_GH_VERSION = "2.0.0";

/** Options for `ghTransport`. */
export interface GhTransportOptions {
  /** The GitHub host: `github.com`, or a GitHub Enterprise Server host. */
  host?: string;
  /** The command to run. Defaults to `gh`, found on the path. */
  command?: string;
}

/**
 * A transport that runs `gh api graphql` with a fixed argument list: the
 * query on standard input, each variable as an argument. `gh` uses its own
 * sign-in for the host.
 */
export function ghTransport(options: GhTransportOptions = {}): GitHubTransport {
  const host = options.host ?? "github.com";
  const command = options.command ?? "gh";
  let checked: Promise<void> | undefined;
  return {
    async graphql<T>(query: string, variables: Record<string, unknown>): Promise<T> {
      checked ??= checkGh(command);
      try {
        await checked;
      } catch (error) {
        checked = undefined;
        throw error;
      }
      const args = ["api", "graphql", "--hostname", host, "-F", "query=@-"];
      for (const [name, value] of Object.entries(variables)) {
        if (value === undefined) continue;
        if (typeof value === "string") args.push("-f", `${name}=${value}`);
        else if (typeof value === "number" || typeof value === "boolean" || value === null) {
          args.push("-F", `${name}=${String(value)}`);
        } else {
          throw new TypeError(`ghTransport can't pass the variable ${name}: ${typeof value}`);
        }
      }
      const result = await run(command, args, query);
      const body = parseBody(result.stdout);
      if (result.code === 0 && body !== undefined && body.errors === undefined) {
        return body.data as T;
      }
      throw ghError(result, body, host);
    },
  };
}

async function checkGh(command: string): Promise<void> {
  const result = await run(command, ["--version"], "");
  const version = /gh version (\d+)\.(\d+)\.(\d+)/.exec(result.stdout);
  if (result.code !== 0 || version === null) {
    throw new ReviewError("gh-too-old", "Couldn't read the GitHub CLI's version (`gh --version`).");
  }
  const found = version.slice(1, 4).map(Number);
  const min = MIN_GH_VERSION.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const a = found[i] ?? 0;
    const b = min[i] ?? 0;
    if (a > b) return;
    if (a < b) {
      throw new ReviewError(
        "gh-too-old",
        `The GitHub CLI is version ${found.join(".")}; review needs ${MIN_GH_VERSION} or later. Update it from https://cli.github.com.`,
      );
    }
  }
}

interface RunResult {
  code: number;
  stdout: string;
  stderr: string;
}

function run(command: string, args: string[], input: string): Promise<RunResult> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: ["pipe", "pipe", "pipe"],
      env: { ...process.env, GH_PROMPT_DISABLED: "1", GH_NO_UPDATE_NOTIFIER: "1", NO_COLOR: "1" },
      windowsHide: true,
    });
    const stdout: Buffer[] = [];
    const stderr: Buffer[] = [];
    child.stdout.on("data", (chunk: Buffer) => stdout.push(chunk));
    child.stderr.on("data", (chunk: Buffer) => stderr.push(chunk));
    child.on("error", (error: NodeJS.ErrnoException) => {
      if (error.code === "ENOENT") {
        reject(
          new ReviewError(
            "gh-missing",
            "The GitHub CLI (`gh`) isn't on the path. Install it from https://cli.github.com and run `gh auth login`.",
            { cause: error },
          ),
        );
      } else {
        reject(
          new ReviewError("network", `Couldn't run \`gh\`: ${error.message}`, { cause: error }),
        );
      }
    });
    child.on("close", (code) => {
      resolve({
        code: code ?? 1,
        stdout: Buffer.concat(stdout).toString("utf8"),
        stderr: Buffer.concat(stderr).toString("utf8"),
      });
    });
    // gh may exit before reading its input; the exit code says what happened.
    child.stdin.on("error", () => undefined);
    child.stdin.end(input);
  });
}

interface GraphQLError {
  type?: string;
  message?: string;
}

interface GraphQLBody {
  data?: unknown;
  errors?: GraphQLError[];
  message?: string;
}

function parseBody(text: string): GraphQLBody | undefined {
  try {
    const value: unknown = JSON.parse(text);
    return typeof value === "object" && value !== null ? (value as GraphQLBody) : undefined;
  } catch {
    return undefined;
  }
}

function ghError(result: RunResult, body: GraphQLBody | undefined, host: string): ReviewError {
  const stderr = result.stderr.replace(/^gh: /gm, "").trim();
  // gh exits with 4 when a command needs authentication it doesn't have.
  if (
    result.code === 4 ||
    /gh auth login|not logged in|HTTP 401|Bad credentials/i.test(result.stderr)
  ) {
    return new ReviewError(
      "not-signed-in",
      `The GitHub CLI isn't signed in to ${host}. Run \`gh auth login --hostname ${host}\`.`,
    );
  }
  if (body?.errors !== undefined) return graphqlError(body.errors);
  if (/rate limit/i.test(stderr)) return rateLimited(undefined, stderr);
  return new ReviewError(
    "refused",
    `GitHub refused the request: ${stderr || body?.message || `gh exited with ${result.code}`}`,
  );
}

function graphqlError(errors: GraphQLError[]): ReviewError {
  const message = errors.map((error) => error.message ?? "unknown error").join("; ");
  if (errors.some((error) => error.type === "RATE_LIMITED")) return rateLimited(undefined, message);
  if (errors.every((error) => error.type === "NOT_FOUND")) {
    return new ReviewError("not-found", `GitHub couldn't find it: ${message}`);
  }
  return new ReviewError("refused", `GitHub refused the request: ${message}`);
}

function rateLimited(retryAfter: number | undefined, detail: string): ReviewError {
  const wait =
    retryAfter === undefined
      ? "Wait a minute"
      : `Wait ${retryAfter} second${retryAfter === 1 ? "" : "s"}`;
  return new ReviewError(
    "rate-limited",
    `GitHub is limiting how fast requests can be made. ${wait} and try again. (${detail})`,
    retryAfter === undefined ? {} : { retryAfter },
  );
}

/** Options for `tokenTransport`. */
export interface TokenTransportOptions {
  /** The GitHub host: `github.com`, or a GitHub Enterprise Server host. */
  host?: string;
  /** The `fetch` to use. Defaults to the global one. */
  fetch?: typeof fetch;
}

/**
 * A transport that posts to the GraphQL API with `fetch`, using a token from
 * `getToken`, which is called for each request so the host can refresh it.
 */
export function tokenTransport(
  getToken: () => Promise<string | undefined> | string | undefined,
  options: TokenTransportOptions = {},
): GitHubTransport {
  const host = options.host ?? "github.com";
  const url = graphqlUrl(host);
  return {
    async graphql<T>(query: string, variables: Record<string, unknown>): Promise<T> {
      const doFetch = options.fetch ?? globalThis.fetch;
      const token = await getToken();
      if (token === undefined || token === "") {
        throw new ReviewError("not-signed-in", `There's no GitHub sign-in for ${host}.`);
      }
      let response: Response;
      try {
        response = await doFetch(url, {
          method: "POST",
          headers: {
            authorization: `bearer ${token}`,
            "content-type": "application/json",
            accept: "application/json",
            "user-agent": "ascribe-review",
          },
          body: JSON.stringify({ query, variables }),
        });
      } catch (error) {
        const detail = error instanceof Error ? error.message : String(error);
        throw new ReviewError("network", `Couldn't reach ${host}: ${detail}`, { cause: error });
      }
      const text = await response.text();
      const body = parseBody(text);
      if (response.status === 401) {
        throw new ReviewError("not-signed-in", `${host} didn't accept the GitHub sign-in.`);
      }
      if (response.status === 403 || response.status === 429) {
        const retryAfter = retryAfterOf(response.headers);
        if (
          response.status === 429 ||
          retryAfter !== undefined ||
          /rate limit/i.test(body?.message ?? text)
        ) {
          throw rateLimited(retryAfter, scrub(body?.message ?? `HTTP ${response.status}`));
        }
      }
      if (!response.ok) {
        throw new ReviewError(
          "refused",
          `GitHub refused the request (HTTP ${response.status}): ${scrub(body?.message ?? text.slice(0, 200))}`,
        );
      }
      if (body === undefined) {
        throw new ReviewError("network", `${host} answered with something that isn't JSON.`);
      }
      if (body.errors !== undefined && body.errors.length > 0) throw graphqlError(body.errors);
      return body.data as T;
    },
  };
}

/** The GraphQL endpoint for a host. */
export function graphqlUrl(host: string): string {
  return host === "github.com" ? "https://api.github.com/graphql" : `https://${host}/api/graphql`;
}

function retryAfterOf(headers: Headers): number | undefined {
  const after = headers.get("retry-after");
  if (after !== null && /^\d+$/.test(after)) return Number(after);
  if (headers.get("x-ratelimit-remaining") === "0") {
    const reset = Number(headers.get("x-ratelimit-reset"));
    if (Number.isFinite(reset) && reset > 0) {
      return Math.max(0, Math.ceil(reset - Date.now() / 1000));
    }
  }
  return undefined;
}
