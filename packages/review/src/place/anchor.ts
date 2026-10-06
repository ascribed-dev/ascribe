// Source anchors: `<path>:<first>-<last>` for a block, `<path>:<line>` for an
// include. The path is a content path, `/`-separated, each segment
// percent-encoded except for ASCII letters, digits, `-`, `.`, `_`, and `~`.

import type { Anchor } from "../shapes.js";

/** Where a rendered block came from: its `data-ascribe-source` and `data-ascribe-via`, as `ascribe diff` writes them. */
export type { Anchor };

/** A block's source, parsed. `path` is decoded. */
export interface SourceRange {
  path: string;
  first: number;
  last: number;
}

/** Parses `<path>:<first>-<last>`, or returns `undefined` if it isn't one. */
export function parseSource(source: string): SourceRange | undefined {
  const colon = source.lastIndexOf(":");
  if (colon <= 0) return undefined;
  const match = /^(\d+)-(\d+)$/.exec(source.slice(colon + 1));
  if (match === null) return undefined;
  const first = Number(match[1]);
  const last = Number(match[2]);
  if (first < 1 || last < first) return undefined;
  let path: string;
  try {
    path = decodePath(source.slice(0, colon));
  } catch {
    return undefined;
  }
  return { path, first, last };
}

/** Writes a block's source for a decoded content path. */
export function formatSource(range: SourceRange): string {
  return `${encodePath(range.path)}:${range.first}-${range.last}`;
}

/** Percent-encodes each segment of a `/`-separated path, as anchors do. */
export function encodePath(path: string): string {
  return path
    .split("/")
    .map((segment) =>
      Array.from(new TextEncoder().encode(segment), (byte) => {
        const char = String.fromCharCode(byte);
        return /[A-Za-z0-9\-._~]/.test(char)
          ? char
          : `%${byte.toString(16).toUpperCase().padStart(2, "0")}`;
      }).join(""),
    )
    .join("/");
}

/** The inverse of `encodePath`. */
export function decodePath(path: string): string {
  return path.split("/").map(decodeURIComponent).join("/");
}

/** A string that identifies an anchor, `via` included. */
export function anchorKey(anchor: Anchor): string {
  return [anchor.source, ...anchor.via].join(" ");
}
