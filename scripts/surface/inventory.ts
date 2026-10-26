// The map of Ascribe's surface, project-docs/surface.md: everything a person
// or an agent touches, read from where the code declares it. A pull request
// that adds a command, a setting, a tool, or a diagnostic changes the map, so
// its diff shows what it added to.
//
// Nothing here is typed that can be read. What no file declares is in
// `UNDECLARED`, below, and the map says so.
//
//   node scripts/surface/inventory.ts      rewrite project-docs/surface.md
//
// scripts/surface/inventory.test.ts fails when the map is stale.
import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "smol-toml";

const root = fileURLToPath(new URL("../..", import.meta.url));

/** The map, relative to the root. */
export const SURFACE = "project-docs/surface.md";

function read(file: string): string {
  return readFileSync(path.join(root, file), "utf8");
}

/** A frontmatter key's value on a page, or "" when it has none. */
function frontmatter(file: string, key: string): string {
  const front = /^---\n([\s\S]*?)\n---/.exec(read(file))?.[1] ?? "";
  return new RegExp(`^${key}: "?(.*?)"?$`, "m").exec(front)?.[1] ?? "";
}

function json<T>(file: string): T {
  return JSON.parse(read(file)) as T;
}

/** The tracked files matching git's pathspecs, sorted. */
function tracked(...pathspecs: string[]): string[] {
  return execFileSync("git", ["ls-files", "--", ...pathspecs], { cwd: root, encoding: "utf8" })
    .split("\n")
    .filter((file) => file !== "")
    .sort();
}

/** Every match of a pattern in a text, which must match at least once. */
function all(text: string, pattern: RegExp, what: string): RegExpExecArray[] {
  const found = Array.from(text.matchAll(pattern));
  if (found.length === 0) throw new Error(`the surface map found no ${what}; its source moved`);
  return found;
}

/** A cell's text: on one line, with `|` escaped. */
function cell(text: string): string {
  return text.replace(/\s+/g, " ").trim().replaceAll("|", "\\|");
}

/** The first sentence of a description. */
function sentence(text: string): string {
  // A link's text without its destination, which is relative to the file it came from.
  const flat = text
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\s+/g, " ")
    .trim();
  const end = /\.(?= |$)/.exec(flat);
  return end ? flat.slice(0, end.index + 1) : flat;
}

function table(headers: string[], rows: string[][]): string {
  if (rows.length === 0) throw new Error(`the surface map has an empty table: ${headers.join()}`);
  return [
    `| ${headers.join(" | ")} |`,
    `|${headers.map(() => "---").join("|")}|`,
    ...rows.map((row) => `| ${row.map(cell).join(" | ")} |`),
  ].join("\n");
}

function code(text: string): string {
  return `\`${text}\``;
}

interface Section {
  title: string;
  /** Who meets it. */
  for: string;
  /** Where it's declared, as the map reads it. */
  from: string[];
  body: string;
}

/**
 * A folder's `index.md` and the folder's other reference pages, its parts, as
 * `content-model/index.md` has `content-model/checks.md`, in order. A guide in
 * the same folder isn't one of them.
 */
function pagesOf(file: string): string[] {
  const folder = path.posix.dirname(file);
  return [
    file,
    ...readdirSync(path.join(root, folder))
      .filter((name) => name.endsWith(".md") && name !== "index.md")
      .sort()
      .map((name) => `${folder}/${name}`)
      .filter((page) => frontmatter(page, "type") === "reference"),
  ];
}

// The command line.

const COMMAND_REFERENCE = "docs/content/command/index.md";

/**
 * The command reference's pages: its own, then each page that includes a
 * fragment generated from the help text, or that a command's permalink leads
 * to, as `ascribe lsp`'s on the editor's page, which has no options to
 * generate, in the folder of the feature its commands are for.
 */
