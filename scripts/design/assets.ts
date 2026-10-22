// Every image of the mark that something needs, drawn from the pixel grids in
// design/mark.toml, the mark's colors in design/tokens.toml, and the headline
// in design/words.toml, which the social card draws: the mark and
// wordmark as SVGs in design/, the favicons, touch icon, and social cards in
// site/public/, the header logo in site/src/assets/, the Marketplace and
// activity bar icons in packages/vscode/media/, and the README headers and the
// avatar in design/out/. design/assets.json records each output and the hash
// of each source it came from; assets.test.ts fails when a source changed and
// this script wasn't run. design/README.md has the table.

import { createHash } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import opentype from "opentype.js";
import sharp from "sharp";
import { parse } from "smol-toml";
import { ROOT, type Scheme } from "./specimen.ts";
import { load } from "./tokens.ts";
import { readWords, WORDS } from "./words.ts";

export const COMMAND = "node scripts/design/assets.ts";
export const MANIFEST = "design/assets.json";

/** The mark, its 16 px cut, and the wordmark's letters, as pixel grids. */
const GRIDS = "design/mark.toml";
/** The mark's colors: `color.brand` in the tokens, by name, light and dark. */
const BRAND = "design/tokens.toml#brand";
/** The social card's line under the wordmark: the headline, from the words' home. */
const HEADLINE = `${WORDS}#headline`;
/** The font the social cards' text is drawn in, for generation only: no reader downloads it. */
const FONT = "design/fonts/Charter-Bold.ttf";
/** The site's own pages, each of which gets a card titled with its heading. */
const HOME = "site/home/content";

/**
 * A source that's a page's level-1 heading, the title its card shows: the
 * page's file and `#heading`. Its hash is of the heading alone, so the rest
 * of the page can change without the card.
 */
const heading = (file: string): string => `${file}#heading`;

/** A social card: the page it's for, by the address it's at, and the heading it shows. */
interface Card {
  /** The card's file under site/public/cards/. */
  name: string;
  /** The source of its title. */
  title: string;
}

/**
 * A card for each page of the site outside the docs: each page of home/, by
 * its entry id, and the changelog, from CHANGELOG.md. The docs' pages share
 * site/public/social-card.png.
 */
export const CARDS: Card[] = [
  ...readdirSync(join(ROOT, HOME))
    .filter((file) => file.endsWith(".md"))
    .sort()
    .map((file) => ({ name: file.slice(0, -3), title: heading(`${HOME}/${file}`) })),
  { name: "changelog", title: heading("CHANGELOG.md") },
];

/** The grids design/mark.toml holds, and the wordmark made from them. */
export interface Grids {
  mark: string[];
  small: string[];
  wordmark: string[];
}

/** The letters after the mark's a. */
const WORD = "scribe";

/**
 * Reads design/mark.toml, checking each grid is a rectangle of known cells,
 * and makes the wordmark: the mark, set on the letters' x-height, then each
 * letter one cell apart.
 */
