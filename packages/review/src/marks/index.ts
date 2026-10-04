// @ascribed/review/marks: marks what changed on a rendered page, in the
// browser.
//
// The page is Ascribe's site output rendered with source anchors (the
// site-render contract, §7): every block's element carries
// `data-ascribe-source`, and `data-ascribe-via` when it came through
// includes. The changes are `ascribe diff --format json`'s, for that page.
// `markChanges` applies them to the page's elements; `marks.css` (exported
// as `@ascribed/review/marks.css`) draws them, and `setShow` switches between
// the changes, the page as it will be, and the page as it was.
//
// Everything the marks add is an element with `data-ascribe-ui`, and every
// element they change gets `data-ascribe-change`, so `clearMarks` can put the
// page back as it was rendered.

/** Where a block is written: the anchor grammar's `source` and `via`. */
export interface Anchor {
  source: string;
  via: string[];
}

/** The words that differ inside a changed block of prose. */
export interface Words {
  /** `[start, end)` ranges, in characters, of `now_text`. */
  now: [number, number][];
  /** `[start, end)` ranges, in characters, of `was_text`. */
  was: [number, number][];
  now_text: string;
  was_text: string;
}

/** One block's change, as `ascribe diff --format json` writes it. */
export interface Change {
  kind: "changed" | "added" | "removed" | "moved";
  now?: Anchor;
  was?: Anchor;
  words?: Words;
  after?: Anchor;
  parent?: Anchor;
  text?: string;
}

/** What the page shows: its changes, the page as it will be, or as it was. */
export type Show = "changes" | "will" | "was";

export interface MarkOptions {
  /**
   * The page as it was, rendered with anchors. Removed blocks, and moved
   * blocks at their old place, are copied from it; without it they're shown
   * from the change's `text`, and "as it was" shows changed prose from its
   * words.
   */
  was?: ParentNode | null;
}

/** A change as marked on the page. */
export interface Mark {
  change: Change;
  /** The element to go to: the block now, or what stands for a removed one. */
  element: HTMLElement;
}

const LABELS: Record<Change["kind"], string> = {
  added: "Added",
  changed: "Changed",
  removed: "Removed",
  moved: "Moved",
};

/** Blocks a label can start, inline, as their first child. */
const LABEL_INSIDE = new Set([
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
]);

/** A parsed anchor source: the path, decoded, and the lines. */
export interface Source {
  path: string;
  first: number;
  last: number;
}

/** Parses `<path>:<first>-<last>`, splitting at the last `:`. */
export function parseSource(source: string): Source | undefined {
  const colon = source.lastIndexOf(":");
  if (colon < 0) return undefined;
  const match = /^(\d+)-(\d+)$/.exec(source.slice(colon + 1));
  if (!match) return undefined;
  let path = source.slice(0, colon);
  try {
    path = decodeURIComponent(path);
  } catch {
    // Not percent-encoded as the grammar says; show it as it is.
  }
  return { path, first: Number(match[1]), last: Number(match[2]) };
}