function commandReference(): string[] {
  const permalinked = new Set(
    [...read("docs/ascribe.permalinks").matchAll(/^cli-\S* = "([^#"]+)/gm)].map(
      ([, page = ""]) => `docs/content/${page}`,
    ),
  );
  return [
    COMMAND_REFERENCE,
    ...tracked("docs/content/*.md", "docs/content/**/*.md").filter(
      (file) =>
        file !== COMMAND_REFERENCE &&
        !file.includes("/_") &&
        (permalinked.has(file) || /^@include: .*\/_generated\/cli-/m.test(read(file))),
    ),
  ];
}

function commands(): Section {
  const synopsis = read("docs/content/_generated/cli-synopsis.md");
  const reference = commandReference().map(read).join("\n");
  const rows = all(synopsis, /^ascribe (\S+(?: [a-z]+)?) *(.*)$/gm, "commands").flatMap(
    ([, name = "", rest = ""]) => {
      // `ascribe build [--build <NAME>]`: the name is one word, and the rest its arguments.
      const [first = "", second] = name.split(" ");
      const group = second !== undefined && /^[a-z]+$/.test(second);
      const command = group ? name : first;
      const tail = group ? rest : [second, rest].filter(Boolean).join(" ");
      if (command.startsWith("--")) return [];
      const heading = new RegExp(
        // A `@summary` above the first paragraph marks it as the section's summary.
        `^#{2,3} \`ascribe ${command}\`\\n(?:@.*\\n)*\\n(?:@summary\\n)?([^\\n]+)`,
        "m",
      ).exec(reference);
      const formats = /--format ([a-z|]+)/.exec(tail)?.[1] ?? "";
      return [
        [
          code(`ascribe ${command}`),
          heading?.[1] === undefined ? "" : sentence(heading[1]),
          tail.replace(/ ?\[--format [a-z|]+\]/, "") === ""
            ? ""
            : code(tail.replace(/ ?\[--format [a-z|]+\]/, "").trim()),
          formats === "" ? "" : formats.split("|").map(code).join(", "),
        ],
      ];
    },
  );
  const global = all(
    read("docs/content/_generated/cli-global-options.md"),
    /^- `(--[^`]+)`: (.+)$/gm,
    "global options",
  ).map(([, option = "", what = ""]) => [code(option), sentence(what)]);
  return {
    title: "The command line",
    for: "people and agents",
    from: [
      "docs/content/_generated/cli-synopsis.md",
      "docs/content/_generated/cli-global-options.md",
      ...commandReference(),
    ],
    body: [
      table(["Command", "What it's for", "Arguments and options", "`--format`"], rows),
      "On every command:",
      table(["Option", "What it's for"], global),
    ].join("\n\n"),
  };
}

// The language.

function language(): Section {
  const schema = read("crates/ascribe-core/src/schema.rs");
  const names = new Map(
    all(schema, /Builtin::(\w+) => "([a-z]+)"/g, "directive names").map(
      ([, variant = "", name = ""]) => [variant, name] as const,
    ),
  );
  const rows = all(schema, /\/\/\/ `@[a-z]+` \((§[\d.]+)\)\.\n\s+(\w+),/g, "directives").map(
    ([, section = "", variant = ""]) => [
      code(`@${names.get(variant) ?? variant}`),
      `SPEC ${section}`,
    ],
  );
  return {
    title: "The language",
    for: "authors",
    from: ["crates/ascribe-core/src/schema.rs"],
    body: table(["Directive", "Defined in"], rows),
  };
}

// `ascribe.toml`.

/** A page's text with each `@include` of a whole file replaced by that file. */
function expanded(file: string): string {
  return read(file).replace(/^@include: (\S+)$/gm, (_, target: string) =>
    expanded(path.posix.join(path.posix.dirname(file), target)).trimEnd(),
  );
}

function model(): Section {
  // The reference takes what it shares with the contract from fragments.
  const reference = pagesOf("docs/content/content-model/index.md").map(expanded).join("\n");
  const sections = reference.split(/^(?=## )/m);
  const rows = sections.flatMap((section) => {
    const heading = /^## (\d+)\. (.*`.*)\n/.exec(section);
    if (heading === null) return [];
    // The first line of prose: not a heading, a directive, a list, a table, or code.
    const prose = section
      .replace(/^```[\s\S]*?^```$/gm, "")
      .split("\n")
      .slice(1)
      .find((line) => /^[A-Z`*]/.test(line));
    return [[heading[2] ?? "", sentence(prose ?? ""), `§${heading[1] ?? ""}`]];
  });
  // The reference's pages, in section order.
  rows.sort((a, b) => Number(a[2]?.slice(1)) - Number(b[2]?.slice(1)));
  return {
    title: "`ascribe.toml`",
    for: "authors",
    from: ["docs/content/content-model/", "docs/content/_fragments/content-model/"],
    body: table(["Section", "What it's for", "In the reference"], rows),
  };
}

// Diagnostics.

interface Diagnostic {
  code: string;
  slug: string;
  severity: string;
  level: string;
  next?: string;
  message?: string;
}

function diagnostics(): Section {
  const registry = parse(read("tests/conformance/diagnostics.toml")) as unknown as {
    diagnostic: Diagnostic[];
  };
  const entries = registry.diagnostic;
  const live = entries;
  const count = (key: "severity" | "level" | "next"): string =>
    Array.from(
      live.reduce(
        (counts, entry) =>
          counts.set(entry[key] ?? "none", (counts.get(entry[key] ?? "none") ?? 0) + 1),
        new Map<string, number>(),
      ),
    )
      .sort(([a], [b]) => a.localeCompare(b))
      .map(([value, n]) => `${n} ${value}`)
      .join(", ");
  return {
    title: "Diagnostics",
    for: "people and agents",
    from: ["tests/conformance/diagnostics.toml"],
    body: [
      `${live.length} in all. By severity: ${count("severity")}. By level: ${count("level")}. By kind of next step: ${count("next")}.`,
      table(
        ["Code", "Name", "Severity", "Next step", "Message"],
        entries.map((entry) => [
          code(entry.code),
          code(entry.slug),
          entry.severity,
          entry.next ?? "",
          entry.message ?? "",
        ]),
      ),
    ].join("\n\n"),
  };
}

// The VS Code extension.

interface Manifest {
  contributes: {
    commands: { command: string; title: string; category?: string }[];
    views: Record<string, { id: string; name: string }[]>;
    keybindings: { command: string; key: string; mac?: string }[];
    walkthroughs: { title: string; steps: { title: string }[] }[];
    configuration: {
      properties: Record<
        string,
        { default?: unknown; markdownDescription?: string; description?: string }
      >;
    };
    languageModelTools: { name: string; displayName: string; userDescription: string }[];
    chatSkills: { path: string; when?: string }[];
    mcpServerDefinitionProviders: { id: string; label: string }[];
    menus: Record<string, unknown[]>;
    taskDefinitions: {
      type: string;
      properties: Record<string, { enum?: string[]; description?: string }>;
    }[];
  };
}

function extension(manifest: Manifest): Section {
  const contributes = manifest.contributes;
  const actions = all(
    read("docs/content/_generated/editor-actions.md"),
    /^\| \*\*(.+?)\*\* \| (.+?) \| (.+?) \|$/gm,
    "actions",
  );
  const actionTitles = new Set(actions.map(([, title = ""]) => title));
  const commandRows = contributes.commands
    .filter((command) => !actionTitles.has(command.title))
    .map((command) => [
      `${command.category ?? ""}: ${command.title}`.replace(/^: /, ""),
      code(command.command),
    ]);
  const settings = Object.entries(contributes.configuration.properties).map(([key, setting]) => [
    code(key),
    code(JSON.stringify(setting.default ?? null)),
    sentence(setting.markdownDescription ?? setting.description ?? ""),
  ]);
  const views = Object.values(contributes.views).flatMap((group) =>
    group.map((view) => [view.name, code(view.id)]),
  );
  const keys = contributes.keybindings.map((binding) => [
    code(binding.command),
    code(binding.key),
    binding.mac === undefined ? "" : code(binding.mac),
  ]);
  const steps = contributes.walkthroughs.flatMap((walkthrough) =>
    walkthrough.steps.map((step, index) => [String(index + 1), step.title, walkthrough.title]),
  );
  const tasks = contributes.taskDefinitions.flatMap((definition) =>
    Object.entries(definition.properties).map(([key, property]) => [
      code(definition.type),
      code(key),
      (property.enum ?? []).map(code).join(", "),
      sentence(property.description ?? ""),
    ]),
  );
  const menus = Object.entries(contributes.menus).map(([menu, items]) => [
    code(menu),
    String(items.length),
  ]);
  return {
    title: "The VS Code extension",
    for: "people",
    from: ["packages/vscode/package.json", "docs/content/_generated/editor-actions.md"],
    body: [
      `${contributes.commands.length} commands, ${actions.length} of them actions.`,
      "Commands that aren't actions:",
      table(["Title", "Id"], commandRows),
      "Actions, which the actions bar and the lightbulb offer where each applies:",
      table(
        ["Action", "What it does", "Where it applies"],
        actions.map(([, title = "", what = "", where = ""]) => [title, sentence(what), where]),
      ),
      "Settings:",
      table(["Setting", "Default", "What it's for"], settings),
      "Views:",
      table(["View", "Id"], views),
      "Keys:",
      table(["Command", "Key", "On macOS"], keys),
      "Task types, and the properties `tasks.json` gives them:",
      table(["Type", "Property", "Values", "What it says"], tasks),
      "The walkthrough:",
      table(["Step", "Title", "Walkthrough"], steps),
      "Menus the commands are placed in:",
      table(["Menu", "Entries"], menus),
    ].join("\n\n"),
  };
}

// The language server.

function server(): Section {
  const source = read("crates/ascribe-lsp/src/server.rs");
  const handler = source.slice(source.indexOf("fn handle_request("));
  const arms = handler.slice(0, handler.indexOf("\n}\n"));
  const standard = all(arms, /(?<![:\w])(\w+)::METHOD =>/g, "standard requests").map(
    ([, name = ""]) => [code(name), "The Language Server Protocol's"],
  );
  const own = tracked("crates/ascribe-lsp/src/*.rs").flatMap((file) =>
    Array.from(read(file).matchAll(/pub const \w*METHOD: &str = "(ascribe\/[^"]+)";/g), (found) => [
      code(found[1] ?? ""),
      code(file),
    ]),
  );
  if (own.length === 0) throw new Error("the surface map found no requests of Ascribe's own");
  return {
    title: "The language server",
    for: "editors, and the extension",
    from: ["crates/ascribe-lsp/src/server.rs", "crates/ascribe-lsp/src/"],
    body: [
      "Ascribe's own requests:",
      table(["Request", "Declared in"], own),
      "The standard requests it answers, by the type that names each in the code:",
      table(["Request", "From"], standard),
    ].join("\n\n"),
  };
}

// The MCP server.

function mcp(): Section {
  const tools = all(
    read("crates/ascribe-cli/src/mcp/tools.rs"),
    /name: "(ascribe_\w+)",\s+title: "([^"]+)",\s+command: "([^"]+)",/g,
    "MCP tools",
  ).map(([, name = "", title = "", command = ""]) => [code(name), title, code(command)]);
  const server = read("crates/ascribe-cli/src/mcp/mod.rs");
  const resources = Array.from(
    new Set(all(server, /"(ascribe:\/\/[^"]+)"/g, "MCP resources").map(([, uri = ""]) => uri)),
  )
    .filter((uri) => !uri.endsWith("/"))
    .map((uri) => [code(uri)]);
  const prompts = all(
    read("crates/ascribe-cli/src/agents/prompts.rs"),
    /^\s+name: "([a-z-]+)",/gm,
    "prompts",
  ).map(([, name = ""]) => [code(name), code(`ascribe agents prompt ${name}`)]);
  return {
    title: "The MCP server",
    for: "agents",
    from: [
      "crates/ascribe-cli/src/mcp/tools.rs",
      "crates/ascribe-cli/src/mcp/mod.rs",
      "crates/ascribe-cli/src/agents/prompts.rs",
    ],
    body: [
      "Tools:",
      table(["Tool", "Title", "The command it answers as"], tools),
      "Resources:",
      table(["Address"], resources),
      "Prompts:",
      table(["Prompt", "The command that writes it"], prompts),
    ].join("\n\n"),
  };
}

// VS Code's agent tools.

function agentTools(manifest: Manifest): Section {
  const contributes = manifest.contributes;
  return {
    title: "VS Code's agent tools",
    for: "agents in VS Code",
    from: ["packages/vscode/package.json"],
    body: [
      table(
        ["Tool", "Shown as", "What it gives"],
        contributes.languageModelTools.map((tool) => [
          code(`#${tool.name}`),
          tool.displayName,
          sentence(tool.userDescription),
        ]),
      ),
      "The MCP server, offered to VS Code's agents:",
      table(
        ["Provider", "Label"],
        contributes.mcpServerDefinitionProviders.map((provider) => [
          code(provider.id),
          provider.label,
        ]),
      ),
      "The skill, offered to VS Code's chat:",
      table(
        ["File", "When"],
        contributes.chatSkills.map((skill) => [
          code(skill.path),
          skill.when === undefined ? "Always" : code(skill.when),
        ]),
      ),
    ].join("\n\n"),
  };
}

