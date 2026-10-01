import { describe, expect, it } from "vitest";
import {
  channelName,
  globFolder,
  nestedProjects,
  owningProject,
  samePath,
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
