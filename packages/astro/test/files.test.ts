import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  agentsMiddleware,
  copyAgentFiles,
  copyPublishedFiles,
  filesMiddleware,
} from "../src/files.js";

const siteRoot = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
mkdirSync(path.join(siteRoot, "_ascribe", "files", "downloads"), { recursive: true });
writeFileSync(path.join(siteRoot, "_ascribe", "files", "downloads", "my config.yaml"), "a: 1\n");
writeFileSync(path.join(siteRoot, "secret.txt"), "not published");

let server: Server;
let origin: string;

beforeAll(async () => {
  const middleware = filesMiddleware(siteRoot, "/docs/");
  server = createServer((req, res) =>
    middleware(req, res, () => {
      res.statusCode = 404;
      res.end("next");
    }),
  );
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  origin = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
});

afterAll(() => new Promise<void>((resolve) => server.close(() => resolve())));

describe("filesMiddleware", () => {
  it("serves a published file at <base>_ascribe/files/, with its type", async () => {
    const response = await fetch(`${origin}/docs/_ascribe/files/downloads/my%20config.yaml`);
    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/yaml; charset=utf-8");
    expect(await response.text()).toBe("a: 1\n");
  });

  it("passes on anything else, and never leaves the published directory", async () => {
    for (const url of [
      "/docs/other",
      "/docs/_ascribe/files/downloads/nope.yaml",
      "/docs/_ascribe/files/downloads",
      "/docs/_ascribe/files/..%2F..%2Fsecret.txt",
      "/docs/_ascribe/files/%E0%A4%A",
      "/_ascribe/files/downloads/my%20config.yaml",
    ]) {
      const response = await fetch(`${origin}${url}`);
      expect(await response.text(), url).toBe("next");
    }
  });
});

describe("copyPublishedFiles", () => {
  it("copies the published files into the build output", async () => {
    const out = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    expect(await copyPublishedFiles(siteRoot, pathToFileURL(out + path.sep))).toBe(true);
    expect(
      readFileSync(path.join(out, "_ascribe", "files", "downloads", "my config.yaml"), "utf8"),
    ).toBe("a: 1\n");
  });

  it("does nothing when a build published none", async () => {
    const empty = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    expect(await copyPublishedFiles(empty, pathToFileURL(empty + path.sep))).toBe(false);
  });
});

// The plain output for agents, laid out by URL: served and copied at the base path.
const plainRoot = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
mkdirSync(path.join(plainRoot, "guides"), { recursive: true });
writeFileSync(path.join(plainRoot, "llms.txt"), "# Docs\n");
writeFileSync(path.join(plainRoot, "guides", "my-setup.md"), "# Set up\n");

describe("agentsMiddleware", () => {
  let agents: Server;
  let agentsOrigin: string;

  beforeAll(async () => {
    const middleware = agentsMiddleware(plainRoot, "/docs/");
    agents = createServer((req, res) =>
      middleware(req, res, () => {
        res.statusCode = 404;
        res.end("next");
      }),
    );
    await new Promise<void>((resolve) => agents.listen(0, "127.0.0.1", resolve));
    agentsOrigin = `http://127.0.0.1:${(agents.address() as AddressInfo).port}`;
  });

  afterAll(() => new Promise<void>((resolve) => agents.close(() => resolve())));

  it("serves llms.txt and each page's Markdown under the base path, with their types", async () => {
    const index = await fetch(`${agentsOrigin}/docs/llms.txt`);
    expect(index.headers.get("content-type")).toBe("text/plain; charset=utf-8");
    expect(await index.text()).toBe("# Docs\n");
    const page = await fetch(`${agentsOrigin}/docs/guides/my-setup.md`);
    expect(page.headers.get("content-type")).toBe("text/markdown; charset=utf-8");
    expect(await page.text()).toBe("# Set up\n");
  });

  it("passes on the pages themselves and anything outside the base path", async () => {
    for (const url of ["/docs/guides/my-setup", "/docs/guides", "/llms.txt", "/docs/../llms.txt"]) {
      const response = await fetch(`${agentsOrigin}${url}`);
      expect(await response.text(), url).toBe("next");
    }
  });
});

describe("copyAgentFiles", () => {
  it("copies the plain output to the root of the build output", async () => {
    const out = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    expect(await copyAgentFiles(plainRoot, pathToFileURL(out + path.sep))).toBe(true);
    expect(readFileSync(path.join(out, "llms.txt"), "utf8")).toBe("# Docs\n");
    expect(readFileSync(path.join(out, "guides", "my-setup.md"), "utf8")).toBe("# Set up\n");
  });
});
