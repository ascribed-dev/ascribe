// The files that map the repository for someone starting cold (ARCHITECTURE.md,
// AGENTS.md, and the crates' READMEs) name paths and commands, and each one
// must exist: a map that points at a moved file sends its reader nowhere.
//
// A path is a relative link, or a code span whose first segment is a file or
// folder next to the file or at the repository's root; it must exist there. A
// command is a `pnpm` or `cargo` line in a code span or a shell block; the
// script, package, crate, test, or benchmark it names must exist.
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "vitest";

const root = fileURLToPath(new URL("../..", import.meta.url));

/** The files checked, relative to the root. */
function files(): string[] {
  const crates = readdirSync(path.join(root, "crates"))
    .map((name) => `crates/${name}/README.md`)
    .filter((file) => existsSync(path.join(root, file)));
  return [
    "ARCHITECTURE.md",
    "AGENTS.md",
    "CONTRIBUTING.md",
    "README.md",
    "tests/zod/README.md",
    ...crates,
  ];
}

/** The text outside fenced code blocks, and the lines of the shell blocks. */
function split(markdown: string): { prose: string; shell: string[] } {
  const prose: string[] = [];
  const shell: string[] = [];
  let fence: { mark: string; sh: boolean } | undefined;
  for (const line of markdown.split("\n")) {
    const run = /^ {0,3}(`{3,}|~{3,})\s*(\S*)/.exec(line);
    if (fence === undefined && run !== null) {
      fence = { mark: run[1] ?? "", sh: run[2] === "sh" };
    } else if (
      fence !== undefined &&
      run !== null &&
      run[2] === "" &&
      run[1]?.[0] === fence.mark[0] &&
      (run[1]?.length ?? 0) >= fence.mark.length
    ) {
      // A closing fence: the opening's character, at least as many, and no info.
      fence = undefined;
    } else if (fence === undefined) {
      prose.push(line);
    } else if (fence.sh) {
      shell.push(line);
    }
  }
  return { prose: prose.join("\n"), shell };
}

/** A relative link's destination, without its fragment. */
function links(markdown: string): string[] {
  return Array.from(split(markdown).prose.matchAll(/\]\(([^)\s]+)\)/g), (match) => match[1] ?? "")
    .filter((link) => !/^[a-z]+:|^#/.test(link))
    .map((link) => decodeURIComponent(link.replace(/#.*$/, "")));
}

/** Every code span, and every line of a shell block. */
function code(markdown: string): string[] {
  const { prose, shell } = split(markdown);
  return [...Array.from(prose.matchAll(/`([^`\n]+)`/g), (m) => m[1] ?? ""), ...shell];
}

/** Paths the files name that aren't in this repository: a project's output. */
const elsewhere = ["_ascribe/", ".ascribe/"];

/**
 * Where a code span that reads as a path in this repository is, from `dir`:
 * next to the file if it's there, else at the root. `undefined` when it
 * doesn't read as one.
 */
function pathIn(span: string, dir: string): string | undefined {
  // `./x` is a reference as a page writes it, not a path in this repository.
  if (span.startsWith("./")) return undefined;
  if (!/^[\w.@-]+(\/[\w.@-]+)*\/?$/.test(span) || !span.includes("/")) return undefined;
  const first = span.split("/")[0] ?? "";
  const bases = [dir, root].filter((at) => existsSync(path.join(at, first)));
  // A span that names a file, with an extension, under a folder that's
  // nowhere is a typo, unless it's in a project's output. (A first segment
  // with a dot, such as `example.com`, is a host, not a folder.)
  if (bases.length === 0) {
    const file = /\.\w+$/.test(span) && !first.includes(".");
    return file && !elsewhere.some((prefix) => span.startsWith(prefix))
      ? path.join(dir, span)
      : undefined;
  }
  return (
    bases.map((at) => path.join(at, span)).find((at) => existsSync(at)) ?? path.join(dir, span)
  );
}

/** Each crate's directory, by its package name. */
function crates(): Map<string, string> {
  const workspace = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  const members = /members\s*=\s*\[([^\]]*)\]/.exec(workspace)?.[1] ?? "";
  const out = new Map<string, string>();
  for (const [, member = ""] of members.matchAll(/"([^"]+)"/g)) {
    const manifest = readFileSync(path.join(root, member, "Cargo.toml"), "utf8");
    const name = /^name\s*=\s*"([^"]+)"/m.exec(manifest)?.[1];
    if (name !== undefined) out.set(name, member);
  }
  return out;
}

