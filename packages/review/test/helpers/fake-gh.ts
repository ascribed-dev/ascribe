// A fake `gh` executable: a Node script, put on the path, that records each
// call and replays answers written by the test.
import { chmodSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";

/** What the fake prints and how it exits, for calls whose arguments contain `match`. */
export interface GhAnswer {
  match: string;
  stdout?: string;
  stderr?: string;
  code?: number;
}

export interface FakeGh {
  dir: string;
  /** The `PATH` with the fake first. */
  path: string;
  /** Each call's arguments and standard input. */
  calls(): { args: string[]; stdin: string }[];
  remove(): void;
}

const SCRIPT = `#!/usr/bin/env node
const fs = require("node:fs");
const path = require("node:path");
const dir = __dirname;
const args = process.argv.slice(2);
let stdin = "";
try { stdin = fs.readFileSync(0, "utf8"); } catch {}
fs.appendFileSync(path.join(dir, "calls.jsonl"), JSON.stringify({ args, stdin }) + "\\n");
const answers = JSON.parse(fs.readFileSync(path.join(dir, "answers.json"), "utf8"));
const joined = args.join(" ");
const answer = answers.find((a) => joined.includes(a.match)) || { stderr: "no answer", code: 1 };
if (answer.stdout) process.stdout.write(answer.stdout);
if (answer.stderr) process.stderr.write(answer.stderr);
process.exitCode = answer.code || 0;
`;

/** Writes a fake `gh` answering with `answers`. POSIX only: Windows can't run it without a shell. */
export function fakeGh(answers: GhAnswer[]): FakeGh {
  const dir = mkdtempSync(path.join(tmpdir(), "fake-gh-"));
  writeFileSync(path.join(dir, "gh"), SCRIPT);
  chmodSync(path.join(dir, "gh"), 0o755);
  writeFileSync(path.join(dir, "answers.json"), JSON.stringify(answers));
  writeFileSync(path.join(dir, "calls.jsonl"), "");
  return {
    dir,
    path: `${dir}${path.delimiter}${process.env["PATH"] ?? ""}`,
    calls: () =>
      readFileSync(path.join(dir, "calls.jsonl"), "utf8")
        .split("\n")
        .filter((line) => line !== "")
        .map((line) => JSON.parse(line) as { args: string[]; stdin: string }),
    remove: () => rmSync(dir, { recursive: true, force: true }),
  };
}
