import { describe, expect, it } from "vitest";
import { ChosenBuilds } from "../../src/ui/chosenBuild.js";

describe("the build chosen in each project", () => {
  it("keeps a choice to the project it was made in", () => {
    const choices = new ChosenBuilds();
    choices.set("/repo/examples/quill", "cloud");
    expect(choices.get("/repo/examples/quill")).toBe("cloud");
    expect(choices.get("/repo/examples/astro-site")).toBeUndefined();
    choices.set("/repo/examples/astro-site", "site");
    expect(choices.get("/repo/examples/quill")).toBe("cloud");
    expect(choices.get("/repo/examples/astro-site")).toBe("site");
  });

  it("goes back to the editor's build when the choice is cleared", () => {
    const choices = new ChosenBuilds();
    choices.set("/repo/docs", "cloud");
    choices.set("/repo/docs", undefined);
    expect(choices.get("/repo/docs")).toBeUndefined();
  });

  it("finds a project's choice however its folder is spelled", () => {
    const choices = new ChosenBuilds();
    choices.set("/repo/docs/", "cloud");
    expect(choices.get("/repo/docs")).toBe("cloud");
    choices.set("C:\\Repo\\Docs", "site");
    expect(choices.get("c:/repo/docs")).toBe("site");
  });

  it("follows the editor's build when that is the one chosen", () => {
    const choices = new ChosenBuilds();
    choices.choose("/repo/docs", "cloud", "site");
    expect(choices.get("/repo/docs")).toBe("cloud");
    choices.choose("/repo/docs", "site", "site");
    expect(choices.get("/repo/docs")).toBeUndefined();
  });

  it("says which project's choice changed, once per change", () => {
    const choices = new ChosenBuilds();
    const changed: string[] = [];
    const listening = choices.onDidChange((folder) => changed.push(folder));
    choices.set("/repo/docs", "cloud");
    choices.set("/repo/docs", "cloud");
    choices.choose("/repo/handbook", "site", "site");
    choices.set("/repo/docs", undefined);
    expect(changed).toEqual(["/repo/docs", "/repo/docs"]);
    listening.dispose();
    choices.set("/repo/docs", "cloud");
    expect(changed).toHaveLength(2);
  });

  it("shows the chosen build while the content model has it, and the editor's otherwise", () => {
    const choices = new ChosenBuilds();
    const builds = [
      { name: "site", editor: true },
      { name: "cloud", editor: false },
    ];
    expect(choices.shown("/repo/docs", builds)).toBe("site");
    choices.set("/repo/docs", "cloud");
    expect(choices.shown("/repo/docs", builds)).toBe("cloud");
    choices.set("/repo/docs", "gone");
    expect(choices.shown("/repo/docs", builds)).toBe("site");
    expect(choices.shown("/repo/docs", [])).toBe("gone");
    expect(choices.shown("/repo/other", [])).toBeUndefined();
  });
});
