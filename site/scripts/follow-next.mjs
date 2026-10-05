// Moves the Ascribe packages to the newest canary, without saving: the one
// step production's build takes beyond `npm ci`, so the site follows `main`.
//
// The lockfile pins the canary that was current when it was written, which is
// what a user wants: they pin a release and move on purpose. This site is
// meant to follow `main`, so after `npm ci` its build installs the canary
// that `next` names now, and everything else stays as the lockfile has it.
//
// All three packages are installed at the version `next` names for
// @ascribed/astro, which pins @ascribed/cli, @ascribed/elements, and
// @ascribed/review to its own version, so they're always one canary even
// while the tags are being moved.
import { execFileSync } from "node:child_process";

const PACKAGES = ["@ascribed/astro", "@ascribed/cli", "@ascribed/elements"];
const npm = process.platform === "win32" ? "npm.cmd" : "npm";
const shell = process.platform === "win32";

const version = execFileSync(npm, ["view", "@ascribed/astro@next", "version"], {
  encoding: "utf8",
  shell,
}).trim();
if (!/^\d+\.\d+\.\d+-/.test(version)) {
  throw new Error(`@ascribed/astro@next is ${JSON.stringify(version)}, not a canary`);
}
console.log(`Installing the canary ${version}, without saving`);
execFileSync(npm, ["install", "--no-save", ...PACKAGES.map((name) => `${name}@${version}`)], {
  stdio: "inherit",
  shell,
});
