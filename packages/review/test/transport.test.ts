import { afterEach, beforeEach, describe, expect, test } from "vitest";
import { ghTransport, graphqlUrl, tokenTransport } from "../src/github/transport.js";
import { ReviewError } from "../src/shared/errors.js";
import { fakeGh, type FakeGh, type GhAnswer } from "./helpers/fake-gh.js";

const VERSION: GhAnswer = { match: "--version", stdout: "gh version 2.89.0 (2026-03-26)\n" };
const QUERY = "query Viewer($n: Int!) { viewer { login } }";

describe.skipIf(process.platform === "win32")("the gh transport", () => {
  let gh: FakeGh | undefined;
  let savedPath: string | undefined;
  beforeEach(() => {
    savedPath = process.env["PATH"];
  });
  afterEach(() => {
    process.env["PATH"] = savedPath;
    gh?.remove();
    gh = undefined;
  });
  const install = (answers: GhAnswer[]) => {
    gh = fakeGh(answers);
    process.env["PATH"] = gh.path;
    return gh;
  };

  test("sends the query on standard input and variables as typed arguments", async () => {
    const fake = install([
      VERSION,
      { match: "api graphql", stdout: JSON.stringify({ data: { viewer: { login: "me" } } }) },
    ]);
    const data = await ghTransport({ host: "ghe.example.com" }).graphql(QUERY, {
      n: 3,
      body: "@not-a-file 42",
      flag: true,
      none: null,
      skipped: undefined,
    });
    expect(data).toEqual({ viewer: { login: "me" } });
    const call = fake.calls()[1];
    expect(call?.args).toEqual([
      "api",
      "graphql",
      "--hostname",
      "ghe.example.com",
      "-F",
      "query=@-",
      "-F",
      "n=3",
      "-f",
      "body=@not-a-file 42",
      "-F",
      "flag=true",
      "-F",
      "none=null",
    ]);
    expect(call?.stdin).toBe(QUERY);
  });

  test("checks the version once", async () => {
    const fake = install([VERSION, { match: "api graphql", stdout: '{"data":{}}' }]);
    const transport = ghTransport();
    await transport.graphql(QUERY, {});
    await transport.graphql(QUERY, {});
    expect(fake.calls().map((c) => c.args[0])).toEqual(["--version", "api", "api"]);
  });

  test("reports gh missing", async () => {
    process.env["PATH"] = "/nonexistent";
    await expect(ghTransport().graphql(QUERY, {})).rejects.toMatchObject({
      code: "gh-missing",
      message: expect.stringContaining("cli.github.com"),
    });
  });

  test("reports gh too old", async () => {
    install([{ match: "--version", stdout: "gh version 1.14.0 (2021-08-04)\n" }]);
    await expect(ghTransport().graphql(QUERY, {})).rejects.toMatchObject({ code: "gh-too-old" });
  });

  test("reports not signed in", async () => {
    install([
      VERSION,
      {
        match: "api graphql",
        stderr: "To get started with GitHub CLI, please run:  gh auth login\n",
        code: 4,
      },
    ]);
    await expect(ghTransport().graphql(QUERY, {})).rejects.toMatchObject({
      code: "not-signed-in",
      message: expect.stringContaining("gh auth login --hostname github.com"),
    });
  });

  test("reports the secondary rate limit", async () => {
    install([
      VERSION,
      {
        match: "api graphql",
        stderr:
          "gh: You have exceeded a secondary rate limit. Please wait a few minutes before you try again. (HTTP 403)\n",
        code: 1,
      },
    ]);
    await expect(ghTransport().graphql(QUERY, {})).rejects.toMatchObject({ code: "rate-limited" });
  });

  test("reports GraphQL errors, and keeps tokens out of them", async () => {
    install([
      VERSION,
      {
        match: "api graphql",
        stdout: JSON.stringify({
          errors: [
            { type: "UNPROCESSABLE", message: "bad ghp_abcdefghijklmnopqrstuvwxyz0123456789" },
          ],
        }),
        stderr: "gh: bad\n",
        code: 1,
      },
    ]);
    const error = await ghTransport()
      .graphql(QUERY, {})
      .catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ReviewError);
    expect((error as ReviewError).code).toBe("refused");
    expect((error as ReviewError).message).not.toContain("ghp_");
  });

  test("reports a missing object as not found", async () => {
    install([
      VERSION,
      {
        match: "api graphql",
        stdout: JSON.stringify({ errors: [{ type: "NOT_FOUND", message: "Could not resolve" }] }),
        code: 1,
      },
    ]);
    await expect(ghTransport().graphql(QUERY, {})).rejects.toMatchObject({ code: "not-found" });
  });
});