/** Each pnpm package's scripts, by its name; the root's under "". */
function packages(): Map<string, string[]> {
  const scripts = (dir: string): [string, string[]] => {
    const manifest = JSON.parse(readFileSync(path.join(root, dir, "package.json"), "utf8")) as {
      name: string;
      scripts?: Record<string, string>;
    };
    return [dir === "" ? "" : manifest.name, Object.keys(manifest.scripts ?? {})];
  };
  const dirs = ["", "tests/zod", "examples/astro-site"];
  for (const parent of ["packages", "packages/cli/platforms"]) {
    for (const name of readdirSync(path.join(root, parent))) {
      if (existsSync(path.join(root, parent, name, "package.json"))) dirs.push(`${parent}/${name}`);
    }
  }
  return new Map(dirs.map(scripts));
}

/** What's wrong with a command, if it names something that isn't there. */
function commandProblem(line: string): string | undefined {
  const words = line
    .replace(/#.*$/, "")
    .trim()
    .split(/\s+/)
    .filter((word) => !/^[A-Z_]+=/.test(word));
  for (const part of words.join(" ").split(/\s*&&\s*/)) {
    const [tool, ...args] = part.split(" ");
    if (tool === "pnpm") {
      const filter = args.indexOf("--filter");
      const scriptsOf = packages();
      if (filter >= 0) {
        const name = args[filter + 1] ?? "";
        const scripts = scriptsOf.get(name);
        if (scripts === undefined) return `no package ${name}`;
        const script = args[filter + 2];
        if (script !== undefined && script !== "exec" && !scripts.includes(script)) {
          return `${name} has no script ${script}`;
        }
      } else if (args[0] !== undefined && !["-r", "install", "exec"].includes(args[0])) {
        if (!(scriptsOf.get("") ?? []).includes(args[0])) return `no root script ${args[0]}`;
      }
    } else if (tool === "cargo") {
      const dir = args.includes("-p") ? crates().get(args[args.indexOf("-p") + 1] ?? "") : root;
      if (dir === undefined) return "no such crate";
      for (const [flag, folder] of [
        ["--test", "tests"],
        ["--bench", "benches"],
      ] as const) {
        const at = args.indexOf(flag);
        if (at >= 0 && !existsSync(path.join(root, dir, folder, `${args[at + 1] ?? ""}.rs`))) {
          return `no ${folder}/${args[at + 1]}.rs in ${dir}`;
        }
      }
    }
  }
  return undefined;
}

test("every path the map names exists", () => {
  const missing: string[] = [];
  let checked = 0;
  for (const file of files()) {
    const dir = path.dirname(path.join(root, file));
    const markdown = readFileSync(path.join(root, file), "utf8");
    const named = [
      ...links(markdown).map((link) => path.join(dir, link)),
      ...code(markdown).flatMap((span) => pathIn(span, dir) ?? []),
    ];
    for (const target of named) {
      checked++;
      if (!existsSync(target)) missing.push(`${file}: ${path.relative(root, target)}`);
    }
  }
  expect(missing).toEqual([]);
  expect(checked).toBeGreaterThan(100);
});

test("every command the map names runs something that exists", () => {
  const problems: string[] = [];
  let checked = 0;
  for (const file of files()) {
    for (const line of code(readFileSync(path.join(root, file), "utf8"))) {
      if (!/^(\S+=\S+\s+)*(pnpm|cargo) /.test(line.trim())) continue;
      checked++;
      const problem = commandProblem(line);
      if (problem !== undefined) problems.push(`${file}: ${line.trim()} (${problem})`);
    }
  }
  expect(problems).toEqual([]);
  expect(checked).toBeGreaterThan(10);
});
