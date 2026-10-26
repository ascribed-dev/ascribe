// The copy button on code blocks (CONTRACT.md §12). The script adds a
// <button> after each code block's <pre>, outside it, so the code's text, and
// a copy of it, never holds the button. The button has no text of its own:
// its words, "Copy", "Copied", and "Couldn't copy", are the stylesheet's
// generated content, and the script reads them from there for the button's
// name and for the visually hidden live region that says a click's outcome.
// Each block is an anchor of its own, with a name the script gives it, so
// the stylesheet can put the button in the block's corner. Without the
// script there's no button, and without a clipboard (a page that isn't a
// secure context) the script adds none.

import { CLASS_COPY, CLASS_COPY_STATUS, DATA_COPY } from "./names.js";

/** How long a button says "Copied" or "Couldn't copy" before it says "Copy" again. */
const SHOWN_MS = 2000;

/** What a button says when the stylesheet gives it no words. */
const FALLBACK = { copy: "Copy", copied: "Copied", failed: "Couldn't copy" } as const;

type Outcome = "copied" | "failed";

const timers = new WeakMap<Element, ReturnType<typeof setTimeout>>();

/** The anchor name each block was given; a copy of a block (review's) has none here. */
const anchors = new WeakMap<Element, string>();

let anchorCount = 0;

let status: HTMLElement | null = null;

/** The code element of a code block: a `<pre>` whose child is a `<code>`. */
function codeOf(element: Element): Element | null {
  if (element.localName !== "pre") return null;
  return Array.from(element.children).find((child) => child.localName === "code") ?? null;
}

/**
 * Give `pre` its button, unless it's not a code block or already has one, and
 * an anchor name of its own for the button. Returns a button it made.
 */
function addButton(pre: Element): HTMLButtonElement | undefined {
  if (codeOf(pre) === null || anchors.has(pre)) return undefined;
  const name = `--ascribe-code-${++anchorCount}`;
  anchors.set(pre, name);
  (pre as HTMLElement).style.setProperty("anchor-name", name);
  // A copy of a block whose button was copied with it keeps that button.
  const next = pre.nextElementSibling;
  if (next instanceof HTMLButtonElement && next.classList.contains(CLASS_COPY)) {
    next.style.setProperty("position-anchor", name);
    return undefined;
  }
  const button = document.createElement("button");
  button.type = "button";
  button.className = CLASS_COPY;
  button.style.setProperty("position-anchor", name);
  pre.after(button);
  return button;
}

/** Give every code block in `root`, and `root` itself, a button. */
function addButtons(root: Element): void {
  const made = [root, ...Array.from(root.querySelectorAll("pre"))]
    .map(addButton)
    .filter((button) => button !== undefined);
  // Named after they're all in, so the page's style is worked out once.
  for (const button of made) button.setAttribute("aria-label", wordsOn(button) || FALLBACK.copy);
}

/** The code block a button copies: the nearest `<pre>` before it. */
function blockOf(button: Element): Element | null {
  for (let el = button.previousElementSibling; el !== null; el = el.previousElementSibling) {
    if (codeOf(el) !== null) return el;
  }
  return null;
}

/** The live region, made once, the first time it's needed. */
function liveRegion(): HTMLElement {
  if (status === null || !status.isConnected) {
    status = document.createElement("div");
    status.className = CLASS_COPY_STATUS;
    status.setAttribute("role", "status");
    document.body.append(status);
  }
  return status;
}

/** The words the stylesheet shows on a button, or none. */
function wordsOn(button: Element): string {
  const content = getComputedStyle(button, "::before").content;
  // One string or several (`"Copied" "!"`), as engines serialize `content`.
  const parts = content.match(/"(?:[^"\\]|\\.)*"/g) ?? [];
  return parts.map((part) => part.slice(1, -1).replace(/\\(.)/g, "$1")).join("");
}

/** Show `outcome` on the button for a moment, and say it in the live region. */
function show(button: Element, outcome: Outcome): void {
  const region = liveRegion();
  button.setAttribute(DATA_COPY, outcome);
  // Empty first, so the same words a second time are announced again.
  region.textContent = "";
  region.textContent = wordsOn(button) || FALLBACK[outcome];
  clearTimeout(timers.get(button));
  timers.set(
    button,
    setTimeout(() => {
      button.removeAttribute(DATA_COPY);
      if (region.textContent !== "") region.textContent = "";
    }, SHOWN_MS),
  );
}

async function copy(button: Element): Promise<void> {
  const block = blockOf(button);
  const code = block === null ? null : codeOf(block);
  if (code === null) return;
  // The code as written, without the newline that ends the block, so a
  // pasted command doesn't run before the reader has read it.
  const text = (code.textContent ?? "").replace(/\n+$/, "");
  try {
    await navigator.clipboard.writeText(text);
    show(button, "copied");
  } catch {
    show(button, "failed");
  }
}

function onClick(event: MouseEvent): void {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const button = target.closest(`button.${CLASS_COPY}`);
  if (button !== null) void copy(button);
}

function setup(): void {
  addButtons(document.body);
  // Code blocks a page adds later, or a new body: a preview that redraws, a
  // site that swaps pages in place.
  new MutationObserver((records) => {
    for (const record of records) {
      for (const node of Array.from(record.addedNodes)) {
        if (node instanceof Element) addButtons(node);
      }
    }
  }).observe(document.documentElement, { childList: true, subtree: true });
}

let started = false;

/** Add copy buttons to the page's code blocks, now and as they're added. */
export function startCopyButtons(): void {
  if (started || typeof document === "undefined") return;
  started = true;
  if (typeof navigator.clipboard?.writeText !== "function") return;
  document.addEventListener("click", onClick);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", setup, { once: true });
  } else {
    setup();
  }
}
