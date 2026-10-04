// Comment bodies, rendered. A body is Markdown written by other people, so it
// becomes a safe subset of it, built as DOM nodes and never as an HTML
// string: paragraphs, emphasis, strikethrough, code, links, lists, and block
// quotes. Raw HTML stays text. A link is kept only when it goes to the web or
// to an email address, and opens apart from the page; an image becomes a link
// to it, so nothing is loaded until the reader asks.

/** The schemes a link in a comment may have. */
const SAFE_SCHEMES = new Set(["http:", "https:", "mailto:"]);

/** `url` when it's a link a comment may carry, else `undefined`. */
export function safeUrl(url: string): string | undefined {
  // URL parsing drops ASCII whitespace and control characters first.
  const kept = Array.from(url.trim())
    .filter((ch) => ch.charCodeAt(0) > 0x20)
    .join("");
  let parsed: URL;
  try {
    parsed = new URL(kept);
  } catch {
    return undefined;
  }
  return SAFE_SCHEMES.has(parsed.protocol) ? parsed.href : undefined;
}

/** Renders a comment's Markdown body in `doc`, as a fragment of plain DOM nodes. */
export function renderMarkdown(doc: Document, text: string): DocumentFragment {
  const fragment = doc.createDocumentFragment();
  renderBlocks(doc, fragment, text.replace(/\r\n?/g, "\n").split("\n"), 0);
  return fragment;
}

const FENCE = /^ {0,3}(`{3,}|~{3,})/;
const QUOTE = /^ {0,3}> ?/;
const BULLET = /^( {0,3})([-*+])[ \t]+/;
const ORDERED = /^( {0,3})(\d{1,9})[.)][ \t]+/;
const HEADING = /^ {0,3}#{1,6}[ \t]+/;
const RULE = /^ {0,3}([-*_])(?:[ \t]*\1){2,}[ \t]*$/;

function isBlank(line: string): boolean {
  return line.trim() === "";
}

function startsBlock(line: string): boolean {
  return (
    FENCE.test(line) ||
    QUOTE.test(line) ||
    BULLET.test(line) ||
    ORDERED.test(line) ||
    HEADING.test(line) ||
    RULE.test(line)
  );
}

function renderBlocks(doc: Document, parent: Node, lines: string[], depth: number): void {
  let i = 0;
  while (i < lines.length) {
    const line = lines[i] ?? "";
    if (isBlank(line)) {
      i++;
      continue;
    }
    const fence = FENCE.exec(line);
    if (fence) {
      const marker = fence[1] ?? "```";
      const body: string[] = [];
      i++;
      while (i < lines.length && !(lines[i] ?? "").trimStart().startsWith(marker)) {
        body.push(lines[i] ?? "");
        i++;
      }
      i++;
      const pre = doc.createElement("pre");
      const code = doc.createElement("code");
      code.textContent = body.join("\n");
      pre.append(code);
      parent.appendChild(pre);
      continue;
    }
    if (QUOTE.test(line) && depth < 8) {
      const body: string[] = [];
      while (i < lines.length && QUOTE.test(lines[i] ?? "")) {
        body.push((lines[i] ?? "").replace(QUOTE, ""));
        i++;
      }
      const quote = doc.createElement("blockquote");
      renderBlocks(doc, quote, body, depth + 1);
      parent.appendChild(quote);
      continue;
    }
    const bullet = BULLET.exec(line);
    const ordered = bullet ? null : ORDERED.exec(line);
    if ((bullet || ordered) && depth < 8) {
      const list = doc.createElement(bullet ? "ul" : "ol");
      const start = ordered ? Number(ordered[2]) : 1;
      if (ordered && start !== 1) list.setAttribute("start", String(start));
      const pattern = bullet ? BULLET : ORDERED;
      while (i < lines.length) {
        const match = pattern.exec(lines[i] ?? "");
        if (!match) break;
        const indent = match[0].length;
        const body = [(lines[i] ?? "").slice(indent)];
        i++;
        // The item goes on over lines indented under it, and lazily over the
        // next lines of its paragraph.
        while (i < lines.length) {
          const next = lines[i] ?? "";
          if (isBlank(next)) {
            const after = lines[i + 1] ?? "";
            if (/^\s/.test(after) && !isBlank(after) && leading(after) >= indent) {
              body.push("");
              i++;
              continue;
            }
            break;
          }
          if (leading(next) >= indent) body.push(next.slice(indent));
          else if (!startsBlock(next) && !isBlank(body.at(-1) ?? "")) body.push(next.trim());
          else break;
          i++;
        }
        const item = doc.createElement("li");
        renderBlocks(doc, item, body, depth + 1);
        // A tight item's one paragraph shows without a paragraph's margins.
        const only = item.firstElementChild;
        if (item.childElementCount === 1 && only?.localName === "p") {
          only.replaceWith(...Array.from(only.childNodes));
        }
        list.append(item);
        if (isBlank(lines[i] ?? "") && pattern.test(lines[i + 1] ?? "")) i++;
      }
      parent.appendChild(list);
      continue;
    }
    if (RULE.test(line)) {
      parent.appendChild(doc.createElement("hr"));
      i++;
      continue;
    }
    if (HEADING.test(line)) {
      // A heading in a comment reads as a strong line, not a page heading.
      const p = doc.createElement("p");
      const strong = doc.createElement("strong");
      renderInline(doc, strong, line.replace(HEADING, "").replace(/[ \t]+#+[ \t]*$/, ""), 0);
      p.append(strong);
      parent.appendChild(p);
      i++;
      continue;
    }
    const body: string[] = [];
    while (i < lines.length) {
      const next = lines[i] ?? "";
      if (isBlank(next) || (body.length > 0 && startsBlock(next))) break;
      body.push(next.trim());
      i++;
    }
    const p = doc.createElement("p");
    // GitHub keeps a comment's line breaks.
    body.forEach((text, n) => {
      if (n > 0) p.append(doc.createElement("br"));
      renderInline(doc, p, text, 0);
    });
    parent.appendChild(p);
  }
}

