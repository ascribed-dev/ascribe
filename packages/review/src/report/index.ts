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
  titleNodes,
  type Mark,
  type Show,
} from "../marks/index.js";
import { DATA_SHOW, ID_REPORT, ID_REPORT_DATA } from "../names.js";
import type { BuildData, ImageRef, PageData, RenderedPage, ReportData } from "../shapes.js";

export type { BuildData, ReportData } from "../shapes.js";

interface State {
  /** The index of the build in the list: a build, or a versioned build's lines. */
  build: number;
  page: number;
  /** The index of the line shown, among the build's lines. */
  line: number;
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

/**
 * Puts `text` on the clipboard, through the old `execCommand` way where the
 * Clipboard API isn't allowed, as on a file the browser opened from disk.
 */
async function copyText(text: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    const area = h("textarea", { readonly: "", class: "r-offscreen" }, [text]);
    document.body.append(area);
    area.select();
    let copied = false;
    try {
      copied = document.execCommand("copy");
    } catch {
      copied = false;
    }
    area.remove();
    return copied;
  }
}

/** `2 changed · 1 added`. */
function describeCounts(counts: PageData["counts"]): string {
  const parts: string[] = [];
  for (const kind of ["changed", "added", "removed", "moved"] as const) {
    if (counts[kind] > 0) parts.push(`${counts[kind]} ${kind}`);
  }
  return parts.join(" · ");
}

/** `1 changed page`, or `3 changed pages in 2 builds`. */
export function changedPages(builds: readonly { pages: readonly { path: string }[] }[]): string {
  const paths = new Set(builds.flatMap((b) => b.pages.map((p) => p.path)));
  if (paths.size === 0) return "no changed pages";
  const pages = paths.size === 1 ? "1 changed page" : `${paths.size} changed pages`;
  return builds.length > 1 ? `${pages} in ${builds.length} builds` : pages;
}

/** A changed page once, with what changed on it in each line. */
export interface ListedPage {
  path: string;
  /** Its data in each of the build's lines, `undefined` where it didn't change. */
  in: (PageData | undefined)[];
}

/** A build in the report's list: a build, or a versioned build's lines together. */
export interface ListedBuild {
  /** The name the list shows: the build's, or the versioned build's. */
  build: string;
  /** The builds it stands for: one, or each line, in the data's order. */
  lines: BuildData[];
  /** Each changed page once, in path order. */
  pages: ListedPage[];
}

/**
 * The report's builds as its list shows them: the lines of a versioned build
 * are one entry, each page that changed in any of them once.
 */
export function listBuilds(builds: readonly BuildData[]): ListedBuild[] {
  const out: ListedBuild[] = [];
  for (const build of builds) {
    const name = build.line?.build;
    const same = name === undefined ? undefined : out.find((b) => b.lines[0]?.line?.build === name);
    if (same) same.lines.push(build);
    else out.push({ build: name ?? build.build, lines: [build], pages: [] });
  }
  for (const listed of out) {
    const paths = [...new Set(listed.lines.flatMap((b) => b.pages.map((p) => p.path)))].sort(
      (a, b) => (a < b ? -1 : a > b ? 1 : 0),
    );
    listed.pages = paths.map((path) => ({
      path,
      in: listed.lines.map((b) => b.pages.find((p) => p.path === path)),
    }));
  }
  return out.filter((b) => b.pages.length > 0);
}

/**
 * Which lines a page changed in, for a versioned build: `in every line`, or
 * `only in 3.5`; nothing for a build that isn't versioned.
 */
export function linesNote(listed: ListedBuild, page: ListedPage): string {
  if (listed.lines.length < 2) return "";
  const lines = listed.lines.filter((_, i) => page.in[i] !== undefined);
  if (lines.length === listed.lines.length) return "in every line";
  return `only in ${andList(lines.map((b) => b.line?.line ?? b.build))}`;
}

/**
 * The notice about the working tree's errors, or `null` with none: a page
 * with an error may render oddly, which shouldn't read as the change.
 */
export function errorsNotice(errors: number | undefined): string | null {
  if (!errors) return null;
  const one = errors === 1;
  const count = one ? "1 error" : `${errors} errors`;
  return (
    `The working tree has ${count}, so a page here may not render as it will once ` +
    `${one ? "it's" : "they're"} fixed. \`ascribe check\` lists ${one ? "it" : "them"}.`
  );
}

/** What a page in the list says changed: a moved page says where from. */
export function pageSummary(page: PageData): string {
  if (page.status === "added") return "New page";
  if (page.status === "removed") return "Removed";
  const parts = [describeCounts(page.counts)];
  if (page.page_changed.length > 0) parts.push(`${andList(page.page_changed)} changed`);
  const changed = parts.filter(Boolean);
  if (page.moved_from === undefined) return changed.join(" · ");
  if (changed.length === 0) changed.push("no change to the page");
  return [`Moved from ${page.moved_from}`, ...changed].join(" · ");
}

