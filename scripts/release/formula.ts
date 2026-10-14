// Writes the Homebrew formula for a release, `Formula/ascribe.rb` in the tap
// ascribed-dev/homebrew-tap, from the release's SHA256SUMS: one archive and
// its hash for each platform Homebrew installs on.
//
//   node scripts/release/formula.ts <version> [--sums <file>] [--out <file>]
//
// Without --sums, it downloads the published release's SHA256SUMS. The
// release workflow's `tap` job writes it this way and opens the tap's pull
// request; a dry run writes it from the packed release's SHA256SUMS
// (RELEASING.md).
import { writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { parseArgs } from "node:util";

const repository = "ascribed-dev/ascribe";

/** The platforms Homebrew installs on, as a release names its archives, and the formula's blocks for each. */
export const PLATFORMS = [
  { target: "darwin-arm64", os: "on_macos", cpu: "on_arm" },
  { target: "linux-arm64", os: "on_linux", cpu: "on_arm" },
  { target: "linux-x64", os: "on_linux", cpu: "on_intel" },
] as const;

/** The hash of each file SHA256SUMS lists, by its name without the folder. */
export function readSums(text: string): Map<string, string> {
  const sums = new Map<string, string>();
  for (const line of text.split(/\r?\n/)) {
    const match = /^([0-9a-f]{64}) [ *]?(\S+)$/.exec(line.trim());
    const [, hash, file] = match ?? [];
    if (hash !== undefined && file !== undefined) sums.set(path.basename(file), hash);
  }
  return sums;
}

/** The formula for `version`, from its release's SHA256SUMS. */
export function formula(version: string, sums: string): string {
  if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`\`${version}\` isn't a version, such as 0.3.0`);
  }
  const hashes = readSums(sums);
  const archive = (target: string) => {
    const name = `ascribe-${version}-${target}.tar.gz`;
    const hash = hashes.get(name);
    if (hash === undefined) throw new Error(`SHA256SUMS has no ${name}`);
    return [
      `      url "https://github.com/${repository}/releases/download/v${version}/${name}"`,
      `      sha256 "${hash}"`,
    ];
  };
  const block = (os: string) => {
    const lines = [`  ${os} do`];
    if (os === "on_macos") lines.push("    depends_on arch: :arm64");
    for (const platform of PLATFORMS.filter((p) => p.os === os)) {
      lines.push(`    ${platform.cpu} do`, ...archive(platform.target), "    end");
    }
    lines.push("  end");
    return lines;
  };
  return [
    `# Written by scripts/release/formula.ts in ${repository} for each release.`,
    "# Change that script, not this file.",
    "class Ascribe < Formula",
    `  desc "Documentation as code: Markdown with directives, checked and built"`,
    `  homepage "https://ascribed-dev.com"`,
    `  version "${version}"`,
    `  license "MPL-2.0"`,
    "",
    ...block("on_macos"),
    "",
    ...block("on_linux"),
    "",
    "  def install",
    `    bin.install "ascribe"`,
    "  end",
    "",
    "  test do",
    `    assert_equal "ascribe #{version}", shell_output("#{bin}/ascribe --version").strip`,
    "  end",
    "end",
    "",
  ].join("\n");
}

async function sumsOf(version: string, file: string | undefined): Promise<string> {
  if (file !== undefined) return readFile(file, "utf8");
  const url = `https://github.com/${repository}/releases/download/v${version}/SHA256SUMS`;
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${url}: ${response.status} ${response.statusText}`);
  return response.text();
}

if (import.meta.main) {
  const { positionals, values } = parseArgs({
    allowPositionals: true,
    options: { sums: { type: "string" }, out: { type: "string" } },
  });
  const [version] = positionals;
  if (version === undefined || positionals.length > 1) {
    process.stderr.write(
      "usage: node scripts/release/formula.ts <version> [--sums <file>] [--out <file>]\n",
    );
    process.exit(2);
  }
  try {
    const text = formula(
      version.replace(/^v/, ""),
      await sumsOf(version.replace(/^v/, ""), values.sums),
    );
    if (values.out === undefined) process.stdout.write(text);
    else writeFileSync(values.out, text);
  } catch (error) {
    process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    process.exit(1);
  }
}
