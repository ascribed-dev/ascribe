// Every image of the mark that something needs, generated from the sources in
// design/: the favicons, touch icon, and social cards in site/public/, the
// header logo in site/src/assets/, the Marketplace and activity bar icons in
// packages/vscode/media/, and the README header and the avatar in design/out/.
// design/assets.json records each output and the hash of each source it came
// from; assets.test.ts fails when a source changed and this script wasn't
// run. design/README.md has the table.

import { createHash } from "node:crypto";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import opentype from "opentype.js";
import sharp from "sharp";
import { readCandidate, ROOT, type Scheme } from "./specimen.ts";

export const COMMAND = "node scripts/design/assets.ts";
export const MANIFEST = "design/assets.json";

const MARK = "design/mark.svg";
const WORDMARK = "design/wordmark.svg";
const TAGLINE = "design/tagline.svg";
const PALETTE = "design/candidates/chosen.toml";
/** The font a social card's title is drawn in, for generation only: no reader downloads it. */
const FONT = "design/fonts/InterDisplay-SemiBold.ttf";
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

/** A path of a source drawing, and whether it takes the second color. */
interface Shape {
  d: string;
  second: boolean;
}

/** A source drawing: its paths in its viewBox. */
export interface Drawing {
  box: [x: number, y: number, width: number, height: number];
  shapes: Shape[];
}

/** Reads a source SVG's viewBox and paths; anything else in it is dropped. */
export function readDrawing(file: string, svg: string): Drawing {
  const box = /<svg [^>]*viewBox="([-\d. ]+)"/.exec(svg)?.[1]?.trim().split(/\s+/).map(Number);
  if (box?.length !== 4 || box.some((n) => !Number.isFinite(n))) {
    throw new Error(`${file}: no viewBox`);
  }
  const shapes = [...svg.matchAll(/<path\b([^>]*)\/>/g)].map(([, attributes = ""]) => {
    const d = / d="([^"]+)"/.exec(attributes)?.[1];
    if (d === undefined) throw new Error(`${file}: a path with no d`);
    return { d: precise(d), second: /class="second"/.test(attributes) };
  });
  if (shapes.length === 0) throw new Error(`${file}: no paths`);
  return { box: box as Drawing["box"], shapes };
}

/** A number as an SVG writes it: `places` decimals at most, no trailing zeros. */
const num = (n: number, places = 2): string =>
  String(Math.round(n * 10 ** places) / 10 ** places + 0);

/** Path data with every number at two decimals at most, so it's the same on every platform. */
const precise = (d: string): string =>
  d.replace(/-?(?:\d+\.?\d*|\.\d+)(?:e-?\d+)?/gi, (n) => num(Number(n)));

/** The colors an asset is drawn in: the mark's text color and its second color. */
interface Ink {
  first: string;
  second: string;
}

/** The colors of the chosen palette the assets use, by scheme. */
export interface Colors {
  text: Record<Scheme, string>;
  accent: Record<Scheme, string>;
  muted: Record<Scheme, string>;
  background: Record<Scheme, string>;
  surface: Record<Scheme, string>;
}

export function readColors(toml: string): Colors {
  const chosen = readCandidate("chosen", toml);
  const pick = (name: string): Record<Scheme, string> => {
    const color = chosen.colors.get(name);
    if (color === undefined) throw new Error(`${PALETTE}: no color.${name}`);
    return { light: color.light.hex, dark: color.dark.hex };
  };
  return {
    text: pick("text"),
    accent: pick("accent"),
    muted: pick("muted"),
    background: pick("background"),
    surface: pick("surface"),
  };
}

/** `drawing`'s paths, each filled with `ink` (or `currentColor`), moved and scaled. */
function paths(drawing: Drawing, ink: Ink | "currentColor" | "none", x = 0, y = 0, scale = 1) {
  const fill = (shape: Shape) =>
    ink === "none"
      ? ""
      : ` fill="${typeof ink === "string" ? ink : shape.second ? ink.second : ink.first}"`;
  const body = drawing.shapes.map((s) => `<path${fill(s)} d="${s.d}"/>`).join("");
  const [bx, by] = drawing.box;
  if (x === 0 && y === 0 && scale === 1 && bx === 0 && by === 0) return body;
  return `<g transform="translate(${num(x - bx * scale, 4)} ${num(y - by * scale, 4)}) scale(${num(scale, 4)})">${body}</g>`;
}

