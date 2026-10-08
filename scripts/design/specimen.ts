// Builds design/specimen.html: every candidate palette and type scale in
// design/candidates/*.toml, and every candidate mark (design/candidates/
// mark-*.svg), shown together in light and dark, with the contrast of every
// pairing in design/candidates/pairs.toml.
//
//   node scripts/design/specimen.ts
//
// The page is one self-contained file to open in a browser. It renders the
// element library's, review's, and the docs site's own stylesheets with each
// candidate's colors, so adding a candidate adds no code.

import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

export const ROOT = fileURLToPath(new URL("../..", import.meta.url));
export const CANDIDATES = join(ROOT, "design", "candidates");
export const SPECIMEN = join(ROOT, "design", "specimen.html");

export type Scheme = "light" | "dark";
export const SCHEMES: readonly Scheme[] = ["light", "dark"];

/** What a pairing must reach: 4.5:1 for text, 3:1 for a graphic. */
export const MINIMUM = { text: 4.5, graphic: 3 } as const;
export type Kind = keyof typeof MINIMUM;

export interface Pair {
  fg: string;
  bg: string;
  kind: Kind;
  where: string;
}

export interface Distinct {
  a: string;
  b: string;
  where: string;
}

export interface TypeStep {
  size: string;
  line: number;
  weight: number;
}

export interface Candidate {
  /** The file's name without `.toml`. */
  id: string;
  name: string;
  summary: string;
  baseline: boolean;
  /** Hue, then step, then hex. */
  palette: Map<string, Map<string, string>>;
  /** Each semantic color's palette reference and hex, per scheme. */
  colors: Map<string, Record<Scheme, { ref: string; hex: string }>>;
  font: { ui: string; mono: string };
  type: Map<string, TypeStep>;
}

export interface Mark {
  /** The file's name without `mark-` and `.svg`, or `chosen` for design/mark.svg. */
  id: string;
  /** Where it is, from the repository's root. */
  path: string;
  title: string;
  description: string;
  svg: string;
  /** The wordmark drawn to go with it, `wordmark-<id>.svg`, if there is one. */
  wordmark?: string;
}

/** The semantic colors every candidate defines: those the pairs and the stylesheets use. */
export const COLORS = [
  "text",
  "muted",
  "background",
  "surface",
  "border",
  "accent",
  "accent-surface",
  "accent-text",
  "code-background",
  "focus",
  "note",
  "note-background",
  "tip",
  "tip-background",
  "important",
  "important-background",
  "warning",
  "warning-background",
  "caution",
  "caution-background",
  "state",
  "state-background",
  "added",
  "added-background",
  "changed",
  "changed-background",
  "removed",
  "removed-background",
  "moved",
  "moved-background",
  "flash",
] as const;

/** The type scale's steps, smallest first. */
export const TYPE_STEPS = ["caption", "small", "body", "lead", "h3", "h2", "h1"] as const;

/**
 * Each stylesheet's custom properties, from the semantic colors and fonts, so
 * the specimen can render today's stylesheets with a candidate's values.
 */
export const EMIT: Record<string, Record<string, string>> = {
  "site/src/styles/site.css": {
    "--font": "font.ui",
    "--font-mono": "font.mono",
    "--text": "color.text",
    "--muted": "color.muted",
    "--background": "color.background",
    "--surface": "color.surface",
    "--border": "color.border",
    "--accent": "color.accent",
    "--accent-surface": "color.accent-surface",
    "--code-background": "color.code-background",
  },
  "packages/elements/css/style.css": {
    "--ascribe-border-color": "color.border",
    "--ascribe-muted-color": "color.muted",
    "--ascribe-note-color": "color.note",
    "--ascribe-note-background": "color.note-background",
    "--ascribe-tip-color": "color.tip",
    "--ascribe-tip-background": "color.tip-background",
    "--ascribe-important-color": "color.important",
    "--ascribe-important-background": "color.important-background",
    "--ascribe-warning-color": "color.warning",
    "--ascribe-warning-background": "color.warning-background",
    "--ascribe-caution-color": "color.caution",
    "--ascribe-caution-background": "color.caution-background",
    "--ascribe-steps-color": "color.accent",
    "--ascribe-steps-marker-text-color": "color.accent-text",
    "--ascribe-tab-color": "color.muted",
    "--ascribe-tab-active-color": "color.accent",
    "--ascribe-tab-focus-color": "color.focus",
    "--ascribe-tab-hover-background": "color.surface",
    "--ascribe-state-color": "color.state",
    "--ascribe-state-background": "color.state-background",
    "--ascribe-state-ga-color": "color.tip",
    "--ascribe-state-ga-background": "color.tip-background",
    "--ascribe-state-preview-color": "color.important",
    "--ascribe-state-preview-background": "color.important-background",
    "--ascribe-state-beta-color": "color.note",
    "--ascribe-state-beta-background": "color.note-background",
    "--ascribe-state-deprecated-color": "color.warning",
    "--ascribe-state-deprecated-background": "color.warning-background",
    "--ascribe-state-removed-color": "color.caution",
    "--ascribe-state-removed-background": "color.caution-background",
  },
  "packages/review/src/marks/marks.css": {
    "--ascribe-review-added": "color.added",
    "--ascribe-review-added-background": "color.added-background",
    "--ascribe-review-changed": "color.changed",
    "--ascribe-review-changed-background": "color.changed-background",
    "--ascribe-review-removed": "color.removed",
    "--ascribe-review-removed-background": "color.removed-background",
    "--ascribe-review-moved": "color.moved",
    "--ascribe-review-moved-background": "color.moved-background",
    "--ascribe-review-muted": "color.muted",
    "--ascribe-review-surface": "color.background",
    "--ascribe-review-flash": "color.flash",
    "--ascribe-review-focus": "color.focus",
    "--ascribe-review-mono": "font.mono",
    "--ascribe-review-text": "color.text",
    "--ascribe-review-border": "color.border",
    "--ascribe-review-thread": "color.surface",
    "--ascribe-review-input": "color.background",
    "--ascribe-review-accent-text": "color.accent-text",
  },
};