/**
 * Where a page's change came from besides its own file, as `ascribe diff`
 * says it: `through _fragments/prereqs.md` (`also through` when its own file
 * changed too), and, for a page that changed only because a page it links
 * to moved, `a link changed because guides/configure.md moved`. Empty when
 * there's nothing to say. `only` leads the files a page changed through
 * when its own file didn't change.
 */
export function causesText(page: PageData, only = "through"): string {
  const moved = page.moved_links ?? [];
  const through = page.because.filter((c) => !moved.some((m) => m.to === c));
  const reasons: string[] = [];
  if (through.length > 0) {
    reasons.push(`${page.own_file_changed ? "also through" : only} ${through.join(", ")}`);
  }
  if (!page.own_file_changed && moved.length > 0) {
    const links = moved.length === 1 ? "a link changed" : "links changed";
    reasons.push(`${links} because ${andList(moved.map((m) => m.from))} moved`);
  }
  return reasons.join("; ");
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
      text = `Image not included: ${ref.path} (${bytes(ref.bytes)}, over the preview's ${bytes(data.image_limit)} limit)`;
    } else if (ref) {
      text = `Image not found: ${ref.path}`;
    } else {
      text = `Image not loaded: ${src} (the preview makes no network requests)`;
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
  const builds = listBuilds(data.builds);
  const state: State = {
    build: 0,
    page: 0,
    line: 0,
    show: "changes",
    at: -1,
    breakdown: false,
    atEnd: false,
  };
  let marks: Mark[] = [];
  let stopSources: (() => void) | undefined;

  /** The line a page shows first: the line asked for when it changed there, else the first it changed in. */
  const lineOf = (page: ListedPage | undefined, wanted: number): number => {
    if (!page) return 0;
    if (page.in[wanted] !== undefined) return wanted;
    return Math.max(
      page.in.findIndex((p) => p !== undefined),
      0,
    );
  };

  /** The page shown, in the line shown. */
  const shown = (): PageData | undefined => builds[state.build]?.pages[state.page]?.in[state.line];

  /** A page's address in the report: its build's, or its line's, name, and its path. */
  const hashOf = (b: number, p: number, line: number): string | undefined => {
    const listed = builds[b];
    const page = listed?.pages[p];
    const build = listed?.lines[line]?.build;
    return page && build !== undefined
      ? `#${encodeURIComponent(`${build}/${page.path}`)}`
      : undefined;
  };

  const fromHash = (): void => {
    const hash = decodeURIComponent(location.hash.slice(1));
    const slash = hash.indexOf("/");
    if (slash < 0) return;
    const name = hash.slice(0, slash);
    const b = builds.findIndex((x) => x.lines.some((l) => l.build === name));
    if (b < 0) return;
    const listed = builds[b];
    const p = listed?.pages.findIndex((x) => x.path === hash.slice(slash + 1)) ?? -1;
    state.build = b;
    state.page = Math.max(p, 0);
    state.line = lineOf(
      listed?.pages[state.page],
      listed?.lines.findIndex((l) => l.build === name) ?? 0,
    );
  };

  const open = (b: number, p: number, push = true, line = state.line): void => {
    state.build = b;
    state.page = p;
    state.line = lineOf(builds[b]?.pages[p], line);
    state.at = -1;
    state.atEnd = false;
    state.breakdown = false;
    const hash = hashOf(b, p, state.line);
    if (push && hash !== undefined) history.replaceState(null, "", hash);
    render();
    root.querySelector<HTMLElement>(".r-main")?.scrollTo?.({ top: 0 });
  };

  const step = (by: number): void => {
    if (state.show !== "changes") {
      state.show = "changes";
      if (article.hasAttribute(DATA_SHOW)) setShow(article, "changes");
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
  const errors = errorsNotice(data.working_tree_errors);
  if (errors !== null) {
    // The command, in backticks, shows as code.
    const words = errors.split("`").map((part, i) => (i % 2 === 1 ? h("code", {}, [part]) : part));
    head.append(h("div", { class: "r-notice", role: "note" }, words));
  }
  if (data.limit.omitted > 0) {
    head.append(
      h("div", { class: "r-notice" }, [
        `This preview renders the first ${data.limit.pages} changed pages. ${data.limit.omitted} more are listed by name at the end of the list, not rendered.`,
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
    // Each page as it changed in the line shown, or the first it changed in.
    const first = (listed: ListedPage): PageData | undefined =>
      listed.in[lineOf(listed, state.line)];
    const included = build.pages.filter((p) => first(p)?.omitted === false);
    const omitted = build.pages.filter((p) => first(p)?.omitted !== false);
    side.append(h("h2", {}, [`Changed pages (${build.pages.length})`]));
    const list = h("ul", { class: "r-pages" });
    build.pages.forEach((listed, i) => {
      const page = first(listed);
      if (!page || !included.includes(listed)) return;
      const lines = linesNote(build, listed);
      const link = h("a", { href: hashOf(state.build, i, lineOf(listed, state.line)) ?? "#" }, [
        h(
          "span",
          { class: "r-page-title" },
          titleNodes(page.formatted_title, page.title ?? page.path),
        ),
        h("span", { class: "r-page-path" }, [page.path]),
        h("span", { class: "r-page-counts" }, [pageSummary(page)]),
        lines === "" ? null : h("span", { class: "r-page-lines" }, [lines]),
        !page.own_file_changed && causesText(page) !== ""
          ? h("span", { class: "r-page-via" }, [causesText(page, "via")])
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
          omitted.map((p) => {
            const page = first(p);
            return h("li", {}, [page ? `${p.path}: ${pageSummary(page)}` : p.path]);
          }),
        ),
      );
    }
  };

  // Where Copy prompt says what happened, read out as it changes.
  const status = h("div", { class: "r-status", role: "status" });
  const renderControls = (): void => {
    controls.replaceChildren();
    const build = builds[state.build];
    const listed = build?.pages[state.page];
    const page = shown();
    if (!build || !listed || !page) return;
    if (build.lines.length > 1) {
      // The lines the page changed in, to see each one's rendering.
      const lines = h("div", { class: "r-row r-lines" }, [
        h("span", { class: "r-show" }, [`Line · ${linesNote(build, listed)}`]),
      ]);
      const seg = h("div", { class: "r-seg", role: "group", "aria-label": "Line" });
      build.lines.forEach((line, i) => {
        const changed = listed.in[i] !== undefined;
        const label = line.line
          ? `${line.line.line}${line.line.current ? " (current)" : ""}`
          : line.build;
        const attributes: Record<string, string> = { "aria-pressed": String(i === state.line) };
        if (!changed) {
          attributes.disabled = "";
          attributes.title = `No change in ${line.line?.line ?? line.build}`;
        }
        seg.append(button(label, "", () => open(state.build, state.page, true, i), attributes));
      });
      lines.append(seg);
      controls.append(lines);
    }
    const row = h("div", { class: "r-row" });
    const info = h("span", { class: "r-grow" }, [h("b", {}, [page.path]), ` ${page.route}`]);
    if (page.moved_from !== undefined) info.append(` · moved from ${page.moved_from}`);
    const causes = causesText(page, "changed only through");
    if (causes !== "") info.append(` · ${causes}`);
    row.append(info);
    // A page whose blocks are all as they were changed only in what the
    // render doesn't show, like its frontmatter, or only moved.
    const blocks = page.status === "changed" && page.changes.length > 0;
    if (page.status === "changed" && !blocks) {
      const quiet =
        page.moved_from !== undefined && page.page_changed.length === 0
          ? "No change to the page"
          : "No changes to the page's content";
      row.append(h("span", { class: "r-quiet" }, [quiet]));
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
    if (page.prompt) {
      const prompt = page.prompt;
      const copy = button("Copy prompt", "r-ghost r-copy", () => {
        void copyText(prompt).then((copied) => {
          status.textContent = copied
            ? "Prompt copied. Paste it into your agent."
            : "The prompt couldn't be copied: the browser didn't allow it.";
        });
      });
      copy.title = "Copy a prompt that asks an agent to review this page's changes";
      row.append(copy);
    }
    controls.append(row, status);
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
      const nextListed = build.pages[nextIndex];
      const next = nextListed?.in[lineOf(nextListed, state.line)];
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
    const page = shown();
    main.replaceChildren(controls);
    status.textContent = "";
    article.replaceChildren();
    article.removeAttribute(DATA_SHOW);
    article.className = "r-page";
    if (!page) return;
    main.append(
      h(
        "h1",
        { class: "r-page-heading" },
        titleNodes(page.formatted_title, page.title ?? page.path),
      ),
    );
    if (page.omitted || (page.now === null && page.was === null)) {
      main.append(
        h("p", { class: "r-empty" }, [
          page.omitted
            ? `${page.path} isn't rendered: the preview renders at most ${data.limit.pages} changed pages.`
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

const data = document.getElementById(ID_REPORT_DATA);
const app = document.getElementById(ID_REPORT);
if (data && app) start(app, JSON.parse(data.textContent ?? "{}") as ReportData);
