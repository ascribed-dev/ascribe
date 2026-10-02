// The preview's script, run in the webview. It draws what the extension
// sends: the page's HTML (already rendered by the same renderer as the site),
// with each asset reference pointed at a webview URL, and reports what the
// author does: picks a build, clicks a link.
//
// The page's HTML is untrusted text from the author's own files: it is parsed
// into an inert `<template>`, its asset references are rewritten there (so no
// request is made for a reference before it is rewritten), and only then moved
// into the document. Nothing here evaluates it: the content security policy
// (`html.ts`) doesn't allow inline scripts, and none is created.

import { canonicalReference, isExternal, splitFragment } from "../preview/refs.js";
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

const violations: string[] = [];
document.addEventListener("securitypolicyviolation", (event) => {
  violations.push(`${event.violatedDirective} ${event.blockedURI}`);
});

function post(message: FromWebview): void {
  vscode.postMessage(message);
}

buildSelect.addEventListener("change", () => post({ type: "build", name: buildSelect.value }));

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
  const wrapper = document.createElement("ascribe-availability");
  wrapper.setAttribute("scope", "page");
  targets.forEach((item, index) => {
    if (index > 0) wrapper.append("; ");
    const element = document.createElement("ascribe-availability-target");
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
  heading?.scrollIntoView({ block: "start" });
}

function report(): RenderReport {
  const elements: Record<string, number> = {};
  for (const tag of ["ascribe-tabs", "ascribe-note", "ascribe-steps", "ascribe-availability"]) {
    elements[tag] = content.querySelectorAll(tag).length;
  }
  return {
    headings: [...content.querySelectorAll("h1, h2, h3, h4, h5, h6")].map((h) => h.id),
    elements,
    elementsDefined: customElements.get("ascribe-tabs") !== undefined,
    violations: [...violations],
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
  if (message.html === null) {
    page.hidden = true;
    content.replaceChildren();
    post({ type: "rendered", seq: message.seq, report: report() });
    return;
  }
  const scroll = { x: window.scrollX, y: window.scrollY };
  const template = document.createElement("template");
  template.innerHTML = message.html;
  rewriteAssets(template.content, message.assets);
  title.textContent = message.title ?? "";
  title.hidden = message.title === null;
  showAvailability(message.available);
  content.replaceChildren(template.content);
  page.hidden = false;
  // Replacing the content moves the document; put the reader back where they were.
  window.scrollTo(scroll.x, scroll.y);
  post({ type: "rendered", seq: message.seq, report: report() });
  imagesSettled(message.seq);
}

window.addEventListener("message", (event: MessageEvent<ToWebview>) => {
  const message = event.data;
  if (message.type === "render") render(message);
  else if (message.type === "reveal") reveal(message.id);
});

post({ type: "ready" });
