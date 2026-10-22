import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { describe, expect, it } from "vitest";
import { gitIgnored } from "../../src/gitIgnored.js";
import {
  channelName,
  expandBraces,
  globFolder,
  inSourceCopies,
  LEFT_OUT_FOLDERS,
  nestedProjects,
  ownedElsewhere,
  owningProject,
  projectExclude,
  projectName,
  samePath,
  throughFolder,
  within,
  type Project,
} from "../../src/projects.js";

const project = (folder: string): Project => ({
  folder,
  config: `${folder}${folder.includes("\\") ? "\\" : "/"}ascribe.toml`,
});

describe("owningProject", () => {
  const root = project("/repo");
  const docs = project("/repo/docs");
  const deep = project("/repo/docs/guide");
  const sibling = project("/repo/site");
  const all = [root, docs, deep, sibling];

  it("picks the nearest ancestor", () => {
    expect(owningProject("/repo/docs/guide/a.md", all)).toBe(deep);
    expect(owningProject("/repo/docs/b.md", all)).toBe(docs);
    expect(owningProject("/repo/readme.md", all)).toBe(root);
  });

  it("doesn't depend on the order of the list", () => {
    expect(owningProject("/repo/docs/guide/a.md", [...all].reverse())).toBe(deep);
  });

  it("keeps siblings apart", () => {
    expect(owningProject("/repo/site/x.md", all)).toBe(sibling);
    expect(owningProject("/repo/site/x.md", [docs, sibling])).toBe(sibling);
    expect(owningProject("/repo/site/x.md", [docs])).toBeUndefined();
  });

  it("returns nothing for a file outside every project", () => {
    expect(owningProject("/elsewhere/a.md", all)).toBeUndefined();
    expect(owningProject("/repo/a.md", [docs])).toBeUndefined();
  });

  it("doesn't mistake a path-prefix lookalike for a parent", () => {
    expect(owningProject("/a/docs-old/x.md", [project("/a/docs")])).toBeUndefined();
    expect(owningProject("/a/docs/x.md", [project("/a/docs-old")])).toBeUndefined();
  });

  it("gives a project's own ascribe.toml to that project", () => {
    expect(owningProject(deep.config, all)).toBe(deep);
  });

  it("handles a project at the filesystem root and a trailing separator", () => {
    expect(owningProject("/a.md", [project("/")])?.folder).toBe("/");
    expect(owningProject("/repo/docs/x.md", [project("/repo/docs/")])).toBeDefined();
  });

  describe("on Windows paths", () => {
    const win = [project("C:\\repo"), project("C:\\repo\\docs")];

    it("folds drive-letter and directory case, and accepts either separator", () => {
      expect(owningProject("c:\\Repo\\docs\\a.md", win)).toBe(win[1]);
      expect(owningProject("C:/repo/docs/a.md", win)).toBe(win[1]);
      expect(owningProject("C:\\REPO\\b.md", win)).toBe(win[0]);
    });

    it("doesn't cross drives or match lookalikes", () => {
      expect(owningProject("D:\\repo\\a.md", win)).toBeUndefined();
      expect(owningProject("C:\\repo-old\\a.md", win)).toBeUndefined();
    });
  });
});

describe("nestedProjects", () => {
  const root = project("/repo");
  const docs = project("/repo/docs");
  const deep = project("/repo/docs/guide");
  const sibling = project("/repo/site");
  const lookalike = project("/repo-old");
  const all = [root, docs, deep, sibling, lookalike];

  it("lists every project below, at any depth, and not the project itself", () => {
    expect(nestedProjects(root, all)).toEqual([docs, deep, sibling]);
    expect(nestedProjects(docs, all)).toEqual([deep]);
  });

  it("is empty for a leaf, a sibling, and a lookalike", () => {
    expect(nestedProjects(deep, all)).toEqual([]);
    expect(nestedProjects(sibling, all)).toEqual([]);
    expect(nestedProjects(lookalike, all)).toEqual([]);
  });

  it("compares Windows paths case-insensitively", () => {
    const parent = project("C:\\Repo");
    const child = project("c:\\repo\\docs");
    expect(nestedProjects(parent, [parent, child])).toEqual([child]);
  });
});