/** `guides/install.md:12`, or `guides/install.md:12-14`, and its includes. */
export function describeSource(source: string, via?: string | null): string {
  const parsed = parseSource(source);
  let text = source;
  if (parsed) {
    const lines =
      parsed.first === parsed.last ? `${parsed.first}` : `${parsed.first}-${parsed.last}`;
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

function viaOf(element: Element): string {
  return element.getAttribute("data-ascribe-via") ?? "";
}

/**
 * The element of a block: the one with exactly its anchor, or, when there's
 * none (a paragraph in a tight list item has no element of its own), the
 * smallest anchored element with the same file and includes whose lines
 * contain it.
 */
export function findBlock(root: ParentNode, anchor: Anchor): HTMLElement | null {
  const via = anchor.via.join(" ");
  const anchored = Array.from(root.querySelectorAll<HTMLElement>("[data-ascribe-source]"));
  const exact = anchored.find(
    (el) => el.getAttribute("data-ascribe-source") === anchor.source && viaOf(el) === via,
  );
  if (exact) return exact;
  const want = parseSource(anchor.source);
  if (!want) return null;
  let best: HTMLElement | null = null;
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

function ui<K extends keyof HTMLElementTagNameMap>(
  doc: Document,
  tag: K,
  className: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const el = doc.createElement(tag);
  el.className = className;
  el.setAttribute("data-ascribe-ui", "");
  if (text !== undefined) el.textContent = text;
  return el;
}

/** A label, inside the block at its start, or just before it. */
function addLabel(element: HTMLElement, kind: Change["kind"]): void {
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

/**
 * What goes in `holder` to show `before`, a block of the page as it was: a
 * copy, or for an item going into an item, the copy's contents.
 */
function contentOf(holder: Element, before: Element): Node[] {
  const copy = copyOf(before);
  if (holder.localName === "li" && copy.localName === "li") return Array.from(copy.childNodes);
  return [copy];
}

/** A copy of a block of the page as it was, without its anchors or ids. */
function copyOf(element: Element): HTMLElement {
  const copy = element.cloneNode(true) as HTMLElement;
  for (const el of [copy, ...Array.from(copy.querySelectorAll("*"))]) {
    el.removeAttribute("data-ascribe-source");
    el.removeAttribute("data-ascribe-via");
    el.removeAttribute("id");
  }
  return copy;
}

/** The element to add beside `sibling`: an item in a list, else a `div`. */
function holderFor(container: Element): "li" | "div" {
  return container.localName === "ul" || container.localName === "ol" ? "li" : "div";
}

/** The nearest heading before `node` in `root`, as text. */
function headingBefore(root: Element, node: Node): string | undefined {
  const headings = Array.from(root.querySelectorAll("h1, h2, h3, h4, h5, h6"));
  let found: Element | undefined;
  for (const heading of headings) {
    if (heading === node || heading.contains(node)) {
      found = heading;
      break;
    }
    if (heading.compareDocumentPosition(node) & Node.DOCUMENT_POSITION_FOLLOWING) found = heading;
    else break;
  }
  return found ? ownText(found).trim() : undefined;
}

/** An element's text without what the marks added. */
function ownText(element: Element): string {
  let text = "";
  const walk = (node: Node): void => {
    for (const child of Array.from(node.childNodes)) {
      if (child.nodeType === Node.TEXT_NODE) text += child.nodeValue ?? "";
      else if (child instanceof Element && !child.hasAttribute("data-ascribe-ui")) walk(child);
    }
  };
  walk(element);
  return text.replace(/\s+/g, " ");
}

/** Where a removed block, or a moved block's old place, goes in the page now. */
class Placer {
  /** The last element placed after each block, so several stay in order. */
  #last = new Map<Element, Element>();
  #first = new Map<Element, Element>();

  constructor(readonly root: HTMLElement) {}

  /** Puts `make(container)`'s element where `change` was. */
  place(change: Change, make: (container: Element) => HTMLElement): HTMLElement {
    const after = change.after ? findBlock(this.root, change.after) : null;
    const parent = change.parent ? findBlock(this.root, change.parent) : null;
    if (after && after !== parent && after.parentElement) {
      let previous: Element = this.#last.get(after) ?? after;
      // Past what the marks put after the block itself: its copy as it
      // was, or its "moved from" link.
      for (
        let next = previous.nextElementSibling;
        next &&
        (next.hasAttribute("data-ascribe-was-only") ||
          next.classList.contains("ascribe-move-after"));
        next = next.nextElementSibling
      ) {
        previous = next;
      }
      const el = make(after.parentElement);
      // A label before the next block belongs to that block, so this goes
      // before it.
      previous.after(el);
      this.#last.set(after, el);
      return el;
    }
    const container = parent ?? this.root;
    const el = make(container);
    const previous = this.#first.get(container);
    if (previous) {
      previous.after(el);
    } else {
      // First among the container's blocks: before its first anchored
      // child, or its first label, or at its end.
      const first = Array.from(container.children).find(
        (c) => c.hasAttribute("data-ascribe-source") || c.hasAttribute("data-ascribe-ui"),
      );
      if (first) first.before(el);
      else container.append(el);
    }
    this.#first.set(container, el);
    return el;
  }
}

/** A text position: the text node, and the offset in it in UTF-16 units. */
interface At {
  node: Text;
  offset: number;
}

/**
 * The block's own text, whitespace collapsed as `ascribe diff` collapses it,
 * with where each character is: text inside the block's anchored children,
 * which are blocks of their own, and inside what the marks added, isn't its.
 */
function textMap(element: Element): { text: string[]; at: At[] } {
  const chars: string[] = [];
  const at: At[] = [];
  let space = false;
  let spaceAt: At | undefined;
  const walk = (node: Node): void => {
    for (const child of Array.from(node.childNodes)) {
      if (child.nodeType === Node.TEXT_NODE) {
        const value = (child as Text).nodeValue ?? "";
        let offset = 0;
        for (const ch of value) {
          if (/\s/.test(ch)) {
            if (!space) spaceAt = { node: child as Text, offset };
            space = true;
          } else {
            if (space && chars.length > 0 && spaceAt) {
              chars.push(" ");
              at.push(spaceAt);
            }
            space = false;
            chars.push(ch);
            at.push({ node: child as Text, offset });
          }
          offset += ch.length;
        }
      } else if (
        child instanceof Element &&
        !child.hasAttribute("data-ascribe-ui") &&
        !child.hasAttribute("data-ascribe-source")
      ) {
        walk(child);
      }
    }
  };
  walk(element);
  return { text: chars, at };
}

/** Wraps characters `[start, end)` of the map in elements made by `make`. */
function wrapRange(map: { at: At[] }, start: number, end: number, make: () => HTMLElement): void {
  // Each text node's part of the range, wrapped by itself, last first so
  // earlier offsets stay right.
  const parts: { node: Text; from: number; to: number }[] = [];
  for (let i = start; i < end; i++) {
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

function charWidth(at: At): number {
  const code = at.node.nodeValue?.codePointAt(at.offset) ?? 0;
  return code > 0xffff ? 2 : 1;
}

/**
 * Marks the changed words of a block: the words added or replacing others
 * in `<ins>`, and the words removed in `<del>` where they were. Skipped when
 * the block's text isn't the text the change was computed from.
 */
function markWords(element: HTMLElement, words: Words): boolean {
  const map = textMap(element);
  const nowChars = Array.from(words.now_text);
  const text = map.text.join("");
  const offset = text === words.now_text ? 0 : findChars(map.text, nowChars);
  if (offset < 0) return false;
  const doc = element.ownerDocument;
  const wasChars = Array.from(words.was_text);
  // Each removed run goes where it was: after as many unchanged characters
  // of the new text as there were before it in the old.
  const insertions: { at: number; text: string }[] = [];
  let removedBefore = 0;
  for (const [start, end] of words.was) {
    const unchangedBefore = start - removedBefore;
    insertions.push({
      at: nowPosition(words.now, unchangedBefore),
      text: wasChars.slice(start, end).join(""),
    });
    removedBefore += end - start;
  }
  // Wrap from the end so earlier positions stay valid.
  type Edit = { at: number; kind: "ins"; end: number } | { at: number; kind: "del"; text: string };
  const edits: Edit[] = [
    ...words.now.map(([start, end]): Edit => ({ at: start, kind: "ins", end })),
    ...insertions.map((i): Edit => ({ at: i.at, kind: "del", text: i.text })),
  ];
  // At one position, the removed words go first, before the words that
  // replace them.
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

/** Where in the new text `unchanged` unchanged characters end. */
function nowPosition(ranges: [number, number][], unchanged: number): number {
  let position = unchanged;
  for (const [start, end] of ranges) {
    if (start < position) position += end - start;
    else break;
  }
  return position;
}

function findChars(haystack: string[], needle: string[]): number {
  if (needle.length === 0) return -1;
  outer: for (let i = 0; i + needle.length <= haystack.length; i++) {
    for (let j = 0; j < needle.length; j++) if (haystack[i + j] !== needle[j]) continue outer;
    return i;
  }
  return -1;
}

/** Inserts `node` before character `index` of the map, or at its end. */
function insertAt(map: { at: At[] }, index: number, node: Node, element: Element): void {
  const at = map.at[index];
  if (at) {
    const rest = at.node.splitText(at.offset);
    rest.before(node);
    // The map's later positions in this node now point into `rest`.
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

/**
 * Marks `changes` on the page in `root`, and returns the marks in page
 * order. Marks already there are cleared first.
 */
export function markChanges(
  root: HTMLElement,
  changes: readonly Change[],
  options: MarkOptions = {},
): Mark[] {
  clearMarks(root);
  const doc = root.ownerDocument;
  const was = options.was ?? null;
  const placer = new Placer(root);
  const marks: Mark[] = [];
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
          const before = was && change.was ? findBlock(was, change.was) : null;
          if (change.words) markWords(element, change.words);
          if (before) {
            // As it was, the block as the old page rendered it.
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
          const note = ui(doc, "span", "ascribe-move-note");
          note.append(doc.createTextNode(`A ${blockWord(element)} moved from here `));
          const link = ui(doc, "a", "ascribe-move-link");
          link.href = `#${id}`;
          note.append(link);
          holder.append(note);
          // As it was, the block itself.
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
        const forward = stub.querySelector<HTMLAnchorElement>(".ascribe-move-link");
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
  marks.sort((a, b) =>
    a.element === b.element
      ? 0
      : a.element.compareDocumentPosition(b.element) & Node.DOCUMENT_POSITION_FOLLOWING
        ? -1
        : 1,
  );
  return marks;
}

/** "paragraph", "heading", "list", or "block". */
function blockWord(element: Element): string {
  const name = element.localName;
  if (name === "p") return "paragraph";
  if (/^h[1-6]$/.test(name)) return "heading";
  if (name === "ul" || name === "ol") return "list";
  if (name === "li") return "list item";
  if (name === "pre") return "code block";
  if (name === "table") return "table";
  return "block";
}

/** Removes every mark from the page in `root`. */
export function clearMarks(root: HTMLElement): void {
  for (const el of Array.from(root.querySelectorAll("[data-ascribe-ui]"))) {
    if (el.localName === "ins") el.replaceWith(...Array.from(el.childNodes));
    else el.remove();
  }
  for (const el of Array.from(root.querySelectorAll("[data-ascribe-change]"))) {
    el.removeAttribute("data-ascribe-change");
    el.removeAttribute("data-ascribe-has-was");
    if (el.hasAttribute("data-ascribe-move")) {
      if (el.id === el.getAttribute("data-ascribe-move")) el.removeAttribute("id");
      el.removeAttribute("data-ascribe-move");
    }
  }
  root.normalize();
}

/** Shows the changes, the page as it will be, or the page as it was. */
export function setShow(root: HTMLElement, show: Show): void {
  root.setAttribute("data-ascribe-show", show);
}

/**
 * Makes `element` visible: opens the `<details>` and selects the tabs it's
 * in.
 */
export function reveal(element: HTMLElement): void {
  for (let el: HTMLElement | null = element; el; el = el.parentElement) {
    if (el instanceof HTMLDetailsElement && !el.open) el.open = true;
    const group: HTMLElement | null = el.parentElement;
    if (el.localName === "ascribe-tab" && el.hidden && group) {
      const tabs: Element[] = Array.from(group.children).filter(
        (c) => c.localName === "ascribe-tab",
      );
      const index = tabs.indexOf(el);
      const buttons = group.querySelectorAll<HTMLButtonElement>(
        ':scope > [role="tablist"] > [role="tab"]',
      );
      buttons[index]?.click();
    }
  }
}

/** Reveals `element`, scrolls to it, and flashes it. */
export function goTo(element: HTMLElement): void {
  reveal(element);
  // Not every DOM implements scrolling (jsdom doesn't).
  if (typeof element.scrollIntoView === "function") {
    element.scrollIntoView({ block: "center", behavior: "smooth" });
  }
  element.classList.remove("ascribe-flash");
  // Restart the animation.
  void element.offsetWidth;
  element.classList.add("ascribe-flash");
  if (!element.hasAttribute("tabindex")) element.setAttribute("tabindex", "-1");
  element.focus({ preventScroll: true });
}

/**
 * Shows each block's source on hover and focus: `guides/install.md:12`,
 * from its anchor. Returns a function that stops.
 */
export function showSources(root: HTMLElement): () => void {
  const doc = root.ownerDocument;
  const tip = ui(doc, "span", "ascribe-where");
  tip.setAttribute("role", "status");
  let current: Element | null = null;
  const show = (target: EventTarget | null): void => {
    const el =
      target instanceof Element
        ? target.closest("[data-ascribe-source], [data-ascribe-was-source]")
        : null;
    if (el === current) return;
    current = el;
    if (!el || !root.contains(el)) {
      tip.remove();
      current = null;
      return;
    }
    const source = el.getAttribute("data-ascribe-source");
    tip.textContent = source
      ? describeSource(source, el.getAttribute("data-ascribe-via"))
      : `was ${describeSource(el.getAttribute("data-ascribe-was-source") ?? "")}`;
    const box = el.getBoundingClientRect();
    const host = root.getBoundingClientRect();
    tip.style.top = `${box.top - host.top}px`;
    root.append(tip);
  };
  const over = (event: Event): void => show(event.target);
  const out = (event: Event): void => {
    const to = (event as MouseEvent).relatedTarget;
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

/**
 * Takes out of a page's HTML what could run script or take the reader
 * elsewhere: scripts, `<meta>` and `<base>`, event handler attributes, and
 * `javascript:` URLs. A page under review comes from the change being
 * reviewed, so a host disarms it before showing it, whatever its content
 * security policy also blocks.
 */
export function disarm(fragment: DocumentFragment): void {
  for (const el of Array.from(fragment.querySelectorAll("script, meta, base"))) el.remove();
  for (const el of Array.from(fragment.querySelectorAll("*"))) {
    for (const attr of Array.from(el.attributes)) {
      if (/^on/i.test(attr.name) || runsScript(attr.value)) el.removeAttribute(attr.name);
    }
  }
}

/** Whether a URL runs script: `javascript:`, however it's spaced or cased. */
function runsScript(value: string): boolean {
  // URL parsing drops ASCII whitespace and control characters first.
  const kept = Array.from(value)
    .filter((ch) => ch.charCodeAt(0) > 0x20)
    .join("");
  return /^javascript:/i.test(kept);
}
