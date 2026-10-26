// `pnpm check:all` (scripts/check.ts) runs what rust.yml and js.yml run on a
// pull request, so a step added to, removed from, or changed in either
// workflow fails here until the script has it too, or says why it doesn't.
import { readFileSync } from "node:fs";
import { expect, it } from "vitest";
import { ciOnly, pinnedTools, steps, type Step } from "./check.ts";

const root = new URL("../", import.meta.url);
const workflows: Step["workflow"][] = ["rust.yml", "js.yml"];

// Each job's steps, by their `name:`, or their `run:` when they have none.
// Steps that only use an action are left out.
function jobs(workflow: string): Map<string, string[]> {
  const text = readFileSync(new URL(`.github/workflows/${workflow}`, root), "utf8");
  const lines = text.slice(text.indexOf("\njobs:\n")).split("\n");
  const found = new Map<string, string[]>();
  let job: string[] | undefined;
  let step: { name?: string; run?: string } | undefined;
  const end = () => {
    const ci = step?.name ?? step?.run;
    if (job && ci !== undefined) job.push(ci);
    step = undefined;
  };
  for (const line of lines) {
    const id = /^ {2}([\w-]+):$/.exec(line)?.[1];
    if (id !== undefined) {
      end();
      job = [];
      found.set(id, job);
      continue;
    }
    if (/^ {6}- /.test(line)) {
      end();
      step = {};
    }
    const key = /^ {6}(?:- | {2})(name|run): (.*)$/.exec(line);
    if (step && key) step[key[1] as "name" | "run"] = key[2] ?? "";
  }
  end();
  return found;
}

it("runs each step of each job rust.yml and js.yml run, in their order", () => {
  for (const workflow of workflows) {
    for (const [job, ci] of jobs(workflow)) {
      const left = ciOnly.some((j) => j.workflow === workflow && j.job === job);
      const here = steps.filter((s) => s.workflow === workflow && s.job === job).map((s) => s.ci);
      if (left) {
        expect(here, `${workflow}'s ${job} is in ciOnly and has steps`).toEqual([]);
        continue;
      }
      // To fix: add the step to scripts/check.ts, where it runs in its job, or
      // add it with `skip` saying why it doesn't run here.
      expect(here, `${workflow}'s ${job} job`).toEqual(ci);
    }
  }
});

it("names only jobs the workflows have", () => {
  const named = [...steps, ...ciOnly].map((s) => `${s.workflow} ${s.job}`);
  const real = workflows.flatMap((w) => [...jobs(w).keys()].map((job) => `${w} ${job}`));
  expect(named.filter((n) => !real.includes(n))).toEqual([]);
});

it("runs as CI does what it doesn't say it runs differently", () => {
  // A step run as its `run:` line is split on spaces, so it can't quote,
  // expand, or chain.
  const shell = steps.filter((s) => !s.run && !s.skip && /["'$|&;<>]/.test(s.ci));
  expect(shell.map((s) => s.ci)).toEqual([]);
});

it("knows the versions of cargo-deny and cargo-machete rust.yml pins", () => {
  const tools = pinnedTools();
  expect([...tools.keys()]).toEqual(["cargo-deny", "cargo-machete"]);
  for (const version of tools.values()) expect(version).toMatch(/^\d+\.\d+\.\d+$/);
});
