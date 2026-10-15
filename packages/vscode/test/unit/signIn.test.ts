// Which GitHub sign-in a pull request's host gets (`src/preview/signIn.ts`):
// a token goes only to the host it was issued for.
import { describe, expect, it } from "vitest";
import {
  enterpriseHost,
  hasSignIn,
  noSignInReason,
  providerFor,
  tokenFor,
  type SignIns,
} from "../../src/preview/signIn.js";

const SCOPES = ["repo"];

/** VS Code with a session for both providers, recording what it's asked. */
function signedIn(enterpriseUri: string | undefined) {
  const asked: { provider: string; interactive: boolean }[] = [];
  const signIns: SignIns = {
    getSession: (provider, _scopes, interactive) => {
      asked.push({ provider, interactive });
      return Promise.resolve({ accessToken: `${provider}-token` });
    },
    enterpriseUri: () => enterpriseUri,
  };
  return { signIns, asked };
}

describe("providerFor", () => {
  it("gives github.com VS Code's github sign-in", () => {
    expect(providerFor("github.com", undefined)).toBe("github");
    expect(providerFor("GitHub.com", "https://ghe.example.com")).toBe("github");
  });

  it("gives the configured GitHub Enterprise server its sign-in, compared as a host name", () => {
    expect(providerFor("ghe.example.com", "https://ghe.example.com")).toBe("github-enterprise");
    expect(providerFor("ghe.example.com", "https://GHE.Example.com:8443/some/path/")).toBe(
      "github-enterprise",
    );
    expect(providerFor("GHE.example.COM", " https://ghe.example.com ")).toBe("github-enterprise");
  });

  it("gives any other host no sign-in", () => {
    expect(providerFor("gitlab.example", undefined)).toBeUndefined();
    expect(providerFor("gitlab.example", "https://ghe.example.com")).toBeUndefined();
    expect(providerFor("ghe.example.com.evil.example", "https://ghe.example.com")).toBeUndefined();
    expect(providerFor("example.com", "https://ghe.example.com")).toBeUndefined();
    expect(providerFor("ghe.example.com", "")).toBeUndefined();
    expect(providerFor("ghe.example.com", "not a url")).toBeUndefined();
  });
});

describe("enterpriseHost", () => {
  it("is the setting's host name, without port or path, in lower case", () => {
    expect(enterpriseHost("https://GHE.example.com:8443/x")).toBe("ghe.example.com");
    expect(enterpriseHost(undefined)).toBeUndefined();
    expect(enterpriseHost("  ")).toBeUndefined();
  });
});

describe("a host that isn't GitHub's", () => {
  it("is never asked about, quietly or by asking the person", async () => {
    const { signIns, asked } = signedIn("https://ghe.example.com");
    expect(await hasSignIn("gitlab.example", SCOPES, false, signIns)).toBe(false);
    expect(await hasSignIn("gitlab.example", SCOPES, true, signIns)).toBe(false);
    expect(await tokenFor("gitlab.example", SCOPES, signIns)).toBeUndefined();
    expect(asked).toEqual([]);
  });

  it("says why, and names the setting", () => {
    expect(noSignInReason("gitlab.example", undefined)).toBe(
      "The remote is on gitlab.example, which isn't GitHub. For a GitHub Enterprise server, set github-enterprise.uri.",
    );
    expect(noSignInReason("gitlab.example", "https://ghe.example.com")).toContain(
      "github-enterprise.uri names (ghe.example.com)",
    );
    expect(noSignInReason("github.com", undefined)).toBeUndefined();
  });
});

describe("tokenFor", () => {
  it("gives each token with the host it was issued for", async () => {
    const { signIns, asked } = signedIn("https://GHE.example.com:8443");
    expect(await tokenFor("github.com", SCOPES, signIns)).toEqual({
      token: "github-token",
      host: "github.com",
    });
    expect(await tokenFor("ghe.example.com", SCOPES, signIns)).toEqual({
      token: "github-enterprise-token",
      host: "ghe.example.com",
    });
    expect(asked).toEqual([
      { provider: "github", interactive: false },
      { provider: "github-enterprise", interactive: false },
    ]);
  });

  it("is undefined when there's no session, or reading one fails", async () => {
    const none: SignIns = {
      getSession: () => Promise.resolve(undefined),
      enterpriseUri: () => undefined,
    };
    const failing: SignIns = {
      getSession: () => Promise.reject(new Error("declined")),
      enterpriseUri: () => undefined,
    };
    expect(await tokenFor("github.com", SCOPES, none)).toBeUndefined();
    expect(await tokenFor("github.com", SCOPES, failing)).toBeUndefined();
    expect(await hasSignIn("github.com", SCOPES, true, failing)).toBe(false);
  });
});
