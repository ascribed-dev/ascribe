// The element library's reference, docs/content/reference/elements.md, lists
// what @ascribed/elements has, taken from the package. Its table of custom
// properties is a fragment generated here from the first `:root` block of
// packages/elements/css/style.css, in that block's order; what each property
// sets is said here, in `sets`, and a property with no entry fails. Run it with
// ASCRIBE_BLESS=1 to rewrite the fragment. The page's table of elements is
// written by hand, and compared here with the elements the package's names.ts
// says the element contract defines.
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));

const STYLESHEET = "packages/elements/css/style.css";
const FRAGMENT = "docs/content/_generated/elements-properties.md";
const PAGE = "docs/content/reference/elements.md";
const NAMES = "packages/elements/src/names.ts";

const BLESS = "ASCRIBE_BLESS=1 pnpm exec vitest run scripts/docs-site/elements.test.ts";

const HEADER =
  `<!-- Generated from ${STYLESHEET} by scripts/docs-site/elements.test.ts. ` +
  `Say what a new property sets in the test, then run \`${BLESS}\`. -->\n`;

function read(file: string): string {
  return readFileSync(path.join(root, file), "utf8");
}

/** What each property sets, by name, for the properties with a name of their own. */
const sets: Record<string, string> = {
  "--ascribe-font-family": "The font of notes, tabs, and availability badges.",
  "--ascribe-font-size-small": "The size of a step's number and of availability badges.",
  "--ascribe-font-weight-strong":
    "The weight of a note's heading line, a step's number, a tab's label when the script isn't running, and the selected tab.",
  "--ascribe-border-width": "The width of the line joining steps and of the line under the tabs.",
  "--ascribe-space":
    "The space above and below notes, steps, and tabs, beside a note's content, between steps and beside their numbers, either side of a tab's label, and above a tab's label when the script isn't running.",
  "--ascribe-space-small":
    "The smaller space: above and below a note's content and under its heading line, between tabs, above and below a tab's label and under the tabs, and above and below a line of availability badges.",
  "--ascribe-border-color": "The line joining steps and the line under the tabs.",
  "--ascribe-radius": "The corners of notes, and the top corners of tabs.",
  "--ascribe-text-color": "The text of a note.",
  "--ascribe-muted-color":
    "A tab's label when the script isn't running, and the lead-in of availability badges.",
  "--ascribe-surface-color": "Nothing yet: no element uses it.",
  "--ascribe-steps-color": "The circle behind a step's number.",
  "--ascribe-steps-marker-size":
    "The size of the circle behind a step's number, which is also how far a step's content is indented.",
  "--ascribe-steps-marker-text-color": "A step's number.",
  "--ascribe-tab-color": "A tab's label, when the script is running.",
  "--ascribe-tab-active-color": "The selected tab's label and its underline.",
  "--ascribe-tab-focus-color": "The outline of a tab focused from the keyboard.",
  "--ascribe-tab-hover-background": "A tab's background under the pointer.",
  "--ascribe-state-color":
    "The text of an availability badge whose state the library doesn't style, such as a project's own.",
  "--ascribe-state-background":
    "The background of an availability badge whose state the library doesn't style, such as a project's own.",
};

/** The note types the stylesheet styles: `note`, the default, and each `ascribe-note[type="…"]`. */
function noteTypes(css: string): Set<string> {
  const types = Array.from(css.matchAll(/^ascribe-note\[type="([a-z-]+)"\]/gm), ([, t = ""]) => t);
  return new Set(["note", ...types]);
}

/** What a property sets, or undefined when nothing here says. */
function describe(property: string, types: Set<string>): string | undefined {
  const own = sets[property];
  if (own !== undefined) return own;
  const state = /^--ascribe-state-([a-z]+)-(color|background)$/.exec(property);
  if (state !== null) {
    const part = state[2] === "color" ? "text" : "background";
    return `The ${part} of an availability badge whose last state is \`${state[1]}\`.`;
  }
  const note = /^--ascribe-([a-z]+)-(color|background)$/.exec(property);
  if (note !== null && types.has(note[1] ?? "")) {
    const which =
      note[1] === "note"
        ? "Notes of the type `note`, and of a type the library doesn't style, such as a project's own"
        : `\`${note[1]}\` notes`;
    return note[2] === "color"
      ? `${which}: their border and heading line.`
      : `${which}: their background.`;
  }
  return undefined;
}

/** The custom properties the first `:root` block declares, in its order. */
function properties(css: string): string[] {
  const block = /^:root \{$([\s\S]*?)^\}$/m.exec(css)?.[1];
  if (block === undefined) throw new Error(`${STYLESHEET} has no :root block`);
  return Array.from(block.matchAll(/^\s*(--ascribe-[a-z0-9-]+):/gm), ([, name = ""]) => name);
}

test("the reference's table of custom properties is the stylesheet's", () => {
  const css = read(STYLESHEET);
  const listed = properties(css);
  const everywhere = new Set(
    Array.from(css.matchAll(/(--ascribe-[a-z0-9-]+[a-z0-9])/g), (m) => m[1]),
  );
  expect(new Set(listed), `every property ${STYLESHEET} uses is declared in :root`).toEqual(
    everywhere,
  );

  const types = noteTypes(css);
  const missing = listed.filter((property) => describe(property, types) === undefined);
  expect(missing, `say what each property sets, in \`sets\` in this test`).toEqual([]);
  const stale = Object.keys(sets).filter((property) => !listed.includes(property));
  expect(stale, `\`sets\` names properties ${STYLESHEET} doesn't have`).toEqual([]);

  const rows = listed.map((property) => `| \`${property}\` | ${describe(property, types)} |`);
  const fragment = `${HEADER}\n| Property | What it sets |\n|---|---|\n${rows.join("\n")}\n`;
  if (process.env.ASCRIBE_BLESS === "1") {
    writeFileSync(path.join(root, FRAGMENT), fragment);
  }
  expect(read(FRAGMENT), `${FRAGMENT} is out of date: run \`${BLESS}\``).toBe(fragment);
});

test("the reference's table of elements is the element contract's", () => {
  // Each element names.ts cites the element contract for. (`ascribe-attributes`
  // is the site-render contract's marker, which no page keeps.)
  const names = read(NAMES);
  const contract = Array.from(
    names.matchAll(/\/\*\*((?:(?!\*\/)[\s\S])*)\*\/\s*export const ELEMENT_\w+ = "([a-z-]+)";/g),
    ([, doc = "", name = ""]) => ({ doc, name }),
  )
    .filter(({ doc }) => doc.includes("packages/elements/CONTRACT.md"))
    .map(({ name }) => name);
  expect(contract.length).toBeGreaterThan(0);

  const page = read(PAGE);
  const section = /^## The elements$([\s\S]*?)^## /m.exec(page)?.[1];
  if (section === undefined) throw new Error(`${PAGE} has no "## The elements" section`);
  const listed = Array.from(section.matchAll(/^\| `<([a-z-]+)>`/gm), ([, name = ""]) => name);
  expect(listed.toSorted(), `the elements ${PAGE} lists`).toEqual(contract.toSorted());
});
