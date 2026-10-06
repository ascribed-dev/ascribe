// Shared by the end-to-end tests: build and serve the site with Astro's own
// JavaScript API, and drive it in Chromium.
import { spawn } from "node:child_process";
import { existsSync } from "node:fs";
import { cp, mkdir, rm } from "node:fs/promises";
import { createServer } from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build, preview } from "astro";
import { chromium, type Browser } from "playwright-core";

/** The example site's directory. */
export const siteDir = fileURLToPath(new URL("../..", import.meta.url));

/** The site's base path, from astro.config.mjs. */
export const BASE = "/docs";

/** The bundled Chromium in this environment, if there is one; otherwise Playwright's own. */
export function launchChromium(): Promise<Browser> {
  const configured = process.env["ASCRIBE_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  const executablePath = configured ?? (existsSync(bundled) ? bundled : undefined);
  return chromium.launch(executablePath === undefined ? {} : { executablePath });
}

/** A port nothing is listening on. */
function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address !== null ? address.port : 0;
      server.close(() => resolve(port));
    });
  });
}

/** Builds the site rooted at `root` (the integration runs `ascribe build` first). */
export function buildSite(root: string = siteDir): Promise<void> {
  return build({ root, logLevel: "warn" });
}

/** Serves a built site, returning its origin and a function that stops it. */
export async function servePreview(
  root: string = siteDir,
): Promise<{ origin: string; stop(): Promise<void> }> {
  const port = await freePort();
  const server = await preview({ root, logLevel: "warn", server: { host: "127.0.0.1", port } });
  return { origin: `http://127.0.0.1:${server.port}`, stop: () => server.stop() };
}

/**
 * Starts the dev server in its own process, with `env` added to its
 * environment, returning its origin and a function that stops it.
 */
export async function serveDev(
  root: string = siteDir,
  env: Record<string, string> = {},
): Promise<{ origin: string; stop(): Promise<void> }> {
  const port = await freePort();
  const script = fileURLToPath(new URL("dev-server.mjs", import.meta.url));
  const child = spawn(process.execPath, [script, root, String(port)], {
    stdio: ["ignore", "pipe", "inherit"],
    // The test runner sets NODE_ENV=test and VITEST*, which Astro's dev server takes for another mode.
    env: {
      ...Object.fromEntries(
        Object.entries(process.env).filter(([name]) => !name.startsWith("VITEST")),
      ),
      NODE_ENV: "development",
      ...env,
    },
  });
  const stop = (): Promise<void> =>
    new Promise((resolve) => {
      if (child.exitCode !== null) return resolve();
      child.once("exit", () => resolve());
      child.kill("SIGTERM");
    });
  return new Promise((resolve, reject) => {
    let output = "";
    child.once("error", reject);
    child.once("exit", (code) =>
      reject(new Error(`the dev server exited with ${code}: ${output}`)),
    );
    child.stdout.on("data", (chunk: Buffer) => {
      output += chunk.toString();
      if (output.includes("READY")) resolve({ origin: `http://127.0.0.1:${port}`, stop });
    });
  });
}

/**
 * A copy of the site's sources under `.e2e-tmp/<name>/`, inside the site's
 * directory so that the copy finds the same `node_modules`. `edit` changes it.
 */
export async function copySite(name: string): Promise<string> {
  const target = path.join(siteDir, ".e2e-tmp", name);
  await rm(target, { recursive: true, force: true });
  await mkdir(target, { recursive: true });
  for (const entry of ["astro.config.mjs", "ascribe.toml", "content", "src"]) {
    await cp(path.join(siteDir, entry), path.join(target, entry), { recursive: true });
  }
  return target;
}
