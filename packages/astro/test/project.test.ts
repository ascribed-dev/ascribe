import { chmodSync, mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import {
  consumerMismatches,
  homeRoute,
  normalizeBase,
  readProject,
  supplyRouting,
} from "../src/project.js";

function project(toml: string): string {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
  writeFileSync(path.join(dir, "ascribe.toml"), toml);
  return dir;
}

// A stand-in for the compiler: a shell script that prints `json` and the
// arguments it was given to a file beside it.
function script(json: string): string {
  const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
  const file = path.join(dir, "ascribe");
  writeFileSync(path.join(dir, "answer.json"), json);
  writeFileSync(file, `#!/bin/sh\necho "$*" > "${dir}/args"\ncat "${dir}/answer.json"\n`);
  chmodSync(file, 0o755);
  return file;
}

describe.skipIf(process.platform === "win32")("readProject with [from]", () => {
  it("asks the binary for [consumer], since what's taken is in another file", () => {
    const dir = project(`
spec = "0.1"
[from."../docs/ascribe.toml"]
take = ["consumer.site", "consumer.trailing-slash"]
[consumer]
base-path = "/"
agents = false
`);
    const binary = script(
      JSON.stringify({
        consumer: [
          { key: "site", value: "https://docs.example.com", set: true },
          { key: "base-path", value: "/", set: true },
          { key: "trailing-slash", value: "never", set: true },
          { key: "agents", value: false, set: true },
          { key: "headers", set: false },
        ],
        builds: [{ name: "site" }],
      }),
    );
    const info = readProject(dir, { binary, root: dir });
    expect(info.consumer).toEqual({
      site: "https://docs.example.com",
      basePath: "/",
      trailingSlash: "never",
      agents: false,
      headers: undefined,
    });
    // Its builds are its own: `[from]` doesn't take them.
    expect(info.builds).toEqual([]);
  });

  it("says what the binary said when it can't read the model", () => {
    const dir = project('spec = "0.1"\n[from."missing.toml"]\ntake = ["consumer.site"]\n');
    const binary = path.join(mkdtempSync(path.join(tmpdir(), "ascribe-astro-")), "ascribe");
    writeFileSync(binary, '#!/bin/sh\necho "error: missing.toml doesn\'t exist" >&2\nexit 2\n');
    chmodSync(binary, 0o755);
    expect(() => readProject(dir, { binary, root: dir })).toThrow(
      /whose \[from\] takes keys from another file, with `ascribe model`: error: missing.toml doesn't exist/,
    );
  });
});

describe("readProject", () => {
  it("reads the output directory and consumer settings", () => {
    const dir = project(`
spec = "0.1"
[project]
output-dir = "out"
[consumer]
site = "https://docs.example.com"
base-path = "/docs"
trailing-slash = "never"
agents = true
headers = "netlify"
`);
    const info = readProject(dir);
    expect(info.consumer).toEqual({
      site: "https://docs.example.com",
      basePath: "/docs/",
      trailingSlash: "never",
      agents: true,
      headers: "netlify",
    });
    expect(info.siteRoot("site")).toBe(path.join(dir, "out", "site", "site"));
    expect(info.plainRoot("site")).toBe(path.join(dir, "out", "site", "plain"));
  });

  it("needs no [builds] table: the implicit `site` build has the same output layout", () => {
    const info = readProject(project('spec = "0.1"\n[project]\ncontent-root = "docs"\n'));
    expect(info.siteRoot("site")).toBe(path.join(info.dir, ".ascribe", "build", "site", "site"));
  });

  it("publishes for agents with a site unless `agents` says otherwise", () => {
    const site = '[consumer]\nsite = "https://docs.example.com"\n';
    expect(readProject(project(`spec = "0.1"\n${site}`)).consumer.agents).toBe(true);
    expect(readProject(project(`spec = "0.1"\n${site}agents = false\n`)).consumer.agents).toBe(
      false,
    );
  });

  it("uses the profile's defaults", () => {
    const info = readProject(project('spec = "0.1"\n'));
    expect(info.contentRoot).toBe(path.join(info.dir, "docs"));
    expect(info.consumer).toEqual({
      site: undefined,
      basePath: "/",
      trailingSlash: "always",
      agents: false,
      headers: undefined,
    });
    expect(info.siteRoot("site")).toBe(path.join(info.dir, ".ascribe", "build", "site", "site"));
  });

  it("says which file it couldn't read", () => {
    const dir = mkdtempSync(path.join(tmpdir(), "ascribe-astro-"));
    mkdirSync(path.join(dir, "empty"));
    expect(() => readProject(path.join(dir, "empty"))).toThrow(/ascribe\.toml/);
  });
});

describe("consumerMismatches", () => {
  const info = readProject(
    project(
      '[consumer]\nsite = "https://a.example"\nbase-path = "/docs/"\ntrailing-slash = "never"\n',
    ),
  );

  it("accepts the same routing however the base path is written", () => {
    expect(
      consumerMismatches(info, {
        base: "/docs",
        trailingSlash: "never",
        site: "https://a.example/",
      }),
    ).toEqual([]);
    expect(
      consumerMismatches(info, { base: "/docs/", trailingSlash: "ignore", site: undefined }),
    ).toEqual([]);
  });

  it("accepts a base path under Astro's base, which leaves the rest of the site to other pages", () => {
    expect(
      consumerMismatches(info, { base: "/", trailingSlash: "never", site: undefined }),
    ).toEqual([]);
    expect(
      consumerMismatches(info, { base: "/doc", trailingSlash: "never", site: undefined }),
    ).toHaveLength(1);
  });

  it("names each difference", () => {
    const problems = consumerMismatches(info, {
      base: "/manual",
      trailingSlash: "always",
      site: "https://b.example",
    });
    expect(problems).toHaveLength(3);
    expect(problems.join("\n")).toMatch(/base-path.*"\/docs\/".*"\/manual".*or a path under it/);
    expect(problems.join("\n")).toMatch(/trailing-slash.*"never".*"always"/);
    expect(problems.join("\n")).toMatch(/site/);
  });
});

describe("supplyRouting", () => {
  const docs = readProject(
    project(
      '[consumer]\nsite = "https://a.example"\nbase-path = "/docs/"\ntrailing-slash = "never"\n',
    ),
  );
  const home = readProject(
    project('[consumer]\nsite = "https://a.example"\ntrailing-slash = "never"\n'),
  );
  const unset = { base: "/", trailingSlash: "ignore", site: undefined } as const;

  it("takes what astro.config leaves unset from [consumer], with one project", () => {
    const { routing, supplied } = supplyRouting([docs], unset, true);
    expect(supplied).toEqual({ site: "https://a.example", trailingSlash: "never", base: "/docs" });
    expect(routing).toEqual(supplied);
    expect(consumerMismatches(docs, routing)).toEqual([]);
  });

  it("keeps what astro.config sets, which must still agree", () => {
    const astro = { base: "/manual/", trailingSlash: "always", site: "https://b.example" } as const;
    const { routing, supplied } = supplyRouting([docs], astro, true);
    expect(supplied).toEqual({});
    expect(routing).toEqual(astro);
    expect(consumerMismatches(docs, routing)).toHaveLength(3);
  });

  it("ends a base astro.config sets the way a supplied trailingSlash says", () => {
    const set = { ...unset, base: "/docs/" };
    expect(supplyRouting([docs], set, true).supplied.base).toBe("/docs");
    expect(supplyRouting([docs], set, false).supplied.base).toBe("/docs");
    expect(supplyRouting([docs], { ...unset, base: "/docs" }, true).supplied.base).toBeUndefined();
  });

  it("leaves the base the site's with several projects, and takes the rest from the first that has it", () => {
    const { routing, supplied } = supplyRouting([home, docs], unset, false);
    expect(supplied).toEqual({ site: "https://a.example", trailingSlash: "never" });
    expect(routing.base).toBe("/");
    expect(consumerMismatches(home, routing)).toEqual([]);
    expect(consumerMismatches(docs, routing)).toEqual([]);
    const bare = readProject(project('spec = "0.1"\n'));
    expect(supplyRouting([bare, docs], unset, false).supplied.site).toBe("https://a.example");
    // The profile's default, which Ascribe writes its links with.
    expect(supplyRouting([bare], unset, true).supplied).toEqual({ trailingSlash: "always" });
  });
});

describe("normalizeBase", () => {
  it("adds the missing slashes", () => {
    expect(normalizeBase("docs")).toBe("/docs/");
    expect(normalizeBase("/")).toBe("/");
  });
});

describe("homeRoute", () => {
  it("is the base, ending as the site's routes do", () => {
    expect(homeRoute("/docs", "never")).toBe("/docs");
    expect(homeRoute("/docs/", "never")).toBe("/docs");
    expect(homeRoute("/docs", "always")).toBe("/docs/");
    expect(homeRoute("/docs", "ignore")).toBe("/docs/");
    expect(homeRoute("/", "never")).toBe("/");
  });
});