// Reading ------------------------------------------------------------------

const HEX = /^#[0-9a-f]{6}$/;

function table(value: unknown, where: string): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error(`${where}: expected a table`);
  }
  return value as Record<string, unknown>;
}

function string(value: unknown, where: string): string {
  if (typeof value !== "string") throw new Error(`${where}: expected a string`);
  return value;
}

function number(value: unknown, where: string): number {
  if (typeof value !== "number") throw new Error(`${where}: expected a number`);
  return value;
}

/** Reads one candidate file; throws, naming the key, on anything missing or wrong. */
export function readCandidate(id: string, text: string): Candidate {
  const file = `${id}.toml`;
  const doc = parse(text);
  const meta = table(doc.candidate, `${file} [candidate]`);

  const palette = new Map<string, Map<string, string>>();
  for (const [hue, steps] of Object.entries(table(doc.palette, `${file} [palette]`))) {
    const ramp = new Map<string, string>();
    for (const [step, hex] of Object.entries(table(steps, `${file} palette.${hue}`))) {
      const value = string(hex, `${file} palette.${hue}.${step}`);
      if (!HEX.test(value)) {
        throw new Error(`${file} palette.${hue}.${step}: "${value}" isn't a lowercase #rrggbb`);
      }
      ramp.set(step, value);
    }
    palette.set(hue, ramp);
  }

  const colorTable = table(doc.color, `${file} [color]`);
  const colors: Candidate["colors"] = new Map();
  for (const name of COLORS) {
    const entry = table(colorTable[name], `${file} color.${name}`);
    const pick = (scheme: Scheme) => {
      const ref = string(entry[scheme], `${file} color.${name}.${scheme}`);
      const [hue = "", step = ""] = ref.split(".");
      const hex = palette.get(hue)?.get(step);
      if (hex === undefined) {
        throw new Error(`${file} color.${name}.${scheme}: no palette entry "${ref}"`);
      }
      return { ref, hex };
    };
    colors.set(name, { light: pick("light"), dark: pick("dark") });
  }
  for (const name of Object.keys(colorTable)) {
    if (!(COLORS as readonly string[]).includes(name)) {
      throw new Error(`${file} color.${name}: not a semantic color the stylesheets use`);
    }
  }

  const font = table(doc.font, `${file} [font]`);
  const typeTable = table(doc.type, `${file} [type]`);
  const type = new Map<string, TypeStep>();
  for (const step of TYPE_STEPS) {
    const where = `${file} type.${step}`;
    const entry = table(typeTable[step], where);
    type.set(step, {
      size: string(entry.size, `${where}.size`),
      line: number(entry.line, `${where}.line`),
      weight: number(entry.weight, `${where}.weight`),
    });
  }

  return {
    id,
    name: string(meta.name, `${file} candidate.name`),
    summary: string(meta.summary, `${file} candidate.summary`),
    baseline: meta.baseline === true,
    palette,
    colors,
    font: { ui: string(font.ui, `${file} font.ui`), mono: string(font.mono, `${file} font.mono`) },
    type,
  };
}

export function readPairs(text: string): { pairs: Pair[]; distinct: Distinct[] } {
  const doc = parse(text) as { pair?: Pair[]; distinct?: Distinct[] };
  const known = (name: string) => {
    if (!(COLORS as readonly string[]).includes(name)) {
      throw new Error(`pairs.toml: "${name}" isn't a semantic color`);
    }
  };
  const pairs = doc.pair ?? [];
  for (const pair of pairs) {
    known(pair.fg);
    known(pair.bg);
    if (!(pair.kind in MINIMUM)) throw new Error(`pairs.toml: kind "${pair.kind}"`);
  }
  const distinct = doc.distinct ?? [];
  for (const d of distinct) {
    known(d.a);
    known(d.b);
  }
  return { pairs, distinct };
}

