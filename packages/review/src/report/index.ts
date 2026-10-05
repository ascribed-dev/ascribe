// The static report's script: `ascribe diff --format html` inlines it, bundled
// with the element library and the marks, into the one HTML file it writes.
// It reads the report's data from the `<script type="application/json">`
// the file holds and draws everything from it: the changed pages of each
// build, and each page rendered with its changes marked. It makes no
// network requests; the file holds every image it shows.

import "@ascribed/elements";
import {
  disarm,
  goTo,
  markChanges,
  setShow,
  showSources,
  type Change,
  type Mark,
  type Show,
} from "../marks/index.js";

/** The report's data, as `ascribe diff --format html` writes it. */
export interface ReportData {
  ascribe_version: string;
  base: { requested: string; commit: string; merge_base: string | null };
  builds: BuildData[];
  /** Each rendered page, once however many builds render it alike. */
  pages: Record<string, RenderedPage>;
  /** Each image, as a `data:` URL, once however many pages use it. */
  images: Record<string, string>;
  /** How many changed pages a report renders, and how many it left out. */
  limit: { pages: number; omitted: number };
  /** The size, in bytes, above which an image isn't included. */
  image_limit: number;
}

export interface BuildData {
  build: string;
  pages: PageData[];
}

/** One changed page: `ascribe diff --format json`'s, and its renderings. */
export interface PageData {
  path: string;
  route: string;
  status: "added" | "removed" | "changed";
  own_file_changed: boolean;
  because: string[];
  page_changed: string[];
  counts: { changed: number; added: number; removed: number; moved: number };
  changes: Change[];
  title: string | null;
  /** The page now and before, as keys of `pages`; null when there's none. */
  now: string | null;
  was: string | null;
  /** Whether it's beyond the report's limit, so not rendered. */
  omitted: boolean;
}

export interface RenderedPage {
  html: string;
  /** Each image reference the HTML writes, and what it is. */
  images: Record<string, ImageRef>;
}

export interface ImageRef {
  /** The image's source file. */
  path: string;
  /** Its key in `images`, when it's included. */
  image?: string;
  /** Its size, when it's too large to include. */
  bytes?: number;
}

interface State {
  build: number;
  page: number;
  show: Show;
  /** The change last gone to, or -1. */
  at: number;
  breakdown: boolean;
  atEnd: boolean;
}

function h<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attributes: Record<string, string> = {},
  children: (Node | string | null)[] = [],
): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [name, value] of Object.entries(attributes)) {
    if (name === "class") el.className = value;
    else el.setAttribute(name, value);
  }
  for (const child of children) if (child !== null) el.append(child);
  return el;
}

function button(
  text: string,
  className: string,
  onClick: () => void,
  attributes: Record<string, string> = {},
): HTMLButtonElement {
  const el = h("button", { type: "button", class: className, ...attributes }, [text]);
  el.addEventListener("click", onClick);
  return el;
}

/** `2 changed · 1 added`. */
export function describeCounts(counts: PageData["counts"]): string {
  const parts: string[] = [];
  for (const kind of ["changed", "added", "removed", "moved"] as const) {
    if (counts[kind] > 0) parts.push(`${counts[kind]} ${kind}`);
  }
  return parts.join(" · ");
}

/** `1 changed page`, or `3 changed pages in 2 builds`. */
export function changedPages(builds: BuildData[]): string {
  const paths = new Set(builds.flatMap((b) => b.pages.map((p) => p.path)));
  if (paths.size === 0) return "no changed pages";
  const pages = paths.size === 1 ? "1 changed page" : `${paths.size} changed pages`;
  return builds.length > 1 ? `${pages} in ${builds.length} builds` : pages;
}

/** What a page in the list says changed. */
function pageSummary(page: PageData): string {
  if (page.status === "added") return "New page";
  if (page.status === "removed") return "Removed";
  const parts = [describeCounts(page.counts)];
  if (page.page_changed.length > 0) parts.push(`${andList(page.page_changed)} changed`);
  return parts.filter(Boolean).join(" · ");
}

function andList(items: string[]): string {
  if (items.length <= 1) return items.join("");
  if (items.length === 2) return `${items[0]} and ${items[1]}`;
  return `${items.slice(0, -1).join(", ")}, and ${items[items.length - 1]}`;
}

