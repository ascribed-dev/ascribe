// The checkout's GitHub repository and branch, from git's configuration.
import { git, gitMaybe } from "../shared/git.js";
import { ReviewError } from "../shared/errors.js";

/** A repository on a GitHub host. */
export interface RepositoryRef {
  /** `github.com`, or a GitHub Enterprise Server host. */
  host: string;
  owner: string;
  name: string;
}

/**
 * The repository a remote URL names, or `undefined` if it isn't a GitHub-style
 * URL: `https://host/owner/name(.git)`, `git@host:owner/name(.git)`, or
 * `ssh://git@host[:port]/owner/name(.git)`.
 */
export function parseRemote(url: string): RepositoryRef | undefined {
  const trimmed = url.trim();
  let host: string;
  let rest: string;
  const scp = /^(?:[^@/]+@)?([^:/]+):(?!\/)(.+)$/.exec(trimmed);
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(trimmed)) {
    let parsed: URL;
    try {
      parsed = new URL(trimmed);
    } catch {
      return undefined;
    }
    if (!["https:", "http:", "ssh:", "git:"].includes(parsed.protocol)) return undefined;
    host = parsed.hostname;
    rest = parsed.pathname;
  } else if (scp !== null) {
    host = scp[1] ?? "";
    rest = scp[2] ?? "";
  } else {
    return undefined;
  }
  const parts = rest.replace(/^\/+|\/+$/g, "").split("/");
  if (parts.length !== 2) return undefined;
  const owner = parts[0] ?? "";
  const name = (parts[1] ?? "").replace(/\.git$/, "");
  if (owner === "" || name === "") return undefined;
  // SSH connections to github.com sometimes go through ssh.github.com.
  if (host === "ssh.github.com") host = "github.com";
  return { host: host.toLowerCase(), owner, name };
}

/** The checkout's branch and the repositories a pull request for it could be in. */
export interface CheckoutInfo {
  /** The repository's top-level directory. */
  root: string;
  /** The local branch, or `undefined` when `HEAD` is detached. */
  branch: string | undefined;
  /** The branch's name on its remote (its upstream's name, or the same name). */
  headRef: string | undefined;
  /** The repository the branch is pushed to. */
  head: RepositoryRef | undefined;
  /** Where to look for the pull request: an `upstream` remote first, then `head`. */
  bases: RepositoryRef[];
  /** The commit `HEAD` names. */
  headOid: string;
}

/** Reads the checkout at `cwd`. Rejects with a `git` error if it isn't a repository. */
export async function readCheckout(cwd: string): Promise<CheckoutInfo> {
  const root = (await git(cwd, ["rev-parse", "--show-toplevel"])).trim();
  const headOid = (await gitMaybe(root, ["rev-parse", "--verify", "--quiet", "HEAD"]))?.trim();
  if (headOid === undefined || headOid === "") {
    throw new ReviewError("git", "The repository has no commits yet.");
  }
  const branch = (await gitMaybe(root, ["symbolic-ref", "--quiet", "--short", "HEAD"]))?.trim();
  if (branch === undefined || branch === "") {
    return { root, branch: undefined, headRef: undefined, head: undefined, bases: [], headOid };
  }
  const config = async (key: string): Promise<string | undefined> =>
    (await gitMaybe(root, ["config", "--get", key]))?.trim() || undefined;
  const remoteName =
    (await config(`branch.${branch}.pushRemote`)) ??
    (await config("remote.pushDefault")) ??
    (await config(`branch.${branch}.remote`)) ??
    "origin";
  const merge = await config(`branch.${branch}.merge`);
  const headRef = merge?.replace(/^refs\/heads\//, "") ?? branch;
  const remoteUrl = async (name: string): Promise<RepositoryRef | undefined> => {
    const url = await config(`remote.${name}.url`);
    return url === undefined ? undefined : parseRemote(url);
  };
  const head = await remoteUrl(remoteName);
  const bases: RepositoryRef[] = [];
  const upstream = remoteName === "upstream" ? undefined : await remoteUrl("upstream");
  if (upstream !== undefined) bases.push(upstream);
  if (head !== undefined) bases.push(head);
  return { root, branch, headRef, head, bases, headOid };
}