/** Reads one candidate mark, taking its name and note from its `<title>` and `<desc>`. */
export function readMark(id: string, svg: string, file = `design/candidates/mark-${id}.svg`): Mark {
  if (!/<svg [^>]*viewBox="0 0 \d+ \d+"/.test(svg)) throw new Error(`${file}: no viewBox`);
  if (/<(image|text|style|foreignObject)\b|font-family|xlink:href="data:/.test(svg)) {
    throw new Error(`${file}: an image, text, font, or style; a mark is paths only`);
  }
  const title = /<title>([^<]*)<\/title>/.exec(svg)?.[1];
  const description = /<desc>([^<]*)<\/desc>/.exec(svg)?.[1];
  if (title === undefined || description === undefined) {
    throw new Error(`${file}: needs a <title> and a <desc>`);
  }
  return {
    id,
    path: file,
    title,
    description: description.trim().replace(/\s+/g, " "),
    svg: svg.trim(),
  };
}

/** Reads a wordmark: paths only, in a viewBox, like a mark. */
export function readWordmark(file: string, svg: string): string {
  if (!/<svg [^>]*viewBox="[-\d. ]+"/.test(svg)) throw new Error(`${file}: no viewBox`);
  if (/<(image|text|style|foreignObject)\b|font-family|xlink:href="data:/.test(svg)) {
    throw new Error(`${file}: an image, text, font, or style; a wordmark is paths only`);
  }
  return svg.trim();
}

export interface Inputs {
  candidates: Candidate[];
  marks: Mark[];
  pairs: Pair[];
  distinct: Distinct[];
  /** Each stylesheet in EMIT, by path. */
  stylesheets: Record<string, string>;
}

/** Everything the specimen shows, read from the repository. Baseline first, then by name. */
export function readInputs(): Inputs {
  const files = readdirSync(CANDIDATES).sort();
  const candidates = files
    .filter((f) => f.endsWith(".toml") && f !== "pairs.toml")
    .map((f) => readCandidate(f.slice(0, -5), readFileSync(join(CANDIDATES, f), "utf8")))
    .sort((a, b) => Number(b.baseline) - Number(a.baseline) || a.id.localeCompare(b.id));
  const marks = files
    .filter((f) => f.startsWith("mark-") && f.endsWith(".svg"))
    .map((f) => {
      const mark = readMark(f.slice(5, -4), readFileSync(join(CANDIDATES, f), "utf8"));
      const wordmark = `wordmark-${mark.id}.svg`;
      if (!files.includes(wordmark)) return mark;
      return {
        ...mark,
        wordmark: readWordmark(wordmark, readFileSync(join(CANDIDATES, wordmark), "utf8")),
      };
    });
  // The chosen mark, once there is one, comes first.
  const design = join(ROOT, "design");
  if (existsSync(join(design, "mark.svg"))) {
    const chosen = readMark(
      "chosen",
      readFileSync(join(design, "mark.svg"), "utf8"),
      "design/mark.svg",
    );
    const wordmark = join(design, "wordmark.svg");
    marks.unshift(
      existsSync(wordmark)
        ? {
            ...chosen,
            wordmark: readWordmark("design/wordmark.svg", readFileSync(wordmark, "utf8")),
          }
        : chosen,
    );
  }
  const { pairs, distinct } = readPairs(readFileSync(join(CANDIDATES, "pairs.toml"), "utf8"));
  const stylesheets: Record<string, string> = {};
  for (const path of Object.keys(EMIT)) {
    stylesheets[path] = readFileSync(join(ROOT, path), "utf8");
  }
  return { candidates, marks, pairs, distinct, stylesheets };
}

// Color ----------------------------------------------------------------------

