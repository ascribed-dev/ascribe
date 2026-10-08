// The preview's script, run in the webview. It draws what the extension
// sends: the page's HTML (already rendered by the same renderer as the site),
// with each asset reference pointed at a webview URL, and reports what the
// author does: picks a build, clicks a link, scrolls, double-clicks a block.
//
// The **Page | Site** switch shows the site preview instead: the dev server's
// page for the same file, in a frame on the dev server's own origin, which
// runs its own scripts there (its toolbar app has the review controls).
//
// The page's blocks carry source anchors (site-render contract §7), so the
// preview scrolls with the editor by block, both ways (`blocks.ts`), and, with
// review on, marks what changed against the base (`review.ts`) and shows the
// pull request's threads beside the blocks (`threads.ts`).
//
// The page's HTML is untrusted text: the author's own files, or, in review,
// the change under review. It is parsed into an inert `<template>`, disarmed
// there (scripts, event handlers, and `javascript:` URLs taken out, as the
// review report does), its asset references are rewritten there (so no
// request is made for a reference before it is rewritten), and only then moved
// into the document. Nothing here evaluates it: the content security policy
// (`html.ts`) doesn't allow inline scripts, and none is created.

import { disarm, titleNodes } from "@ascribed/review/marks";
import { blockAt, linesInPage, type Lines } from "../preview/blocks.js";
import { canonicalReference, isExternal, splitFragment } from "../preview/refs.js";
import { Review } from "./review.js";
import { Threads } from "./threads.js";
import type {
  AvailabilityTarget,
  FromWebview,
  ImageReport,
  PreviewBuild,
  RenderReport,
  ShownProblem,
  ToWebview,
  WebviewAsset,
} from "../preview/protocol.js";
import {
  DATA_SOURCE,
  ELEMENT_AVAILABILITY,
  ELEMENT_AVAILABILITY_TARGET,
  ELEMENT_NOTE,
  ELEMENT_STEPS,
  ELEMENT_TABS,
} from "../names.js";

declare function acquireVsCodeApi(): {
  postMessage(message: FromWebview): void;
  getState(): unknown;
  setState(state: unknown): void;
};

const vscode = acquireVsCodeApi();

const role = <T extends HTMLElement>(name: string): T => {
  const element = document.querySelector<T>(`[data-role="${name}"]`);
  if (!element) throw new Error(`the preview shell has no ${name}`);
  return element;
};

const buildSelect = role<HTMLSelectElement>("build");
const buildDescription = role("build-description");
const problemList = role("problems");
const page = role("page");
const title = role("title");
const availability = role("availability");
const content = role("content");
const surfaceGroup = role("surface");
const site = role("site");
const siteProblem = role("site-problem");
const siteFrame = role<HTMLIFrameElement>("site-frame");
const threads = new Threads(content, post, () => review.redraw());
const review = new Review(role("review"), content, post, threads);

const violations: string[] = [];
document.addEventListener("securitypolicyviolation", (event) => {
  violations.push(`${event.violatedDirective} ${event.blockedURI}`);
});

function post(message: FromWebview): void {
  vscode.postMessage(message);
}

buildSelect.addEventListener("change", () => post({ type: "build", name: buildSelect.value }));

for (const button of surfaceGroup.querySelectorAll<HTMLButtonElement>("button[data-surface]")) {
  button.addEventListener("click", () => {
    const surface = button.dataset["surface"] === "site" ? "site" : "page";
    showSurface(surface);
    post({ type: "surface", surface });
  });
}

siteFrame.addEventListener("load", () => {
  if (siteFrame.src) post({ type: "siteShown", url: siteFrame.src });
});

/** Shows the page or the site, and marks the switch. */
function showSurface(surface: "page" | "site"): void {
  document.body.classList.toggle("site-mode", surface === "site");
  site.hidden = surface !== "site";
  for (const button of surfaceGroup.querySelectorAll<HTMLButtonElement>("button[data-surface]")) {
    button.setAttribute("aria-pressed", String(button.dataset["surface"] === surface));
  }
}

/** The site: its page in the frame, or why it can't show, with Try again. */
function showSite(message: Extract<ToWebview, { type: "surface" }>): void {
  showSurface(message.surface);
  if (message.surface === "page") return;
  if ("problem" in message) {
    siteFrame.hidden = true;
    const retry = document.createElement("button");
    retry.type = "button";
    retry.textContent = "Try again";
    retry.addEventListener("click", () => post({ type: "surface", surface: "site" }));
    siteProblem.replaceChildren(message.problem, " ", retry);
    siteProblem.hidden = false;
    return;
  }
  siteProblem.hidden = true;
  siteFrame.hidden = false;
  // Only a new page navigates the frame, so a link followed in it stays.
  if (siteFrame.dataset["url"] !== message.url) {
    siteFrame.dataset["url"] = message.url;
    siteFrame.src = message.url;
  }
}