/** An SVG document: a fixed form, so its bytes are the same wherever it's written. */
function svg(width: number, height: number, body: string, size = true): string {
  const dimensions = size ? ` width="${num(width)}" height="${num(height)}"` : "";
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${num(width)} ${num(height)}"${dimensions}>${body}</svg>\n`;
}

const title = (text: string) => `<title>${text}</title>`;

/** A square of `size` with the mark centered at `scale` of it, on `background` if given. */
function tile(
  mark: Drawing,
  ink: Ink,
  size: number,
  scale: number,
  background?: string,
  radius = 0,
): string {
  const [, , width] = mark.box;
  const drawn = size * scale;
  const offset = (size - drawn) / 2;
  const back =
    background === undefined
      ? ""
      : `<rect width="${size}" height="${size}"${radius > 0 ? ` rx="${num(radius)}"` : ""} fill="${background}"/>`;
  return svg(size, size, back + paths(mark, ink, offset, offset, drawn / width));
}

// The mark beside the wordmark. The mark's a sits on y = 29.13 of its 32; the
// wordmark's baseline is y = 0. The wordmark is 25/32 of the mark's height, as
// the specimen's lockup has it, and the gap between them 0.225 of it.
const MARK_BASELINE = 29.13;
const WORDMARK_HEIGHT = 25 / 32;
const LOCKUP_GAP = 0.225;

/** The mark beside the wordmark, sharing a baseline, and the tagline under both. */
function card(sources: Sources, colors: Colors): string {
  const [W, H] = [1200, 630];
  const { mark, wordmark, tagline } = sources;
  const ink = { first: colors.text.light, second: colors.accent.light };
  const markScale = 5;
  const markSize = mark.box[2] * markScale;
  const wordScale = (markSize * WORDMARK_HEIGHT) / wordmark.box[3];
  const gap = markSize * LOCKUP_GAP;
  const lineScale = 44 / 24;
  const lockupWidth = markSize + gap + wordmark.box[2] * wordScale;
  const lineGap = 72;
  const lineHeight = tagline.box[3] * lineScale;
  const blockHeight = markSize + lineGap + lineHeight;
  const left = (W - lockupWidth) / 2;
  const top = (H - blockHeight) / 2;
  const baseline = top + MARK_BASELINE * markScale;
  const wordX = left + markSize + gap;
  const wordY = baseline + wordmark.box[1] * wordScale;
  const lineWidth = tagline.box[2] * lineScale;
  const lineX = (W - lineWidth) / 2;
  const lineY = top + markSize + lineGap;
  return svg(
    W,
    H,
    `<rect width="${W}" height="${H}" fill="${colors.background.light}"/>` +
      paths(mark, ink, left, top, markScale) +
      paths(
        wordmark,
        { first: colors.text.light, second: colors.text.light },
        wordX,
        wordY,
        wordScale,
      ) +
      paths(
        tagline,
        { first: colors.muted.light, second: colors.muted.light },
        lineX,
        lineY,
        lineScale,
      ),
  );
}

// A page's card: the mark beside the wordmark at the top, and the page's
// title below in the pinned font, as outlines, so it's the same wherever it's
// drawn. A title too wide for one line wraps at spaces, into even lines.
const CARD_MARGIN = 96;
const TITLE_SIZE = 84;
const TITLE_LEADING = 96;
/**
 * `text`'s glyphs at `size`, each with where it starts, from `x`: each
 * character's own glyph, with the font's kerning between them. Laid out here
 * rather than by opentype.js, which applies substitutions it can't read all
 * of in Inter; a title has none to lose.
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

/** The card for a page whose title is `title`. */
function pageCard(sources: Sources, title: string): string {
  const [W, H] = [1200, 630];
  const { mark, wordmark, colors } = sources;
  const font = sources.font();
  const markScale = 2;
  const markSize = mark.box[2] * markScale;
  const wordScale = (markSize * WORDMARK_HEIGHT) / wordmark.box[3];
  const left = CARD_MARGIN;
  const top = CARD_MARGIN - 16;
  const baseline = top + MARK_BASELINE * markScale;
  // As few lines as fit, as even as they go: the narrowest measure that
  // still takes no more lines, so a last line isn't one word.
  const fewest = wrap(font, title, TITLE_SIZE, W - 2 * CARD_MARGIN).length;
  let measure = W - 2 * CARD_MARGIN;
  while (measure > 0 && wrap(font, title, TITLE_SIZE, measure - 8).length === fewest) measure -= 8;
  const lines = wrap(font, title, TITLE_SIZE, measure);
  // The title's last line sits on the card's bottom margin.
  const last = H - CARD_MARGIN - 8;
  const titlePaths = lines
    .map((line, i) => {
      const y = last - (lines.length - 1 - i) * TITLE_LEADING;
      const d = layout(font, line, left, TITLE_SIZE)
        .glyphs.map(({ glyph, x }) => glyph.getPath(x, y, TITLE_SIZE).toPathData(2))
        .join("");
      return `<path fill="${colors.text.light}" d="${precise(d)}"/>`;
    })
    .join("");
  return svg(
    W,
    H,
    `<rect width="${W}" height="${H}" fill="${colors.background.light}"/>` +
      paths(mark, lightInk(colors), left, top, markScale) +
      paths(
        wordmark,
        { first: colors.text.light, second: colors.text.light },
        left + markSize + markSize * LOCKUP_GAP,
        baseline + wordmark.box[1] * wordScale,
        wordScale,
      ) +
      titlePaths,
  );
}

