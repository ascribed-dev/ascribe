// An older line of a versioned build against the Web Documentation Delivery
// Spec's checker, as agent-docs.test.ts checks the site: 3.4's `llms.txt`
// and Markdown, at `/docs/3.4/`, are a whole set of docs for agents of their
// own. Run by `pnpm test:e2e`, with the same needs as e2e.test.ts.
import { rm } from "node:fs/promises";
import path from "node:path";
import type { AgentDocsConfig } from "afdocs";
import { describeAgentDocsPerCheck } from "afdocs/helpers";
import { afterAll, beforeAll } from "vitest";
import { BASE, buildSite, servePreview, siteDir } from "./harness.js";
import { versioned } from "./sites.js";

let server: Awaited<ReturnType<typeof servePreview>> | undefined;

beforeAll(async () => {
  const root = await versioned("agent-docs-lines");
  await buildSite(root);
  server = await servePreview(root);
}, 180_000);

afterAll(async () => {
  await server?.stop();
  await rm(path.join(siteDir, ".e2e-tmp"), { recursive: true, force: true });
});

const config: AgentDocsConfig = {
  // Read when the checks run, after the preview server has started.
  get url() {
    if (server === undefined) throw new Error("the preview server didn't start");
    return `${server.origin}${BASE}/3.4`;
  },
  options: {
    canonicalOrigin: "https://docs.example.com",
    samplingStrategy: "deterministic",
  },
  // As agent-docs.test.ts: the host's part, which `astro preview` doesn't do.
  skipChecks: ["content-negotiation"],
};

describeAgentDocsPerCheck(config);