function bytes(n: number): string {
  if (n >= 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${Math.ceil(n / 1024)} KB`;
}

/** What an image reference is: as the HTML writes it, or decoded. */
function imageRef(rendered: RenderedPage | undefined, src: string): ImageRef | undefined {
  if (!rendered) return undefined;
  const exact = rendered.images[src];
  if (exact) return exact;
  const decode = (s: string): string => {
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
  return undefined;
}

/**
 * A rendered page as a fragment, with its images from the report and, for
 * images it doesn't hold, a placeholder saying which.
 */
export function pageFragment(data: ReportData, key: string): DocumentFragment {
  const rendered = data.pages[key];
  const template = document.createElement("template");
  template.innerHTML = rendered?.html ?? "";
  const fragment = template.content;
  for (const img of Array.from(fragment.querySelectorAll("img"))) {
    const src = img.getAttribute("src") ?? "";
    const ref = imageRef(rendered, src.split("#")[0] ?? "");
    const url = ref?.image === undefined ? undefined : data.images[ref.image];
    if (url !== undefined) {
      img.setAttribute("src", url);
      img.removeAttribute("srcset");
      continue;
    }
    let text: string;
    if (ref?.bytes !== undefined) {
      text = `Image not included: ${ref.path} (${bytes(ref.bytes)}, over the report's ${bytes(data.image_limit)} limit)`;
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
  // Media and frames would load from the network; none is shown.
  for (const el of Array.from(
    fragment.querySelectorAll("[src], [srcset], [poster], [data], [srcdoc]"),
  )) {
    if (el.localName === "img") continue;
    for (const name of ["src", "srcset", "poster", "data", "srcdoc"]) el.removeAttribute(name);
  }
  disarm(fragment);
  return fragment;
}

/** Reads the report's data and draws the report in `root`. */
export function start(root: HTMLElement, data: ReportData): void {
  const builds = data.builds.filter((b) => b.pages.length > 0);
  const state: State = {
    build: 0,
    page: 0,
    show: "changes",
    at: -1,
    breakdown: false,
    atEnd: false,
  };
  let marks: Mark[] = [];
  let stopSources: (() => void) | undefined;

  const fromHash = (): void => {
    const hash = decodeURIComponent(location.hash.slice(1));
    const slash = hash.indexOf("/");
    if (slash < 0) return;
    const b = builds.findIndex((x) => x.build === hash.slice(0, slash));
    if (b < 0) return;
    const p = builds[b]?.pages.findIndex((x) => x.path === hash.slice(slash + 1)) ?? -1;
    state.build = b;
    state.page = Math.max(p, 0);
  };

  const open = (b: number, p: number, push = true): void => {
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
    root.querySelector<HTMLElement>(".r-main")?.scrollTo?.({ top: 0 });
  };

  const step = (by: number): void => {
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

  const short = (commit: string): string => commit.slice(0, 7);
  const compared =
    data.base.merge_base && data.base.merge_base !== data.base.commit
      ? `${data.base.requested} (${short(data.base.commit)}), from its merge base with HEAD (${short(data.base.merge_base)})`
      : `${data.base.requested} (${short(data.base.commit)})`;
  head.append(
    h("div", { class: "r-title" }, [
      h("b", {}, ["Ascribe review"]),
      ` · ${changedPages(builds)}, compared with ${compared}`,
    ]),
    h("div", { class: "r-sub" }, [
      "Each page as Ascribe renders it, without the site's layout, navigation, or styles.",
    ]),
  );
  if (data.limit.omitted > 0) {
    head.append(
      h("div", { class: "r-notice" }, [
        `This report renders the first ${data.limit.pages} changed pages. ${data.limit.omitted} more are listed by name at the end of the list, not rendered.`,
      ]),
    );
  }

  if (builds.length === 0) {
    main.append(
      h("p", { class: "r-empty" }, ["Nothing changed: no build publishes a page that differs."]),
    );
    side.remove();
    return;
  }

  const renderSide = (): void => {
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
        !page.own_file_changed && page.because.length > 0
          ? h("span", { class: "r-page-via" }, [`via ${page.because.join(", ")}`])
          : null,
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
          omitted.map((p) => h("li", {}, [`${p.path}: ${pageSummary(p)}`])),
        ),
      );
    }
  };

  const renderControls = (): void => {
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
    // A page whose blocks are all as they were changed only in what the
    // render doesn't show, like its frontmatter.
    const blocks = page.status === "changed" && page.changes.length > 0;
    if (page.status === "changed" && !blocks) {
      row.append(h("span", { class: "r-quiet" }, ["No changes to the page's content"]));
    } else if (blocks) {
      const n = marks.length;
      const at = state.at >= 0 && state.at < n && state.show === "changes";
      const text = at
        ? `${state.at + 1} of ${n} on this page`
        : `${n} ${n === 1 ? "change" : "changes"} on this page`;
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
            "aria-expanded": String(state.breakdown),
          },
        ),
      );
    }
    const seg = h("div", { class: "r-seg", role: "group", "aria-label": "Show" });
    for (const [value, label] of [
      ["changes", "Changes"],
      ["will", "As it will be"],
      ["was", "As it was"],
    ] as const) {
      seg.append(
        button(
          label,
          "",
          () => {
            state.show = value;
            render();
          },
          { "aria-pressed": String(state.show === value) },
        ),
      );
    }
    row.append(h("span", { class: "r-show" }, ["Show"]), seg);
    if (blocks) {
      row.append(
        button("↑", "r-ghost r-sq", () => step(-1), { "aria-label": "Previous change" }),
        button("↓", "r-ghost r-sq", () => step(1), { "aria-label": "Next change" }),
      );
    }
    controls.append(row);
    if (state.breakdown)
      controls.append(h("div", { class: "r-legend" }, [describeCounts(page.counts)]));
    if (page.page_changed.length > 0) {
      controls.append(
        h("div", { class: "r-legend" }, [
          blocks
            ? `Also changed: the page's ${andList(page.page_changed)}, which this render doesn't show.`
            : `The page's ${andList(page.page_changed)} changed, which this render doesn't show.`,
        ]),
      );
    }
    if (state.atEnd) {
      const nextIndex = (state.page + 1) % build.pages.length;
      const next = build.pages[nextIndex];
      const notice = h("div", { class: "r-notice" }, [
        h("span", {}, ["That was the last change on this page."]),
      ]);
      if (next && build.pages.length > 1) {
        notice.append(
          button(
            `${nextIndex === 0 ? "First changed page" : "Next changed page"}: ${next.title ?? next.path}`,
            "r-primary",
            () => {
              open(state.build, nextIndex);
              step(1);
            },
          ),
        );
      }
      controls.append(notice);
    }
  };

  const render = (): void => {
    renderSide();
    stopSources?.();
    stopSources = undefined;
    marks = [];
    const build = builds[state.build];
    const page = build?.pages[state.page];
    main.replaceChildren(controls);
    article.replaceChildren();
    article.removeAttribute("data-ascribe-show");
    article.className = "r-page";
    if (!page) return;
    main.append(h("h1", { class: "r-page-heading" }, [page.title ?? page.path]));
    if (page.omitted || (page.now === null && page.was === null)) {
      main.append(
        h("p", { class: "r-empty" }, [
          page.omitted
            ? `${page.path} isn't rendered: the report renders at most ${data.limit.pages} changed pages.`
            : `${page.path} couldn't be rendered.`,
        ]),
      );
      renderControls();
      return;
    }
    const banner = (text: string): void => {
      main.append(h("p", { class: "r-banner" }, [text]));
    };
    if (page.status === "added") {
      if (state.show === "was") {
        banner("This page is new: it didn't exist before.");
      } else {
        if (state.show === "changes") banner("New page: everything on it is added.");
        article.append(pageFragment(data, page.now ?? ""));
      }
    } else if (page.status === "removed") {
      if (state.show === "will") {
        banner("This page is removed: the build doesn't publish it any more.");
      } else {
        if (state.show === "changes")
          banner("Removed page: the build doesn't publish it any more.");
        article.append(pageFragment(data, page.was ?? ""));
      }
    } else {
      article.append(pageFragment(data, page.now ?? ""));
      // In the document first, so its tabs have the labels the marks hint on.
      main.append(article);
      const was = page.was === null ? null : pageFragment(data, page.was);
      marks = markChanges(article, page.changes, { was });
      setShow(article, state.show);
    }
    // Moving it again would rebuild its tabs, without the hints.
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

const data = document.getElementById("ascribe-review-data");
const app = document.getElementById("ascribe-review");
if (data && app) start(app, JSON.parse(data.textContent ?? "{}") as ReportData);