/**
 * The mark beside the wordmark, as tall as the mark, in `currentColor` for
 * the page to color. The mark's asterisk keeps `class="second"`, so the page
 * can give it a second color.
 */
function logo(mark: Drawing, wordmark: Drawing): string {
  const size = mark.box[3];
  const wordScale = (size * WORDMARK_HEIGHT) / wordmark.box[3];
  const wordX = size + size * LOCKUP_GAP;
  const width = Math.ceil(wordX + wordmark.box[2] * wordScale);
  const marked = mark.shapes
    .map((s) => `<path${s.second ? ' class="second"' : ""} fill="currentColor" d="${s.d}"/>`)
    .join("");
  return svg(
    width,
    size,
    marked +
      paths(
        wordmark,
        "currentColor",
        wordX,
        MARK_BASELINE + wordmark.box[1] * wordScale,
        wordScale,
      ),
    false,
  );
}

/** One color, from the reader's scheme: a favicon is shown on a light or a dark tab. */
function favicon(mark: Drawing, colors: Colors): string {
  const [, , width, height] = mark.box;
  const style = `<style>path{fill:${colors.text.light}}@media (prefers-color-scheme:dark){path{fill:${colors.text.dark}}}</style>`;
  return svg(width, height, style + paths(mark, "none"), false);
}

/**
 * The mark in one color on a 24 pixel square, for VS Code to tint. Its 32
 * units take the middle 20 pixels, so it stands as tall as the codicons
 * beside it, which are drawn on 16 and shown at 24.
 */
function activity(mark: Drawing): string {
  return svg(24, 24, paths(mark, "currentColor", 2, 2, 20 / mark.box[2]));
}

