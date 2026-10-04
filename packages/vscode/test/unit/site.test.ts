// Finding the site preview: `.ascribe/dev.json`, whether its server answers,
// and the page's address on it (`src/preview/site.ts`).
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  answers,
  devFilePath,
  findDevServer,
  noDevServerMessage,
  pageUrl,
  parseDevFile,
  sectionAt,
} from "../../src/preview/site.js";

let server: Server;
let origin: string;

beforeAll(async () => {
  server = createServer((_request, response) => {
    response.writeHead(404).end();
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  origin = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
});

afterAll(() => {
  server.close();
});

/** A port nothing listens on: one that was just closed. */
async function closedPort(): Promise<number> {
  const probe = createServer();
  await new Promise<void>((resolve) => probe.listen(0, "127.0.0.1", resolve));
  const { port } = probe.address() as AddressInfo;
  await new Promise((resolve) => probe.close(resolve));
  return port;
}

function project(devFile?: string): string {
  const folder = mkdtempSync(path.join(tmpdir(), "ascribe-site-"));
  if (devFile !== undefined) {
    mkdirSync(path.dirname(devFilePath(folder)), { recursive: true });
    writeFileSync(devFilePath(folder), devFile);
  }
  return folder;
}

describe("dev.json", () => {
  it("is read for its address and build", () => {
    expect(
      parseDevFile('{ "url": "http://localhost:4321/docs/", "build": "site", "pid": 7 }'),
    ).toEqual({ url: "http://localhost:4321/docs/", build: "site" });
  });

  it("is ignored when it isn't one", () => {
    expect(parseDevFile("{")).toBeUndefined();
    expect(parseDevFile("[]")).toBeUndefined();
    expect(parseDevFile('{ "url": "http://localhost:4321/" }')).toBeUndefined();
    expect(parseDevFile('{ "url": "not a url", "build": "site" }')).toBeUndefined();
    expect(parseDevFile('{ "url": "file:///etc/passwd", "build": "site" }')).toBeUndefined();
  });

  it("names only a server on this machine", () => {
    for (const url of ["http://localhost:4321/", "http://127.0.0.1:4321/", "https://[::1]:4321/"]) {
      expect(parseDevFile(JSON.stringify({ url, build: "site" })), url).toEqual({
        url,
        build: "site",
      });
    }
    for (const url of [
      "https://example.com/",
      "http://192.168.0.41:4321/",
      "http://localhost.example.com/",
    ]) {
      expect(parseDevFile(JSON.stringify({ url, build: "site" })), url).toBeUndefined();
    }
  });
});

describe("findDevServer", () => {
  it("finds a server that answers, whatever it answers", async () => {
    const folder = project(JSON.stringify({ url: `${origin}/docs/`, build: "site", pid: 1 }));
    expect(await findDevServer(folder)).toEqual({
      state: "running",
      server: { url: `${origin}/docs/`, build: "site" },
    });
  });

  it("finds none without dev.json, or when its server is gone", async () => {
    expect(await findDevServer(project())).toEqual({ state: "none" });
    const stale = project(
      JSON.stringify({ url: `http://127.0.0.1:${await closedPort()}/`, build: "site", pid: 1 }),
    );
    expect(await findDevServer(stale)).toEqual({ state: "none" });
    expect(await findDevServer(project("not json"))).toEqual({ state: "none" });
  });

  it("gives up on a server that doesn't answer in time", async () => {
    const silent = createServer(() => undefined);
    await new Promise<void>((resolve) => silent.listen(0, "127.0.0.1", resolve));
    const { port } = silent.address() as AddressInfo;
    try {
      expect(await answers(`http://127.0.0.1:${port}/`, 200)).toBe(false);
    } finally {
      silent.closeAllConnections();
      silent.close();
    }
  });
});

describe("the page's address", () => {
  const dev = { url: "http://localhost:4321/docs/", build: "site" };

  it("is the route on the server's origin, the route having the base path", () => {
    expect(pageUrl(dev, "/docs/guides/my-setup")).toBe(
      "http://localhost:4321/docs/guides/my-setup",
    );
    expect(pageUrl(dev, "/docs/guides/my-setup", "weave-configuration")).toBe(
      "http://localhost:4321/docs/guides/my-setup#weave-configuration",
    );
  });

  it("opens at the last heading at or above the editor's top line", () => {
    const sections = [
      { id: "set-up", line: 4 },
      { id: "weave", line: 20 },
      { id: "after", line: 40 },
    ];
    expect(sectionAt(sections, 0)).toBeUndefined();
    expect(sectionAt(sections, 4)).toBe("set-up");
    expect(sectionAt(sections, 39)).toBe("weave");
    expect(sectionAt([...sections].reverse(), 39)).toBe("weave");
  });

  it("says how to start a dev server when there's none", () => {
    expect(noDevServerMessage("handbook")).toContain("Start its dev server (astro dev)");
  });
});
