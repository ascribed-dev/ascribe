// The mark's generated assets are current with their sources, and each is
// there at its size. Pixels aren't compared: rasterizers differ by platform.
// Run `node scripts/design/assets.ts` to rewrite them; design/README.md has
// the table.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import {
  ASSETS,
  COMMAND,
  hash,
  ico,
  manifest,
  MANIFEST,
  readDrawing,
  readSources,
  render,
  SOURCES,
  type Manifest,
} from "./assets.ts";
import { ROOT } from "./specimen.ts";

const read = (file: string): string => readFileSync(join(ROOT, file), "utf8");
const committed = JSON.parse(read(MANIFEST)) as Manifest;

/** Each source whose hash isn't the one `recorded` was made from, as a message. */
function stale(recorded: Manifest, reader: (file: string) => string): string[] {
  const fresh = manifest(reader);
  const changed = new Set<string>();
  for (const asset of fresh.assets) {
    const was = recorded.assets.find((a) => a.path === asset.path);
    for (const [source, digest] of Object.entries(asset.sources)) {
      if (was?.sources[source] !== digest) changed.add(source);
    }
  }
  return [...changed].map((source) => `${source} changed: run \`${COMMAND}\``);
}

/** A PNG's width and height, from its header. */
function pngSize(bytes: Buffer): [number, number] {
  expect(bytes.subarray(1, 4).toString("latin1")).toBe("PNG");
  return [bytes.readUInt32BE(16), bytes.readUInt32BE(20)];
}

describe("design/assets.json", () => {
  test("is current with the sources", () => {
    expect(stale(committed, read)).toEqual([]);
  });

  test("lists every asset, at its size, and nothing else", () => {
    expect(committed).toEqual(manifest(read));
  });

  test("a changed source names the command that rewrites the assets", () => {
    const changed = (file: string) => {
      const text = read(file);
      return file === "design/mark.svg" ? text.replace("29.5", "29.6") : text;
    };
    expect(stale(committed, changed)).toEqual([
      "design/mark.svg changed: run `node scripts/design/assets.ts`",
    ]);
  });

  test("a hash doesn't change with line endings", () => {
    expect(hash("a\r\nb\n")).toBe(hash("a\nb\n"));
  });
});

describe.each(ASSETS)("$path", (asset) => {
  const file = join(ROOT, asset.path);

  test("exists, at its size", () => {
    expect(existsSync(file), `missing: run \`${COMMAND}\``).toBe(true);
    const bytes = readFileSync(file);
    if (asset.form === "png") {
      expect(pngSize(bytes)).toEqual([asset.width, asset.height]);
    } else if (asset.form === "ico") {
      expect(bytes.readUInt16LE(2)).toBe(1);
      expect(bytes.readUInt16LE(4)).toBe(1);
      expect([bytes[6], bytes[7]]).toEqual([asset.width % 256, asset.height % 256]);
      expect(pngSize(bytes.subarray(bytes.readUInt32LE(18)))).toEqual([asset.width, asset.height]);
    } else {
      const text = bytes.toString("utf8");
      const box = /viewBox="0 0 ([\d.]+) ([\d.]+)"/.exec(text);
      const [width, height] = [Number(box?.[1]), Number(box?.[2])];
      expect(width / height).toBeCloseTo(asset.width / asset.height, 5);
      const sized = /<svg [^>]*width="([\d.]+)" height="([\d.]+)"/.exec(text);
      if (sized !== null)
        expect([Number(sized[1]), Number(sized[2])]).toEqual([asset.width, asset.height]);
    }
  });

  if (asset.form === "svg") {
    test("is what the script writes", async () => {
      const fresh = (await render(asset, readSources())).toString("utf8");
      expect(read(asset.path).replace(/\r\n/g, "\n"), `stale: run \`${COMMAND}\``).toBe(fresh);
    });

    test("is paths only: no text, image, font, or editor metadata", () => {
      expect(read(asset.path)).not.toMatch(
        /<(text|image|foreignObject|metadata|sodipodi|inkscape)\b|<!--|font-family|xlink:href/,
      );
    });
  }

  if (asset.oneColor === true) {
    test("has no color but currentColor", () => {
      const colors = read(asset.path).match(
        /(fill|stroke|color|stop-color)="[^"]*"|#[0-9a-f]{3,8}\b/gi,
      );
      expect(colors?.every((c) => c.endsWith('"currentColor"'))).toBe(true);
    });
  }
});

test("every source is read", () => {
  expect(SOURCES).toEqual([
    "design/candidates/chosen.toml",
    "design/mark.svg",
    "design/tagline.svg",
    "design/wordmark.svg",
  ]);
});

test("the favicon is one color, from the reader's scheme", () => {
  const { colors } = readSources();
  const text = read("site/public/favicon.svg");
  expect(text.match(/#[0-9a-f]{6}/g)).toEqual([colors.text.light, colors.text.dark]);
  expect(text).toContain("@media (prefers-color-scheme:dark)");
});

test("a source drawing is read as its paths, at fixed precision", () => {
  const drawing = readDrawing(
    "x.svg",
    '<svg viewBox="0 0 32 32"><!-- a note --><path fill="red" d="M1.004 2.5L3 4.0000001Z"/><path class="second" d="M0 0"/></svg>',
  );
  expect(drawing).toEqual({
    box: [0, 0, 32, 32],
    shapes: [
      { d: "M1 2.5L3 4Z", second: false },
      { d: "M0 0", second: true },
    ],
  });
  expect(() => readDrawing("x.svg", "<svg><path d='M0 0'/></svg>")).toThrow("x.svg: no viewBox");
});

test("an icon file holds one PNG", () => {
  const png = Buffer.from("\x89PNG....", "latin1");
  const out = ico(png, 32);
  expect([...out.subarray(0, 8)]).toEqual([0, 0, 1, 0, 1, 0, 32, 32]);
  expect(out.readUInt32LE(14)).toBe(png.length);
  expect(out.subarray(22).equals(png)).toBe(true);
});

test("the extension's Marketplace banner is the icon's tile, so the two read as one", () => {
  const { colors } = readSources();
  const extension = JSON.parse(read("packages/vscode/package.json")) as {
    icon: string;
    galleryBanner: { color: string; theme: string };
  };
  expect(ASSETS.map((a) => a.path)).toContain(`packages/vscode/${extension.icon}`);
  expect(extension.galleryBanner).toEqual({ color: colors.surface.dark, theme: "dark" });
});
