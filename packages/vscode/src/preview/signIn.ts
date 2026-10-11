// Which of VS Code's GitHub sign-ins a pull request's host may use. Apart from
// VS Code, so it can be tested by itself; `threads.ts` gives it VS Code's
// authentication and settings.
import type { HostToken } from "@ascribed/review/github";

/** VS Code's authentication providers for GitHub. */
type Provider = "github" | "github-enterprise";

/** What reading a sign-in needs from VS Code. */
export interface SignIns {
  /** `vscode.authentication.getSession`, for one of the GitHub providers. */
  getSession(
    provider: Provider,
    scopes: readonly string[],
    interactive: boolean,
  ): Promise<{ accessToken: string } | undefined>;
  /** The `github-enterprise.uri` setting: the one server the `github-enterprise` sign-in is for. */
  enterpriseUri(): string | undefined;
}

/** The host of the `github-enterprise.uri` setting, or `undefined` when it's unset or not a URL. */
export function enterpriseHost(uri: string | undefined): string | undefined {
  if (uri === undefined || uri.trim() === "") return undefined;
  try {
    const host = new URL(uri.trim()).hostname;
    return host === "" ? undefined : host.toLowerCase();
  } catch {
    return undefined;
  }
}

/**
 * The sign-in a host gets: VS Code's `github` for `github.com`, its
 * `github-enterprise` for the server `github-enterprise.uri` names, and none
 * for any other host, whose remote isn't GitHub's.
 */
export function providerFor(host: string, enterpriseUri: string | undefined): Provider | undefined {
  // Compared by name, without the port: a remote's port is git's, not the API's.
  const name = host.toLowerCase();
  if (name === "github.com") return "github";
  if (name === enterpriseHost(enterpriseUri)) return "github-enterprise";
  return undefined;
}

/** Why a host gets no sign-in, as a sentence; `undefined` when it gets one. */
export function noSignInReason(
  host: string,
  enterpriseUri: string | undefined,
): string | undefined {
  if (providerFor(host, enterpriseUri) !== undefined) return undefined;
  const configured = enterpriseHost(enterpriseUri);
  return configured === undefined
    ? `The remote is on ${host}, which isn't GitHub. For a GitHub Enterprise server, set github-enterprise.uri.`
    : `The remote is on ${host}, which isn't GitHub or the GitHub Enterprise server github-enterprise.uri names (${configured}).`;
}

/**
 * Whether there's a sign-in for the host; with `interactive`, asks for one.
 * A host that gets no sign-in is never asked about.
 */
export async function hasSignIn(
  host: string,
  scopes: readonly string[],
  interactive: boolean,
  signIns: SignIns,
): Promise<boolean> {
  const provider = providerFor(host, signIns.enterpriseUri());
  if (provider === undefined) return false;
  try {
    return (await signIns.getSession(provider, scopes, interactive)) !== undefined;
  } catch {
    // Declined, or no provider for the host (GitHub Enterprise isn't set up).
    return false;
  }
}

/**
 * The host's token, with the host it was issued for, so the transport can
 * refuse to send it anywhere else. Read for each request so VS Code can
 * refresh it; `undefined` for a host that gets no sign-in.
 */
export async function tokenFor(
  host: string,
  scopes: readonly string[],
  signIns: SignIns,
): Promise<HostToken | undefined> {
  const enterpriseUri = signIns.enterpriseUri();
  const provider = providerFor(host, enterpriseUri);
  if (provider === undefined) return undefined;
  const session = await signIns.getSession(provider, scopes, false).then(
    (s) => s,
    () => undefined,
  );
  if (session === undefined) return undefined;
  const issuedFor = provider === "github" ? "github.com" : enterpriseHost(enterpriseUri);
  return issuedFor === undefined ? undefined : { token: session.accessToken, host: issuedFor };
}
