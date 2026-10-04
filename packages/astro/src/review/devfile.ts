// `.ascribe/dev.json`: where the dev server is, for the editor's Open Site
// Preview. Written when the server starts, removed when it stops. A server
// that crashed leaves it behind, so a reader checks the URL answers first.
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import type { AddressInfo } from "node:net";
import path from "node:path";

/** What `dev.json` holds. */
export interface DevFile {
  /** The site's address: origin and base path, `http://localhost:4321/docs/`. */
  url: string;
  /** The build the dev server shows. */
  build: string;
  /** The dev server's process, so another server's file isn't removed. */
  pid: number;
}

/** `<project>/.ascribe/dev.json`. */
export function devFilePath(projectDir: string): string {
  return path.join(projectDir, ".ascribe", "dev.json");
}

/** The site's address from the server's: a wildcard or loopback host is `localhost`. */
export function siteUrl(address: AddressInfo, options: { https: boolean; base: string }): string {
  let host = address.address;
  if (["::", "0.0.0.0", "::1", "", "localhost"].includes(host)) host = "localhost";
  else if (host.includes(":")) host = `[${host}]`;
  const base = options.base.endsWith("/") ? options.base : `${options.base}/`;
  return `${options.https ? "https" : "http"}://${host}:${address.port}${base.startsWith("/") ? base : `/${base}`}`;
}

/** Writes `dev.json`. A failure is the caller's to log: it only costs the editor its link. */
export function writeDevFile(projectDir: string, file: DevFile): void {
  const target = devFilePath(projectDir);
  mkdirSync(path.dirname(target), { recursive: true });
  writeFileSync(target, `${JSON.stringify(file, null, 2)}\n`);
}

/** Removes `dev.json` if this process wrote it. */
export function removeDevFile(projectDir: string, pid: number): void {
  const target = devFilePath(projectDir);
  try {
    const current = JSON.parse(readFileSync(target, "utf8")) as Partial<DevFile>;
    if (current.pid !== pid) return;
  } catch {
    return;
  }
  rmSync(target, { force: true });
}
