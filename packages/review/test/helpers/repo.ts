// Temporary git repositories for tests.
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

export interface TempRepo {
  root: string;
  git(...args: string[]): string;
  write(file: string, text: string): void;
  read(file: string): string;
  commit(message: string): string;
  remove(): void;
}

/** A new repository in a temporary directory, with an initial branch `main`. */
export function tempRepo(): TempRepo {
  const root = realpathSync.native(mkdtempSync(path.join(tmpdir(), "ascribe-review-")));
  const git = (...args: string[]) =>
    execFileSync("git", ["-c", "core.quotepath=off", ...args], {
      cwd: root,
      encoding: "utf8",
      env: {
        ...process.env,
        GIT_AUTHOR_NAME: "Test",
        GIT_AUTHOR_EMAIL: "test@example.com",
        GIT_COMMITTER_NAME: "Test",
        GIT_COMMITTER_EMAIL: "test@example.com",
        GIT_CONFIG_NOSYSTEM: "1",
      },
    });
  git("init", "--quiet", "--initial-branch=main");
  git("config", "core.autocrlf", "false");
  git("config", "commit.gpgsign", "false");
  return {
    root,
    git,
    write(file, text) {
      const full = path.join(root, ...file.split("/"));
      mkdirSync(path.dirname(full), { recursive: true });
      writeFileSync(full, text);
    },
    read(file) {
      return readFileSync(path.join(root, ...file.split("/")), "utf8");
    },
    commit(message) {
      git("add", "-A");
      git("commit", "--quiet", "--allow-empty", "-m", message);
      return git("rev-parse", "HEAD").trim();
    },
    remove() {
      rmSync(root, { recursive: true, force: true });
    },
  };
}

/** `count` lines, `line 1` to `line <count>`. */
export function numbered(count: number, prefix = "line"): string[] {
  return Array.from({ length: count }, (_, i) => `${prefix} ${i + 1}`);
}
