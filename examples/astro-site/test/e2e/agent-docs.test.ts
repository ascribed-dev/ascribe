// The built site against the Web Documentation Delivery Spec, with its own
// checker, `afdocs` (pinned in package.json, with the spec version Ascribe
// targets in `ascribe_resolve::llms::DELIVERY_SPEC`): one test per check, so
// a regression in llms.txt, the Markdown pages, or the pointer to the index
// fails by name. Run by `pnpm test:e2e`, with the same needs as e2e.test.ts.
import type { AgentDocsConfig } from "afdocs";
import { describeAgentDocsPerCheck } from "afdocs/helpers";
import { afterAll, beforeAll } from "vitest";
import { BASE, buildSite, servePreview } from "./harness.js";

let server: Awaited<ReturnType<typeof servePreview>> | undefined;

beforeAll(async () => {
  await buildSite();
  server = await servePreview();
}, 180_000);

afterAll(async () => {
  await server?.stop();
});

const config: AgentDocsConfig = {
  // Read when the checks run, after the preview server has started.
  get url() {
    if (server === undefined) throw new Error("the preview server didn't start");
    return `${server.origin}${BASE}`;
  },
  options: {
    // llms.txt and the pages link to the published origin, ascribe.toml's `site`.
    canonicalOrigin: "https://docs.example.com",
    samplingStrategy: "deterministic",
  },
  skipChecks: [
    // Answering `Accept: text/markdown` with the Markdown is the web server's
    // part, which `astro preview` doesn't do: docs/content/guides/astro.md
    // says what to set on a host.
    "content-negotiation",
  ],
};

describeAgentDocsPerCheck(config);