// The plugin.

function plugin(): Section {
  const part = (file: string): string => {
    if (file.includes("/commands/")) return `A command, \`/ascribe:${path.basename(file, ".md")}\``;
    if (file.includes("/hooks/")) {
      const hooks = json<{ hooks: Record<string, unknown> }>(file).hooks;
      return `Hooks: ${Object.keys(hooks).join(", ")}`;
    }
    if (file.endsWith(".lsp.json")) return "The language server";
    if (file.endsWith(".mcp.json")) return "The MCP server";
    if (file.includes("/launcher/")) return "The launcher, which finds the `ascribe` binary to run";
    if (file.endsWith("SKILL.md")) return "The skill";
    if (file.includes("/skills/")) return "A reference the skill reads when it needs it";
    if (file.endsWith("README.md")) return "Its README, written by hand";
    return "The manifest";
  };
  return {
    title: "The plugin",
    for: "agents",
    from: ["plugins/ascribe/"],
    body: table(
      ["File", "What it is"],
      tracked("plugins/ascribe").map((file) => [code(file), part(file)]),
    ),
  };
}

// The npm packages.

type Exports = Record<string, unknown>;

function exportsOf(dir: string): string[][] {
  const manifest = json<{ name: string; exports?: Exports; bin?: Record<string, string> }>(
    `${dir}/package.json`,
  );
  return [
    ...Object.keys(manifest.exports ?? {}).map((entry) => [
      code(path.posix.join(manifest.name, entry)),
      "An import",
    ]),
    ...Object.keys(manifest.bin ?? {}).map((bin) => [
      code(bin),
      `A program, from ${manifest.name}`,
    ]),
  ];
}