function leading(line: string): number {
  return line.length - line.trimStart().length;
}

const ESCAPABLE = /[\\`*_{}[\]()#+\-.!~<>|"']/;

/**
 * What one line's searches for closing markers found, so each search runs
 * once: a body is anyone's text, and an unclosed marker repeated must not
 * make a line take quadratic time.
 */
interface Scan {
  /** For each `[`, the index of the `]` that closes it, or -1. Made on the first `[`. */
  brackets?: Int32Array;
  /** By closing marker: the first index from which a search for it found none. */
  none: Map<string, number>;
}

/** Whether a search for `marker` from `from` already found nothing. */
function knownMissing(scan: Scan, marker: string, from: number): boolean {
  const none = scan.none.get(marker);
  return none !== undefined && from >= none;
}

function markMissing(scan: Scan, marker: string, from: number): void {
  const none = scan.none.get(marker);
  if (none === undefined || from < none) scan.none.set(marker, from);
}

/** Renders one line's inline Markdown into `parent`. */
function renderInline(doc: Document, parent: Node, text: string, depth: number): void {
  const scan: Scan = { none: new Map() };
  let plain = "";
  const flush = (): void => {
    if (plain) parent.appendChild(doc.createTextNode(plain));
    plain = "";
  };
  let i = 0;
  while (i < text.length) {
    const ch = text[i] ?? "";
    const rest = text.slice(i);
    if (ch === "\\" && ESCAPABLE.test(text[i + 1] ?? "")) {
      plain += text[i + 1];
      i += 2;
      continue;
    }
    if (ch === "`") {
      const ticks = /^`+/.exec(rest)?.[0] ?? "`";
      const from = i + ticks.length;
      const end = knownMissing(scan, ticks, from) ? -1 : text.indexOf(ticks, from);
      if (end < 0) markMissing(scan, ticks, from);
      if (end > 0) {
        flush();
        const code = doc.createElement("code");
        let inner = text.slice(i + ticks.length, end);
        if (/^ .*[^ ].* $/.test(inner)) inner = inner.slice(1, -1);
        code.textContent = inner;
        parent.appendChild(code);
        i = end + ticks.length;
        continue;
      }
      plain += ticks;
      i += ticks.length;
      continue;
    }
    if (ch === "!" && text[i + 1] === "[") {
      const link = parseLink(text, i + 1, scan);
      if (link) {
        flush();
        const url = safeUrl(link.url);
        const label = link.label.trim() ? `Image: ${link.label.trim()}` : "Image";
        if (url) parent.appendChild(anchor(doc, url, label));
        else parent.appendChild(doc.createTextNode(label));
        i = link.end;
        continue;
      }
    }
    if (ch === "[") {
      const link = parseLink(text, i, scan);
      if (link) {
        flush();
        const url = safeUrl(link.url);
        if (url && depth < 4) {
          const a = anchor(doc, url, "");
          renderInline(doc, a, link.label, depth + 1);
          parent.appendChild(a);
        } else {
          renderInline(doc, parent, link.label, depth + 1);
        }
        i = link.end;
        continue;
      }
    }
    if (ch === "<") {
      const auto = /^<((?:https?|mailto):[^\s<>]+)>/i.exec(rest);
      const url = auto ? safeUrl(auto[1] ?? "") : undefined;
      if (auto && url) {
        flush();
        parent.appendChild(anchor(doc, url, auto[1] ?? url));
        i += auto[0].length;
        continue;
      }
    }
    if ((ch === "h" || ch === "H") && /^https?:\/\//i.test(rest) && !/\w/.test(text[i - 1] ?? "")) {
      // Longer than any address a browser keeps: text, and a bound on the
      // search, so a line of failed addresses can't take quadratic time.
      const found = /^https?:\/\/[^\s<]{1,2048}/i.exec(rest)?.[0] ?? "";
      const match = /[^\s<]/.test(rest[found.length] ?? " ") ? "" : found;
      // Trailing punctuation ends the sentence, not the address.
      const bare = match.replace(/[.,:;!?'")\]]+$/, "");
      const url = safeUrl(bare);
      if (url) {
        flush();
        parent.appendChild(anchor(doc, url, bare));
        i += bare.length;
        continue;
      }
    }
    if ((ch === "*" || ch === "_" || ch === "~") && depth < 4) {
      const span = parseEmphasis(text, i, scan);
      if (span) {
        flush();
        const el = doc.createElement(span.tag);
        renderInline(doc, el, span.inner, depth + 1);
        parent.appendChild(el);
        i = span.end;
        continue;
      }
    }
    plain += ch;
    i++;
  }
  flush();
}

function anchor(doc: Document, url: string, text: string): HTMLAnchorElement {
  const a = doc.createElement("a");
  a.href = url;
  a.target = "_blank";
  a.rel = "noopener noreferrer";
  if (text) a.textContent = text;
  return a;
}

/** `[label](url "title")` starting at `start`, or `undefined`. */
function parseLink(
  text: string,
  start: number,
  scan: Scan,
): { label: string; url: string; end: number } | undefined {
  scan.brackets ??= matchBrackets(text);
  const close = scan.brackets[start] ?? -1;
  if (close < 0 || text[close + 1] !== "(") return undefined;
  const target =
    /^\(\s*(<[^<>]*>|[^\s()]*(?:\([^\s()]*\)[^\s()]*)*)(?:\s+(?:"[^"]*"|'[^']*'))?\s*\)/.exec(
      text.slice(close + 1),
    );
  if (!target) return undefined;
  const url = (target[1] ?? "").replace(/^<|>$/g, "");
  return { label: text.slice(start + 1, close), url, end: close + 1 + target[0].length };
}

/** For each `[` in `text`, the index of the `]` that closes it (escapes skipped), or -1. */
function matchBrackets(text: string): Int32Array {
  const close = new Int32Array(text.length).fill(-1);
  const open: number[] = [];
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === "\\") i++;
    else if (ch === "[") open.push(i);
    else if (ch === "]") {
      const at = open.pop();
      if (at !== undefined) close[at] = i;
    }
  }
  return close;
}

