import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import { parseRemote, readCheckout } from "../src/github/repository.js";
import { tempRepo, type TempRepo } from "./helpers/repo.js";

describe("remotes", () => {
  test("read GitHub and GitHub Enterprise Server URLs in each form", () => {
    const acme = { host: "github.com", owner: "acme", name: "docs" };
    expect(parseRemote("https://github.com/acme/docs.git")).toEqual(acme);
    expect(parseRemote("https://github.com/acme/docs")).toEqual(acme);
    expect(parseRemote("git@github.com:acme/docs.git")).toEqual(acme);
    expect(parseRemote("ssh://git@ssh.github.com:443/acme/docs.git")).toEqual(acme);
    expect(parseRemote("https://GHE.example.com/team/site.git")).toEqual({
      host: "ghe.example.com",
      owner: "team",
      name: "site",
    });
    expect(parseRemote("/srv/git/docs.git")).toBeUndefined();
    expect(parseRemote("https://gitlab.example.com/group/sub/docs.git")).toBeUndefined();
  });
});

describe("the checkout", () => {
  let repo: TempRepo;
  afterEach(() => repo.remove());

  test("names the branch, its remote name, and where to look", async () => {
    repo = tempRepo();
    repo.commit("init");
    repo.git("checkout", "--quiet", "-b", "local-name");
    repo.git("remote", "add", "origin", "git@github.com:me/docs.git");
    repo.git("remote", "add", "upstream", "https://github.com/acme/docs.git");
    repo.git("config", "branch.local-name.remote", "origin");
    repo.git("config", "branch.local-name.merge", "refs/heads/fix-typo");
    const checkout = await readCheckout(repo.root);
    expect(checkout.branch).toBe("local-name");
    expect(checkout.headRef).toBe("fix-typo");
    expect(checkout.head).toEqual({ host: "github.com", owner: "me", name: "docs" });
    expect(checkout.bases.map((b) => b.owner)).toEqual(["acme", "me"]);
  });

  test("has no branch when HEAD is detached", async () => {
    repo = tempRepo();
    const commit = repo.commit("init");
    repo.git("checkout", "--quiet", commit);
    const checkout = await readCheckout(repo.root);
    expect(checkout.branch).toBeUndefined();
    expect(checkout.bases).toEqual([]);
  });

  test("rejects a directory that isn't a repository", async () => {
    repo = tempRepo();
    const outside = mkdtempSync(path.join(tmpdir(), "not-a-repo-"));
    try {
      await expect(readCheckout(outside)).rejects.toMatchObject({ code: "git" });
    } finally {
      rmSync(outside, { recursive: true, force: true });
    }
  });
});
