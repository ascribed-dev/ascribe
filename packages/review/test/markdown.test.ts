// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { renderMarkdown, safeUrl } from "../src/overlay/markdown.js";

/** The rendered body's HTML, for comparing shapes. */
function html(text: string): string {
  const div = document.createElement("div");
  div.append(renderMarkdown(document, text));
  return div.innerHTML;
}

describe("renderMarkdown", () => {
  it("renders paragraphs, keeping a comment's line breaks", () => {
    expect(html("One\ntwo\n\nThree")).toBe("<p>One<br>two</p><p>Three</p>");
  });

  it("renders emphasis, strong emphasis, strikethrough, and code", () => {
    expect(html("*a* **b** _c_ __d__ ~~e~~ `f <g>`")).toBe(
      "<p><em>a</em> <strong>b</strong> <em>c</em> <strong>d</strong> <del>e</del> <code>f &lt;g&gt;</code></p>",
    );
  });

  it("leaves underscores inside words alone", () => {
    expect(html("snake_case_name")).toBe("<p>snake_case_name</p>");
  });

  it("renders links that open apart from the page", () => {
    expect(html("See [the guide](https://example.com/a?b=1).")).toBe(
      '<p>See <a href="https://example.com/a?b=1" target="_blank" rel="noopener noreferrer">the guide</a>.</p>',
    );
    expect(html("At https://example.com/x, and <mailto:a@example.com>")).toBe(
      '<p>At <a href="https://example.com/x" target="_blank" rel="noopener noreferrer">https://example.com/x</a>, and <a href="mailto:a@example.com" target="_blank" rel="noopener noreferrer">mailto:a@example.com</a></p>',
    );
  });

  it("renders lists and block quotes", () => {
    expect(html("- one\n- two\n  more\n\n1. first\n2. second")).toBe(
      "<ul><li>one</li><li>two<br>more</li></ul><ol><li>first</li><li>second</li></ol>",
    );
    expect(html("> quoted\n> *still*\n\nafter")).toBe(
      "<blockquote><p>quoted<br><em>still</em></p></blockquote><p>after</p>",
    );
  });

  it("renders fenced code as text", () => {
    expect(html("```js\nconst a = '<b>';\n```")).toBe(
      "<pre><code>const a = '&lt;b&gt;';</code></pre>",
    );
  });

  it("keeps raw HTML as text, so a script is inert", () => {
    const div = document.createElement("div");
    div.append(
      renderMarkdown(
        document,
        '<script>alert(1)</script>\n<img src=x onerror="alert(2)">\n<b onclick="x()">bold</b>',
      ),
    );
    expect(div.querySelector("script, img, b")).toBeNull();
    expect(div.textContent).toContain("<script>alert(1)</script>");
    for (const el of Array.from(div.querySelectorAll("*"))) {
      for (const attr of Array.from(el.attributes)) expect(attr.name).not.toMatch(/^on/);
    }
  });

  it("drops links that would run script or leave the web", () => {
    for (const url of [
      "javascript:alert(1)",
      "JaVaScRiPt:alert(1)",
      " java\tscript:alert(1)",
      "data:text/html,<script>alert(1)</script>",
      "vbscript:x",
      "file:///etc/passwd",
      "relative/path",
    ]) {
      const div = document.createElement("div");
      div.append(renderMarkdown(document, `[click](${url})`));
      expect(div.querySelector("a"), url).toBeNull();
      expect(div.textContent).toContain("click");
    }
  });

  it("makes an image a link to it, so nothing loads without a click", () => {
    expect(html("![diagram](https://example.com/d.png)")).toBe(
      '<p><a href="https://example.com/d.png" target="_blank" rel="noopener noreferrer">Image: diagram</a></p>',
    );
    expect(html("![x](javascript:alert(1))")).toBe("<p>Image: x</p>");
  });

  it("renders a heading as a strong line", () => {
    expect(html("## Summary")).toBe("<p><strong>Summary</strong></p>");
  });

  it("honors backslash escapes", () => {
    expect(html("\\*not emphasis\\*")).toBe("<p>*not emphasis*</p>");
  });

  it("finds the bracket that closes a link inside others", () => {
    expect(html("[[a](https://x.test/) b")).toBe(
      '<p>[<a href="https://x.test/" target="_blank" rel="noopener noreferrer">a</a> b</p>',
    );
  });

  it("renders a long body of unclosed markers in linear time", () => {
    // GitHub allows about 65,000 characters in a comment.
    for (const unit of [
      "[",
      "*a ",
      "_a ",
      "~~a ",
      "`` `",
      "[a](<",
      "http://[ ",
      "http://[",
      "**",
    ]) {
      const body = unit.repeat(Math.ceil(65_000 / unit.length));
      const start = performance.now();
      const text = html(body);
      expect(performance.now() - start, unit).toBeLessThan(2000);
      expect(text.length).toBeGreaterThan(0);
    }
  });
});

describe("safeUrl", () => {
  it("keeps web and email links only", () => {
    expect(safeUrl("https://example.com")).toBe("https://example.com/");
    expect(safeUrl("mailto:a@example.com")).toBe("mailto:a@example.com");
    expect(safeUrl("javascript:alert(1)")).toBeUndefined();
    expect(safeUrl("\u0001javascript:alert(1)")).toBeUndefined();
  });
});
