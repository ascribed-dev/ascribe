/* The search script of an archive written by `ascribe versions archive`, generated from
   packages/archive by `pnpm --filter @ascribed/archive embed`. Don't edit it. */
"use strict";
(() => {
  // src/search.ts
  function words(query) {
    return [
      ...new Set(
        query.toLowerCase().split(/\s+/u).filter((w) => w.length > 0)
      )
    ];
  }
  var ORDER = ["title", "heading", "text"];
  var BEFORE = 50;
  var LENGTH = 160;
  function search(index, query) {
    const terms = words(query);
    if (terms.length === 0) return [];
    const has = (lower) => terms.every((t) => lower.includes(t));
    const found = { title: [], heading: [], text: [] };
    for (const page of index.pages) {
      if (has(lowered(page, page.title))) {
        const first = page.sections.find((s) => s.text.length > 0)?.text ?? "";
        found.title.push(result("title", page, null, around(first, terms)));
      }
      for (const section of page.sections) {
        if (section.heading.length > 0 && has(section.heading.toLowerCase())) {
          found.heading.push(result("heading", page, section, around(section.text, terms)));
        } else if (has(lowered(section, `${section.heading} ${section.text}`))) {
          found.text.push(result("text", page, section, around(section.text, terms)));
        }
      }
    }
    const seen = /* @__PURE__ */ new Set();
    return ORDER.flatMap((rank) => found[rank]).filter((r) => !seen.has(r.href) && seen.add(r.href));
  }
  var lowercased = /* @__PURE__ */ new WeakMap();
  function lowered(of, text) {
    let lower = lowercased.get(of);
    if (lower === void 0) {
      lower = text.toLowerCase();
      lowercased.set(of, lower);
    }
    return lower;
  }
  function result(rank, page, section, context) {
    const id = section?.id ?? "";
    return {
      rank,
      href: id.length > 0 ? `${page.href}#${id}` : page.href,
      title: page.title,
      heading: section?.heading ?? "",
      context
    };
  }
  function around(text, terms) {
    if (text.length === 0) return [];
    let lower = text.toLowerCase();
    const shown = lower.length === text.length ? text : lower;
    const first = Math.min(
      ...terms.map((t) => lower.indexOf(t)).filter((i2) => i2 >= 0),
      Number.MAX_SAFE_INTEGER
    );
    const at = first === Number.MAX_SAFE_INTEGER ? 0 : first;
    let start = Math.max(0, at - BEFORE);
    if (start > 0) {
      const space = shown.indexOf(" ", start);
      start = space >= 0 && space < at ? space + 1 : start;
    }
    let end = Math.min(shown.length, start + LENGTH);
    if (end < shown.length) {
      const space = shown.lastIndexOf(" ", end);
      end = space > at ? space : end;
    }
    const cut = shown.slice(start, end);
    lower = lower.slice(start, end);
    const pieces = [];
    const push = (piece) => {
      if (piece.text.length > 0) pieces.push(piece);
    };
    if (start > 0) push({ text: "…", match: false });
    let i = 0;
    while (i < cut.length) {
      let next = -1;
      let length = 0;
      for (const t of terms) {
        const j = lower.indexOf(t, i);
        if (j >= 0 && (next < 0 || j < next || j === next && t.length > length)) {
          next = j;
          length = t.length;
        }
      }
      if (next < 0) {
        push({ text: cut.slice(i), match: false });
        break;
      }
      push({ text: cut.slice(i, next), match: false });
      push({ text: cut.slice(next, next + length), match: true });
      i = next + length;
    }
    if (end < shown.length) push({ text: "…", match: false });
    return pieces;
  }

  // src/index.ts
  var SHOWN = 20;
  function mount(script) {
    const doc = script.ownerDocument;
    const indexSrc = script.dataset["index"] ?? "search-index.js";
    const root = script.dataset["root"] ?? "./";
    const box = doc.createElement("div");
    box.className = "archive-search";
    box.setAttribute("role", "search");
    const label = doc.createElement("label");
    label.htmlFor = "archive-search-input";
    label.textContent = script.dataset["label"] ?? "Search this version";
    const input = doc.createElement("input");
    input.type = "search";
    input.id = "archive-search-input";
    input.autocomplete = "off";
    input.spellcheck = false;
    input.setAttribute("aria-describedby", "archive-search-status");
    input.setAttribute("aria-controls", "archive-search-results");
    const status = doc.createElement("p");
    status.id = "archive-search-status";
    status.className = "archive-search-status";
    status.setAttribute("role", "status");
    const list = doc.createElement("ol");
    list.id = "archive-search-results";
    list.className = "archive-search-results";
    list.hidden = true;
    box.append(label, input, status, list);
    script.replaceWith(box);
    let loading;
    const load = () => {
      loading ??= new Promise((resolve) => {
        if (globalThis.ascribeArchiveIndex !== void 0) {
          resolve(globalThis.ascribeArchiveIndex);
          return;
        }
        const tag = doc.createElement("script");
        tag.src = indexSrc;
        tag.addEventListener("load", () => resolve(globalThis.ascribeArchiveIndex));
        tag.addEventListener("error", () => resolve(void 0));
        doc.head.append(tag);
      });
      return loading;
    };
    const show = (index, query) => {
      list.replaceChildren();
      if (words(query).length === 0) {
        list.hidden = true;
        status.textContent = "";
        return;
      }
      if (index === void 0) {
        list.hidden = true;
        status.textContent = "The search index couldn't be loaded.";
        return;
      }
      const results = search(index, query);
      for (const result2 of results.slice(0, SHOWN)) list.append(item(doc, result2, root));
      list.hidden = results.length === 0;
      status.textContent = summary(results.length, query);
    };
    let latest = "";
    const run = async () => {
      const query = input.value;
      latest = query;
      if (loading === void 0 && words(query).length > 0)
        status.textContent = "Loading the search index…";
      const index = await load();
      if (latest === query) show(index, query);
    };
    input.addEventListener("focus", () => void load());
    input.addEventListener("input", () => void run());
    input.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown") {
        const first = list.querySelector("a");
        if (first !== null) {
          event.preventDefault();
          first.focus();
        }
      } else if (event.key === "Escape" && input.value !== "") {
        event.preventDefault();
        input.value = "";
        void run();
      }
    });
    list.addEventListener("keydown", (event) => {
      const links = [...list.querySelectorAll("a")];
      const at = links.indexOf(doc.activeElement);
      if (at < 0) return;
      if (event.key === "ArrowDown" || event.key === "ArrowUp") {
        event.preventDefault();
        const next = event.key === "ArrowDown" ? at + 1 : at - 1;
        (links[next] ?? (next < 0 ? input : links[at]))?.focus();
      } else if (event.key === "Escape") {
        event.preventDefault();
        input.focus();
      }
    });
    return box;
  }
  function summary(count, query) {
    const quoted = `“${query.trim()}”`;
    if (count === 0) return `Nothing found for ${quoted}.`;
    if (count > SHOWN) return `${count} results for ${quoted}; the first ${SHOWN} are shown.`;
    return `${count} ${count === 1 ? "result" : "results"} for ${quoted}.`;
  }
  function item(doc, result2, root) {
    const li = doc.createElement("li");
    const link = doc.createElement("a");
    link.href = root + result2.href;
    const where = doc.createElement("span");
    where.className = "archive-search-where";
    where.textContent = result2.heading.length > 0 ? `${result2.title} › ${result2.heading}` : result2.title;
    link.append(where);
    if (result2.context.length > 0) {
      const context = doc.createElement("span");
      context.className = "archive-search-context";
      for (const piece of result2.context) {
        if (piece.match) {
          const mark = doc.createElement("mark");
          mark.textContent = piece.text;
          context.append(mark);
        } else {
          context.append(piece.text);
        }
      }
      link.append(context);
    }
    li.append(link);
    return li;
  }
  var current = typeof document === "undefined" ? null : document.currentScript;
  if (current instanceof HTMLScriptElement && current.dataset["index"] !== void 0) mount(current);
})();
