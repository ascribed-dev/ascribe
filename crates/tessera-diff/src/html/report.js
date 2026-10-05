/* The script of `ascribe diff --format html`, generated from packages/review and
   packages/elements by `pnpm --filter @ascribed/review embed`. Don't edit it. */
"use strict";
(() => {
  // ../elements/src/group.ts
  var AscribeGroup = class extends HTMLElement {
  };

  // ../elements/src/tabs.ts
  var STORAGE_PREFIX = "ascribe-tabs:";
  var pageChoices = /* @__PURE__ */ new Map();
  var syncedGroups = /* @__PURE__ */ new Set();
  var idCounter = 0;
  function readChoice(dimension) {
    const chosen = pageChoices.get(dimension);
    if (chosen !== void 0) return chosen;
    try {
      return localStorage.getItem(STORAGE_PREFIX + dimension);
    } catch {
      return null;
    }
  }
  function writeChoice(dimension, value) {
    pageChoices.set(dimension, value);
    try {
      localStorage.setItem(STORAGE_PREFIX + dimension, value);
    } catch {
    }
  }
  function valuesOf(tab) {
    return (tab.getAttribute("value") ?? "").split(/\s+/).filter(Boolean);
  }
  var AscribeTab = class extends HTMLElement {
  };
  var AscribeTabs = class extends HTMLElement {
    #tablist = null;
    #tabs = [];
    #buttons = [];
    #selected = 0;
    #ready = false;
    connectedCallback() {
      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", () => this.#setup(), { once: true });
      } else {
        this.#setup();
      }
    }
    disconnectedCallback() {
      this.#teardown();
    }
    #setup() {
      if (!this.isConnected || this.#ready) return;
      this.#tabs = Array.from(this.children).filter(
        (child) => child.localName === "ascribe-tab"
      );
      if (this.#tabs.length === 0) return;
      this.#ready = true;
      const tablist = document.createElement("div");
      tablist.setAttribute("role", "tablist");
      this.#tablist = tablist;
      this.#buttons = this.#tabs.map((tab) => {
        if (!tab.id) tab.id = `ascribe-tab-${++idCounter}`;
        const button2 = document.createElement("button");
        button2.type = "button";
        button2.id = `${tab.id}-button`;
        button2.setAttribute("role", "tab");
        button2.setAttribute("aria-controls", tab.id);
        button2.textContent = tab.getAttribute("label") ?? "";
        button2.addEventListener("click", () => this.#choose(this.#buttons.indexOf(button2)));
        button2.addEventListener("keydown", (event) => this.#onKeydown(event));
        tablist.append(button2);
        tab.setAttribute("role", "tabpanel");
        tab.setAttribute("aria-labelledby", button2.id);
        return button2;
      });
      this.prepend(tablist);
      const sync = this.getAttribute("sync");
      let initial = 0;
      if (sync !== null) {
        syncedGroups.add(this);
        const remembered = readChoice(sync);
        const match = remembered === null ? -1 : this.#indexOfValue(remembered);
        if (match >= 0) initial = match;
      }
      this.#select(initial);
    }
    #teardown() {
      if (!this.#ready) return;
      syncedGroups.delete(this);
      this.#tablist?.remove();
      for (const tab of this.#tabs) {
        tab.removeAttribute("role");
        tab.removeAttribute("aria-labelledby");
        tab.removeAttribute("hidden");
      }
      this.#tablist = null;
      this.#tabs = [];
      this.#buttons = [];
      this.#ready = false;
    }
    #indexOfValue(value) {
      return this.#tabs.findIndex((tab) => valuesOf(tab).includes(value));
    }
    /** Select a tab without recording or syncing the choice. */
    #select(index) {
      this.#selected = index;
      this.#tabs.forEach((tab, i) => {
        tab.toggleAttribute("hidden", i !== index);
      });
      this.#buttons.forEach((button2, i) => {
        button2.setAttribute("aria-selected", String(i === index));
        button2.tabIndex = i === index ? 0 : -1;
      });
    }
    /** The reader picked a tab: select it and, for a synced group, tell the rest. */
    #choose(index) {
      this.#select(index);
      const sync = this.getAttribute("sync");
      const tab = this.#tabs[index];
      const choice = tab ? valuesOf(tab)[0] : void 0;
      if (sync === null || choice === void 0) return;
      writeChoice(sync, choice);
      for (const group of syncedGroups) {
        if (group !== this && group.getAttribute("sync") === sync) group.#follow(choice);
      }
    }
    /** Another group with the same `sync` made a choice; keep our selection if we have no match. */
    #follow(value) {
      const index = this.#indexOfValue(value);
      if (index >= 0) this.#select(index);
    }
    #onKeydown(event) {
      const last = this.#tabs.length - 1;
      let next;
      switch (event.key) {
        case "ArrowRight":
          next = this.#selected === last ? 0 : this.#selected + 1;
          break;
        case "ArrowLeft":
          next = this.#selected === 0 ? last : this.#selected - 1;
          break;
        case "Home":
          next = 0;
          break;
        case "End":
          next = last;
          break;
        default:
          return;
      }
      event.preventDefault();
      this.#choose(next);
      this.#buttons[next]?.focus();
    }
  };

  // ../elements/src/index.ts
  var registry = [
    ["ascribe-tabs", AscribeTabs],
    ["ascribe-tab", AscribeTab],
    ["ascribe-group", AscribeGroup]
  ];
  for (const [name, constructor] of registry) {
    if (!customElements.get(name)) customElements.define(name, constructor);
  }

  // src/marks/index.ts
  var LABELS = {
    added: "Added",
    changed: "Changed",
    removed: "Removed",
    moved: "Moved"
  };
  var LABEL_INSIDE = /* @__PURE__ */ new Set([
    "p",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "dt",
    "dd",
    "summary",
    "td",
    "th",
    // A tab's panel: before it would put the label among the tab list's panels.
    "ascribe-tab"
  ]);
  function parseSource(source) {
    const colon = source.lastIndexOf(":");
    if (colon < 0) return void 0;
    const match = /^(\d+)-(\d+)$/.exec(source.slice(colon + 1));
    if (!match) return void 0;
    let path = source.slice(0, colon);
    try {
      path = decodeURIComponent(path);
    } catch {
    }
    return { path, first: Number(match[1]), last: Number(match[2]) };
  }
  function describeSource(source, via) {
    const parsed = parseSource(source);
    let text = source;
    if (parsed) {
      const lines = parsed.first === parsed.last ? `${parsed.first}` : `${parsed.first}-${parsed.last}`;
      text = `${parsed.path}:${lines}`;
    }
    if (via) {
      const outer = via.split(" ").map((v) => {
        const colon = v.lastIndexOf(":");
        if (colon < 0) return v;
        try {
          return `${decodeURIComponent(v.slice(0, colon))}${v.slice(colon)}`;
        } catch {
          return v;
        }
      });
      text += ` via ${outer.join(", ")}`;
    }
    return text;
  }
  function viaOf(element) {
    return element.getAttribute("data-ascribe-via") ?? "";
  }
  function findBlock(root, anchor) {
    const via = anchor.via.join(" ");
    const anchored = Array.from(root.querySelectorAll("[data-ascribe-source]"));
    const exact = anchored.find(
      (el) => el.getAttribute("data-ascribe-source") === anchor.source && viaOf(el) === via
    );
    if (exact) return exact;
    const want = parseSource(anchor.source);
    if (!want) return null;
    let best = null;
    let bestSpan = Infinity;
    for (const el of anchored) {
      if (viaOf(el) !== via) continue;
      const has = parseSource(el.getAttribute("data-ascribe-source") ?? "");
      if (!has || has.path !== want.path) continue;
      if (has.first > want.first || has.last < want.last) continue;
      const span = has.last - has.first;
      if (span < bestSpan) {
        best = el;
        bestSpan = span;
      }
    }
    return best;
  }
  function ui(doc, tag, className, text) {
    const el = doc.createElement(tag);
    el.className = className;
    el.setAttribute("data-ascribe-ui", "");
    if (text !== void 0) el.textContent = text;
    return el;
  }
  function addLabel(element, kind) {
    const doc = element.ownerDocument;
    const label = ui(doc, "span", "ascribe-label", LABELS[kind]);
    label.setAttribute("data-ascribe-label", kind);
    if (LABEL_INSIDE.has(element.localName)) {
      element.prepend(label);
    } else {
      label.classList.add("ascribe-label-before");
      element.before(label);
    }
  }
  function contentOf(holder, before) {
    const copy = copyOf(before);
    if (holder.localName === "li" && copy.localName === "li") return Array.from(copy.childNodes);
    return [copy];
  }
  function copyOf(element) {
    const copy = element.cloneNode(true);
    for (const el of [copy, ...Array.from(copy.querySelectorAll("*"))]) {
      el.removeAttribute("data-ascribe-source");
      el.removeAttribute("data-ascribe-via");
      el.removeAttribute("id");
    }
    return copy;
  }
  function holderFor(container) {
    return container.localName === "ul" || container.localName === "ol" ? "li" : "div";
  }
  function headingBefore(root, node) {
    const headings = Array.from(root.querySelectorAll("h1, h2, h3, h4, h5, h6"));
    let found;
    for (const heading of headings) {
      if (heading === node || heading.contains(node)) {
        found = heading;
        break;
      }
      if (heading.compareDocumentPosition(node) & Node.DOCUMENT_POSITION_FOLLOWING) found = heading;
      else break;
    }
    return found ? ownText(found).trim() : void 0;
  }
  function ownText(element) {
    let text = "";
    const walk = (node) => {
      for (const child of Array.from(node.childNodes)) {
        if (child.nodeType === Node.TEXT_NODE) text += child.nodeValue ?? "";
        else if (child instanceof Element && !child.hasAttribute("data-ascribe-ui")) walk(child);
      }
    };
    walk(element);
    return text.replace(/\s+/g, " ");
  }
  var Placer = class {
    constructor(root) {
      this.root = root;
    }
    root;
    /** The last element placed after each block, so several stay in order. */
    #last = /* @__PURE__ */ new Map();
    #first = /* @__PURE__ */ new Map();
    /** Puts `make(container)`'s element where `change` was. */
    place(change, make) {
      const after = change.after ? findBlock(this.root, change.after) : null;
      const parent = change.parent ? findBlock(this.root, change.parent) : null;
      if (after && after !== parent && after.parentElement) {
        let previous2 = this.#last.get(after) ?? after;
        for (let next = previous2.nextElementSibling; next && (next.hasAttribute("data-ascribe-was-only") || next.classList.contains("ascribe-move-after")); next = next.nextElementSibling) {
          previous2 = next;
        }
        const el2 = make(after.parentElement);
        previous2.after(el2);
        this.#last.set(after, el2);
        return el2;
      }
      const container = parent ?? this.root;
      const el = make(container);
      const previous = this.#first.get(container);
      if (previous) {
        previous.after(el);
      } else {
        const first = Array.from(container.children).find(
          (c) => c.hasAttribute("data-ascribe-source") || c.hasAttribute("data-ascribe-ui")
        );
        if (first) first.before(el);
        else container.append(el);
      }
      this.#first.set(container, el);
      return el;
    }
  };
  function textMap(element) {
    const chars = [];
    const at = [];
    let space = false;
    let spaceAt;
    const walk = (node) => {
      for (const child of Array.from(node.childNodes)) {
        if (child.nodeType === Node.TEXT_NODE) {
          const value = child.nodeValue ?? "";
          let offset = 0;
          for (const ch of value) {
            if (/\s/.test(ch)) {
              if (!space) spaceAt = { node: child, offset };
              space = true;
            } else {
              if (space && chars.length > 0 && spaceAt) {
                chars.push(" ");
                at.push(spaceAt);
              }
              space = false;
              chars.push(ch);
              at.push({ node: child, offset });
            }
            offset += ch.length;
          }
        } else if (child instanceof Element && !child.hasAttribute("data-ascribe-ui") && !child.hasAttribute("data-ascribe-source")) {
          walk(child);
        }
      }
    };
    walk(element);
    return { text: chars, at };
  }
  function wrapRange(map, start2, end, make) {
    const parts = [];
    for (let i = start2; i < end; i++) {
      const at = map.at[i];
      if (!at) continue;
      const width = charWidth(at);
      const part = parts[parts.length - 1];
      if (part && part.node === at.node) part.to = at.offset + width;
      else parts.push({ node: at.node, from: at.offset, to: at.offset + width });
    }
    for (const part of parts.reverse()) {
      const range = part.node.ownerDocument.createRange();
      range.setStart(part.node, part.from);
      range.setEnd(part.node, part.to);
      const wrapper = make();
      range.surroundContents(wrapper);
    }
  }
  function charWidth(at) {
    const code = at.node.nodeValue?.codePointAt(at.offset) ?? 0;
    return code > 65535 ? 2 : 1;
  }
  function markWords(element, words) {
    const map = textMap(element);
    const nowChars = Array.from(words.now_text);
    const text = map.text.join("");
    const offset = text === words.now_text ? 0 : findChars(map.text, nowChars);
    if (offset < 0) return false;
    const doc = element.ownerDocument;
    const wasChars = Array.from(words.was_text);
    const insertions = [];
    let removedBefore = 0;
    for (const [start2, end] of words.was) {
      const unchangedBefore = start2 - removedBefore;
      insertions.push({
        at: nowPosition(words.now, unchangedBefore),
        text: wasChars.slice(start2, end).join("")
      });
      removedBefore += end - start2;
    }
    const edits = [
      ...words.now.map(([start2, end]) => ({ at: start2, kind: "ins", end })),
      ...insertions.map((i) => ({ at: i.at, kind: "del", text: i.text }))
    ];
    edits.sort((a, b) => b.at - a.at || (a.kind === "del" ? -1 : 1));
    for (const edit of edits) {
      if (edit.kind === "ins") {
        wrapRange(map, offset + edit.at, offset + edit.end, () => ui(doc, "ins", "ascribe-ins"));
      } else {
        const del = ui(doc, "del", "ascribe-del", edit.text);
        insertAt(map, offset + edit.at, del, element);
      }
    }
    return true;
  }
  function nowPosition(ranges, unchanged) {
    let position = unchanged;
    for (const [start2, end] of ranges) {
      if (start2 < position) position += end - start2;
      else break;
    }
    return position;
  }
  function findChars(haystack, needle) {
    if (needle.length === 0) return -1;
    outer: for (let i = 0; i + needle.length <= haystack.length; i++) {
      for (let j = 0; j < needle.length; j++) if (haystack[i + j] !== needle[j]) continue outer;
      return i;
    }
    return -1;
  }
  function insertAt(map, index, node, element) {
    const at = map.at[index];
    if (at) {
      const rest = at.node.splitText(at.offset);
      rest.before(node);
      for (let i = index; i < map.at.length; i++) {
        const later = map.at[i];
        if (later && later.node === at.node && later.offset >= at.offset) {
          map.at[i] = { node: rest, offset: later.offset - at.offset };
        }
      }
      return;
    }
    const last = map.at[map.at.length - 1];
    if (last) {
      const rest = last.node.splitText(last.offset + charWidth(last));
      rest.before(node);
    } else {
      element.append(node);
    }
  }
  function markChanges(root, changes, options = {}) {
    clearMarks(root);
    const doc = root.ownerDocument;
    const was = options.was ?? null;
    const placer = new Placer(root);
    const marks = [];
    root.classList.add("ascribe-marks");
    if (!root.hasAttribute("data-ascribe-show")) root.setAttribute("data-ascribe-show", "changes");
    let moveCount = 0;
    for (const change of changes) {
      switch (change.kind) {
        case "added":
        case "changed": {
          const element = change.now ? findBlock(root, change.now) : null;
          if (!element || element.hasAttribute("data-ascribe-change")) break;
          element.setAttribute("data-ascribe-change", change.kind);
          addLabel(element, change.kind);
          if (change.kind === "changed") {
            if (change.was) element.setAttribute("data-ascribe-was-source", change.was.source);
            const before = was && change.was ? findBlock(was, change.was) : null;
            if (change.words) markWords(element, change.words);
            if (before) {
              const copy = copyOf(before);
              copy.setAttribute("data-ascribe-ui", "");
              copy.setAttribute("data-ascribe-was-only", "");
              element.after(copy);
              element.setAttribute("data-ascribe-has-was", "");
            }
          }
          marks.push({ change, element });
          break;
        }
        case "removed": {
          const before = was && change.was ? findBlock(was, change.was) : null;
          const element = placer.place(change, (container) => {
            const holder = ui(doc, holderFor(container), "ascribe-removed");
            holder.setAttribute("data-ascribe-change", "removed");
            if (change.was) holder.setAttribute("data-ascribe-was-source", change.was.source);
            const label = ui(doc, "span", "ascribe-label", LABELS.removed);
            label.setAttribute("data-ascribe-label", "removed");
            const body = ui(doc, "div", "ascribe-removed-body");
            if (before) body.append(...contentOf(holder, before));
            else body.textContent = change.text ?? "";
            const toggle = ui(doc, "button", "ascribe-toggle", "Show");
            toggle.type = "button";
            toggle.setAttribute("aria-expanded", "false");
            toggle.addEventListener("click", (event) => {
              event.stopPropagation();
              const open = holder.classList.toggle("ascribe-open");
              toggle.textContent = open ? "Collapse" : "Show";
              toggle.setAttribute("aria-expanded", String(open));
            });
            holder.append(label, body, toggle);
            return holder;
          });
          marks.push({ change, element });
          break;
        }
        case "moved": {
          const element = change.now ? findBlock(root, change.now) : null;
          if (!element || element.hasAttribute("data-ascribe-change")) break;
          const id = `ascribe-move-${++moveCount}`;
          const before = was && change.was ? findBlock(was, change.was) : null;
          const stub = placer.place(change, (container) => {
            const holder = ui(doc, holderFor(container), "ascribe-moved-from");
            holder.id = `${id}-from`;
            holder.setAttribute("data-ascribe-change", "moved-from");
            if (change.was) holder.setAttribute("data-ascribe-was-source", change.was.source);
            const note = ui(doc, "span", "ascribe-move-note");
            note.append(doc.createTextNode(`A ${blockWord(element)} moved from here `));
            const link = ui(doc, "a", "ascribe-move-link");
            link.href = `#${id}`;
            note.append(link);
            holder.append(note);
            const copy = ui(doc, "div", "ascribe-was-copy");
            copy.setAttribute("data-ascribe-was-only", "");
            if (before) copy.append(...contentOf(holder, before));
            else copy.textContent = change.text ?? ownText(element);
            holder.append(copy);
            return holder;
          });
          element.setAttribute("data-ascribe-change", "moved");
          element.setAttribute("data-ascribe-move", id);
          if (!element.id) element.id = id;
          addLabel(element, "moved");
          const back = ui(doc, "a", "ascribe-move-link");
          back.href = `#${stub.id}`;
          back.textContent = `from “${headingBefore(root, stub) ?? "the top of the page"}” ↑`;
          const forward = stub.querySelector(".ascribe-move-link");
          if (forward) {
            forward.href = `#${element.id}`;
            forward.textContent = `to “${headingBefore(root, element) ?? "the top of the page"}” ↓`;
          }
          if (LABEL_INSIDE.has(element.localName)) {
            const note = ui(doc, "span", "ascribe-move-note");
            note.append(" ", back);
            element.append(note);
          } else {
            const holder = ui(doc, "div", "ascribe-move-note ascribe-move-after");
            holder.append(back);
            element.after(holder);
          }
          for (const link of [back, forward]) {
            link?.addEventListener("click", (event) => {
              const target = doc.getElementById(link.hash.slice(1));
              if (!target || !root.contains(target)) return;
              event.preventDefault();
              goTo(target);
            });
          }
          marks.push({ change, element });
          break;
        }
      }
    }
    marks.sort(
      (a, b) => a.element === b.element ? 0 : a.element.compareDocumentPosition(b.element) & Node.DOCUMENT_POSITION_FOLLOWING ? -1 : 1
    );
    hintHidden(root, marks);
    return marks;
  }
  function hintHidden(root, marks) {
    const tabs = /* @__PURE__ */ new Map();
    const summaries = /* @__PURE__ */ new Map();
    for (const { change, element } of marks) {
      for (let el = element; el && el !== root; el = el.parentElement) {
        if (el.localName === "ascribe-tab") {
          const tab = tabs.get(el) ?? { added: false, count: 0 };
          if (el === element && change.kind === "added") tab.added = true;
          else tab.count++;
          tabs.set(el, tab);
        }
        const summary = el !== element ? summaryOf(el) : null;
        if (summary && !summary.contains(element)) {
          summaries.set(summary, (summaries.get(summary) ?? 0) + 1);
        }
      }
    }
    const doc = root.ownerDocument;
    const hint = (kind, text) => {
      const el = ui(doc, "span", "ascribe-hint", text);
      el.setAttribute("data-ascribe-hint", kind);
      return el;
    };
    const watching = [];
    for (const [tab, { added, count }] of tabs) {
      const el = added ? hint("added", "new") : hint("changed", plural(count, "change"));
      const place = () => {
        const button2 = tabButton(tab);
        if (button2 && !button2.contains(el)) button2.append(el);
      };
      place();
      const group = tab.parentElement;
      const Observer = doc.defaultView?.MutationObserver;
      if (group && Observer) {
        const observer = new Observer(place);
        observer.observe(group, { childList: true });
        watching.push(observer);
      }
    }
    if (watching.length > 0) tabWatchers.set(root, watching);
    for (const [summary, count] of summaries)
      summary.append(hint("changed", plural(count, "change")));
  }
  var tabWatchers = /* @__PURE__ */ new WeakMap();
  function plural(count, word) {
    return `${count} ${word}${count === 1 ? "" : "s"}`;
  }
  function summaryOf(element) {
    if (element.localName !== "details") return null;
    return element.querySelector(":scope > summary");
  }
  function tabButton(tab) {
    const group = tab.parentElement;
    if (tab.localName !== "ascribe-tab" || !group) return null;
    const tabs = Array.from(group.children).filter((c) => c.localName === "ascribe-tab");
    const buttons = group.querySelectorAll(
      ':scope > [role="tablist"] > [role="tab"]'
    );
    return buttons[tabs.indexOf(tab)] ?? null;
  }
  function blockWord(element) {
    const name = element.localName;
    if (name === "p") return "paragraph";
    if (/^h[1-6]$/.test(name)) return "heading";
    if (name === "ul" || name === "ol") return "list";
    if (name === "li") return "list item";
    if (name === "pre") return "code block";
    if (name === "table") return "table";
    return "block";
  }
  function clearMarks(root) {
    for (const observer of tabWatchers.get(root) ?? []) observer.disconnect();
    tabWatchers.delete(root);
    for (const el of Array.from(root.querySelectorAll("[data-ascribe-ui]"))) {
      if (el.localName === "ins") el.replaceWith(...Array.from(el.childNodes));
      else el.remove();
    }
    for (const el of Array.from(root.querySelectorAll("[data-ascribe-change]"))) {
      el.removeAttribute("data-ascribe-change");
      el.removeAttribute("data-ascribe-has-was");
      el.removeAttribute("data-ascribe-was-source");
      if (el.hasAttribute("data-ascribe-move")) {
        if (el.id === el.getAttribute("data-ascribe-move")) el.removeAttribute("id");
        el.removeAttribute("data-ascribe-move");
      }
    }
    root.normalize();
  }
  function setShow(root, show) {
    root.setAttribute("data-ascribe-show", show);
  }
  function reveal(element) {
    for (let el = element; el; el = el.parentElement) {
      if (el instanceof HTMLDetailsElement && !el.open) el.open = true;
      if (el.localName === "ascribe-tab" && el.hidden) tabButton(el)?.click();
    }
  }
  function goTo(element) {
    reveal(element);
    if (typeof element.scrollIntoView === "function") {
      element.scrollIntoView({ block: "center", behavior: "smooth" });
    }
    element.classList.remove("ascribe-flash");
    void element.offsetWidth;
    element.classList.add("ascribe-flash");
    if (!element.hasAttribute("tabindex")) element.setAttribute("tabindex", "-1");
    element.focus({ preventScroll: true });
  }
  function showSources(root) {
    const doc = root.ownerDocument;
    const tip = ui(doc, "span", "ascribe-where");
    tip.setAttribute("role", "status");
    let current = null;
    const show = (target) => {
      const el = target instanceof Element ? target.closest("[data-ascribe-source], [data-ascribe-was-source]") : null;
      if (el === current) return;
      current = el;
      if (!el || !root.contains(el)) {
        tip.remove();
        current = null;
        return;
      }
      const source = el.getAttribute("data-ascribe-source");
      tip.textContent = source ? describeSource(source, el.getAttribute("data-ascribe-via")) : `was ${describeSource(el.getAttribute("data-ascribe-was-source") ?? "")}`;
      const box = el.getBoundingClientRect();
      const host = root.getBoundingClientRect();
      tip.style.top = `${box.top - host.top}px`;
      root.append(tip);
    };
    const over = (event) => show(event.target);
    const out = (event) => {
      const to = event.relatedTarget;
      if (!(to instanceof Node) || !root.contains(to)) show(null);
    };
    root.addEventListener("mouseover", over);
    root.addEventListener("focusin", over);
    root.addEventListener("mouseout", out);
    return () => {
      root.removeEventListener("mouseover", over);
      root.removeEventListener("focusin", over);
      root.removeEventListener("mouseout", out);
      tip.remove();
    };
  }
  function disarm(fragment) {
    for (const el of Array.from(fragment.querySelectorAll("script, meta, base"))) el.remove();
    for (const el of Array.from(fragment.querySelectorAll("*"))) {
      for (const attr of Array.from(el.attributes)) {
        if (/^on/i.test(attr.name) || runsScript(attr.value)) el.removeAttribute(attr.name);
      }
    }
  }
  function runsScript(value) {
    const kept = Array.from(value).filter((ch) => ch.charCodeAt(0) > 32).join("");
    return /^javascript:/i.test(kept);
  }

  // src/report/index.ts
  function h(tag, attributes = {}, children = []) {
    const el = document.createElement(tag);
    for (const [name, value] of Object.entries(attributes)) {
      if (name === "class") el.className = value;
      else el.setAttribute(name, value);
    }
    for (const child of children) if (child !== null) el.append(child);
    return el;
  }
  function button(text, className, onClick, attributes = {}) {
    const el = h("button", { type: "button", class: className, ...attributes }, [text]);
    el.addEventListener("click", onClick);
    return el;
  }
  function describeCounts(counts) {
    const parts = [];
    for (const kind of ["changed", "added", "removed", "moved"]) {
      if (counts[kind] > 0) parts.push(`${counts[kind]} ${kind}`);
    }
    return parts.join(" · ");
  }
  function changedPages(builds) {
    const paths = new Set(builds.flatMap((b) => b.pages.map((p) => p.path)));
    if (paths.size === 0) return "no changed pages";
    const pages = paths.size === 1 ? "1 changed page" : `${paths.size} changed pages`;
    return builds.length > 1 ? `${pages} in ${builds.length} builds` : pages;
  }
  function pageSummary(page) {
    if (page.status === "added") return "New page";
    if (page.status === "removed") return "Removed";
    const parts = [describeCounts(page.counts)];
    if (page.page_changed.length > 0) parts.push(`${andList(page.page_changed)} changed`);
    return parts.filter(Boolean).join(" · ");
  }
  function andList(items) {
    if (items.length <= 1) return items.join("");
    if (items.length === 2) return `${items[0]} and ${items[1]}`;
    return `${items.slice(0, -1).join(", ")}, and ${items[items.length - 1]}`;
  }
  function bytes(n) {
    if (n >= 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
    return `${Math.ceil(n / 1024)} KB`;
  }
  function imageRef(rendered, src) {
    if (!rendered) return void 0;
    const exact = rendered.images[src];
    if (exact) return exact;
    const decode = (s) => {
      try {
        return decodeURIComponent(s);
      } catch {
        return s;
      }
    };
    const want = decode(src);
    for (const [reference, ref] of Object.entries(rendered.images)) {
      if (decode(reference) === want) return ref;
    }
    return void 0;
  }
  function pageFragment(data2, key) {
    const rendered = data2.pages[key];
    const template = document.createElement("template");
    template.innerHTML = rendered?.html ?? "";
    const fragment = template.content;
    for (const img of Array.from(fragment.querySelectorAll("img"))) {
      const src = img.getAttribute("src") ?? "";
      const ref = imageRef(rendered, src.split("#")[0] ?? "");
      const url = ref?.image === void 0 ? void 0 : data2.images[ref.image];
      if (url !== void 0) {
        img.setAttribute("src", url);
        img.removeAttribute("srcset");
        continue;
      }
      let text;
      if (ref?.bytes !== void 0) {
        text = `Image not included: ${ref.path} (${bytes(ref.bytes)}, over the report's ${bytes(data2.image_limit)} limit)`;
      } else if (ref) {
        text = `Image not found: ${ref.path}`;
      } else {
        text = `Image not loaded: ${src} (the report makes no network requests)`;
      }
      const placeholder = document.createElement("span");
      placeholder.className = "r-image";
      placeholder.setAttribute("role", "img");
      placeholder.setAttribute("aria-label", img.getAttribute("alt") ?? "");
      placeholder.textContent = text;
      img.replaceWith(placeholder);
    }
    for (const el of Array.from(
      fragment.querySelectorAll("[src], [srcset], [poster], [data], [srcdoc]")
    )) {
      if (el.localName === "img") continue;
      for (const name of ["src", "srcset", "poster", "data", "srcdoc"]) el.removeAttribute(name);
    }
    disarm(fragment);
    return fragment;
  }
  function start(root, data2) {
    const builds = data2.builds.filter((b) => b.pages.length > 0);
    const state = {
      build: 0,
      page: 0,
      show: "changes",
      at: -1,
      breakdown: false,
      atEnd: false
    };
    let marks = [];
    let stopSources;
    const fromHash = () => {
      const hash = decodeURIComponent(location.hash.slice(1));
      const slash = hash.indexOf("/");
      if (slash < 0) return;
      const b = builds.findIndex((x) => x.build === hash.slice(0, slash));
      if (b < 0) return;
      const p = builds[b]?.pages.findIndex((x) => x.path === hash.slice(slash + 1)) ?? -1;
      state.build = b;
      state.page = Math.max(p, 0);
    };
    const open = (b, p, push = true) => {
      state.build = b;
      state.page = p;
      state.at = -1;
      state.atEnd = false;
      state.breakdown = false;
      const page = builds[b]?.pages[p];
      if (push && page) {
        history.replaceState(null, "", `#${encodeURIComponent(`${builds[b]?.build}/${page.path}`)}`);
      }
      render();
      root.querySelector(".r-main")?.scrollTo?.({ top: 0 });
    };
    const step = (by) => {
      if (state.show !== "changes") {
        state.show = "changes";
        if (article.hasAttribute("data-ascribe-show")) setShow(article, "changes");
        else render();
      }
      if (marks.length === 0) return;
      const next = state.at + by;
      if (next >= marks.length || next < 0) {
        state.atEnd = next >= marks.length;
        state.at = next >= marks.length ? marks.length : -1;
        renderControls();
        return;
      }
      state.at = next;
      state.atEnd = false;
      renderControls();
      const mark = marks[next];
      if (mark) goTo(mark.element);
    };
    root.replaceChildren();
    root.className = "r";
    const head = h("header", { class: "r-head" });
    const body = h("div", { class: "r-body" });
    const side = h("nav", { class: "r-side", "aria-label": "Changed pages" });
    const main = h("main", { class: "r-main" });
    const controls = h("div", { class: "r-controls" });
    const article = h("article", { class: "r-page" });
    body.append(side, main);
    root.append(head, body);
    const short = (commit) => commit.slice(0, 7);
    const compared = data2.base.merge_base && data2.base.merge_base !== data2.base.commit ? `${data2.base.requested} (${short(data2.base.commit)}), from its merge base with HEAD (${short(data2.base.merge_base)})` : `${data2.base.requested} (${short(data2.base.commit)})`;
    head.append(
      h("div", { class: "r-title" }, [
        h("b", {}, ["Ascribe review"]),
        ` · ${changedPages(builds)}, compared with ${compared}`
      ]),
      h("div", { class: "r-sub" }, [
        "Each page as Ascribe renders it, without the site's layout, navigation, or styles."
      ])
    );
    if (data2.limit.omitted > 0) {
      head.append(
        h("div", { class: "r-notice" }, [
          `This report renders the first ${data2.limit.pages} changed pages. ${data2.limit.omitted} more are listed by name at the end of the list, not rendered.`
        ])
      );
    }
    if (builds.length === 0) {
      main.append(
        h("p", { class: "r-empty" }, ["Nothing changed: no build publishes a page that differs."])
      );
      side.remove();
      return;
    }
    const renderSide = () => {
      side.replaceChildren();
      if (builds.length > 1) {
        const select = h("select", { id: "r-build", "aria-label": "Build" });
        builds.forEach((b, i) => {
          const option = h("option", { value: String(i) }, [`${b.build} (${b.pages.length})`]);
          if (i === state.build) option.selected = true;
          select.append(option);
        });
        select.addEventListener("change", () => open(Number(select.value), 0));
        side.append(h("label", { class: "r-build", for: "r-build" }, ["Build"]), select);
      } else {
        side.append(h("div", { class: "r-build" }, [`Build: ${builds[0]?.build ?? ""}`]));
      }
      const build = builds[state.build];
      if (!build) return;
      const included = build.pages.filter((p) => !p.omitted);
      const omitted = build.pages.filter((p) => p.omitted);
      side.append(h("h2", {}, [`Changed pages (${build.pages.length})`]));
      const list = h("ul", { class: "r-pages" });
      build.pages.forEach((page, i) => {
        if (!included.includes(page)) return;
        const link = h("a", { href: `#${encodeURIComponent(`${build.build}/${page.path}`)}` }, [
          h("span", { class: "r-page-title" }, [page.title ?? page.path]),
          h("span", { class: "r-page-path" }, [page.path]),
          h("span", { class: "r-page-counts" }, [pageSummary(page)]),
          !page.own_file_changed && page.because.length > 0 ? h("span", { class: "r-page-via" }, [`via ${page.because.join(", ")}`]) : null
        ]);
        if (i === state.page) link.setAttribute("aria-current", "page");
        link.addEventListener("click", (event) => {
          event.preventDefault();
          open(state.build, i);
        });
        list.append(h("li", {}, [link]));
      });
      side.append(list);
      if (omitted.length > 0) {
        side.append(
          h("h3", {}, [`Not rendered (${omitted.length})`]),
          h(
            "ul",
            { class: "r-omitted" },
            omitted.map((p) => h("li", {}, [`${p.path}: ${pageSummary(p)}`]))
          )
        );
      }
    };
    const renderControls = () => {
      controls.replaceChildren();
      const build = builds[state.build];
      const page = build?.pages[state.page];
      if (!build || !page) return;
      const row = h("div", { class: "r-row" });
      const info = h("span", { class: "r-grow" }, [h("b", {}, [page.path]), ` ${page.route}`]);
      if (!page.own_file_changed && page.because.length > 0) {
        info.append(` · changed only through ${page.because.join(", ")}`);
      } else if (page.because.length > 0) {
        info.append(` · also through ${page.because.join(", ")}`);
      }
      row.append(info);
      const blocks = page.status === "changed" && page.changes.length > 0;
      if (page.status === "changed" && !blocks) {
        row.append(h("span", { class: "r-quiet" }, ["No changes to the page's content"]));
      } else if (blocks) {
        const n = marks.length;
        const at = state.at >= 0 && state.at < n && state.show === "changes";
        const text = at ? `${state.at + 1} of ${n} on this page` : `${n} ${n === 1 ? "change" : "changes"} on this page`;
        row.append(
          button(
            text,
            "r-link r-pos",
            () => {
              state.breakdown = !state.breakdown;
              renderControls();
            },
            {
              title: describeCounts(page.counts),
              "aria-label": `${at ? `Change ${state.at + 1} of ${n}` : `${n} changes`} on this page. Show the breakdown.`,
              "aria-expanded": String(state.breakdown)
            }
          )
        );
      }
      const seg = h("div", { class: "r-seg", role: "group", "aria-label": "Show" });
      for (const [value, label] of [
        ["changes", "Changes"],
        ["will", "As it will be"],
        ["was", "As it was"]
      ]) {
        seg.append(
          button(
            label,
            "",
            () => {
              state.show = value;
              render();
            },
            { "aria-pressed": String(state.show === value) }
          )
        );
      }
      row.append(h("span", { class: "r-show" }, ["Show"]), seg);
      if (blocks) {
        row.append(
          button("↑", "r-ghost r-sq", () => step(-1), { "aria-label": "Previous change" }),
          button("↓", "r-ghost r-sq", () => step(1), { "aria-label": "Next change" })
        );
      }
      controls.append(row);
      if (state.breakdown)
        controls.append(h("div", { class: "r-legend" }, [describeCounts(page.counts)]));
      if (page.page_changed.length > 0) {
        controls.append(
          h("div", { class: "r-legend" }, [
            blocks ? `Also changed: the page's ${andList(page.page_changed)}, which this render doesn't show.` : `The page's ${andList(page.page_changed)} changed, which this render doesn't show.`
          ])
        );
      }
      if (state.atEnd) {
        const nextIndex = (state.page + 1) % build.pages.length;
        const next = build.pages[nextIndex];
        const notice = h("div", { class: "r-notice" }, [
          h("span", {}, ["That was the last change on this page."])
        ]);
        if (next && build.pages.length > 1) {
          notice.append(
            button(
              `${nextIndex === 0 ? "First changed page" : "Next changed page"}: ${next.title ?? next.path}`,
              "r-primary",
              () => {
                open(state.build, nextIndex);
                step(1);
              }
            )
          );
        }
        controls.append(notice);
      }
    };
    const render = () => {
      renderSide();
      stopSources?.();
      stopSources = void 0;
      marks = [];
      const build = builds[state.build];
      const page = build?.pages[state.page];
      main.replaceChildren(controls);
      article.replaceChildren();
      article.removeAttribute("data-ascribe-show");
      article.className = "r-page";
      if (!page) return;
      main.append(h("h1", { class: "r-page-heading" }, [page.title ?? page.path]));
      if (page.omitted || page.now === null && page.was === null) {
        main.append(
          h("p", { class: "r-empty" }, [
            page.omitted ? `${page.path} isn't rendered: the report renders at most ${data2.limit.pages} changed pages.` : `${page.path} couldn't be rendered.`
          ])
        );
        renderControls();
        return;
      }
      const banner = (text) => {
        main.append(h("p", { class: "r-banner" }, [text]));
      };
      if (page.status === "added") {
        if (state.show === "was") {
          banner("This page is new: it didn't exist before.");
        } else {
          if (state.show === "changes") banner("New page: everything on it is added.");
          article.append(pageFragment(data2, page.now ?? ""));
        }
      } else if (page.status === "removed") {
        if (state.show === "will") {
          banner("This page is removed: the build doesn't publish it any more.");
        } else {
          if (state.show === "changes")
            banner("Removed page: the build doesn't publish it any more.");
          article.append(pageFragment(data2, page.was ?? ""));
        }
      } else {
        article.append(pageFragment(data2, page.now ?? ""));
        main.append(article);
        const was = page.was === null ? null : pageFragment(data2, page.was);
        marks = markChanges(article, page.changes, { was });
        setShow(article, state.show);
      }
      if (article.parentNode !== main) main.append(article);
      stopSources = showSources(article);
      renderControls();
    };
    window.addEventListener("hashchange", () => {
      fromHash();
      open(state.build, state.page, false);
    });
    fromHash();
    render();
  }
  var data = document.getElementById("ascribe-review-data");
  var app = document.getElementById("ascribe-review");
  if (data && app) start(app, JSON.parse(data.textContent ?? "{}"));
})();
