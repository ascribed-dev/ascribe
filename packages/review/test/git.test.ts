// Running `git` runs nothing the repository supplies: the options and the
// cleared environment are the compiler's (`crates/ascribe-core/src/git.rs`),
// and a committed folder shaped like a bare repository, whose configuration
// names a command, isn't taken for the repository.
import {
  existsSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { readCheckout } from "../src/github/repository.js";
import { lineMap } from "../src/place/lines.js";
import { CLEARED, OPTIONS, git } from "../src/shared/git.js";
import { tempRepo, type TempRepo } from "./helpers/repo.js";

/** The string literals of the Rust constant `name` in `ascribe_core::git`. */
function rustList(name: string): string[] {
  const rust = readFileSync(
    new URL("../../../crates/ascribe-core/src/git.rs", import.meta.url),
    "utf8",
  );
  const body = new RegExp(`pub const ${name}: &\\[&str\\] = &\\[([^\\]]*)\\];`).exec(rust)?.[1];
  expect(body, `ascribe_core::git::${name}`).toBeDefined();
  return [...(body ?? "").matchAll(/^\s*"([^"]*)",/gm)].map((m) => m[1] ?? "");
}

describe("the options", () => {
  it("are the compiler's", () => {
    expect(OPTIONS).toEqual(rustList("OPTIONS"));
    expect(CLEARED).toEqual(rustList("CLEARED"));
  });
});

describe("a committed folder shaped like a bare repository", () => {
  let repo: TempRepo;
  let marks: string;
  let site: string;
  beforeAll(() => {
    repo = tempRepo();
    marks = realpathSync.native(mkdtempSync(path.join(tmpdir(), "ascribe-review-marks-")));
    site = path.join(repo.root, "site");
    repo.git("init", "--quiet", "--bare", "--initial-branch=main", "site");
    // A commit in the planted repository, so its `HEAD` names one.
    const work = realpathSync.native(mkdtempSync(path.join(tmpdir(), "ascribe-review-work-")));
    writeFileSync(path.join(work, "page.md"), "# Page\n");
    repo.git(`--git-dir=${site}`, `--work-tree=${work}`, "add", "-A");
    repo.git(`--git-dir=${site}`, `--work-tree=${work}`, "commit", "--quiet", "-m", "planted");
    rmSync(work, { recursive: true, force: true });
    const mark = path.join(marks, "ran").replace(/\\/g, "/");
    writeFileSync(
      path.join(site, "config"),
      "[core]\n\trepositoryformatversion = 0\n\tbare = false\n\tworktree = .\n" +
        `\tfsmonitor = "touch '${mark}' #"\n`,
    );
    repo.write("site/page.md", "# Page, changed\n");
    repo.commit("start");
  });
  afterAll(() => {
    repo.remove();
    rmSync(marks, { recursive: true, force: true });
  });

  it("isn't read: its command doesn't run", async () => {
    await readCheckout(site).catch(() => undefined);
    await lineMap(site, "HEAD", "page.md", "to-worktree").catch(() => undefined);
    await git(site, ["status", "--porcelain"]).catch(() => undefined);
    expect(existsSync(path.join(marks, "ran"))).toBe(false);
  });
});