document.addEventListener("click", (event) => {
  if (event.defaultPrevented || event.button !== 0) return;
  const target = event.target instanceof Element ? event.target.closest("a[href]") : null;
  if (!target || !content.contains(target)) return;
  const href = target.getAttribute("href") ?? "";
  event.preventDefault();
  if (href.startsWith("#")) {
    reveal(decodeURIComponent(href.slice(1)));
    return;
  }
  post({ type: "open", href });
});

let builds: PreviewBuild[] = [];

function showBuilds(next: PreviewBuild[], current: string): void {
  builds = next;
  buildSelect.replaceChildren(
    ...next.map((build) => {
      const option = document.createElement("option");
      option.value = build.name;
      option.textContent = build.editor ? `${build.name} (editor)` : build.name;
      option.title = build.description;
      return option;
    }),
  );
  buildSelect.disabled = next.length === 0;
  buildSelect.value = current;
  buildDescription.textContent = builds.find((b) => b.name === current)?.description ?? "";
}

function showProblems(problems: ShownProblem[]): void {
  problemList.replaceChildren(
    ...problems.map((problem) => {
      const item = document.createElement("li");
      item.dataset["severity"] = problem.severity;
      item.textContent = problem.message;
      if (problem.action === "showOutput") {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = "Show Output";
        button.addEventListener("click", () => post({ type: "showOutput" }));
        item.append(" ", button);
      }
      return item;
    }),
  );
}

// The preview draws the title and the page-level availability itself.
/** The page-level availability, as the sample layout writes it (element contract §4). */
function showAvailability(targets: AvailabilityTarget[]): void {
  if (targets.length === 0) {
    availability.replaceChildren();
    return;
  }
  const wrapper = document.createElement(ELEMENT_AVAILABILITY);
  wrapper.setAttribute("scope", "page");
  targets.forEach((item, index) => {
    if (index > 0) wrapper.append("; ");
    const element = document.createElement(ELEMENT_AVAILABILITY_TARGET);
    element.setAttribute("target", item.target);
    element.setAttribute("dimension", item.dimension);
    element.setAttribute("states", item.states.join(" "));
    if (item.versions && item.versions.length > 0) {
      element.setAttribute("versions", item.versions.join(" "));
    }
    element.textContent = item.text;
    wrapper.append(element);
  });
  availability.replaceChildren(wrapper);
}

/** Points each `<img src>` and `<a href>` that names an asset at the URL of its source file. */
function rewriteAssets(root: ParentNode, assets: WebviewAsset[]): void {
  if (assets.length === 0) return;
  const byReference = new Map(assets.map((a) => [canonicalReference(a.reference), a.uri]));
  const rewrite = (element: Element, attribute: string): void => {
    const value = element.getAttribute(attribute);
    if (value === null || isExternal(value) || value.startsWith("#")) return;
    const uri = byReference.get(canonicalReference(value));
    if (uri === undefined) return;
    element.setAttribute(attribute, uri + splitFragment(value).fragment);
  };
  root.querySelectorAll("img[src]").forEach((img) => rewrite(img, "src"));
  // A link target (a downloadable file) keeps its `href`, since a click is
  // handled by the extension, which maps it back to the source file; the
  // rewrite would only make the address a webview URL nobody navigates to.
}

function reveal(id: string): void {
  const heading = content.querySelector(`[id="${CSS.escape(id)}"]`);
  quietScroll(() => heading?.scrollIntoView({ block: "start" }));
}

/** The page's anchored blocks, in document order, and the lines each stands for. */
let blocks: { element: HTMLElement; lines: Lines | undefined }[] = [];
let pagePath: string | null = null;
/** The block last scrolled to for the editor, so moving within it scrolls nothing. */
let revealedBlock: HTMLElement | undefined;
/** Until when scroll events are the preview's own (a reveal, a re-render), not the reader's. */
let quietUntil = 0;

function quietScroll(scroll: () => void): void {
  quietUntil = Date.now() + 250;
  scroll();
}

function readBlocks(): void {
  blocks = [...content.querySelectorAll<HTMLElement>(`[${DATA_SOURCE}]`)].map((element) => ({
    element,
    lines:
      pagePath === null
        ? undefined
        : linesInPage(
            element.dataset["ascribeSource"] ?? "",
            element.dataset["ascribeVia"],
            pagePath,
          ),
  }));
  revealedBlock = undefined;
}

