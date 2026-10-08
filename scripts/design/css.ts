// Just enough of a CSS reader for the design tokens' checks: the declarations
// of a stylesheet, each with the at-rules and selector it sits in and its
// line. A stylesheet held in a TypeScript module, as the review overlay's and
// the Astro toolbar's are, is the module's `…_CSS` template literal.

/** One `property: value` of a stylesheet. */
export interface Declaration {
  property: string;
  /** The value, with runs of whitespace as one space. */
  value: string;
  /** The at-rule preludes and the selector it sits in, outermost first. */
  context: string[];
  /** 1-based, in the file the stylesheet came from. */
  line: number;
}

/** A stylesheet's text, and the line of the file it starts on. */
export interface Sheet {
  css: string;
  /** Lines of the file before the stylesheet's first. */
  offset: number;
}

/** The stylesheet in a file: all of a `.css` file, a module's `…_CSS` literal. */
export function sheet(path: string, text: string): Sheet {
  if (path.endsWith(".css")) return { css: text, offset: 0 };
  const match = /export const \w+_CSS = `([^`]*)`/.exec(text);
  if (match?.[1] === undefined) throw new Error(`${path}: no \`export const …_CSS = \`…\`\` found`);
  const start = match.index + match[0].indexOf("`") + 1;
  return { css: match[1], offset: lineCount(text.slice(0, start)) - 1 };
}

const lineCount = (text: string): number => text.split("\n").length;

/** `text` with every character but line breaks in `[start, end)` made a space. */
export function blank(text: string, start: number, end: number): string {
  return text.slice(0, start) + text.slice(start, end).replace(/[^\n]/g, " ") + text.slice(end);
}

/** `css` with its comments blanked, so offsets and lines stay where they were. */
function uncomment(css: string): string {
  let out = css;
  for (const match of css.matchAll(/\/\*[\s\S]*?\*\//g)) {
    out = blank(out, match.index, match.index + match[0].length);
  }
  return out;
}

/** Every declaration in `sheet`, in order. */
export function declarations({ css, offset }: Sheet): Declaration[] {
  const text = uncomment(css);
  const found: Declaration[] = [];
  const context: string[] = [];
  let start = 0;
  let depth = 0;
  let quote: string | undefined;
  const flush = (end: number) => {
    const chunk = text.slice(start, end);
    const colon = topLevelColon(chunk);
    const lead = chunk.length - chunk.trimStart().length;
    if (colon !== -1 && chunk.trim() !== "") {
      found.push({
        property: chunk.slice(0, colon).trim(),
        value: squash(chunk.slice(colon + 1)),
        context: [...context],
        line: offset + lineCount(text.slice(0, start + lead)),
      });
    }
  };
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (quote !== undefined) {
      if (c === "\\") i++;
      else if (c === quote) quote = undefined;
      continue;
    }
    if (c === '"' || c === "'") quote = c;
    else if (c === "(") depth++;
    else if (c === ")") depth--;
    else if (depth > 0) continue;
    else if (c === "{") {
      context.push(squash(text.slice(start, i)));
      start = i + 1;
    } else if (c === ";") {
      flush(i);
      start = i + 1;
    } else if (c === "}") {
      flush(i);
      context.pop();
      start = i + 1;
    }
  }
  return found;
}

const squash = (text: string): string => text.trim().replace(/\s+/g, " ");

function topLevelColon(chunk: string): number {
  let depth = 0;
  for (let i = 0; i < chunk.length; i++) {
    const c = chunk[i];
    if (c === "(") depth++;
    else if (c === ")") depth--;
    else if (c === ":" && depth === 0) return i;
  }
  return -1;
}