describe("samePath", () => {
  it("ignores a trailing separator, and case on Windows only", () => {
    expect(samePath("/a/b/", "/a/b")).toBe(true);
    expect(samePath("/a/B", "/a/b")).toBe(false);
    expect(samePath("C:\\A\\b", "c:/a/B")).toBe(true);
  });
});

describe("globFolder", () => {
  it("uses forward slashes and escapes glob characters", () => {
    expect(globFolder("C:\\repo\\docs")).toBe("C:/repo/docs");
    expect(globFolder("/a/[b]/c*/d?/{e}")).toBe("/a/[[]b]/c[*]/d[?]/[{]e}");
  });
});

describe("channelName", () => {
  const workspace = { path: "/repo", name: "repo" };

  it("is plain for a workspace with one project", () => {
    expect(channelName(project("/repo/docs"), workspace, true)).toBe("Ascribe");
  });

  it("shows the folder relative to its workspace folder", () => {
    expect(channelName(project("/repo/examples/quill"), workspace, false)).toBe(
      "Ascribe (examples/quill)",
    );
  });

  it("uses the workspace folder's name for a project at its root", () => {
    expect(channelName(project("/repo"), workspace, false)).toBe("Ascribe (repo)");
    expect(channelName(project("/repo/"), workspace, false)).toBe("Ascribe (repo)");
  });

  it("keeps Windows case and shows forward slashes", () => {
    expect(
      channelName(project("C:\\Repo\\Docs\\Quill"), { path: "c:\\repo", name: "Repo" }, false),
    ).toBe("Ascribe (Docs/Quill)");
  });

  it("falls back to the folder when there's no workspace folder", () => {
    expect(channelName(project("/elsewhere/x"), undefined, false)).toBe("Ascribe (/elsewhere/x)");
  });
});

describe("projectName", () => {
  it("is the folder relative to its workspace folder, or the workspace folder's name", () => {
    const workspace = { path: "/repo", name: "repo" };
    expect(projectName(project("/repo/examples/quill"), workspace)).toBe("examples/quill");
    expect(projectName(project("/repo"), workspace)).toBe("repo");
  });

  it("is the whole folder outside every workspace folder", () => {
    expect(projectName(project("/elsewhere/x"), undefined)).toBe("/elsewhere/x");
    expect(projectName(project("C:\\elsewhere\\x"), { path: "C:\\repo", name: "repo" })).toBe(
      "C:/elsewhere/x",
    );
  });
});

describe("within", () => {
  it("is true for the folder and what lies below it, and false for a lookalike", () => {
    expect(within("/repo/docs", "/repo/docs")).toBe(true);
    expect(within("/repo/docs/a/b.md", "/repo/docs/")).toBe(true);
    expect(within("/repo/docs-old/a.md", "/repo/docs")).toBe(false);
    expect(within("/repo/a.md", "/repo/docs")).toBe(false);
  });

  it("folds case and separators on Windows paths", () => {
    expect(within("c:\\Repo\\Docs\\a.md", "C:/repo/docs")).toBe(true);
    expect(within("D:\\repo\\docs\\a.md", "C:\\repo\\docs")).toBe(false);
  });
});

describe("ownedElsewhere", () => {
  const a = project("/repo/a");
  const b = project("/repo/b");
  const nested = project("/repo/a/docs/sub");
  const all = [a, b, nested];

  it("is false for the project's own files, including its ascribe.toml", () => {
    expect(ownedElsewhere(a, "/repo/a/docs/x.md", all)).toBe(false);
    expect(ownedElsewhere(a, a.config, all)).toBe(false);
  });

  it("is true for a sibling project's files", () => {
    expect(ownedElsewhere(a, "/repo/b/docs/x.md", all)).toBe(true);
    expect(ownedElsewhere(b, "/repo/a/docs/x.md", all)).toBe(true);
  });

  it("is true for a nested project's files, and false for the nested project's own", () => {
    expect(ownedElsewhere(a, "/repo/a/docs/sub/x.md", all)).toBe(true);
    expect(ownedElsewhere(nested, "/repo/a/docs/sub/x.md", all)).toBe(false);
  });

  it("is true for a file outside every project", () => {
    expect(ownedElsewhere(a, "/repo/target/debug/out.md", all)).toBe(true);
  });

  it("compares Windows paths case-insensitively", () => {
    const win = project("C:\\repo\\a");
    expect(ownedElsewhere(win, "c:\\Repo\\A\\x.md", [win])).toBe(false);
    expect(ownedElsewhere(win, "C:\\repo\\b\\x.md", [win, project("C:\\repo\\b")])).toBe(true);
  });
});