/** A wordmark `height` pixels tall in one scheme's text color. */
function header(wordmark: Drawing, color: string, height: number): string {
  const [, , w, h] = wordmark.box;
  const scale = height / h;
  return svg(
    Math.ceil(w * scale),
    height,
    title("Ascribe") + paths(wordmark, { first: color, second: color }, 0, 0, scale),
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
  wordmark: Drawing;
  tagline: Drawing;
  colors: Colors;
  /** The cards' font, read only when a card is drawn. */
  font: () => opentype.Font;
  /** A source's text, for the cards' titles. */
  text: (source: string) => string;
}

/** An output: where it goes, its size, which sources make it, and how. */
/**
 * The wordmark `height` pixels tall on a rounded plate of `background`, with
 * `pad` around it: a header that reads on a light or a dark page, where an
 * image can't follow the reader's scheme.
 */
function plate(wordmark: Drawing, color: string, background: string, height: number, pad: number) {
  const [, , w, h] = wordmark.box;
  const scale = height / h;
  const [W, H] = [Math.ceil(w * scale) + 2 * pad, height + 2 * pad];
  return svg(
    W,
    H,
    title("Ascribe") +
      `<rect width="${W}" height="${H}" rx="${num(pad / 2)}" fill="${background}"/>` +
      paths(wordmark, { first: color, second: color }, pad, pad, scale),
  );
}

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

export const ASSETS: Asset[] = [
  {
    path: "site/public/favicon.svg",
    width: 32,
    height: 32,
    from: [MARK, PALETTE],
    form: "svg",
    draw: (s) => favicon(s.mark, s.colors),
  },
  {
    path: "site/public/favicon.ico",
    width: 32,
    height: 32,
    from: [MARK, PALETTE],
    form: "ico",
    draw: (s) => tile(s.mark, lightInk(s.colors), 32, 0.84, s.colors.background.light, 6),
  },
  {
    path: "site/public/apple-touch-icon.png",
    width: 180,
    height: 180,
    from: [MARK, PALETTE],
    form: "png",
    draw: (s) => tile(s.mark, lightInk(s.colors), 180, 0.7, s.colors.background.light),
  },
  {
    path: "site/public/social-card.png",
    width: 1200,
    height: 630,
    from: [MARK, WORDMARK, TAGLINE, PALETTE],
    form: "png",
    draw: (s) => card(s, s.colors),
  },
  ...CARDS.map((c): Asset => ({
    path: `site/public/cards/${c.name}.png`,
    width: 1200,
    height: 630,
    from: [MARK, WORDMARK, PALETTE, FONT, c.title],
    form: "png",
    draw: (s) => pageCard(s, s.text(c.title)),
  })),
  {
    path: "site/src/assets/logo.svg",
    width: 153,
    height: 32,
    from: [MARK, WORDMARK],
    form: "svg",
    oneColor: true,
    draw: (s) => logo(s.mark, s.wordmark),
  },
  {
    path: "packages/vscode/media/icon.png",
    width: 256,
    height: 256,
    from: [MARK, PALETTE],
    form: "png",
    draw: (s) => tile(s.mark, darkInk(s.colors), 256, 0.68, s.colors.surface.dark, 48),
  },
  {
    path: "packages/vscode/media/activity.svg",
    width: 24,
    height: 24,
    from: [MARK],
    form: "svg",
    oneColor: true,
    draw: (s) => activity(s.mark),
  },
  {
    path: "design/out/header-light.svg",
    width: 218,
    height: 48,
    from: [WORDMARK, PALETTE],
    form: "svg",
    draw: (s) => header(s.wordmark, s.colors.text.light, 48),
  },
  {
    path: "design/out/header-dark.svg",
    width: 218,
    height: 48,
    from: [WORDMARK, PALETTE],
    form: "svg",
    draw: (s) => header(s.wordmark, s.colors.text.dark, 48),
  },
  {
    path: "design/out/header-vscode.png",
    width: 532,
    height: 192,
    from: [WORDMARK, PALETTE],
    form: "png",
    draw: (s) => plate(s.wordmark, s.colors.text.dark, s.colors.surface.dark, 96, 48),
  },
  {
    path: "design/out/avatar.png",
    width: 512,
    height: 512,
    from: [MARK, PALETTE],
    form: "png",
    draw: (s) => tile(s.mark, lightInk(s.colors), 512, 0.58, s.colors.background.light),
  },
];

/** The full-color mark for a light background: mark-color.svg's colors. */
function lightInk(colors: Colors): Ink {
  return { first: colors.text.light, second: colors.accent.light };
}

/** The full-color mark for a dark background, made from mark.svg's second color. */
function darkInk(colors: Colors): Ink {
  return { first: colors.text.dark, second: colors.accent.dark };
}

/** Every source an asset is made from. */
export const SOURCES = [...new Set(ASSETS.flatMap((a) => a.from))].sort();

/**
 * A source as text: a file's text, a font's bytes in base64, or, for
 * `<file>#heading`, the file's level-1 heading.
 */
export function readSource(source: string): string {
  const [file = source, part] = source.split("#");
  if (file.endsWith(".ttf")) return readFileSync(join(ROOT, file)).toString("base64");
  // A checkout may write CRLF; the hash is of the text, not the line endings.
  const text = readFileSync(join(ROOT, file), "utf8").replace(/\r\n/g, "\n");
  if (part === undefined) return text;
  const found = /^# (.+)$/m.exec(text)?.[1];
  if (part !== "heading" || found === undefined) throw new Error(`${file} has no ${part}`);
  return found.trim();
}

const read = readSource;

export const hash = (text: string): string =>
  createHash("sha256").update(text.replace(/\r\n/g, "\n")).digest("hex");

export function readSources(reader: (file: string) => string = read): Sources {
  let font: opentype.Font | undefined;
  return {
    mark: readDrawing(MARK, reader(MARK)),
    wordmark: readDrawing(WORDMARK, reader(WORDMARK)),
    tagline: readDrawing(TAGLINE, reader(TAGLINE)),
    colors: readColors(reader(PALETTE)),
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

export function manifest(reader: (file: string) => string = read): Manifest {
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
  const drawing = asset.draw(sources);
  if (asset.form === "svg") return Buffer.from(drawing);
  const png = await sharp(Buffer.from(drawing))
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
