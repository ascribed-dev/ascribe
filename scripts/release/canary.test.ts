// The test canary.ts's `failed` makes: whether a failure only waits for a
// canary. Each case is a small repository whose commits stand for `main`, with
// one of them as the canary's.
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, expect, test } from "vitest";
import { verdict } from "./canary.ts";

const dirs: string[] = [];
afterEach(() => {
  for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true });
});

function repository(): { dir: string; commit: (file: string) => string } {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-canary-"));
  dirs.push(dir);
  git(dir, "init", "--quiet", "--initial-branch", "main");
  let n = 0;
  return {
    dir,
    commit(file) {
      mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
      writeFileSync(path.join(dir, file), `${n++}\n`);
      git(dir, "add", "--all");
      git(dir, "commit", "--quiet", "--message", file);
      return git(dir, "rev-parse", "HEAD").trim();
    },
  };
}

function git(cwd: string, ...args: string[]): string {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    env: {
      ...process.env,
      GIT_AUTHOR_NAME: "test",
      GIT_AUTHOR_EMAIL: "test@example.com",
      GIT_COMMITTER_NAME: "test",
      GIT_COMMITTER_EMAIL: "test@example.com",
    },
  });
  if (result.status !== 0) throw new Error(result.stderr);
  return result.stdout;
}

test("a commit after the canary that changes what ships waits", () => {
  const { dir, commit } = repository();
  const canary = commit("crates/cli/src/main.rs");
  commit("crates/cli/src/main.rs");
  commit("docs/content/page.md");
  expect(verdict(dir, canary).waiting).toBe(true);
});

test("a commit the canary includes fails", () => {
  const { dir, commit } = repository();
  commit("crates/cli/src/main.rs");
  const canary = commit("docs/content/page.md");
  expect(verdict(dir, canary)).toMatchObject({ waiting: false, why: expect.stringContaining("includes") });
  // The canary is newer than the commit checked.
  commit("crates/cli/src/main.rs");
  const newer = commit("packages/astro/index.ts");
  git(dir, "checkout", "--quiet", canary);
  expect(verdict(dir, newer).waiting).toBe(false);
});

test("a commit after the canary that changes nothing that ships fails", () => {
  const { dir, commit } = repository();
  const canary = commit("crates/cli/src/main.rs");
  commit("docs/content/page.md");
  commit("site/src/pages/index.astro");
  expect(verdict(dir, canary)).toMatchObject({ waiting: false, why: expect.stringContaining("Nothing that ships") });
});

test("a pull request's merge commit that changes what ships waits", () => {
  const { dir, commit } = repository();
  const canary = commit("Cargo.lock");
  git(dir, "checkout", "--quiet", "-b", "change");
  commit("crates/cli/src/new.rs");
  git(dir, "checkout", "--quiet", "main");
  commit("docs/content/page.md");
  git(dir, "merge", "--quiet", "--no-ff", "--no-edit", "change");
  expect(verdict(dir, canary).waiting).toBe(true);
});

test("a canary that can't be compared never excuses a failure", () => {
  const { dir, commit } = repository();
  commit("crates/cli/src/main.rs");
  expect(verdict(dir, undefined).waiting).toBe(false);
  expect(verdict(dir, "").waiting).toBe(false);
  expect(verdict(dir, "not a commit").waiting).toBe(false);
  expect(verdict(dir, "0123456789abcdef0123456789abcdef01234567")).toMatchObject({
    waiting: false,
    why: expect.stringContaining("isn't in this clone"),
  });
});