describe("throughFolder", () => {
  // Host paths, so the cases hold on Windows too.
  const p = (file: string) => path.resolve(file);

  it("writes a real path through the links the folder was opened with", () => {
    expect(throughFolder(p("/private/var/w/docs"), p("/var/w"), p("/private/var/w"))).toBe(
      p("/var/w/docs"),
    );
    expect(throughFolder(p("/private/var/w"), p("/var/w"), p("/private/var/w"))).toBe(p("/var/w"));
  });

  it("reaches a path outside the folder from it", () => {
    expect(throughFolder(p("/private/var/content"), p("/var/w"), p("/private/var/w"))).toBe(
      p("/var/content"),
    );
  });

  it("leaves a path alone when the folder has no links", () => {
    expect(throughFolder(p("/repo/docs"), p("/repo"), p("/repo"))).toBe(p("/repo/docs"));
  });

  it.runIf(process.platform === "win32")("keeps the folder's drive letter as opened", () => {
    expect(throughFolder("C:\\ws\\proj\\docs", "c:\\ws\\proj", "C:\\ws\\proj")).toBe(
      "c:\\ws\\proj\\docs",
    );
  });

  it.skipIf(process.platform === "win32")("matches an editor opened through a symlink", () => {
    const scratch = realpathSync(mkdtempSync(path.join(tmpdir(), "through-")));
    try {
      const real = path.join(scratch, "real");
      mkdirSync(path.join(real, "docs"), { recursive: true });
      const link = path.join(scratch, "link");
      symlinkSync(real, link);
      const contentRoot = realpathSync(path.join(link, "docs"));
      expect(contentRoot).toBe(path.join(real, "docs"));
      expect(throughFolder(contentRoot, link, realpathSync(link))).toBe(path.join(link, "docs"));
    } finally {
      rmSync(scratch, { recursive: true, force: true });
    }
  });
});

describe("projectExclude", () => {
  const defaults = LEFT_OUT_FOLDERS.map((folder) => `**/${folder}/**`).join(",");

  it("leaves out the default folders alone when nothing else is set", () => {
    expect(projectExclude({}, [])).toBe(`{${defaults}}`);
  });

  it("leaves out Ascribe's outputs, dependencies, and build output, but not build, out, or bin", () => {
    for (const folder of [
      ".ascribe",
      "node_modules",
      "vendor",
      ".venv",
      "target",
      "dist",
      ".next",
    ]) {
      expect(LEFT_OUT_FOLDERS).toContain(folder);
    }
    for (const folder of ["build", "out", "bin"]) {
      expect(LEFT_OUT_FOLDERS).not.toContain(folder);
    }
  });

  it("adds the files.exclude patterns that are on, and the setting's, after the defaults", () => {
    expect(
      projectExclude(
        { "**/.git": true, "**/build": false, "**/*.js": { when: "$(basename).ts" } },
        ["tests/**", "packages/vscode/test/**"],
      ),
    ).toBe(`{${defaults},**/.git,tests/**,packages/vscode/test/**}`);
  });

  it("expands a pattern's own braces, since VS Code's don't nest", () => {
    expect(projectExclude({}, ["fixtures/{a,b}/**"])).toBe(
      `{${defaults},fixtures/a/**,fixtures/b/**}`,
    );
  });

  it("drops what isn't a pattern, blanks, and repeats", () => {
    expect(projectExclude({}, ["", "  ", 3, "**/node_modules/**", "tests/**"])).toBe(
      `{${defaults},tests/**}`,
    );
  });

  it("names each default folder in the setting's description and the editor's settings page", () => {
    const manifest = JSON.parse(
      readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
    ) as {
      contributes: {
        configuration: { properties: Record<string, { markdownDescription: string }> };
      };
    };
    const description =
      manifest.contributes.configuration.properties["ascribe.projects.exclude"]
        ?.markdownDescription ?? "";
    const page = readFileSync(
      new URL("../../../../docs/content/editor/settings.md", import.meta.url),
      "utf8",
    );
    for (const folder of LEFT_OUT_FOLDERS) {
      expect(description, "the setting's description").toContain(`\`${folder}\``);
      expect(page, "docs/content/editor/settings.md").toContain(`\`${folder}\``);
    }
  });
});

