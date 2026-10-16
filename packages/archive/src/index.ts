// The search box of an archive's pages. `ascribe versions archive` writes
// this script, bundled, as the folder's `search.js`, and a `<script>` for it
// in each page's sidebar: the box is drawn where that element is, so with
// scripts off there's none. The index is `search-index.js`, added to the page
// as a script the first time the box is used, since a page opened from a
// disk can't fetch a file. Nothing is requested from outside the folder, and
// nothing is stored.
import { search, words, type Index, type Result } from "./search.js";

declare global {
  // What `search-index.js` sets.
  var ascribeArchiveIndex: Index | undefined;
}

/** How many results are listed. */
export const SHOWN = 20;

/** Draws the box in place of `script`, its `data-index` and `data-root` saying where the index and the folder's top are. */
export function mount(script: HTMLScriptElement): HTMLElement {
  const doc = script.ownerDocument;
  const indexSrc = script.dataset["index"] ?? "search-index.js";
  const root = script.dataset["root"] ?? "./";

  const box = doc.createElement("div");
  box.className = "archive-search";
  box.setAttribute("role", "search");
  const label = doc.createElement("label");
  label.htmlFor = "archive-search-input";
  label.textContent = "Search this version";
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

  let loading: Promise<Index | undefined> | undefined;
  const load = (): Promise<Index | undefined> => {
    loading ??= new Promise((resolve) => {
      if (globalThis.ascribeArchiveIndex !== undefined) {
        resolve(globalThis.ascribeArchiveIndex);
        return;
      }
      const tag = doc.createElement("script");
      tag.src = indexSrc;
      tag.addEventListener("load", () => resolve(globalThis.ascribeArchiveIndex));
      tag.addEventListener("error", () => resolve(undefined));
      doc.head.append(tag);
    });
    return loading;
  };

  const show = (index: Index | undefined, query: string): void => {
    list.replaceChildren();
    if (words(query).length === 0) {
      list.hidden = true;
      status.textContent = "";
      return;
    }
    if (index === undefined) {
      list.hidden = true;
      status.textContent = "The search index couldn't be loaded.";
      return;
    }
    const results = search(index, query);
    for (const result of results.slice(0, SHOWN)) list.append(item(doc, result, root));
    list.hidden = results.length === 0;
    status.textContent = summary(results.length, query);
  };

  let latest = "";
  const run = async (): Promise<void> => {
    const query = input.value;
    latest = query;
    if (loading === undefined && words(query).length > 0)
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
    const at = links.indexOf(doc.activeElement as HTMLAnchorElement);
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

/** What the status says about a query's results. */
export function summary(count: number, query: string): string {
  const quoted = `“${query.trim()}”`;
  if (count === 0) return `Nothing found for ${quoted}.`;
  if (count > SHOWN) return `${count} results for ${quoted}; the first ${SHOWN} are shown.`;
  return `${count} ${count === 1 ? "result" : "results"} for ${quoted}.`;
}

function item(doc: Document, result: Result, root: string): HTMLLIElement {
  const li = doc.createElement("li");
  const link = doc.createElement("a");
  link.href = root + result.href;
  const where = doc.createElement("span");
  where.className = "archive-search-where";
  where.textContent =
    result.heading.length > 0 ? `${result.title} › ${result.heading}` : result.title;
  link.append(where);
  if (result.context.length > 0) {
    const context = doc.createElement("span");
    context.className = "archive-search-context";
    for (const piece of result.context) {
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

const current = typeof document === "undefined" ? null : document.currentScript;
if (current instanceof HTMLScriptElement && current.dataset["index"] !== undefined) mount(current);
