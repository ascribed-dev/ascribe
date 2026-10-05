#!/usr/bin/env node
// The coverage back-test: for each pull request merged to main, the docs pages
// whose covered code it changed without changing the page.
//
//   node project-docs/docs/back-test/back-test.mjs [<rev>] > results.tsv
//
// <rev> defaults to origin/main. Needs the full history (not a shallow clone).
// Prints one tab-separated row per pull request and page reported, then one
// `#` line per pull request with its counts.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const rev = process.argv[2] ?? "origin/main";
const map = JSON.parse(readFileSync(new URL("map.json", import.meta.url), "utf8")).pages;
const git = (...args) => execFileSync("git", args, { encoding: "utf8", maxBuffer: 1 << 28 });

// `*` matches within a path segment, `**` across segments.
const globToRegExp = (glob) =>
  new RegExp(
    "^" +
      glob
        .replace(/[.+^${}()|[\]\\]/g, "\\$&")
        .replace(/\*\*\/?|\*/g, (m) => (m === "*" ? "[^/]*" : ".*")) +
      "$",
  );
const pages = Object.entries(map).map(([page, { paths, covers }]) => ({
  page,
  paths,
  covers: covers.map(globToRegExp),
}));

// Pull requests: first-parent commits that are GitHub merges or squashes.
const prs = git(
  "log",
  "--first-parent",
  "--reverse",
  "--format=%H%x09%ad%x09%s",
  "--date=short",
  rev,
)
  .trim()
  .split("\n")
  .map((line) => {
    const [sha, date, subject] = line.split("\t");
    const pr = /^Merge pull request #(\d+)/.exec(subject) ?? /\(#(\d+)\)$/.exec(subject);
    return pr && { sha, date, pr: Number(pr[1]) };
  })
  .filter(Boolean);

const exists = (sha, path) =>
  git("ls-tree", "--full-tree", "--name-only", sha, "--", path).trim() === path;

console.log(["pr", "date", "page", "covered_files_changed", "examples"].join("\t"));
for (const { sha, date, pr } of prs) {
  // Every path touched, with both sides of a rename.
  const changed = new Set(
    git("diff", "--name-status", "-M", `${sha}^1`, sha)
      .trim()
      .split("\n")
      .filter(Boolean)
      .flatMap((line) => line.split("\t").slice(1)),
  );
  let reported = 0;
  let together = 0;
  for (const { page, paths, covers } of pages) {
    const path = paths.find((p) => exists(`${sha}^1`, p) || exists(sha, p));
    if (!path) continue;
    const hits = [...changed].filter((file) => covers.some((re) => re.test(file)));
    if (hits.length === 0) continue;
    if (changed.has(path)) {
      together++;
      continue;
    }
    reported++;
    console.log([pr, date, page, hits.length, hits.slice(0, 4).join(" ")].join("\t"));
  }
  console.log(
    `# #${pr} ${date}: ${changed.size} files, ${reported} reported, ${together} changed with their code`,
  );
}
