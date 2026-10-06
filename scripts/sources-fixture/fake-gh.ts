// A stand-in for the GitHub CLI, for `setup.ts --local`: the few `gh pr`
// commands the update recipe runs, kept in a JSON file (FAKE_GH_STATE)
// instead of on GitHub. `--delete-branch` deletes the branch from the
// checkout's `origin`, as GitHub would.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import process from "node:process";
import { parseArgs } from "node:util";

export interface Pull {
  number: number;
  head: string;
  state: "OPEN" | "CLOSED";
  title: string;
  body: string;
}

export interface State {
  pulls: Pull[];
  /** What each command did, in order: `opened 1`, `updated 1`, `closed 1`. */
  log: string[];
}

const file = process.env["FAKE_GH_STATE"];
if (file === undefined) fail("FAKE_GH_STATE isn't set.");
const state: State = existsSync(file)
  ? (JSON.parse(readFileSync(file, "utf8")) as State)
  : { pulls: [], log: [] };

const [group, command, ...rest] = process.argv.slice(2);
const { values, positionals } = parseArgs({
  args: rest,
  allowPositionals: true,
  options: {
    head: { type: "string" },
    state: { type: "string" },
    json: { type: "string" },
    jq: { type: "string" },
    title: { type: "string" },
    "body-file": { type: "string" },
    comment: { type: "string" },
    "delete-branch": { type: "boolean" },
  },
});
const pull = (): Pull => {
  const found = state.pulls.find((p) => String(p.number) === positionals[0]);
  if (found === undefined) fail(`no pull request ${positionals[0]}`);
  return found;
};
const body = () => readFileSync(values["body-file"] ?? fail("--body-file is missing"), "utf8");

if (group !== "pr") fail(`gh ${group} isn't faked`);
switch (command) {
  case "list": {
    // The recipe asks for the open pull request's number, or nothing.
    const open = state.pulls.find((p) => p.head === values.head && p.state === "OPEN");
    if (open !== undefined) console.log(open.number);
    break;
  }
  case "create": {
    const number = state.pulls.length + 1;
    state.pulls.push({
      number,
      head: values.head ?? fail("--head is missing"),
      state: "OPEN",
      title: values.title ?? "",
      body: body(),
    });
    state.log.push(`opened ${number}`);
    break;
  }
  case "edit": {
    const found = pull();
    found.title = values.title ?? found.title;
    found.body = body();
    state.log.push(`updated ${found.number}`);
    break;
  }
  case "close": {
    const found = pull();
    found.state = "CLOSED";
    if (values["delete-branch"]) {
      execFileSync("git", ["push", "-q", "origin", "--delete", found.head]);
    }
    state.log.push(`closed ${found.number}`);
    break;
  }
  default:
    fail(`gh pr ${command} isn't faked`);
}
writeFileSync(file, JSON.stringify(state, null, 2));

function fail(message: string): never {
  console.error(`fake gh: ${message}`);
  process.exit(1);
}
