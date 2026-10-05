// @vitest-environment jsdom
import { beforeEach, describe, expect, it } from "vitest";
import {
  clearMarks,
  describeSource,
  findBlock,
  markChanges,
  parseSource,
  setShow,
  type Change,
} from "../src/marks/index.js";
import { blockText } from "../src/overlay/text.js";

/** A page rendered with anchors, in a root element. */
function page(html: string): HTMLElement {
  const root = document.createElement("article");
  root.innerHTML = html;
  document.body.append(root);
  return root;
}

const NOW = `
<h2 data-ascribe-source="guide.md:1-1">Install</h2>
<p data-ascribe-source="guide.md:3-3">Run the installer now.</p>
<p data-ascribe-source="guide.md:5-5">A brand new paragraph.</p>
<ul data-ascribe-source="guide.md:7-8">
<li data-ascribe-source="guide.md:7-7">Linux</li>
<li data-ascribe-source="guide.md:8-8">macOS</li>
</ul>
<h2 data-ascribe-source="guide.md:10-10">Configure</h2>
<p data-ascribe-source="guide.md:12-12">Moved here.</p>
<p data-ascribe-source="_fragments/pre.md:1-1" data-ascribe-via="guide.md:14">Node 22.</p>
`;

const WAS = `
<h2 data-ascribe-source="guide.md:1-1">Install</h2>
<p data-ascribe-source="guide.md:3-3">Run the <em>old</em> installer.</p>
<p data-ascribe-source="guide.md:5-5">Moved here.</p>
<p data-ascribe-source="guide.md:7-7">A paragraph that is <strong>gone</strong>.</p>
<ul data-ascribe-source="guide.md:9-11">
<li data-ascribe-source="guide.md:9-9">Linux</li>
<li data-ascribe-source="guide.md:10-10">Windows</li>
<li data-ascribe-source="guide.md:11-11">macOS</li>
</ul>
<h2 data-ascribe-source="guide.md:13-13">Configure</h2>
<p data-ascribe-source="_fragments/pre.md:1-1" data-ascribe-via="guide.md:15">Node 20.</p>
`;

const a = (source: string, via: string[] = []) => ({ source, via });

const CHANGES: Change[] = [
  {
    kind: "changed",
    now: a("guide.md:3-3"),
    was: a("guide.md:3-3"),
    words: {
      now: [[18, 22]],
      was: [
        [8, 11],
        [21, 22],
      ],
      now_text: "Run the installer now.",
      was_text: "Run the old installer.",
    },
  },
  { kind: "added", now: a("guide.md:5-5") },
  {
    kind: "removed",
    was: a("guide.md:7-7"),
    after: a("guide.md:5-5"),
    text: "A paragraph that is gone.",
  },
  {
    kind: "removed",
    was: a("guide.md:10-10"),
    after: a("guide.md:7-7"),
    parent: a("guide.md:7-8"),
    text: "Windows",
  },
  { kind: "moved", now: a("guide.md:12-12"), was: a("guide.md:5-5"), after: a("guide.md:3-3") },
  {
    kind: "changed",
    now: a("_fragments/pre.md:1-1", ["guide.md:14"]),
    was: a("_fragments/pre.md:1-1", ["guide.md:15"]),
    words: { now: [[5, 7]], was: [[5, 7]], now_text: "Node 22.", was_text: "Node 20." },
  },
];

function wasPage(): HTMLElement {
  const root = document.createElement("div");
  root.innerHTML = WAS;
  return root;
}

beforeEach(() => {
  document.body.innerHTML = "";
});

describe("anchors", () => {
  it("parses and describes a source", () => {
    expect(parseSource("guides/my%20page.md:12-14")).toEqual({
      path: "guides/my page.md",
      first: 12,
      last: 14,
    });
    expect(parseSource("nope")).toBeUndefined();
    expect(describeSource("guides/install.md:12-12")).toBe("guides/install.md:12");
    expect(describeSource("_f/a.md:3-9", "guides/install.md:20")).toBe(
      "_f/a.md:3-9 via guides/install.md:20",
    );
  });

  it("finds a block by its exact anchor, and its includes must match", () => {
    const root = page(NOW);
    expect(findBlock(root, a("guide.md:3-3"))?.textContent).toBe("Run the installer now.");
    expect(findBlock(root, a("_fragments/pre.md:1-1"))).toBeNull();
    expect(findBlock(root, a("_fragments/pre.md:1-1", ["guide.md:14"]))?.textContent).toBe(
      "Node 22.",
    );
  });

  it("finds a block with no element of its own by the smallest one containing it", () => {
    const root = page(`
      <ul data-ascribe-source="a.md:1-4">
        <li data-ascribe-source="a.md:1-2">One, and
        more</li>
        <li data-ascribe-source="a.md:3-4">Two</li>
      </ul>`);
    expect(findBlock(root, a("a.md:2-2"))?.textContent?.trim()).toMatch(/^One/);
    expect(findBlock(root, a("a.md:9-9"))).toBeNull();
  });
});