export function readGrids(toml: string): Grids {
  const doc = parse(toml) as {
    mark?: unknown;
    small?: unknown;
    letters?: Record<string, unknown>;
  };
  const grid = (value: unknown, where: string): string[] => {
    if (!Array.isArray(value) || value.length === 0 || value.some((r) => typeof r !== "string")) {
      throw new Error(`${GRIDS} ${where}: a grid is a list of rows`);
    }
    const rows = value as string[];
    if (rows.some((r) => r.length !== rows[0]?.length)) {
      throw new Error(`${GRIDS} ${where}: every row is as wide as the first`);
    }
    if (rows.some((r) => /[^#|.]/.test(r))) {
      throw new Error(`${GRIDS} ${where}: a cell is "#", "|", or "."`);
    }
    return rows;
  };
  const mark = grid(doc.mark, "mark");
  const small = grid(doc.small, "small");
  const letters = WORD.split("").map((ch) => grid(doc.letters?.[ch], `letters.${ch}`));
  const height = letters[0]?.length ?? 0;
  if (letters.some((l) => l.length !== height)) {
    throw new Error(`${GRIDS} letters: every letter has as many rows`);
  }
  if (height < mark.length) throw new Error(`${GRIDS} letters: shorter than the mark`);
  // The mark sits on the letters' baseline, its rows their x-height.
  const above = height - mark.length;
  const first = [...Array<string>(above).fill(".".repeat(mark[0]?.length ?? 0)), ...mark];
  const wordmark = first.map((row, r) => [row, ...letters.map((l) => l[r])].join("."));
  return { mark, small, wordmark };
}

/** A path of a drawing, and whether it takes the second color. */
interface Shape {
  d: string;
  second: boolean;
}

/** A drawing: its paths in its box, one unit a cell. */
interface Drawing {
  box: [width: number, height: number];
  shapes: Shape[];
}

/**
 * A grid as paths: one for the letters' cells and one for the change marks'.
 * Each row's runs are merged, and a run is joined to the same run in the row
 * above, so a stem is one rectangle and no seam shows between its cells.
 */
function drawing(rows: string[]): Drawing {
  const rects = (cell: string): string => {
    const done: { x: number; y: number; w: number; h: number }[] = [];
    let open = new Map<string, { x: number; y: number; w: number; h: number }>();
    rows.forEach((row, y) => {
      const next = new Map<string, { x: number; y: number; w: number; h: number }>();
      for (const m of row.matchAll(new RegExp(`\\${cell}+`, "g"))) {
        const key = `${m.index} ${m[0].length}`;
        const above = open.get(key);
        if (above === undefined) {
          const rect = { x: m.index, y, w: m[0].length, h: 1 };
          done.push(rect);
          next.set(key, rect);
        } else {
          above.h += 1;
          next.set(key, above);
        }
      }
      open = next;
    });
    return done.map((r) => `M${r.x} ${r.y}h${r.w}v${r.h}h-${r.w}Z`).join("");
  };
  const shapes = [
    { d: rects("#"), second: false },
    { d: rects("|"), second: true },
  ].filter((s) => s.d !== "");
  return { box: [rows[0]?.length ?? 0, rows.length], shapes };
}

/** A number as an SVG writes it: `places` decimals at most, no trailing zeros. */
const num = (n: number, places = 2): string =>
  String(Math.round(n * 10 ** places) / 10 ** places + 0);

/** The colors an asset is drawn in: the letters' color and the change marks'. */
interface Ink {
  first: string;
  second: string;
}

/** The mark's colors, by scheme. */
export interface Colors {
  ink: Record<Scheme, string>;
  muted: Record<Scheme, string>;
  gutter: Record<Scheme, string>;
  reviewed: Record<Scheme, string>;
  ground: Record<Scheme, string>;
  surface: Record<Scheme, string>;
}

const BRAND_NAMES = ["ink", "muted", "gutter", "reviewed", "ground", "surface"] as const;

/** `color.brand`'s colors as hex values, from the tokens' text: what BRAND's hash is of. */
function brandColors(tokensText: string): string {
  const tokens = load(tokensText);
  const colors = Object.fromEntries(
    BRAND_NAMES.map((name) => {
      const token = tokens.colors.get(`color.brand.${name}`);
      if (token === undefined) throw new Error(`design/tokens.toml: no color.brand.${name}`);
      const hex = (scheme: Scheme) => tokens.palette.get(token[scheme]) ?? "";
      return [name, { light: hex("light"), dark: hex("dark") }];
    }),
  );
  return `${JSON.stringify(colors, null, 2)}\n`;
}

export const readColors = (json: string): Colors => JSON.parse(json) as Colors;

/** The full-color mark for a light background. */
const lightInk = (colors: Colors): Ink => ({
  first: colors.ink.light,
  second: colors.gutter.light,
});

/** The full-color mark for a dark background: on Deep River, its home. */
const darkInk = (colors: Colors): Ink => ({ first: colors.ink.dark, second: colors.gutter.dark });

/**
 * `drawing`'s paths, moved and scaled: filled with `ink`, or with
 * `currentColor` for the host to color, or unfilled for a stylesheet to. Unless
 * filled with `ink`, the change marks carry `class="second"`.
 */
function paths(drawing: Drawing, ink: Ink | "currentColor" | "none", x = 0, y = 0, scale = 1) {
  const attributes = (shape: Shape) => {
    if (typeof ink === "object") return ` fill="${shape.second ? ink.second : ink.first}"`;
    const marked = shape.second ? ' class="second"' : "";
    return ink === "none" ? marked : `${marked} fill="currentColor"`;
  };
  const body = drawing.shapes.map((s) => `<path${attributes(s)} d="${s.d}"/>`).join("");
  if (x === 0 && y === 0 && scale === 1) return body;
  return `<g transform="translate(${num(x, 4)} ${num(y, 4)}) scale(${num(scale, 4)})">${body}</g>`;
}

/** An SVG document: a fixed form, so its bytes are the same wherever it's written. */
function svg(width: number, height: number, body: string, size = true): string {
  const dimensions = size ? ` width="${num(width)}" height="${num(height)}"` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${num(width)} ${num(height)}"${dimensions}>${body}</svg>\n`;
}

const title = (text: string) => `<title>${text}</title>`;

/** A drawing as a file of its own, one unit a cell, with its name and a note. */
function source(drawn: Drawing, name: string, note: string, ink: Ink | "currentColor"): string {
  const [w, h] = drawn.box;
  return svg(w, h, `${title(name)}<desc>${note}</desc>${paths(drawn, ink)}`, false);
}

/**
 * A square of `size` with the mark `cell` pixels a cell, on `background` if
 * given. It's placed on whole pixels, so every cell's edge is sharp.
 */
function tile(
  mark: Drawing,
  ink: Ink,
  size: number,
  cell: number,
  background?: string,
  radius = 0,
): string {
  const [w, h] = mark.box;
  const x = Math.floor((size - w * cell) / 2);
  const y = Math.floor((size - h * cell) / 2);
  const back =
    background === undefined
      ? ""
      : `<rect width="${size}" height="${size}"${radius > 0 ? ` rx="${num(radius)}"` : ""} fill="${background}"/>`;
  return svg(size, size, back + paths(mark, ink, x, y, cell));
}

/**
 * `text`'s glyphs at `size`, each with where it starts, from `x`: each
 * character's own glyph, with the font's kerning between them. Laid out here
 * rather than by opentype.js, so no substitution the font has changes a title.
 */
function layout(font: opentype.Font, text: string, x: number, size: number) {
  const scale = size / font.unitsPerEm;
  // By code point, which is what a glyph is looked up by.
  const glyphs = Array.from(text, (char) => font.charToGlyph(char));
  let at = x;
  return {
    glyphs: glyphs.map((glyph, i) => {
      const start = at;
      const next = glyphs[i + 1];
      const kern = next === undefined ? 0 : font.getKerningValue(glyph, next);
      at += ((glyph.advanceWidth ?? 0) + kern) * scale;
      return { glyph, x: start };
    }),
    width: at - x,
  };
}

/** `text` in lines no wider than `width` at `size`, broken at spaces. */
function wrap(font: opentype.Font, text: string, size: number, width: number): string[] {
  const lines: string[] = [];
  for (const word of text.split(/\s+/)) {
    const last = lines.at(-1);
    if (last !== undefined && layout(font, `${last} ${word}`, 0, size).width <= width) {
      lines[lines.length - 1] = `${last} ${word}`;
    } else {
      lines.push(word);
    }
  }
  return lines;
}

/**
 * A glyph's outline as path data, written here: opentype.js's own
 * `toPathData` writes `NaN` for some coordinates.
 */
function pathData(path: opentype.Path): string {
  return path.commands
    .map((c) => {
      if (c.type === "Z") return "Z";
      const end = `${num(c.x)} ${num(c.y)}`;
      if (c.type === "Q") return `Q${num(c.x1)} ${num(c.y1)} ${end}`;
      if (c.type === "C") return `C${num(c.x1)} ${num(c.y1)} ${num(c.x2)} ${num(c.y2)} ${end}`;
      return `${c.type}${end}`;
    })
    .join("");
}

/** One line of `text` as outlines, starting at `x` on the baseline `y`. */
function outline(
  font: opentype.Font,
  text: string,
  x: number,
  y: number,
  size: number,
  fill: string,
) {
  const d = layout(font, text, x, size)
    .glyphs.map(({ glyph, x: at }) => pathData(glyph.getPath(at, y, size)))
    .join("");
  return `<path fill="${fill}" d="${d}"/>`;
}

/** The card for the docs and the repository: the wordmark, and the line under it, on Deep River. */
function card(sources: Sources): string {
  const [W, H] = [1200, 630];
  const { wordmark, colors, line } = sources;
  const font = sources.font();
  const cell = 16;
  const [w, h] = wordmark.box;
  const size = 48;
  const gap = 64;
  // The line's cap height, so the block centers on what's drawn.
  const cap = (font.charToGlyph("D").getBoundingBox().y2 * size) / font.unitsPerEm;
  const top = Math.round((H - (h * cell + gap + cap)) / 2);
  const width = layout(font, line, 0, size).width;
  return svg(
    W,
    H,
    `<rect width="${W}" height="${H}" fill="${colors.ground.dark}"/>` +
      paths(wordmark, darkInk(colors), (W - w * cell) / 2, top, cell) +
      outline(font, line, (W - width) / 2, top + h * cell + gap + cap, size, colors.muted.dark),
  );
}

// A page's card: the wordmark at the top, and the page's title below in the
// pinned font, as outlines, so it's the same wherever it's drawn. A title too
// wide for one line wraps at spaces, into even lines.
const CARD_MARGIN = 96;
const TITLE_SIZE = 84;
const TITLE_LEADING = 96;

/** The card for a page whose title is `title`. */
function pageCard(sources: Sources, title: string): string {
  const [W, H] = [1200, 630];
  const { wordmark, colors } = sources;
  const font = sources.font();
  // As few lines as fit, as even as they go: the narrowest measure that
  // still takes no more lines, so a last line isn't one word.
  const fewest = wrap(font, title, TITLE_SIZE, W - 2 * CARD_MARGIN).length;
  let measure = W - 2 * CARD_MARGIN;
  while (measure > 0 && wrap(font, title, TITLE_SIZE, measure - 8).length === fewest) measure -= 8;
  const lines = wrap(font, title, TITLE_SIZE, measure);
  // The title's last line sits on the card's bottom margin.
  const last = H - CARD_MARGIN - 8;
  const titlePaths = lines
    .map((line, i) =>
      outline(
        font,
        line,
        CARD_MARGIN,
        last - (lines.length - 1 - i) * TITLE_LEADING,
        TITLE_SIZE,
        colors.ink.dark,
      ),
    )
    .join("");
  return svg(
    W,
    H,
    `<rect width="${W}" height="${H}" fill="${colors.ground.dark}"/>` +
      paths(wordmark, darkInk(colors), CARD_MARGIN, CARD_MARGIN - 16, 6) +
      titlePaths,
  );
}

/**
 * The mark and its 16 px cut in the reader's scheme: a favicon is shown on a
 * light or a dark tab. Drawn on 16 units, the cut's cells are whole pixels.
 */
function favicon(small: Drawing, colors: Colors): string {
  const style =
    `<style>path{fill:${colors.ink.light}}.second{fill:${colors.gutter.light}}` +
    `@media (prefers-color-scheme:dark){path{fill:${colors.ink.dark}}.second{fill:${colors.gutter.dark}}}</style>`;
  const [w, h] = small.box;
  return svg(16, 16, style + paths(small, "none", (16 - 2 * w) / 2, (16 - 2 * h) / 2, 2), false);
}

/**
 * The mark in one color on a 24 pixel square, for VS Code to tint: 2 pixels
 * a cell, 20 pixels wide, as tall as the codicons beside it.
 */
function activity(mark: Drawing): string {
  const [w, h] = mark.box;
  return svg(
    24,
    24,
    paths(mark, "currentColor", (24 - 2 * w) / 2, Math.floor((24 - 2 * h) / 2), 2),
  );
}

/** The wordmark `cell` pixels a cell, in `ink`. */
function header(wordmark: Drawing, ink: Ink, cell: number): string {
  const [w, h] = wordmark.box;
  return svg(w * cell, h * cell, title("Ascribe") + paths(wordmark, ink, 0, 0, cell));
}

/**
 * The wordmark `cell` pixels a cell on a rounded plate of `background`, with
 * `pad` around it: a header that reads on a light or a dark page, where an
 * image can't follow the reader's scheme.
 */
function plate(wordmark: Drawing, ink: Ink, background: string, cell: number, pad: number) {
  const [w, h] = wordmark.box;
  const [W, H] = [w * cell + 2 * pad, h * cell + 2 * pad];
  return svg(
    W,
    H,
    title("Ascribe") +
      `<rect width="${W}" height="${H}" rx="${num(pad / 2)}" fill="${background}"/>` +
      paths(wordmark, ink, pad, pad, cell),
  );
}

/** A Windows icon holding one PNG, as every browser since 2009 reads one. */
export function ico(png: Buffer, size: number): Buffer {
  const head = Buffer.alloc(22);
  head.writeUInt16LE(0, 0);
  head.writeUInt16LE(1, 2); // An icon.
  head.writeUInt16LE(1, 4); // One image.
  head.writeUInt8(size % 256, 6);
  head.writeUInt8(size % 256, 7);
  head.writeUInt16LE(1, 10); // Color planes.
  head.writeUInt16LE(32, 12); // Bits per pixel.
  head.writeUInt32LE(png.length, 14);
  head.writeUInt32LE(22, 18);
  return Buffer.concat([head, png]);
}

interface Sources {
  mark: Drawing;
  small: Drawing;
  wordmark: Drawing;
  /** The social card's line: the headline. */
  line: string;
  colors: Colors;
  /** The cards' font, read only when a card is drawn. */
  font: () => opentype.Font;
  /** A source's text, for the cards' titles. */
  text: (source: string) => string;
}

/** An output: where it goes, its size, which sources make it, and how. */
export interface Asset {
  path: string;
  width: number;
  height: number;
  from: string[];
  /** The SVG it is, or that's rasterized to make it. */
  draw: (sources: Sources) => string;
  /** How the drawing becomes the file. */
  form: "svg" | "png" | "ico";
  /** Drawn only from `currentColor`, for its host to color. */
  oneColor?: boolean;
}

/**
 * A source as text: a file's text, a font's bytes in base64, for
 * `<file>#heading` the file's level-1 heading, for the tokens' `#brand` the
 * mark's colors, and for the words' `#headline` the headline.
 */
export function readSource(source: string): string {
  const [file = source, part] = source.split("#");
  if (file.endsWith(".ttf")) return readFileSync(join(ROOT, file)).toString("base64");
  // A checkout may write CRLF; the hash is of the text, not the line endings.
  const text = readFileSync(join(ROOT, file), "utf8").replace(/\r\n/g, "\n");
  if (part === undefined) return text;
  if (part === "brand" && file === "design/tokens.toml") return brandColors(text);
  if (part === "headline" && file === WORDS) return readWords(text).headline;
  const found = /^# (.+)$/m.exec(text)?.[1];
  if (part !== "heading" || found === undefined) throw new Error(`${file} has no ${part}`);
  return found.trim();
}

// The sizes of the assets drawn to the wordmark's size, from the grids.
const GRID = readGrids(readSource(GRIDS));
const [WORD_W, WORD_H] = [GRID.wordmark[0]?.length ?? 0, GRID.wordmark.length];
const [MARK_W, MARK_H] = [GRID.mark[0]?.length ?? 0, GRID.mark.length];
const HEADER_CELL = 5;
const PLATE_CELL = 10;
const PLATE_PAD = 51;

export const ASSETS: Asset[] = [
  {
    path: "design/mark.svg",
    width: MARK_W,
    height: MARK_H,
    from: [GRIDS],
    form: "svg",
    oneColor: true,
    draw: (s) =>
      source(
        s.mark,
        "Ascribe",
        'The mark: a heavy single-storey a, with the change marks of a gutter beside it. One color, from currentColor; the change marks carry class="second", the part a second color goes on. Drawn from design/mark.toml, one unit a cell.',
        "currentColor",
      ),
  },
  {
    path: "design/mark-color.svg",
    width: MARK_W,
    height: MARK_H,
    from: [GRIDS, BRAND],
    form: "svg",
    draw: (s) =>
      source(
        s.mark,
        "Ascribe",
        "The mark in full color for a light background: color.brand.ink and color.brand.gutter. Drawn from design/mark.toml, one unit a cell.",
        lightInk(s.colors),
      ),
  },
  {
    path: "design/wordmark.svg",
    width: WORD_W,
    height: WORD_H,
    from: [GRIDS],
    form: "svg",
    oneColor: true,
    draw: (s) =>
      source(
        s.wordmark,
        "ascribe",
        'The wordmark: the mark grown into the word. One color, from currentColor; the change marks carry class="second". Drawn from design/mark.toml, one unit a cell.',
        "currentColor",
      ),
  },
  {
    path: "site/public/favicon.svg",
    width: 32,
    height: 32,
    from: [GRIDS, BRAND],
    form: "svg",
    draw: (s) => favicon(s.small, s.colors),
  },
  {
    path: "site/public/favicon.ico",
    width: 32,
    height: 32,
    from: [GRIDS, BRAND],
    form: "ico",
    draw: (s) => tile(s.mark, darkInk(s.colors), 32, 3, s.colors.ground.dark, 6),
  },
  {
    path: "site/public/apple-touch-icon.png",
    width: 180,
    height: 180,
    from: [GRIDS, BRAND],
    form: "png",
    draw: (s) => tile(s.mark, darkInk(s.colors), 180, 12, s.colors.ground.dark),
  },
  {
    path: "site/public/social-card.png",
    width: 1200,
    height: 630,
    from: [GRIDS, BRAND, FONT, HEADLINE],
    form: "png",
    draw: (s) => card(s),
  },
  ...CARDS.map((c): Asset => ({
    path: `site/public/cards/${c.name}.png`,
    width: 1200,
    height: 630,
    from: [GRIDS, BRAND, FONT, c.title],
    form: "png",
    draw: (s) => pageCard(s, s.text(c.title)),
  })),
  {
    path: "site/src/assets/logo.svg",
    width: WORD_W,
    height: WORD_H,
    from: [GRIDS],
    form: "svg",
    oneColor: true,
    draw: (s) => svg(WORD_W, WORD_H, paths(s.wordmark, "currentColor"), false),
  },
  {
    path: "packages/vscode/media/icon.png",
    width: 256,
    height: 256,
    from: [GRIDS, BRAND],
    form: "png",
    draw: (s) => tile(s.mark, darkInk(s.colors), 256, 16, s.colors.ground.dark, 48),
  },
  {
    path: "packages/vscode/media/activity.svg",
    width: 24,
    height: 24,
    from: [GRIDS],
    form: "svg",
    oneColor: true,
    draw: (s) => activity(s.mark),
  },
  {
    path: "design/out/header-light.svg",
    width: WORD_W * HEADER_CELL,
    height: WORD_H * HEADER_CELL,
    from: [GRIDS, BRAND],
    form: "svg",
    draw: (s) => header(s.wordmark, lightInk(s.colors), HEADER_CELL),
  },
  {
    path: "design/out/header-dark.svg",
    width: WORD_W * HEADER_CELL,
    height: WORD_H * HEADER_CELL,
    from: [GRIDS, BRAND],
    form: "svg",
    draw: (s) => header(s.wordmark, darkInk(s.colors), HEADER_CELL),
  },
  {
    path: "design/out/header-vscode.png",
    width: WORD_W * PLATE_CELL + 2 * PLATE_PAD,
    height: WORD_H * PLATE_CELL + 2 * PLATE_PAD,
    from: [GRIDS, BRAND],
    form: "png",
    draw: (s) => plate(s.wordmark, darkInk(s.colors), s.colors.ground.dark, PLATE_CELL, PLATE_PAD),
  },
  {
    path: "design/out/avatar.png",
    width: 512,
    height: 512,
    from: [GRIDS, BRAND],
    form: "png",
    draw: (s) => tile(s.mark, darkInk(s.colors), 512, 28, s.colors.ground.dark),
  },
];

/** Every source an asset is made from. */
export const SOURCES = [...new Set(ASSETS.flatMap((a) => a.from))].sort();

export const hash = (text: string): string =>
  createHash("sha256").update(text.replace(/\r\n/g, "\n")).digest("hex");

export function readSources(reader: (file: string) => string = readSource): Sources {
  let font: opentype.Font | undefined;
  const grids = readGrids(reader(GRIDS));
  return {
    mark: drawing(grids.mark),
    small: drawing(grids.small),
    wordmark: drawing(grids.wordmark),
    line: reader(HEADLINE),
    colors: readColors(reader(BRAND)),
    font: () => {
      const bytes = Buffer.from(reader(FONT), "base64");
      font ??= opentype.parse(
        bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.length),
      );
      return font;
    },
    text: reader,
  };
}

export interface Manifest {
  command: string;
  assets: { path: string; width: number; height: number; sources: Record<string, string> }[];
}

export function manifest(reader: (file: string) => string = readSource): Manifest {
  return {
    command: COMMAND,
    assets: ASSETS.map(({ path, width, height, from }) => ({
      path,
      width,
      height,
      sources: Object.fromEntries(from.map((f) => [f, hash(reader(f))])),
    })),
  };
}

/** The file `asset` is, from its sources. */
export async function render(asset: Asset, sources: Sources): Promise<Buffer> {
  const drawn = asset.draw(sources);
  if (asset.form === "svg") return Buffer.from(drawn);
  const png = await sharp(Buffer.from(drawn))
    .resize(asset.width, asset.height)
    .png({ compressionLevel: 9 })
    .toBuffer();
  return asset.form === "ico" ? ico(png, asset.width) : png;
}

if (import.meta.main) {
  const sources = readSources();
  for (const asset of ASSETS) {
    const file = join(ROOT, asset.path);
    mkdirSync(dirname(file), { recursive: true });
    writeFileSync(file, await render(asset, sources));
    console.log(`Wrote ${asset.path}`);
  }
  writeFileSync(join(ROOT, MANIFEST), `${JSON.stringify(manifest(), null, 2)}\n`);
  console.log(`Wrote ${MANIFEST}`);
}
