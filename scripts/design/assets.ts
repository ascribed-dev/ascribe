// Every image of the mark that something needs, generated from the sources in
// design/: the favicons and touch icon in site/public/, the Marketplace and
// activity bar icons in packages/vscode/media/, and the README header and the
// avatar in design/out/. design/assets.json records each output and the hash
// of each source it came from; assets.test.ts fails when a source changed and
// this script wasn't run. design/README.md has the table.

import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import sharp from "sharp";
import { readCandidate, ROOT, type Scheme } from "./specimen.ts";

export const COMMAND = "node scripts/design/assets.ts";
export const MANIFEST = "design/assets.json";

const MARK = "design/mark.svg";
const WORDMARK = "design/wordmark.svg";
const TAGLINE = "design/tagline.svg";
const PALETTE = "design/candidates/chosen.toml";

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

/** The mark beside the wordmark, sharing a baseline, and the tagline under both. */
function card(sources: Sources, colors: Colors): string {
  const [W, H] = [1200, 630];
  const { mark, wordmark, tagline } = sources;
  const ink = { first: colors.text.light, second: colors.accent.light };
  // The mark's a sits on y = 29.13 of its 32; the wordmark's baseline is y = 0.
  const MARK_BASELINE = 29.13;
  const markScale = 5;
  const markSize = mark.box[2] * markScale;
  // The wordmark beside the mark at 25/32 of its height, as the specimen's lockup has it.
  const wordScale = (markSize * (25 / 32)) / wordmark.box[3];
  const gap = 36;
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

const read = (file: string): string =>
  // A checkout may write CRLF; the hash is of the text, not the line endings.
  readFileSync(join(ROOT, file), "utf8").replace(/\r\n/g, "\n");

export const hash = (text: string): string =>
  createHash("sha256").update(text.replace(/\r\n/g, "\n")).digest("hex");

export function readSources(reader: (file: string) => string = read): Sources {
  return {
    mark: readDrawing(MARK, reader(MARK)),
    wordmark: readDrawing(WORDMARK, reader(WORDMARK)),
    tagline: readDrawing(TAGLINE, reader(TAGLINE)),
    colors: readColors(reader(PALETTE)),
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