describe("markChanges", () => {
  it("marks an added block with a label", () => {
    const root = page(NOW);
    markChanges(root, [{ kind: "added", now: a("guide.md:5-5") }]);
    const block = findBlock(root, a("guide.md:5-5"));
    expect(block?.getAttribute("data-ascribe-change")).toBe("added");
    expect(block?.querySelector(".ascribe-label")?.textContent).toBe("Added");
    expect(root.getAttribute("data-ascribe-show")).toBe("changes");
  });

  it("puts a label before a block that can't start with one", () => {
    const root = page(NOW);
    markChanges(root, [{ kind: "added", now: a("guide.md:7-8") }]);
    const list = findBlock(root, a("guide.md:7-8"));
    expect(list?.previousElementSibling?.textContent).toBe("Added");
    expect(list?.querySelector(".ascribe-label")).toBeNull();
  });

  it("highlights the changed words, with the removed words where they were", () => {
    const root = page(NOW);
    markChanges(root, [CHANGES[0] as Change]);
    const block = findBlock(root, a("guide.md:3-3"));
    expect(block?.getAttribute("data-ascribe-change")).toBe("changed");
    expect(Array.from(block?.querySelectorAll("ins") ?? []).map((e) => e.textContent)).toEqual([
      "now.",
    ]);
    expect(Array.from(block?.querySelectorAll("del") ?? []).map((e) => e.textContent)).toEqual([
      "old",
      ".",
    ]);
    // Reading order: Run the <del>old</del> installer <del>.</del><ins>now.</ins>
    const text = Array.from(block?.childNodes ?? [])
      .filter((n) => !(n instanceof Element && n.classList.contains("ascribe-label")))
      .map((n) => (n instanceof Element ? `[${n.localName}:${n.textContent}]` : n.textContent))
      .join("");
    expect(text).toBe("Run the [del:old]installer [del:.][ins:now.]");
  });

  it("highlights words across inline elements and skips text it can't match", () => {
    const root = page(`<p data-ascribe-source="a.md:1-1">Use the <code>quill</code>
      command today.</p>`);
    markChanges(root, [
      {
        kind: "changed",
        now: a("a.md:1-1"),
        was: a("a.md:1-1"),
        words: {
          now: [[8, 21]],
          was: [[8, 11]],
          now_text: "Use the quill command today.",
          was_text: "Use the cli today.",
        },
      },
    ]);
    const ins = Array.from(root.querySelectorAll("ins")).map((e) => e.textContent);
    expect(ins.join("|")).toBe("quill|\n      command");

    const other = page(`<p data-ascribe-source="b.md:1-1">Something else entirely.</p>`);
    markChanges(other, [
      {
        kind: "changed",
        now: a("b.md:1-1"),
        was: a("b.md:1-1"),
        words: { now: [[0, 3]], was: [[0, 3]], now_text: "New text.", was_text: "Old text." },
      },
    ]);
    expect(other.querySelector("ins")).toBeNull();
    expect(other.querySelector("p")?.getAttribute("data-ascribe-change")).toBe("changed");
  });

  it("puts a removed block after the block it followed, from the old page", () => {
    const root = page(NOW);
    markChanges(root, CHANGES.slice(1, 3), { was: wasPage() });
    const added = findBlock(root, a("guide.md:5-5"));
    const removed = added?.nextElementSibling as HTMLElement;
    expect(removed.getAttribute("data-ascribe-change")).toBe("removed");
    expect(removed.querySelector(".ascribe-label")?.textContent).toBe("Removed");
    // Rendered from the base: its markup, without anchors.
    expect(removed.querySelector(".ascribe-removed-body strong")?.textContent).toBe("gone");
    expect(removed.querySelector("[data-ascribe-source]")).toBeNull();
    expect(removed.getAttribute("data-ascribe-was-source")).toBe("guide.md:7-7");
    const toggle = removed.querySelector("button");
    expect(toggle?.textContent).toBe("Show");
    toggle?.click();
    expect(removed.classList.contains("ascribe-open")).toBe(true);
    expect(toggle?.textContent).toBe("Collapse");
  });

  it("puts a removed item in its list as an item, and uses the text without the old page", () => {
    const root = page(NOW);
    markChanges(root, [CHANGES[3] as Change]);
    const linux = findBlock(root, a("guide.md:7-7"));
    const removed = linux?.nextElementSibling;
    expect(removed?.localName).toBe("li");
    expect(removed?.textContent).toContain("Windows");
  });

  it("puts a removed block first in its container when it followed nothing", () => {
    const root = page(NOW);
    markChanges(root, [
      { kind: "removed", was: a("guide.md:1-1"), text: "Old title" },
      { kind: "removed", was: a("guide.md:2-2"), text: "Old intro" },
    ]);
    const first = root.firstElementChild;
    expect(first?.textContent).toContain("Old title");
    expect(first?.nextElementSibling?.textContent).toContain("Old intro");
  });

  it("links a moved block and the stub at its old place", () => {
    const root = page(NOW);
    const marks = markChanges(root, [CHANGES[4] as Change], { was: wasPage() });
    const moved = findBlock(root, a("guide.md:12-12"));
    expect(moved?.getAttribute("data-ascribe-change")).toBe("moved");
    expect(moved?.querySelector(".ascribe-label")?.textContent).toBe("Moved");
    const back = moved?.querySelector("a");
    expect(back?.textContent).toBe("from “Install” ↑");
    const stub = root.querySelector(".ascribe-moved-from") as HTMLElement;
    expect(stub.previousElementSibling).toBe(findBlock(root, a("guide.md:3-3")));
    expect(back?.getAttribute("href")).toBe(`#${stub.id}`);
    const forward = stub.querySelector("a");
    expect(forward?.textContent).toBe("to “Configure” ↓");
    expect(forward?.getAttribute("href")).toBe(`#${moved?.id}`);
    expect(stub.textContent).toContain("A paragraph moved from here");
    // As it was, the stub holds the block.
    expect(stub.querySelector("[data-ascribe-was-only]")?.textContent).toBe("Moved here.");
    expect(marks.map((m) => m.change.kind)).toEqual(["moved"]);
  });

  it("keeps a changed block's old rendering for the page as it was", () => {
    const root = page(NOW);
    markChanges(root, [CHANGES[0] as Change], { was: wasPage() });
    const block = findBlock(root, a("guide.md:3-3"));
    expect(block?.hasAttribute("data-ascribe-has-was")).toBe(true);
    const copy = block?.nextElementSibling;
    expect(copy?.hasAttribute("data-ascribe-was-only")).toBe(true);
    expect(copy?.innerHTML).toBe("Run the <em>old</em> installer.");
  });

  it("returns every mark in page order, and marks blocks through includes", () => {
    const root = page(NOW);
    const marks = markChanges(root, CHANGES, { was: wasPage() });
    expect(marks.map((m) => m.change.kind)).toEqual([
      "changed",
      "added",
      "removed",
      "removed",
      "moved",
      "changed",
    ]);
    const fragment = findBlock(root, a("_fragments/pre.md:1-1", ["guide.md:14"]));
    expect(fragment?.querySelector("ins")?.textContent).toBe("22");
    // The removed paragraph comes before the stub that follows it, and both
    // after the copy of the changed block they follow.
    const order = Array.from(root.children).map(
      (el) => el.getAttribute("data-ascribe-change") ?? el.localName,
    );
    expect(order.slice(0, 7)).toEqual([
      "h2",
      "changed",
      "p",
      "moved-from",
      "added",
      "removed",
      "ul",
    ]);
  });

  it("switches what the page shows", () => {
    const root = page(NOW);
    markChanges(root, CHANGES);
    setShow(root, "was");
    expect(root.getAttribute("data-ascribe-show")).toBe("was");
  });

  it("clears every mark, leaving the page as it was rendered", () => {
    const root = page(NOW);
    const before = root.innerHTML;
    markChanges(root, CHANGES, { was: wasPage() });
    expect(root.innerHTML).not.toBe(before);
    clearMarks(root);
    root.removeAttribute("data-ascribe-show");
    root.classList.remove("ascribe-marks");
    root.removeAttribute("class");
    expect(root.innerHTML).toBe(before);
  });

  it("marks again from scratch", () => {
    const root = page(NOW);
    markChanges(root, CHANGES);
    const marks = markChanges(root, CHANGES);
    expect(marks).toHaveLength(6);
    expect(root.querySelectorAll(".ascribe-removed")).toHaveLength(2);
    expect(root.querySelectorAll(".ascribe-label")).toHaveLength(6);
  });
});

