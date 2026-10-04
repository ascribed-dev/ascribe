import { describe, expect, it } from "vitest";
import { contentSecurityPolicy, shellHtml } from "../../src/preview/html.js";
import type { PreviewResult } from "../../src/preview/protocol.js";
import { canonicalReference, isExternal, splitFragment } from "../../src/preview/refs.js";
import { BuildChoices, previewProblems, type PreviewSituation } from "../../src/preview/routing.js";

describe("asset references", () => {
  it("compares references however the encoding is spelled", () => {
    expect(canonicalReference("./My Diagrams/a.png")).toBe(
      canonicalReference("./My%20Diagrams/a.png"),
    );
    expect(canonicalReference("./a%25b.png")).toBe(canonicalReference("./a%25b.png"));
    expect(canonicalReference("./a%25b.png")).not.toBe(canonicalReference("./a%b.png"));
    expect(canonicalReference("../_fragments/x.png")).toBe("/_fragments/x.png");
    expect(canonicalReference("/docs/_ascribe/files/a b.yaml")).toBe(
      canonicalReference("/docs/_ascribe/files/a%20b.yaml"),
    );
  });

  it("ignores the fragment", () => {
    expect(canonicalReference("./manual.pdf#page=2")).toBe(canonicalReference("./manual.pdf"));
    expect(splitFragment("./manual.pdf#page=2")).toEqual({
      path: "./manual.pdf",
      fragment: "#page=2",
    });
    expect(splitFragment("./x.png")).toEqual({ path: "./x.png", fragment: "" });
  });

  it("tells external destinations from local ones", () => {
    for (const href of ["https://a.b/c", "mailto:a@b.c", "//cdn.example/x", "javascript:x()"]) {
      expect(isExternal(href), href).toBe(true);
    }
    for (const href of ["./a.png", "/docs/x", "#top", "./a:b/c"]) {
      expect(isExternal(href), href).toBe(false);
    }
  });
});

describe("the webview's content security policy", () => {
  const cspSource = "https://x.vscode-cdn.net";
  const policy = contentSecurityPolicy(cspSource);

  it("allows only the extension's and the project's resources", () => {
    const directives = Object.fromEntries(
      policy.split("; ").map((d) => [d.split(" ")[0], d.split(" ").slice(1)]),
    );
    expect(directives["default-src"]).toEqual(["'none'"]);
    for (const name of ["script-src", "style-src", "img-src", "font-src"]) {
      expect(directives[name], name).toEqual([cspSource]);
    }
    expect(Object.keys(directives).sort()).toEqual([
      "default-src",
      "font-src",
      "img-src",
      "script-src",
      "style-src",
    ]);
  });

  it("has no way to run inline code or reach the network", () => {
    expect(policy).not.toMatch(
      /unsafe-inline|unsafe-eval|nonce-|https?:\/\/(?!x\.vscode)|\*|data:|blob:/,
    );
  });

  it("is in the shell, which loads only files, with no inline script or style", () => {
    const html = shellHtml({
      cspSource,
      elementsScript: `${cspSource}/e.js`,
      elementsStyle: `${cspSource}/e.css`,
      marksStyle: `${cspSource}/m.css`,
      previewScript: `${cspSource}/p.js`,
      previewStyle: `${cspSource}/p.css`,
    });
    expect(html).toContain(`http-equiv="Content-Security-Policy" content="${policy}"`);
    expect(html.match(/<script[^>]*>/g)).toEqual([
      `<script src="${cspSource}/e.js">`,
      `<script src="${cspSource}/p.js">`,
    ]);
    expect(html).not.toMatch(/<script>|<style|\sstyle=|\son\w+=/);
  });
});

describe("the build chosen in each project", () => {
  it("keeps a choice to the project it was made in", () => {
    const choices = new BuildChoices();
    choices.set("/repo/examples/quill", "cloud");
    expect(choices.get("/repo/examples/quill")).toBe("cloud");
    expect(choices.get("/repo/examples/astro-site")).toBeUndefined();
    choices.set("/repo/examples/astro-site", "site");
    expect(choices.get("/repo/examples/quill")).toBe("cloud");
    expect(choices.get("/repo/examples/astro-site")).toBe("site");
  });

  it("goes back to the editor's build when the choice is cleared", () => {
    const choices = new BuildChoices();
    choices.set("/repo/docs", "cloud");
    choices.set("/repo/docs", undefined);
    expect(choices.get("/repo/docs")).toBeUndefined();
  });

  it("finds a project's choice however its folder is spelled", () => {
    const choices = new BuildChoices();
    choices.set("/repo/docs/", "cloud");
    expect(choices.get("/repo/docs")).toBe("cloud");
    choices.set("C:\\Repo\\Docs", "site");
    expect(choices.get("c:/repo/docs")).toBe("site");
  });
});

