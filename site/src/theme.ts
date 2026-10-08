// The page's background in each scheme, for `<meta name="theme-color">`: the
// color a browser gives its own UI around the page. Read from site.css's
// generated blocks, so design/tokens.toml stays the one place it's written.
import css from "./styles/site.css?raw";

const MARKER = "/* Generated from design/tokens.toml by scripts/design/tokens.ts: root,";

function background(scheme: "light" | "dark"): string {
  const start = css.indexOf(`${MARKER} ${scheme}. */`);
  const end = css.indexOf("/* End of generated root. */", start);
  const value = /--background: (#[0-9a-f]{6});/.exec(css.slice(start, end))?.[1];
  if (start === -1 || value === undefined) {
    throw new Error(`site.css has no --background in its ${scheme} root block`);
  }
  return value;
}

export const themeColor = { light: background("light"), dark: background("dark") };
