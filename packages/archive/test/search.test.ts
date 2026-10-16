// @vitest-environment jsdom
import { afterEach, describe, expect, it } from "vitest";
import { SHOWN, mount, summary } from "../src/index.js";
import { around, search, words, type Index } from "../src/search.js";

const index: Index = {
  version: 1,
  pages: [
    {
      href: "index.html",
      title: "Quill",
      sections: [{ id: "", heading: "", text: "Quill syncs your docs repository." }],
    },
    {
      href: "install-agent.html",
      title: "Install the agent",
      sections: [
        { id: "", heading: "", text: "The agent watches your docs." },
        {
          id: "connect",
          heading: "Connect to Quill",
          text: "Run quill login --token $TOKEN to connect.",
        },
        { id: "proxy", heading: "Behind a proxy", text: "Set HTTPS_PROXY before you connect." },
      ],
    },
    {
      href: "guide/webhooks.html",
      title: "Webhooks",
      sections: [
        { id: "retries", heading: "Retries", text: "Quill retries a webhook three times." },
      ],
    },
  ],
};

describe("search", () => {
  it("lists title matches, then heading matches, then text, each in the sidebar's order", () => {
    const results = search(index, "quill");
    expect(results.map((r) => [r.rank, r.href])).toEqual([
      ["title", "index.html"],
      ["heading", "install-agent.html#connect"],
      ["text", "guide/webhooks.html#retries"],
    ]);
  });

  it("needs every word, in any order and any case, inside longer words too", () => {
    expect(search(index, "CONNECT proxy").map((r) => r.href)).toEqual(["install-agent.html#proxy"]);
    expect(search(index, "retr").map((r) => r.href)).toEqual(["guide/webhooks.html#retries"]);
    expect(search(index, "proxy webhook")).toEqual([]);
    expect(search(index, "   ")).toEqual([]);
  });

  it("finds a word in code", () => {
    expect(search(index, "--token").map((r) => r.href)).toEqual(["install-agent.html#connect"]);
  });

  it("marks each word in the words around the match", () => {
    expect(around("Run quill login --token $TOKEN to connect.", words("token"))).toEqual([
      { text: "Run quill login --", match: false },
      { text: "token", match: true },
      { text: " $", match: false },
      { text: "TOKEN", match: true },
      { text: " to connect.", match: false },
    ]);
    const long = `${"word ".repeat(40)}needle ${"more ".repeat(60)}`;
    const pieces = around(long, ["needle"]);
    expect(pieces[0]).toEqual({ text: "…", match: false });
    expect(pieces.at(-1)).toEqual({ text: "…", match: false });
    expect(pieces.some((p) => p.match && p.text === "needle")).toBe(true);
  });

  it("says how many it found", () => {
    expect(summary(0, " nope ")).toBe("Nothing found for “nope”.");
    expect(summary(1, "a")).toBe("1 result for “a”.");
    expect(summary(SHOWN + 5, "a")).toBe(
      `${SHOWN + 5} results for “a”; the first ${SHOWN} are shown.`,
    );
  });
});

describe("the box", () => {
  afterEach(() => {
    document.body.replaceChildren();
    document.head.replaceChildren();
    globalThis.ascribeArchiveIndex = undefined;
  });

  function box(): HTMLElement {
    const script = document.createElement("script");
    script.dataset["index"] = "../search-index.js";
    script.dataset["root"] = "../";
    const nav = document.createElement("nav");
    nav.append(script);
    document.body.append(nav);
    return mount(script);
  }

  async function type(input: HTMLInputElement, text: string): Promise<void> {
    input.value = text;
    input.dispatchEvent(new Event("input"));
    await new Promise((resolve) => setTimeout(resolve, 0));
  }

  it("is drawn where its script was, and loads the index once it's used", async () => {
    const drawn = box();
    expect(drawn.parentElement?.tagName).toBe("NAV");
    expect(document.querySelector("nav > script")).toBeNull();
    const input = drawn.querySelector("input");
    if (input === null) throw new Error("no input");
    expect(drawn.querySelector("label")?.htmlFor).toBe(input.id);
    expect(document.head.querySelector("script")).toBeNull();
    input.dispatchEvent(new Event("focus"));
    const tag = document.head.querySelector("script");
    expect(tag?.getAttribute("src")).toBe("../search-index.js");
    globalThis.ascribeArchiveIndex = index;
    tag?.dispatchEvent(new Event("load"));
    await type(input, "proxy");
    const links = [...drawn.querySelectorAll<HTMLAnchorElement>("li a")];
    expect(links.map((a) => a.getAttribute("href"))).toEqual(["../install-agent.html#proxy"]);
    expect(links[0]?.querySelector(".archive-search-where")?.textContent).toBe(
      "Install the agent › Behind a proxy",
    );
    expect(drawn.querySelector("[role=status]")?.textContent).toBe("1 result for “proxy”.");
  });

  it("moves between the input and the results with the arrow keys, and clears with Escape", async () => {
    globalThis.ascribeArchiveIndex = index;
    const drawn = box();
    const input = drawn.querySelector("input");
    if (input === null) throw new Error("no input");
    await type(input, "quill");
    const links = [...drawn.querySelectorAll<HTMLAnchorElement>("li a")];
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(links[0]);
    links[0]?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(links[1]);
    links[1]?.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(document.activeElement).toBe(input);
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(input.value).toBe("");
    expect(drawn.querySelectorAll("li")).toHaveLength(0);
  });

  it("says so when the index can't be loaded", async () => {
    const drawn = box();
    const input = drawn.querySelector("input");
    if (input === null) throw new Error("no input");
    input.dispatchEvent(new Event("focus"));
    document.head.querySelector("script")?.dispatchEvent(new Event("error"));
    await type(input, "quill");
    expect(drawn.querySelector("[role=status]")?.textContent).toBe(
      "The search index couldn't be loaded.",
    );
  });
});