describe("what the preview says", () => {
  const answer = (extra: Partial<PreviewResult> = {}): PreviewResult => ({
    build: "site",
    builds: [{ name: "site", editor: true, description: "" }],
    projectRoot: "/repo/examples/quill",
    contentRoot: "/repo/examples/quill/docs",
    assetRoots: [],
    documentVersion: 1,
    page: null,
    problems: [],
    ...extra,
  });
  const situation = (extra: Partial<PreviewSituation> = {}): PreviewSituation => ({
    file: "/repo/examples/quill/docs/index.md",
    project: "examples/quill",
    state: "running",
    result: answer(),
    show: (file) => file.replace(/^\/repo\//, ""),
    ...extra,
  });

  it("asks for a page before one has been active", () => {
    expect(previewProblems(situation({ file: undefined }))).toEqual([
      { severity: "info", message: "Open an Ascribe page to preview it." },
    ]);
  });

  it("says a file outside every project isn't part of one", () => {
    const problems = previewProblems(
      situation({
        file: "/repo/README.md",
        project: undefined,
        state: undefined,
        result: undefined,
      }),
    );
    expect(problems).toEqual([
      {
        severity: "info",
        message: "This file isn't part of an Ascribe project (no ascribe.toml above it).",
      },
    ]);
  });

  it("names the project and its content root for a file outside the content root", () => {
    const outside = "isn't a source of the project (it is outside the content root)";
    const problems = previewProblems(
      situation({
        file: "/repo/examples/quill/README.md",
        result: answer({
          problems: [
            { severity: "warning", message: "ascribe.toml currently has errors" },
            { severity: "info", message: `/repo/examples/quill/README.md ${outside}` },
          ],
        }),
      }),
    );
    expect(problems).toEqual([
      { severity: "warning", message: "ascribe.toml currently has errors" },
      {
        severity: "info",
        message:
          "examples/quill/README.md is in the project examples/quill, but outside its content root (examples/quill/docs), so there is no page to preview.",
      },
    ]);
    expect(problems.map((p) => p.message).join("\n")).not.toContain("not part of");
  });

  it("compares Windows paths to the content root without regard to case or separator", () => {
    const windows = situation({
      file: "c:\\Repo\\Docs\\_fragments\\a.md",
      result: answer({
        contentRoot: "C:\\repo\\docs",
        problems: [{ severity: "info", message: "_fragments/a.md is a fragment." }],
      }),
    });
    expect(previewProblems(windows)).toEqual([
      { severity: "info", message: "_fragments/a.md is a fragment." },
    ]);
    const outside = situation({
      file: "c:\\Repo\\README.md",
      result: answer({ contentRoot: "C:\\repo\\docs" }),
    });
    expect(previewProblems(outside)[0]?.message).toMatch(/outside its content root/);
  });

  it("keeps the server's own reasons for a file in the content root", () => {
    const fragment = { severity: "info" as const, message: "a.md is a fragment." };
    expect(previewProblems(situation({ result: answer({ problems: [fragment] }) }))).toEqual([
      fragment,
    ]);
    const page = answer({
      page: {
        path: "index.md",
        route: "/",
        title: null,
        frontmatter: {},
        html: "",
        assets: [],
        links: [],
        sections: [],
      },
      problems: [{ severity: "warning", message: "an image is missing" }],
    });
    expect(previewProblems(situation({ result: page }))).toEqual(page.problems);
  });

  it("says the project's server failed, and offers its output", () => {
    expect(previewProblems(situation({ state: "failed", result: undefined }))).toEqual([
      {
        severity: "error",
        message:
          "The language server for examples/quill failed, so there is nothing to preview. Its output says why.",
        action: "showOutput",
      },
    ]);
  });

  it("says the project's server isn't running when it gave no answer", () => {
    const [problem] = previewProblems(situation({ state: "running", result: undefined }));
    expect(problem?.message).toBe(
      "The language server for examples/quill isn't running, so there is nothing to preview.",
    );
    expect(problem?.action).toBe("showOutput");
  });
});