function astro(): Section {
  const source = read("packages/astro/src/index.ts");
  // The options every form takes, then each form's own: one project, or several.
  const forms = ["OneProjectOptions", "ProjectsOptions", "SharedOptions"];
  const options = forms.flatMap((form) => {
    const start = source.indexOf(`interface ${form} `);
    if (start < 0) throw new Error(`the surface map found no ${form}; its source moved`);
    return all(
      source.slice(start, source.indexOf("\n}\n", start)),
      /\/\*\*([\s\S]*?)\*\/\s+(\w+)(\??): ([^;]+);/g,
      "integration options",
    ).map(([, comment = "", name = "", optional = "", type = ""]) => [
      code(name),
      code(type),
      optional === "?" ? "No" : form === "SharedOptions" ? "Yes" : "In its form",
      sentence(comment.replace(/^\s*\* ?/gm, "")),
    ]);
  });
  return {
    title: "`@ascribed/astro`",
    for: "site builders",
    from: ["packages/astro/src/index.ts", "packages/astro/package.json"],
    body: [
      "The integration's options:",
      table(["Option", "Type", "Required", "What it's for"], options),
      "What a site imports:",
      table(["Entry", "Kind"], exportsOf("packages/astro")),
    ].join("\n\n"),
  };
}

function elements(): Section {
  const names = all(
    read("crates/ascribe-core/src/names.rs"),
    /ELEMENT_\w+ = "(ascribe-[a-z-]+)";/g,
    "elements",
  ).map(([, name = ""]) => [code(`<${name}>`)]);
  const properties = Array.from(
    new Set(
      all(
        read("packages/elements/css/style.css"),
        /(--ascribe-[a-z0-9-]+[a-z0-9])/g,
        "CSS properties",
      ).map(([, property = ""]) => property),
    ),
  )
    .sort()
    .map((property) => [code(property)]);
  return {
    title: "`@ascribed/elements`, `@ascribed/review`, and `@ascribed/cli`",
    for: "site builders",
    from: [
      "crates/ascribe-core/src/names.rs",
      "packages/elements/css/style.css",
      "packages/elements/package.json",
      "packages/review/package.json",
      "packages/cli/package.json",
    ],
    body: [
      "Elements the site output writes:",
      table(["Element"], names),
      "CSS properties a site can set:",
      table(["Property"], properties),
      "What a site or a tool imports or runs:",
      table(
        ["Entry", "Kind"],
        ["packages/elements", "packages/review", "packages/cli"].flatMap(exportsOf),
      ),
    ].join("\n\n"),
  };
}