describe("the token transport", () => {
  const TOKEN = "gho_abcdefghijklmnopqrstuvwxyz0123456789";
  interface Init {
    status?: number;
    headers?: Record<string, string>;
    body: unknown;
  }
  const fakeFetch = (answer: Init) => {
    const requests: { url: string; init: RequestInit }[] = [];
    const fn = (async (url: string, init: RequestInit) => {
      requests.push({ url, init });
      return new Response(
        typeof answer.body === "string" ? answer.body : JSON.stringify(answer.body),
        {
          status: answer.status ?? 200,
          headers: answer.headers ?? {},
        },
      );
    }) as unknown as typeof fetch;
    return { fn, requests };
  };

  test("posts the query with the token, to the host's endpoint", async () => {
    const { fn, requests } = fakeFetch({ body: { data: { viewer: { login: "me" } } } });
    const data = await tokenTransport(() => TOKEN, { host: "ghe.example.com", fetch: fn }).graphql(
      QUERY,
      { n: 1 },
    );
    expect(data).toEqual({ viewer: { login: "me" } });
    expect(requests[0]?.url).toBe("https://ghe.example.com/api/graphql");
    const headers = requests[0]?.init.headers as Record<string, string> | undefined;
    expect(headers?.["authorization"]).toBe(`bearer ${TOKEN}`);
    expect(JSON.parse(requests[0]?.init.body as string)).toEqual({
      query: QUERY,
      variables: { n: 1 },
    });
    expect(graphqlUrl("github.com")).toBe("https://api.github.com/graphql");
  });

  test("reports no sign-in without asking GitHub", async () => {
    const { fn, requests } = fakeFetch({ body: {} });
    await expect(
      tokenTransport(async () => undefined, { fetch: fn }).graphql(QUERY, {}),
    ).rejects.toMatchObject({
      code: "not-signed-in",
    });
    expect(requests).toEqual([]);
  });

  test("reports a refused token", async () => {
    const { fn } = fakeFetch({ status: 401, body: { message: "Bad credentials" } });
    await expect(
      tokenTransport(() => TOKEN, { fetch: fn }).graphql(QUERY, {}),
    ).rejects.toMatchObject({
      code: "not-signed-in",
    });
  });

  test("reports the secondary rate limit with how long to wait", async () => {
    const { fn } = fakeFetch({
      status: 403,
      headers: { "retry-after": "60" },
      body: { message: "You have exceeded a secondary rate limit." },
    });
    await expect(
      tokenTransport(() => TOKEN, { fetch: fn }).graphql(QUERY, {}),
    ).rejects.toMatchObject({
      code: "rate-limited",
      retryAfter: 60,
    });
    const { fn: limited } = fakeFetch({
      status: 200,
      body: { errors: [{ type: "RATE_LIMITED", message: "slow" }] },
    });
    await expect(
      tokenTransport(() => TOKEN, { fetch: limited }).graphql(QUERY, {}),
    ).rejects.toMatchObject({
      code: "rate-limited",
    });
  });

  test("reports a network failure without the token", async () => {
    const fn = (async () => {
      throw new TypeError("fetch failed");
    }) as unknown as typeof fetch;
    const error = (await tokenTransport(() => TOKEN, { fetch: fn })
      .graphql(QUERY, {})
      .catch((e: unknown) => e)) as ReviewError;
    expect(error.code).toBe("network");
    expect(`${error.message} ${String(error.stack)}`).not.toContain(TOKEN);
  });

  test("reports other refusals, with any token in the answer scrubbed", async () => {
    const { fn } = fakeFetch({ status: 500, body: { message: `oops ${TOKEN}` } });
    const error = (await tokenTransport(() => TOKEN, { fetch: fn })
      .graphql(QUERY, {})
      .catch((e: unknown) => e)) as ReviewError;
    expect(error.code).toBe("refused");
    expect(error.message).not.toContain(TOKEN);
  });
});
