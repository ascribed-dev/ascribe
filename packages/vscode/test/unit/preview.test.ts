import { describe, expect, it } from "vitest";
import { contentSecurityPolicy, shellHtml } from "../../src/preview/html.js";
import { canonicalReference, isExternal, splitFragment } from "../../src/preview/refs.js";

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