// The outputs.

function outputs(): Section {
  const synopsis = read("docs/content/_generated/cli-synopsis.md");
  const emitters = /--emit ([a-z,]+)/.exec(synopsis)?.[1];
  if (emitters === undefined) throw new Error("the surface map found no --emit; its source moved");
  const shapes = tracked("schemas/*.schema.json").map((file) => {
    const schema = json<{ title?: string; description?: string }>(file);
    return [code(file), schema.title ?? "", sentence(schema.description ?? "")];
  });
  return {
    title: "The outputs",
    for: "site builders, and tools",
    from: ["docs/content/_generated/cli-synopsis.md", "schemas/"],
    body: [
      `A build's outputs, by \`--emit\`: ${emitters.split(",").map(code).join(", ")}. Each has a contract under \`docs/content/contracts/\`.`,
      "The JSON the commands and the language server write, each with a schema:",
      table(["Schema", "Shape", "What it is"], shapes),
    ].join("\n\n"),
  };
}

// The docs.

function docs(): Section {
  const pages = tracked("docs/content/*.md", "docs/content/**/*.md")
    .filter((file) => !file.split("/").some((part) => part.startsWith("_")))
    .map((file) => [
      code(file.replace("docs/content/", "")),
      frontmatter(file, "title"),
      frontmatter(file, "type"),
    ]);
  const readmes = tracked("README.md", "*/README.md", "**/README.md")
    .filter((file) => !/^(tests|examples|project-docs|design|site|scripts|\.)/.test(file))
    .map((file) => [code(file)]);
  return {
    title: "The docs",
    for: "people and agents",
    from: ["docs/content/"],
    body: [
      "Pages of the site. Each command's `--help` is the same text as its options in the command reference.",
      table(["Page", "Title", "Type"], pages),
      "READMEs published with a package, a crate, or the repository:",
      table(["File"], readmes),
    ].join("\n\n"),
  };
}