function channels(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

function linear(channel: number): number {
  const c = channel / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

function luminance(hex: string): number {
  const [r, g, b] = channels(hex).map(linear) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG 2's contrast ratio between two colors, from 1 to 21. */
export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/** Linear-RGB matrices that show a color as a reader with no red or no green cones sees it (Viénot, Brettel, and Mollon, 1999). */
const VISION = {
  normal: null,
  protan: [
    [0.11238, 0.88762, 0],
    [0.11238, 0.88762, 0],
    [0.00401, -0.00401, 1],
  ],
  deutan: [
    [0.29275, 0.70725, 0],
    [0.29275, 0.70725, 0],
    [-0.02234, 0.02234, 1],
  ],
} as const;
export type Vision = keyof typeof VISION;

function oklab([r, g, b]: [number, number, number]): [number, number, number] {
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return [
    0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  ];
}

function seen(hex: string, vision: Vision): [number, number, number] {
  const rgb = channels(hex).map(linear) as [number, number, number];
  const matrix = VISION[vision];
  if (matrix === null) return oklab(rgb);
  const [a, b, c] = matrix;
  const dot = (row: readonly number[]) =>
    Math.max(0, (row[0] ?? 0) * rgb[0] + (row[1] ?? 0) * rgb[1] + (row[2] ?? 0) * rgb[2]);
  return oklab([dot(a), dot(b), dot(c)]);
}

/** How far apart two colors look, as a distance in OKLab, to a reader with the given vision. */
export function difference(a: string, b: string, vision: Vision): number {
  const [l1, a1, b1] = seen(a, vision);
  const [l2, a2, b2] = seen(b, vision);
  return Math.hypot(l1 - l2, a1 - a2, b1 - b2);
}

/** Below this OKLab distance, two colors side by side are hard to tell apart. */
export const DISTINCT = 0.1;

export interface Check {
  pair: Pair;
  scheme: Scheme;
  fg: string;
  bg: string;
  ratio: number;
  pass: boolean;
}

/** Every pairing in both schemes, in the order of pairs.toml. */
export function checkPairs(candidate: Candidate, pairs: Pair[]): Check[] {
  return pairs.flatMap((pair) =>
    SCHEMES.map((scheme) => {
      const fg = hexOf(candidate, pair.fg, scheme);
      const bg = hexOf(candidate, pair.bg, scheme);
      const ratio = contrast(fg, bg);
      // Compared as shown, to two places, so a pair the page shows as 4.50 passes.
      const pass = Math.round(ratio * 100) / 100 >= MINIMUM[pair.kind];
      return { pair, scheme, fg, bg, ratio, pass };
    }),
  );
}

function hexOf(candidate: Candidate, name: string, scheme: Scheme): string {
  const color = candidate.colors.get(name);
  if (color === undefined) throw new Error(`${candidate.id}: no color "${name}"`);
  return color[scheme].hex;
}

/** The value a token reference (`color.accent`, `font.mono`) has in a scheme. */
function tokenValue(candidate: Candidate, token: string, scheme: Scheme): string {
  const [group, name = ""] = token.split(/\.(.*)/);
  if (group === "font") return name === "mono" ? candidate.font.mono : candidate.font.ui;
  return hexOf(candidate, name, scheme);
}

// Writing --------------------------------------------------------------------

function esc(text: string): string {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function ratio(value: number): string {
  return value.toFixed(2);
}

/** The mark at a size, with its title as its accessible name. */
function sized(mark: Mark, size: number): string {
  return mark.svg
    .replace(/<desc>[^<]*<\/desc>\s*/, "")
    .replace("<svg ", `<svg width="${size}" height="${size}" role="img" `);
}

/** The custom properties a scheme column uses for its own chrome. */
function schemeVars(candidate: Candidate, scheme: Scheme): string {
  return COLORS.map((name) => `--c-${name}: ${hexOf(candidate, name, scheme)}`).join("; ");
}

/** The declarations that give a preview page a candidate's colors and type. */
function previewCss(candidate: Candidate, scheme: Scheme): string {
  const props = new Map<string, string>();
  for (const map of Object.values(EMIT)) {
    for (const [prop, token] of Object.entries(map)) {
      props.set(prop, tokenValue(candidate, token, scheme));
    }
  }
  const decls = [...props].map(([prop, value]) => `${prop}: ${value};`).join(" ");
  const t = (step: string) => {
    const s = candidate.type.get(step);
    return s === undefined
      ? ""
      : `font-size: ${s.size}; line-height: ${s.line}; font-weight: ${s.weight};`;
  };
  // Three :root classes outrank every scheme rule in the stylesheets.
  return [
    `:root:root:root { color-scheme: ${scheme}; ${decls} }`,
    `body { ${t("body")} }`,
    `.page h1 { ${t("h1")} } .page h2 { ${t("h2")} } .page h3 { ${t("h3")} }`,
    `.page .description { ${t("lead")} }`,
    `.sidebar, .toc, .page figure.code-title figcaption { ${t("small")} }`,
    `.sidebar h2, .toc h2 { ${t("caption")} }`,
  ].join("\n");
}

function marksBlock(marks: Mark[], scheme: Scheme): string {
  const rows = marks.map((mark) => {
    const sizes = [16, 24, 32, 64, 128];
    const one = sizes.map((s) => sized(mark, s)).join("");
    const full = sizes.map((s) => sized(mark, s)).join("");
    const bar = scheme === "light" ? "vs-light" : "vs-dark";
    const tab = scheme === "light" ? "tab-light" : "tab-dark";
    return `<div class="mark">
<p class="mark-name">${esc(mark.title)}</p>
<div class="sizes one">${one}</div>
<div class="sizes full">${full}</div>
<div class="uses">
<div class="lockup full">${sized(mark, 32)}${
      mark.wordmark === undefined
        ? "<span>ascribe</span>"
        : mark.wordmark.replace("<svg ", '<svg height="25" role="img" class="wordmark" ')
    }</div>
<div class="activity ${bar}" title="VS Code's activity bar">
<span class="active">${sized(mark, 24)}</span><span class="glyph"></span>
</div>
<div class="browser ${tab}" title="A browser tab"><span class="tab full">${sized(mark, 16)}<span>Getting started · Ascribe</span></span></div>
</div>
</div>`;
  });
  return `<div class="marks">${rows.join("\n")}</div>`;
}

function semanticBlock(candidate: Candidate, scheme: Scheme): string {
  const background = hexOf(candidate, "background", scheme);
  const rows = COLORS.map((name) => {
    const { ref, hex } = candidate.colors.get(name)?.[scheme] ?? { ref: "", hex: "" };
    const onBackground = name === "background" ? "" : ratio(contrast(hex, background));
    return `<tr><td><span class="chip" style="background: ${hex}"></span>${esc(name)}</td><td><code>${esc(ref)}</code></td><td><code>${hex}</code></td><td class="num">${onBackground}</td></tr>`;
  });
  return `<table class="semantic"><thead><tr><th>Color</th><th>From</th><th>Value</th><th class="num">On background</th></tr></thead><tbody>
${rows.join("\n")}
</tbody></table>`;
}

function paletteBlock(candidate: Candidate): string {
  const ramps = [...candidate.palette].map(([hue, steps]) => {
    const chips = [...steps].map(([step, hex]) => {
      const ink = contrast(hex, "#000000") >= contrast(hex, "#ffffff") ? "#000000" : "#ffffff";
      return `<div class="swatch" style="background: ${hex}; color: ${ink}"><b>${esc(step)}</b><span>${hex}</span></div>`;
    });
    return `<div class="ramp"><span class="hue">${esc(hue)}</span>${chips.join("")}</div>`;
  });
  return `<div class="palette">${ramps.join("\n")}</div>`;
}

function contrastBlock(candidate: Candidate, pairs: Pair[]): string {
  const checks = checkPairs(candidate, pairs);
  const cell = (c: Check) =>
    `<td class="num ${c.pass ? "pass" : "fail"}"><span class="chip" style="background: ${c.bg}; color: ${c.fg}">Aa</span>${ratio(c.ratio)}${c.pass ? "" : " ✗"}</td>`;
  const rows = pairs.map((pair, i) => {
    const [light, dark] = [checks[2 * i], checks[2 * i + 1]] as [Check, Check];
    return `<tr><td><code>${esc(pair.fg)}</code> on <code>${esc(pair.bg)}</code></td><td>${esc(pair.where)}</td><td>${pair.kind} ${MINIMUM[pair.kind]}:1</td>${cell(light)}${cell(dark)}</tr>`;
  });
  const failing = checks.filter((c) => !c.pass);
  const summary =
    failing.length === 0
      ? `<p class="verdict pass">Every pairing passes, in light and in dark.</p>`
      : `<p class="verdict fail">${failing.length} failing:</p><ul class="failing">${failing
          .map(
            (c) =>
              `<li>${c.scheme}: <code>${esc(c.pair.fg)}</code> on <code>${esc(c.pair.bg)}</code>, ${ratio(c.ratio)}:1 where ${MINIMUM[c.pair.kind]}:1 is needed (${esc(c.pair.where)})</li>`,
          )
          .join("")}</ul>`;
  return `${summary}
<table class="contrast"><thead><tr><th>Pair</th><th>Where</th><th>Needs</th><th class="num">Light</th><th class="num">Dark</th></tr></thead><tbody>
${rows.join("\n")}
</tbody></table>`;
}

function distinctBlock(candidate: Candidate, distinct: Distinct[]): string {
  const visions: Vision[] = ["normal", "protan", "deutan"];
  const rows = distinct.map((d) => {
    const cells = SCHEMES.flatMap((scheme) => {
      const a = hexOf(candidate, d.a, scheme);
      const b = hexOf(candidate, d.b, scheme);
      return visions.map((v) => {
        const value = difference(a, b, v);
        return `<td class="num ${value < DISTINCT ? "fail" : ""}">${value.toFixed(2)}</td>`;
      });
    });
    return `<tr><td><code>${esc(d.a)}</code> and <code>${esc(d.b)}</code></td><td>${esc(d.where)}</td>${cells.join("")}</tr>`;
  });
  return `<table class="distinct"><thead>
<tr><th rowspan="2">Colors</th><th rowspan="2">Where</th><th colspan="3">Light</th><th colspan="3">Dark</th></tr>
<tr>${["Typical", "Protan", "Deutan", "Typical", "Protan", "Deutan"].map((v) => `<th class="num">${v}</th>`).join("")}</tr>
</thead><tbody>
${rows.join("\n")}
</tbody></table>`;
}

function typeBlock(candidate: Candidate): string {
  const rows = TYPE_STEPS.map((step) => {
    const s = candidate.type.get(step);
    if (s === undefined) return "";
    const sample =
      step === "caption"
        ? "ON THIS PAGE"
        : step.startsWith("h")
          ? "Variants and availability"
          : "One source, resolved into a site, plain Markdown, and JSON.";
    return `<tr><th>${step}</th><td class="spec"><code>${esc(s.size)} / ${s.line} / ${s.weight}</code></td><td style="font-size: ${s.size}; line-height: ${s.line}; font-weight: ${s.weight}">${sample}</td></tr>`;
  });
  return `<table class="type">${rows.join("\n")}</table>`;
}

/** The docs page each preview frame shows: the site's chrome, the elements, and review's marks. */
const PAGE = `<header class="site-header"><a class="site-name" href="#">Ascribe</a><button class="search-button" type="button">Search<kbd>/</kbd></button><a class="repo-link" href="#"><span>GitHub</span></a></header>
<div class="site-body">
<nav class="sidebar"><section><h2>Start</h2><ul><li><a href="#">Introduction</a></li><li><a href="#" aria-current="page">Getting started</a></li><li><a href="#">Writing pages</a></li></ul></section><section><h2>Reference</h2><ul><li><a href="#">Directives</a></li><li><a href="#">ascribe.toml</a></li><li><a href="#">Commands</a></li></ul></section></nav>
<main class="page"><article>
<h1>Getting started</h1>
<p class="description">Write a page, check it, and build a site from it.</p>
<ascribe-availability scope="block"><ascribe-availability-target states="ga">Cloud (GA)</ascribe-availability-target>; <ascribe-availability-target states="beta">Self-managed (beta)</ascribe-availability-target>; <ascribe-availability-target states="preview">Desktop (preview)</ascribe-availability-target>; <ascribe-availability-target states="deprecated">Legacy (deprecated)</ascribe-availability-target>; <ascribe-availability-target states="removed">Classic (removed)</ascribe-availability-target>; <ascribe-availability-target states="planned">Mobile (planned)</ascribe-availability-target></ascribe-availability>
<p>A page is Markdown with <code>@</code> directives. Run <code>ascribe check</code> to see what's wrong, and <a href="#">read about directives</a>.</p>
<ascribe-note type="note" label="Note"><p>A note says something worth knowing.</p></ascribe-note>
<ascribe-note type="tip" label="Tip"><p>A tip says how to do it better.</p></ascribe-note>
<ascribe-note type="important" label="Important" heading="Before you build"><p>Something the reader needs to succeed.</p></ascribe-note>
<ascribe-note type="warning" label="Warning"><p>Something that could go wrong.</p></ascribe-note>
<ascribe-note type="caution" label="Caution"><p>Something that will go wrong, or can't be undone.</p></ascribe-note>
<h2>Install</h2>
<ascribe-steps><ol><li>Install the command line tool.</li><li>Write <code>ascribe.toml</code> in your project's root.</li><li>Run <code>ascribe build</code>.</li></ol></ascribe-steps>
<ascribe-tabs><div role="tablist"><button role="tab" aria-selected="true" type="button">npm</button><button role="tab" aria-selected="false" type="button">pnpm</button><button role="tab" aria-selected="false" type="button">Cargo</button></div><ascribe-tab role="tabpanel" label="npm"><figure class="code-title"><figcaption>terminal</figcaption><pre><code>npm install --save-dev @ascribed/cli
npx ascribe build</code></pre></figure></ascribe-tab><ascribe-tab role="tabpanel" label="pnpm" hidden=""><p>pnpm</p></ascribe-tab></ascribe-tabs>
<h2>What changed</h2>
<div class="ascribe-marks" data-ascribe-show="changes">
<p data-ascribe-change="added"><span class="ascribe-label" data-ascribe-label="added">Added</span>A paragraph that's new in this change.</p>
<p data-ascribe-change="changed"><span class="ascribe-label" data-ascribe-label="changed">Changed</span>A paragraph with <del class="ascribe-del">a few</del> <ins class="ascribe-ins">some</ins> words changed.</p>
<div class="ascribe-removed" data-ascribe-change="removed"><span class="ascribe-label" data-ascribe-label="removed">Removed</span><div class="ascribe-removed-body"><p>A paragraph this change removes, struck through and folded to one line.</p></div><button class="ascribe-toggle" type="button">Show</button></div>
<p data-ascribe-change="moved"><span class="ascribe-label" data-ascribe-label="moved">Moved</span>A paragraph that moved here. <span class="ascribe-move-note">from <button class="ascribe-move-link" type="button">Install</button></span></p>
<p class="ascribe-flash-still">A paragraph flashed after a jump to it.</p>
</div>
</article></main>
</div>`;

const STYLE = `
:root { color-scheme: light dark; --page: #ffffff; --ink: #1f2328; --soft: #59636e; --line: #d1d9e0; }
@media (prefers-color-scheme: dark) { :root { --page: #0d1117; --ink: #e6edf3; --soft: #9198a1; --line: #3d444d; } }
* { box-sizing: border-box; }
body { margin: 0; padding: 2rem; background: var(--page); color: var(--ink); font: 15px/1.5 system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; }
h1 { margin: 0 0 0.25rem; font-size: 1.75rem; }
h2 { margin: 3rem 0 0.25rem; padding-top: 1rem; border-top: 3px solid var(--ink); font-size: 1.5rem; }
h3 { margin: 2rem 0 0.5rem; font-size: 1.125rem; }
code { font: 0.875em ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
.lede, .summary, .note { color: var(--soft); max-width: 60rem; }
nav.toc a { margin-right: 1rem; color: inherit; }
.candidate-marks { display: grid; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); gap: 1rem; }
.candidate-marks div { padding: 1rem; border: 1px solid var(--line); border-radius: 0.5rem; }
.candidate-marks h3 { margin: 0 0 0.25rem; }
.candidate-marks p { margin: 0; color: var(--soft); }
.schemes { display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; }
.scheme { padding: 1rem; border: 1px solid var(--line); border-radius: 0.5rem; background: var(--c-background); color: var(--c-text); }
.scheme > h3:first-child { margin-top: 0; }
.scheme h3 { color: var(--c-muted); font-size: 0.8rem; letter-spacing: 0.06em; text-transform: uppercase; }
.mark { padding-block: 0.5rem; border-bottom: 1px solid var(--c-border); }
.mark-name { margin: 0 0 0.5rem; font-weight: 600; }
.sizes { display: flex; align-items: end; gap: 1rem; margin-bottom: 0.75rem; }
.sizes svg, .uses svg { display: block; flex: none; }
.full .second { color: var(--c-accent); }
.uses { display: flex; flex-wrap: wrap; align-items: center; gap: 1.25rem; margin-bottom: 0.5rem; }
.lockup .wordmark { width: auto; margin-bottom: 2px; }
.lockup { display: flex; align-items: flex-end; gap: 0.5rem; font-size: 1.75rem; font-weight: 650; letter-spacing: -0.01em; }
.activity { display: flex; flex-direction: column; gap: 0; width: 48px; padding-block: 4px; border-radius: 4px; }
.activity > span { display: flex; justify-content: center; align-items: center; height: 48px; }
.activity .active { border-left: 2px solid var(--vs-border); color: var(--vs-active); }
.activity .glyph::before { content: ""; width: 20px; height: 20px; border: 2px solid var(--vs-inactive); border-radius: 4px; }
.vs-light { --vs-border: #005fb8; --vs-active: #1f1f1f; --vs-inactive: #616161; background: #f8f8f8; }
.vs-dark { --vs-border: #0078d4; --vs-active: #d7d7d7; --vs-inactive: #868686; background: #181818; }
.browser { padding: 6px 8px 0; border-radius: 6px 6px 0 0; }
.browser .tab { display: flex; align-items: center; gap: 8px; padding: 7px 14px; border-radius: 8px 8px 0 0; font-size: 12px; }
.tab-light { background: #dee1e6; } .tab-light .tab { background: #ffffff; color: #1f1f1f; }
.tab-dark { background: #202124; } .tab-dark .tab { background: #35363a; color: #e8eaed; }
.palette { display: grid; gap: 4px; margin-block: 1rem; }
.ramp { display: flex; gap: 4px; align-items: stretch; }
.hue { width: 5rem; flex: none; align-self: center; font-weight: 600; }
.swatch { flex: 1; min-width: 0; padding: 0.4rem; border-radius: 4px; font-size: 11px; }
.swatch b, .swatch span { display: block; }
table { border-collapse: collapse; font-size: 13px; }
th, td { padding: 0.25rem 0.5rem; border-bottom: 1px solid var(--line); text-align: left; vertical-align: middle; }
.scheme th, .scheme td { border-color: var(--c-border); }
.num { text-align: right; white-space: nowrap; font-variant-numeric: tabular-nums; }
.chip { display: inline-block; min-width: 1.25rem; height: 1.25rem; margin-right: 0.4rem; padding: 0 0.3rem; border: 1px solid rgb(128 128 128 / 0.35); border-radius: 3px; vertical-align: middle; font-weight: 600; font-size: 12px; line-height: 1.15rem; }
.fail { color: #d1242f; font-weight: 600; }
@media (prefers-color-scheme: dark) { .fail { color: #ff8182; } }
.verdict { font-weight: 600; }
.verdict.pass { color: inherit; }
.failing { margin-top: 0; }
.frame { position: relative; overflow: hidden; border: 1px solid var(--c-border); border-radius: 0.375rem; }
.frame iframe { position: absolute; top: 0; left: 0; width: 1000px; border: 0; transform-origin: 0 0; }
.type th { width: 5rem; }
.type .spec { width: 11rem; color: var(--soft); white-space: nowrap; }
`;

/** Sizes each preview frame to its page, scaled to the column. */
const SCRIPT = `
const css = document.getElementById("stylesheets").textContent;
const page = document.getElementById("page").innerHTML;
for (const frame of document.querySelectorAll(".frame")) {
  const iframe = frame.querySelector("iframe");
  const scheme = frame.dataset.scheme;
  const vars = frame.querySelector("template").innerHTML;
  const fit = () => {
    const doc = iframe.contentDocument;
    if (!doc || !doc.body) return;
    const scale = frame.clientWidth / 1000;
    const height = doc.documentElement.scrollHeight;
    iframe.style.height = height + "px";
    iframe.style.transform = "scale(" + scale + ")";
    frame.style.height = height * scale + "px";
  };
  iframe.addEventListener("load", fit);
  addEventListener("resize", fit);
  iframe.srcdoc = '<!doctype html><html lang="en" data-ascribe-scheme="' + scheme + '"><head><meta charset="utf-8"><style>' + css + "</style><style>" + vars + ".ascribe-flash-still { background: var(--ascribe-review-flash); }</style></head><body>" + page + "</body></html>";
}
`;

function candidateSection(candidate: Candidate, inputs: Inputs): string {
  const columns = SCHEMES.map(
    (scheme) => `<div class="scheme" style="${schemeVars(candidate, scheme)}">
<h3>${scheme === "light" ? "Light" : "Dark"}: the marks</h3>
${marksBlock(inputs.marks, scheme)}
<h3>A docs page</h3>
<div class="frame" data-scheme="${scheme}"><template>${esc(previewCss(candidate, scheme))}</template><iframe title="${esc(candidate.name)}, ${scheme}"></iframe></div>
<h3>Semantic colors</h3>
${semanticBlock(candidate, scheme)}
</div>`,
  );
  return `<section id="${esc(candidate.id)}">
<h2>${esc(candidate.name)}</h2>
<p class="summary">${esc(candidate.summary)} <code>design/candidates/${esc(candidate.id)}.toml</code></p>
<div class="schemes">
${columns.join("\n")}
</div>
<h3>The palette</h3>
${paletteBlock(candidate)}
<h3>Contrast</h3>
${contrastBlock(candidate, inputs.pairs)}
<h3>With red-green color blindness</h3>
<p class="note">How far apart colors that sit side by side look, as a distance in OKLab, to a typical reader and to readers without red (protan) or green (deutan) cones. Below ${DISTINCT} is marked: those rely on the label and the bar's shape.</p>
${distinctBlock(candidate, inputs.distinct)}
<h3>The type scale</h3>
<p class="note">Over the system font stack: <code>${esc(candidate.font.ui)}</code>; code in <code>${esc(candidate.font.mono)}</code>. Size / line height / weight.</p>
${typeBlock(candidate)}
</section>`;
}

/** The specimen page, as text. */
export function renderSpecimen(inputs: Inputs): string {
  const stylesheets = Object.entries(inputs.stylesheets)
    .map(([path, text]) => `/* ${path} */\n${text}`)
    .join("\n");
  if (/<\/script/i.test(stylesheets)) throw new Error("a stylesheet contains </script");
  const marks = inputs.marks
    .map(
      (m) =>
        `<div><h3>${esc(m.title)}</h3><p>${esc(m.description)}</p><p><code>${esc(m.path)}</code></p></div>`,
    )
    .join("\n");
  const toc = inputs.candidates.map((c) => `<a href="#${esc(c.id)}">${esc(c.name)}</a>`).join("");
  return `<!doctype html>
<!-- Generated by scripts/design/specimen.ts from design/candidates/. Don't edit it: run
     node scripts/design/specimen.ts, or its test with ASCRIBE_BLESS=1. -->
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Ascribe design candidates</title>
<style>${STYLE}</style>
</head>
<body>
<h1>Ascribe design candidates</h1>
<p class="lede">The candidate marks, palettes, and type scales, each in light and dark. The docs pages are the element library's, review's, and the docs site's own stylesheets with the candidate's colors. Contrast is WCAG 2's: ${MINIMUM.text}:1 for text, ${MINIMUM.graphic}:1 for a bar, border, or mark that carries meaning.</p>
<nav class="toc">${toc}</nav>
<h2>The marks</h2>
<p class="note">Each palette below shows every mark: in one color, in full color with the palette's accent as the second color, at 16, 24, 32, 64, and 128 pixels, and in VS Code's activity bar and a browser tab.</p>
<div class="candidate-marks">
${marks}
</div>
${inputs.candidates.map((c) => candidateSection(c, inputs)).join("\n")}
<template id="page">${PAGE}</template>
<script type="text/plain" id="stylesheets">${stylesheets}</script>
<script>${SCRIPT}</script>
</body>
</html>
`;
}

if (import.meta.main) {
  writeFileSync(SPECIMEN, renderSpecimen(readInputs()));
  console.log(`Wrote ${SPECIMEN}`);
}
