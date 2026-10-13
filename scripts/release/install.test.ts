// The install script, site/public/install.sh, against a release made in the
// test: a folder of files it downloads through a stand-in `curl`, with
// stand-ins for `uname` and `getconf` to choose the platform. Nothing goes to
// the network. CI runs it against the real last release too (ci.yml, `install`).
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { afterEach, expect, test } from "vitest";

const script = path.join(import.meta.dirname, "../../site/public/install.sh");
const dirs: string[] = [];
afterEach(() => {
  for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true });
});

interface Machine {
  os?: string;
  arch?: string;
  /** What `getconf GNU_LIBC_VERSION` prints, or nothing when it fails. */
  glibc?: string;
  /** Change the archive's bytes after SHA256SUMS is written. */
  tamper?: boolean;
}

/** Runs the script on a made-up machine with a made-up release, 1.2.3 and latest. */
function install(machine: Machine, ...args: string[]) {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-install-"));
  dirs.push(dir);
  const { os = "Linux", arch = "x86_64", glibc = "glibc 2.31", tamper = false } = machine;
  const targets: Record<string, Record<string, string>> = {
    Linux: { x86_64: "linux-x64", aarch64: "linux-arm64" },
    Darwin: { arm64: "darwin-arm64" },
  };
  const target = targets[os]?.[arch];

  // The release, at the paths its addresses end with.
  const release = path.join(dir, "release");
  const tag = path.join(release, "v1.2.3");
  mkdirSync(tag, { recursive: true });
  if (target !== undefined) {
    const name = `ascribe-1.2.3-${target}`;
    const stage = path.join(dir, "stage", name);
    mkdirSync(stage, { recursive: true });
    writeFileSync(path.join(stage, "ascribe"), "#!/bin/sh\necho 'ascribe 1.2.3'\n");
    chmodSync(path.join(stage, "ascribe"), 0o755);
    run("tar", ["-czf", path.join(tag, `${name}.tar.gz`), "-C", path.join(dir, "stage"), name]);
    const hash = createHash("sha256")
      .update(readFileSync(path.join(tag, `${name}.tar.gz`)))
      .digest("hex");
    writeFileSync(
      path.join(tag, "SHA256SUMS"),
      `${"0".repeat(64)}  npm/ascribed-cli-1.2.3.tgz\n${hash}  github/${name}.tar.gz\n`,
    );
    if (tamper) writeFileSync(path.join(tag, `${name}.tar.gz`), "not the archive");
  }

  const bin = path.join(dir, "bin");
  mkdirSync(bin);
  tool(
    bin,
    "curl",
    `url=""; out=""
while [ $# -gt 0 ]; do
  case "$1" in
    -o) out=$2; shift 2 ;;
    https://*) url=$1; shift ;;
    *) shift ;;
  esac
done
case "$url" in
  https://github.com/ascribed-dev/ascribe/releases/latest/download/*) file=${JSON.stringify(tag)}/\${url##*/} ;;
  https://github.com/ascribed-dev/ascribe/releases/download/*) file=${JSON.stringify(release)}/\${url#https://github.com/ascribed-dev/ascribe/releases/download/} ;;
  *) exit 22 ;;
esac
[ -f "$file" ] || exit 22
cp "$file" "$out"`,
  );
  tool(bin, "uname", `[ "$1" = -s ] && echo ${os} || echo ${arch}`);
  tool(bin, "getconf", glibc === "" ? "exit 1" : `echo '${glibc}'`);
  tool(bin, "sysctl", "exit 1");

  const to = path.join(dir, "to");
  const result = spawnSync("sh", [script, "--to", to, ...args], {
    encoding: "utf8",
    env: { HOME: dir, PATH: `${bin}:${process.env.PATH}` },
  });
  return { ...result, installed: existsSync(path.join(to, "ascribe")) };
}

function tool(bin: string, name: string, body: string): void {
  writeFileSync(path.join(bin, name), `#!/bin/sh\n${body}\n`);
  chmodSync(path.join(bin, name), 0o755);
}

function run(command: string, args: string[]): void {
  const result = spawnSync(command, args, { encoding: "utf8" });
  if (result.status !== 0) throw new Error(`${command} failed: ${result.stderr}`);
}

const posix = process.platform !== "win32";
const byName = (a: string, b: string) => a.localeCompare(b);

test("names the targets a release builds, but Windows's", () => {
  const workflow = readFileSync(
    path.join(import.meta.dirname, "../../.github/workflows/release-build.yml"),
    "utf8",
  );
  const build = workflow.slice(workflow.indexOf("  build:"), workflow.indexOf("  pack:"));
  const released = Array.from(build.matchAll(/\{ target: ([\w-]+),/g), ([, target = ""]) => target)
    .filter((target) => !target.startsWith("win32"))
    .sort(byName);
  expect(released.length).toBeGreaterThan(0);
  const installs = Array.from(
    readFileSync(script, "utf8").matchAll(/\btarget=([a-z]+-\w+)/g),
    ([, target = ""]) => target,
  );
  expect([...new Set(installs)].sort(byName)).toEqual(released);
});

test.runIf(posix)("installs the latest release and runs its --version", () => {
  const result = install({});
  expect(result.stderr).toBe("");
  expect(result.status).toBe(0);
  expect(result.installed).toBe(true);
  expect(result.stdout).toContain("Installed ascribe 1.2.3 in");
  expect(result.stdout).toContain("isn't on your PATH");
});

test.runIf(posix)("installs the version it's given, on each platform released", () => {
  for (const machine of [
    { os: "Linux", arch: "x86_64" },
    { os: "Linux", arch: "aarch64" },
    { os: "Darwin", arch: "arm64" },
  ]) {
    const result = install(machine, "--version", "v1.2.3");
    expect(result.status, `${machine.os} ${machine.arch}: ${result.stderr}`).toBe(0);
    expect(result.installed).toBe(true);
  }
});

test.runIf(posix)("stops before installing when the archive doesn't match SHA256SUMS", () => {
  const result = install({ tamper: true });
  expect(result.status).toBe(1);
  expect(result.installed).toBe(false);
  expect(result.stderr).toContain("doesn't match the release's SHA256SUMS");
  expect(result.stderr).toContain("nothing was installed");
});

test.runIf(posix)("stops on a version that isn't released", () => {
  const result = install({}, "--version", "9.9.9");
  expect(result.status).toBe(1);
  expect(result.installed).toBe(false);
  expect(result.stderr).toContain("Is 9.9.9 a release?");
});

test.runIf(posix)("names what's released on a platform that isn't", () => {
  const result = install({ os: "Darwin", arch: "x86_64" });
  expect(result.status).toBe(1);
  expect(result.stderr).toContain("there's no ascribe for Darwin on x86_64");
  expect(result.stderr).toContain("macOS on Apple silicon");
});

test.runIf(posix)("refuses a glibc older than the floor, and a system without glibc", () => {
  const old = install({ glibc: "glibc 2.17" });
  expect(old.status).toBe(1);
  expect(old.stderr).toContain(
    "ascribe runs on Linux with glibc 2.28 or later, and this system has glibc 2.17",
  );
  const musl = install({ glibc: "" });
  expect(musl.status).toBe(1);
  expect(musl.stderr).toContain("this system has no glibc");
  expect(install({ glibc: "glibc 2.28" }).status).toBe(0);
});