// What Ascribe writes into a project.

/** The page that lists the files Ascribe puts in a project, one table under `## The files`. */
const PROJECT_FILES = "docs/content/command/project-files.md";

function projectFiles(): Section {
  const text = read(PROJECT_FILES);
  const start = text.indexOf("\n## The files\n");
  if (start === -1) throw new Error(`the surface map found no "## The files" in ${PROJECT_FILES}`);
  const rows = all(
    text.slice(start),
    /^\| (?!File \|)(`.*?) \| (.*?) \| (Yes|No) \|$/gm,
    "file Ascribe writes",
  )
    // A cell's attributes, such as `{available="ascribe planned"}`, and its links to other pages are the
    // page's, not the map's.
    .map(([, file = "", what = "", commit = ""]) => [
      file.replace(/ \{[^}]*\}$/, ""),
      what.replace(/ \(\[[^\]]*\]\([^)]*\)\)/g, "").replace(/\[([^\]]*)\]\([^)]*\)/g, "$1"),
      commit,
    ]);
  return {
    title: "Files Ascribe puts in a project",
    for: "people and agents",
    from: [PROJECT_FILES],
    body: table(["File", "What writes it", "Commit it"], rows),
  };
}

// What nothing declares.

/**
 * Parts of the surface that no file a script can read declares. Each is typed
 * here, which is the thing this map exists to avoid: giving one a single home
 * in the code, and reading it from there, takes its row out of this list.
 */
const UNDECLARED: { surface: string; what: string; where: string }[] = [
  {
    surface: "The command line",
    what: "What each command's exit codes mean",
    where:
      "The prose of the command reference's pages, `docs/content/command/index.md` and the pages it lists, command by command; the three codes are in `crates/ascribe-cli/src/exit.rs`",
  },
  {
    surface: "The language",
    what: "The reserved frontmatter keys: `available`, `variant`, `formatted`, `source`, `permalink`, `intended`, and `slug` under the `astro` profile",
    where: "Checked in `crates/ascribe-model/src/sections.rs`; described in SPEC §2.1",
  },
  {
    surface: "The language",
    what: "Attributes each directive takes, phrases (`{key}`), and the inline forms",
    where: "`Builtin::schema` in `crates/ascribe-core/src/schema.rs`, as code, not as a list",
  },
  {
    surface: "The VS Code extension",
    what: "The status bar item (the build, and review's state), the preview and its header, the build lens, and review's marks and threads",
    where: "`packages/vscode/src/`, as code",
  },
  {
    surface: "`@ascribed/astro`",
    what: "The dev toolbar's review app, and what the integration serves: published files, `llms.txt`, each page's Markdown, and a page for each permalink",
    where: "`packages/astro/src/`, as code",
  },
  {
    surface: "The outputs",
    what: "The HTML preview `ascribe diff --format html` writes, and the layout of each output's folder",
    where: "`packages/review/`, and `docs/content/contracts/output-layout.md`",
  },
];

function undeclared(): string {
  return [
    "## What nothing declares",
    "These are part of the surface, and no file a script can read lists them, so they're typed in `scripts/surface/inventory.ts`. Each is a surface with no single home.",
    table(
      ["Surface", "What", "Where it lives instead"],
      UNDECLARED.map((entry) => [entry.surface, entry.what, entry.where]),
    ),
  ].join("\n\n");
}

/** The map's text. */
export function generate(): string {
  const manifest = json<Manifest>("packages/vscode/package.json");
  const sections = [
    commands(),
    language(),
    model(),
    diagnostics(),
    extension(manifest),
    server(),
    mcp(),
    agentTools(manifest),
    plugin(),
    astro(),
    elements(),
    outputs(),
    docs(),
    projectFiles(),
  ];
  return [
    "<!-- Generated by scripts/surface/inventory.ts from the files each section names. Don't edit it: change the source, then run `node scripts/surface/inventory.ts`. scripts/surface/inventory.test.ts fails when it's stale. -->",
    "# Ascribe's surface",
    "Everything a person or an agent touches, read from where the code declares it. It says what exists, not whether it should: a plan that adds a command, a setting, a tool, or a diagnostic reads it to see what it's adding to, and its pull request's diff of this file shows what it added.",
    ...sections.map((section) =>
      [
        `## ${section.title}`,
        `For ${section.for}. Read from ${section.from.map(code).join(", ")}.`,
        section.body,
      ].join("\n\n"),
    ),
    undeclared(),
    "",
  ]
    .join("\n\n")
    .replace(/\n+$/, "\n");
}

if (
  process.argv[1] !== undefined &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  writeFileSync(path.join(root, SURFACE), generate());
}
