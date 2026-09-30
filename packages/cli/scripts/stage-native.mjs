import { chmod, copyFile, mkdir, stat } from "node:fs/promises";
import { resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const targets = [
  ["darwin", "arm64", "ASCRIBE_BIN_DARWIN_ARM64"],
  ["linux", "arm64", "ASCRIBE_BIN_LINUX_ARM64"],
  ["linux", "x64", "ASCRIBE_BIN_LINUX_X64"],
  ["win32", "x64", "ASCRIBE_BIN_WIN32_X64"],
];
const root = resolve(fileURLToPath(import.meta.url), "..", "..", "platforms");

if (process.argv.length > 2) {
  const [target] = process.argv.slice(2);
  if (
    target !== undefined &&
    !targets.some(([platform, arch]) => `${platform}-${arch}` === target)
  ) {
    throw new Error(
      `Unknown target ${target}. Choose one of ${targets.map(([p, a]) => `${p}-${a}`).join(", ")}.`,
    );
  }
}

const selected = targets.filter(([platform, arch]) => {
  const requested = process.argv[2];
  return requested === undefined || requested === `${platform}-${arch}`;
});
const missing = [];
for (const [platform, arch, variable] of selected) {
  const source = process.env[variable];
  if (source === undefined || source === "") {
    missing.push(variable);
    continue;
  }
  const destination = resolve(
    root,
    `cli-${platform}-${arch}`,
    "bin",
    platform === "win32" ? "ascribe.exe" : "ascribe",
  );
  await mkdir(resolve(destination, ".."), { recursive: true });
  const info = await stat(source);
  if (!info.isFile()) throw new Error(`${variable} must point to a regular file: ${source}`);
  await copyFile(source, destination);
  if (platform !== "win32") await chmod(destination, 0o755);
  process.stdout.write(`Staged ${platform}-${arch}: ${destination}\n`);
}
if (missing.length > 0) {
  throw new Error(`Missing native binary environment variable(s): ${missing.join(", ")}.`);
}