describe("inSourceCopies", () => {
  const configs = ["/repo/docs/ascribe.toml", "/repo/docs/sources/api/ascribe.toml"];

  it("is true for an ascribe.toml in a project's sources/<name>/", () => {
    expect(inSourceCopies("/repo/docs/sources/api/ascribe.toml", configs)).toBe(true);
    expect(inSourceCopies("/repo/docs/sources/api/site/ascribe.toml", configs)).toBe(true);
  });

  it("is false for a project's own, one in sources/ itself, and one with no project above", () => {
    expect(inSourceCopies("/repo/docs/ascribe.toml", configs)).toBe(false);
    expect(inSourceCopies("/repo/docs/sources/ascribe.toml", configs)).toBe(false);
    expect(inSourceCopies("/repo/sources/api/ascribe.toml", configs)).toBe(false);
    expect(inSourceCopies("/repo/docs/sourcesx/api/ascribe.toml", configs)).toBe(false);
  });

  it("compares Windows paths whatever their case and separators", () => {
    const docs = ["C:\\repo\\docs\\ascribe.toml"];
    expect(inSourceCopies("c:\\Repo\\Docs\\Sources\\api\\ascribe.toml", docs)).toBe(true);
    expect(inSourceCopies("c:/repo/docs/sources/api/ascribe.toml", docs)).toBe(true);
    expect(inSourceCopies("c:\\Repo\\Sources\\api\\ascribe.toml", docs)).toBe(false);
  });
});

describe("gitIgnored", () => {
  const git = (cwd: string, ...args: string[]) =>
    execFileSync("git", ["-c", "user.name=Test", "-c", "user.email=test@example.com", ...args], {
      cwd,
    });
  const write = (file: string) => {
    mkdirSync(path.dirname(file), { recursive: true });
    writeFileSync(file, "");
  };

  it("gives the files the repository ignores, but not one git tracks", async () => {
    const repo = mkdtempSync(path.join(tmpdir(), "ascribe-ignored-"));
    try {
      git(repo, "init", "-q");
      writeFileSync(path.join(repo, ".gitignore"), "generated/\nforced/\n");
      const files = ["docs", "generated/a", "forced/b", "with space/c"].map((folder) =>
        path.join(repo, folder, "ascribe.toml"),
      );
      files.forEach(write);
      git(repo, "add", "-f", path.join("forced", "b", "ascribe.toml"));
      const ignored = await gitIgnored(repo, files);
      expect([...ignored]).toEqual([path.join(repo, "generated", "a", "ascribe.toml")]);
      // Asked from a folder inside the repository, as a workspace folder can be.
      const generated = path.join(repo, "generated");
      expect([...(await gitIgnored(generated, [files[1] ?? ""]))]).toEqual([files[1]]);
    } finally {
      rmSync(repo, { recursive: true, force: true });
    }
  });

  it("gives none outside a repository", async () => {
    const folder = mkdtempSync(path.join(tmpdir(), "ascribe-no-repo-"));
    try {
      const file = path.join(folder, "target", "ascribe.toml");
      write(file);
      // Git looks no higher than the temporary directory's parent.
      process.env["GIT_CEILING_DIRECTORIES"] = path.dirname(folder);
      expect([...(await gitIgnored(folder, [file]))]).toEqual([]);
    } finally {
      delete process.env["GIT_CEILING_DIRECTORIES"];
      rmSync(folder, { recursive: true, force: true });
    }
  });
});

describe("expandBraces", () => {
  it("gives each alternative of a group", () => {
    expect(expandBraces("*.{md,mdx}")).toEqual(["*.md", "*.mdx"]);
  });

  it("expands nested and repeated groups", () => {
    expect(expandBraces("{a,b{1,2}}/{x,y}")).toEqual([
      "a/x",
      "a/y",
      "b1/x",
      "b1/y",
      "b2/x",
      "b2/y",
    ]);
  });

  it("leaves a pattern with no group, or an unclosed one, as it is", () => {
    expect(expandBraces("tests/**")).toEqual(["tests/**"]);
    expect(expandBraces("odd{name")).toEqual(["odd{name"]);
  });
});