/** Emphasis, strong emphasis, or strikethrough starting at `start`. */
function parseEmphasis(
  text: string,
  start: number,
  scan: Scan,
): { tag: "em" | "strong" | "del"; inner: string; end: number } | undefined {
  const ch = text[start] ?? "";
  // Only the first three characters of a run decide its size.
  const run = (/^([*_~])\1*/.exec(text.slice(start, start + 3))?.[0] ?? "").length;
  const size = ch === "~" ? (run >= 2 ? 2 : 0) : Math.min(run, 2);
  if (size === 0) return undefined;
  const marker = ch.repeat(size);
  const open = start + size;
  // An opening marker has text right after it; `_` inside a word isn't one.
  if (/\s/.test(text[open] ?? " ")) return undefined;
  if (ch === "_" && /\w/.test(text[start - 1] ?? "")) return undefined;
  // Whether a marker closes depends only on what's beside it, so a search
  // that found no closer from one place finds none from any later one.
  if (knownMissing(scan, marker, open + 1)) return undefined;
  let from = open + 1;
  while (from <= text.length) {
    const close = text.indexOf(marker, from);
    if (close < 0) {
      markMissing(scan, marker, open + 1);
      return undefined;
    }
    const closesHere =
      !/\s/.test(text[close - 1] ?? " ") &&
      text[close + size] !== ch &&
      !(ch === "_" && /\w/.test(text[close + size] ?? ""));
    if (closesHere) {
      const tag = ch === "~" ? "del" : size === 2 ? "strong" : "em";
      return { tag, inner: text.slice(open, close), end: close + size };
    }
    from = close + 1;
  }
  markMissing(scan, marker, open + 1);
  return undefined;
}
