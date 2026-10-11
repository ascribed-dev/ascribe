#!/usr/bin/env node
// `npx @ascribed/astro`: the package's command, `ascribe-astro`. Its one
// subcommand, `init`, writes the starter (src/init.ts).
import { parseArgs } from "node:util";
import { init, STARTER_FILES } from "./init.js";

const USAGE = `Usage: npx @ascribed/astro init [--project <dir>] [--build <name>]

Writes the files every site built with Ascribe starts from into the Astro
project in the current folder, once: ${STARTER_FILES.join(", ")}.
It writes nothing if any of them is there.

Options:
  --project <dir>   The folder holding ascribe.toml, from the current folder. Default: the current folder.
  --build <name>    The build the site shows. Default: the project's one build.
  -h, --help        Print this.`;

function main(argv: string[]): Promise<number> | number {
  let parsed;
  try {
    parsed = parseArgs({
      args: argv,
      allowPositionals: true,
      options: {
        project: { type: "string" },
        build: { type: "string" },
        help: { type: "boolean", short: "h" },
      },
    });
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n\n${USAGE}\n`);
    return 2;
  }
  const { values, positionals } = parsed;
  if (values.help === true) {
    process.stdout.write(`${USAGE}\n`);
    return 0;
  }
  if (positionals.length !== 1 || positionals[0] !== "init") {
    process.stderr.write(`${USAGE}\n`);
    return 2;
  }
  return run(values.project, values.build);
}

async function run(project: string | undefined, build: string | undefined): Promise<number> {
  try {
    const result = await init({ root: process.cwd(), project, build });
    const options = [`build: ${JSON.stringify(result.build)}`];
    if (project !== undefined) options.push(`project: ${JSON.stringify(project)}`);
    process.stdout.write(
      `${result.written.map((file) => `wrote ${file}`).join("\n")}\n\n` +
        `Next, if astro.config.mjs doesn't have it, add the integration:\n` +
        `  integrations: [ascribe({ ${options.join(", ")} })],\n` +
        `then run \`npx astro dev\`.\n`,
    );
    return 0;
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    return 2;
  }
}

process.exitCode = await main(process.argv.slice(2));