function revealLine(line: number, ifHidden: boolean): void {
  const index = blockAt(
    blocks.map((b) => b.lines),
    line,
  );
  const block = index === undefined ? undefined : blocks[index];
  if (block === undefined || block.element === revealedBlock) return;
  const rect = block.element.getBoundingClientRect();
  if (ifHidden && rect.bottom > 0 && rect.top < window.innerHeight) return;
  revealedBlock = block.element;
  quietScroll(() => block.element.scrollIntoView({ block: "start" }));
  post({ type: "revealedLine", line, source: block.element.dataset["ascribeSource"] ?? "" });
}

/** The block at the top of the view: the innermost one across the top edge, else the first below it. */
function topBlock(): { element: HTMLElement; lines: Lines } | undefined {
  let across: { element: HTMLElement; lines: Lines } | undefined;
  for (const { element, lines } of blocks) {
    if (lines === undefined) continue;
    const rect = element.getBoundingClientRect();
    if (rect.top <= 1 && rect.bottom > 1) across = { element, lines };
    else if (rect.top > 1) return across ?? { element, lines };
  }
  return across;
}

let scrollQueued = false;
window.addEventListener("scroll", () => {
  if (Date.now() < quietUntil || scrollQueued) return;
  scrollQueued = true;
  requestAnimationFrame(() => {
    scrollQueued = false;
    const block = topBlock();
    if (block === undefined) return;
    revealedBlock = block.element;
    post({ type: "scrolled", line: block.lines.first });
  });
});

content.addEventListener("dblclick", (event) => {
  const target = event.target instanceof Element ? event.target : null;
  for (let node = target; node !== null && content.contains(node); node = node.parentElement) {
    const block = blocks.find((b) => b.element === node);
    if (block?.lines !== undefined) {
      post({ type: "openLine", line: block.lines.first });
      return;
    }
  }
});

function report(): RenderReport {
  const elements: Record<string, number> = {};
  for (const tag of [ELEMENT_TABS, ELEMENT_NOTE, ELEMENT_STEPS, ELEMENT_AVAILABILITY]) {
    elements[tag] = content.querySelectorAll(tag).length;
  }
  return {
    headings: [...content.querySelectorAll("h1, h2, h3, h4, h5, h6")].map((h) => h.id),
    anchored: blocks.filter((b) => b.lines !== undefined).length,
    elements,
    elementsDefined: customElements.get(ELEMENT_TABS) !== undefined,
    violations: [...violations],
    marks: review.counts(),
    reviewHeader: review.headerText(),
  };
}

function imagesSettled(seq: number): void {
  const images = [...content.querySelectorAll("img")];
  const settle = (img: HTMLImageElement): Promise<ImageReport> =>
    new Promise((resolve) => {
      const done = (): void =>
        resolve({
          src: img.src,
          loaded: img.complete && img.naturalWidth > 0,
          width: img.naturalWidth,
        });
      if (img.complete) return done();
      img.addEventListener("load", done, { once: true });
      img.addEventListener("error", done, { once: true });
    });
  void Promise.all(images.map(settle)).then((reports) =>
    post({ type: "images", seq, images: reports }),
  );
}

function render(message: Extract<ToWebview, { type: "render" }>): void {
  showBuilds(message.builds, message.build);
  showProblems(message.problems);
  pagePath = message.path;
  if (message.html === null) {
    page.hidden = true;
    content.replaceChildren();
    review.draw(null, null);
    threads.draw(null, []);
    readBlocks();
    post({ type: "rendered", seq: message.seq, report: report() });
    return;
  }
  const scroll = { x: window.scrollX, y: window.scrollY };
  const template = document.createElement("template");
  template.innerHTML = message.html;
  disarm(template.content);
  rewriteAssets(template.content, message.assets);
  title.replaceChildren(...titleNodes(message.formattedTitle, message.title ?? ""));
  title.hidden = message.title === null;
  showAvailability(message.available);
  content.replaceChildren(template.content);
  page.hidden = false;
  review.draw(message.review ?? null, message.path, (root) => rewriteAssets(root, message.assets));
  threads.draw(message.review?.threads ?? null, message.review?.page?.changes ?? []);
  readBlocks();
  // Replacing the content moves the document; put the reader back where they were.
  quietScroll(() => window.scrollTo(scroll.x, scroll.y));
  post({ type: "rendered", seq: message.seq, report: report() });
  imagesSettled(message.seq);
}

window.addEventListener("message", (event: MessageEvent<ToWebview>) => {
  const message = event.data;
  if (message.type === "render") render(message);
  else if (message.type === "reveal") reveal(message.id);
  else if (message.type === "revealLine") revealLine(message.line, message.ifHidden);
  else if (message.type === "nextPage") review.nextPage(message.page, message.first);
  else if (message.type === "surface") showSite(message);
  else threads.receive(message);
});

post({ type: "ready" });