describe("hints on what hides changes", () => {
  const TABS = `
<ascribe-tabs data-ascribe-source="guide.md:1-9">
<div role="tablist"><button role="tab">npm</button><button role="tab">pnpm</button><button role="tab">yarn</button></div>
<ascribe-tab label="npm" data-ascribe-source="guide.md:1-3"><p data-ascribe-source="guide.md:2-2">npm i</p></ascribe-tab>
<ascribe-tab label="pnpm" hidden data-ascribe-source="guide.md:4-6"><p data-ascribe-source="guide.md:5-5">pnpm add</p></ascribe-tab>
<ascribe-tab label="yarn" hidden data-ascribe-source="guide.md:7-9"><p data-ascribe-source="guide.md:8-8">yarn add</p></ascribe-tab>
</ascribe-tabs>
<details data-ascribe-source="guide.md:11-15"><summary>Why?</summary>
<p data-ascribe-source="guide.md:12-12">One.</p>
<p data-ascribe-source="guide.md:14-14">Two.</p>
</details>`;

  it("says on a tab's label and a summary what changed inside", () => {
    const root = page(TABS);
    markChanges(root, [
      { kind: "changed", now: a("guide.md:5-5"), was: a("guide.md:5-5") },
      { kind: "added", now: a("guide.md:7-9") },
      { kind: "added", now: a("guide.md:12-12") },
      { kind: "changed", now: a("guide.md:14-14"), was: a("guide.md:13-13") },
    ]);
    const buttons = root.querySelectorAll('[role="tab"]');
    const hint = (el: Element | null | undefined): string | undefined =>
      el?.querySelector(".ascribe-hint")?.textContent ?? undefined;
    expect(hint(buttons[0])).toBeUndefined();
    expect(hint(buttons[1])).toBe("1 change");
    expect(hint(buttons[2])).toBe("new");
    expect(buttons[2]?.querySelector(".ascribe-hint")?.getAttribute("data-ascribe-hint")).toBe(
      "added",
    );
    expect(hint(root.querySelector("summary"))).toBe("2 changes");
    // An added tab's label is in its panel, not among the group's panels.
    expect(findBlock(root, a("guide.md:7-9"))?.firstElementChild?.textContent).toBe("Added");
    // Not part of the blocks' text, and cleared with the marks.
    expect(blockText(findBlock(root, a("guide.md:11-15")) as HTMLElement)).toBe("Why?\nOne.\nTwo.");
    clearMarks(root);
    expect(root.querySelector(".ascribe-hint")).toBeNull();
  });

  it("waits for a tab list made after the marks, and stops when they're cleared", async () => {
    const list = /<div role="tablist">.*<\/div>/.exec(TABS)?.[0] ?? "";
    const root = page(TABS.replace(list, ""));
    markChanges(root, [{ kind: "added", now: a("guide.md:7-9") }]);
    expect(root.querySelector(".ascribe-hint")).toBeNull();
    const group = root.querySelector("ascribe-tabs") as HTMLElement;
    group.insertAdjacentHTML("afterbegin", list);
    await Promise.resolve();
    expect(root.querySelectorAll('[role="tab"]')[2]?.textContent).toBe("yarnnew");
    // Made again, as when the tabs are put back in the page.
    group.querySelector('[role="tablist"]')?.remove();
    group.insertAdjacentHTML("afterbegin", list);
    await Promise.resolve();
    expect(root.querySelectorAll('[role="tab"]')[2]?.textContent).toBe("yarnnew");
    clearMarks(root);
    group.querySelector('[role="tablist"]')?.remove();
    group.insertAdjacentHTML("afterbegin", list);
    await Promise.resolve();
    expect(root.querySelector(".ascribe-hint")).toBeNull();
  });
});

describe("a marked block's text, for quoting", () => {
  it("leaves out the marks' labels and removed words, and keeps inserted ones", () => {
    const root = page(NOW);
    markChanges(root, CHANGES);
    const block = (source: string) =>
      root.querySelector(`[data-ascribe-source="${source}"]`) as Element;
    expect(blockText(block("guide.md:3-3"))).toBe("Run the installer now.");
    expect(blockText(block("guide.md:5-5"))).toBe("A brand new paragraph.");
    expect(blockText(block("guide.md:7-8"))).toBe("Linux\nmacOS");
  });
});
