// The preview against the published site (phase 25): for each page of the
// Astro end-to-end slice under each build, the HTML the language server's
// `ascribe/preview` returns has to be the HTML Astro built, for the page's
// content: the same elements, attributes, heading ids, and image attributes.
// Asset URLs differ by design (the preview points at source files, Astro at
// its optimized copies), so they're compared by the source file they resolve to.
//
//   cargo build -p tessera-cli
//   pnpm --filter @ascribed/elements build && pnpm --filter @ascribed/astro build
//   pnpm --filter ascribe-vscode test:parity
//
// The site is built with its own script (`pnpm --filter
// @ascribed/example-astro-site build`), in place for its one build, `site`,
// and in copies under the site's `.e2e-tmp/` for two more, so that the
// example's own configuration is untouched: `all` (every arm, badges) and
// `cloud-only` (the cloud arm, availability filtered to cloud).
import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { cp, mkdir, rm } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { chromium, type Browser, type Page } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import type { PreviewResult } from "../../src/preview/protocol.js";
import { canonicalReference } from "../../src/preview/refs.js";
import { LspClient } from "./lsp.js";
import { treeOf, walk, type Tree } from "./normalize.js";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const siteDir = path.join(repository, "examples/astro-site");
const scratch = path.join(siteDir, ".e2e-tmp");
const BASE = "/docs";

const binary = process.env["ASCRIBE_BIN"] ?? path.join(repository, "target", "debug", "ascribe");

interface Variant {
  name: string;
  build: string;
  /** Added to ascribe.toml of the copy. */
  toml?: string;
}

const VARIANTS: Variant[] = [
  { name: "site", build: "site" },
  {
    name: "all",
    build: "all",
    toml: '\n[builds.all]\nvariants = "switch"\navailability = "badge"\n',
  },
  {
    name: "cloud-only",
    build: "cloud-only",
    toml: '\n[builds.cloud-only]\nvariants = { deployment = "cloud" }\navailability = { filter = "cloud" }\n',
  },
];

const roots = new Map<string, string>();
const servers = new Map<string, LspClient>();
let browser: Browser;
let page: Page;

function pnpmBuild(root?: string): void {
  const args = ["--filter", "@ascribed/example-astro-site", "build"];
  if (root !== undefined) args.push("--root", root);
  execFileSync("pnpm", args, {
    cwd: repository,
    stdio: "pipe",
    env: { ...process.env, ASCRIBE_BIN: binary },
  });
}

async function copyVariant(variant: Variant): Promise<string> {
  const target = path.join(scratch, `parity-${variant.name}`);
  await rm(target, { recursive: true, force: true });
  await mkdir(target, { recursive: true });
  for (const entry of ["astro.config.mjs", "ascribe.toml", "content", "src"]) {
    await cp(path.join(siteDir, entry), path.join(target, entry), { recursive: true });
  }
  const edit = (file: string, change: (text: string) => string): void =>
    writeFileSync(path.join(target, file), change(readFileSync(path.join(target, file), "utf8")));
  edit("ascribe.toml", (text) => text + (variant.toml ?? ""));
  edit("astro.config.mjs", (text) => text.replace('build: "site"', `build: "${variant.build}"`));
  // The site imports the generated schema from a path that names the build.
  edit("src/content.config.ts", (text) =>
    text.replace("build/site/site/_ascribe", `build/${variant.build}/site/_ascribe`),
  );
  return target;
}

beforeAll(async () => {
  expect(existsSync(binary), `${binary} exists (cargo build -p tessera-cli, or ASCRIBE_BIN)`).toBe(
    true,
  );
  for (const variant of VARIANTS) {
    if (variant.name === "site") {
      pnpmBuild();
      roots.set(variant.name, siteDir);
    } else {
      const root = await copyVariant(variant);
      pnpmBuild(root);
      roots.set(variant.name, root);
    }
    servers.set(variant.name, await LspClient.start(binary, roots.get(variant.name) as string));
  }
  const configured = process.env["ASCRIBE_CHROMIUM"];
  const bundled = "/opt/pw-browsers/chromium";
  const executablePath = configured ?? (existsSync(bundled) ? bundled : undefined);
  browser = await chromium.launch(executablePath === undefined ? {} : { executablePath });
  page = await (await browser.newContext({ javaScriptEnabled: true })).newPage();
});

afterAll(async () => {
  await browser?.close();
  await Promise.all([...servers.values()].map((server) => server.close()));
  for (const variant of VARIANTS) {
    if (variant.name !== "site") {
      await rm(path.join(scratch, `parity-${variant.name}`), { recursive: true, force: true });
    }
  }
});

/** Every page of the example's content, by content path. */
function pagesOf(root: string): string[] {
  const out: string[] = [];
  const walkDir = (dir: string, prefix: string): void => {
    for (const entry of readdirSync(dir, { withFileTypes: true })) {
      if (entry.name.startsWith("_")) continue;
      if (entry.isDirectory()) walkDir(path.join(dir, entry.name), `${prefix}${entry.name}/`);
      else if (entry.name.endsWith(".md")) out.push(`${prefix}${entry.name}`);
    }
  };
  walkDir(path.join(root, "content"), "");
  return out.sort();
}

async function previewOf(variant: Variant, contentPath: string): Promise<PreviewResult> {
  const root = roots.get(variant.name) as string;
  const uri = pathToFileURL(path.join(root, "content", contentPath)).toString();
  return (await servers
    .get(variant.name)
    ?.request("ascribe/preview", { textDocument: { uri }, build: variant.build })) as PreviewResult;
}

/** The built HTML of a route, if Astro built one: `/docs/guides/my-setup` is `dist/guides/my-setup/index.html`. */
function builtHtml(root: string, route: string): string | undefined {
  const rel = route.slice(BASE.length).replace(/^\/|\/$/g, "");
  const file = path.join(root, "dist", rel, "index.html");
  return existsSync(file) ? readFileSync(file, "utf8") : undefined;
}

/** Sets a page's HTML with its scripts removed, so the element library has not run. */
async function load(html: string): Promise<void> {
  await page.setContent(html.replace(/<script\b[\s\S]*?<\/script>/gi, ""), { waitUntil: "commit" });
}

/** Puts every asset URL of a tree in one form: `asset:<content-relative path>`. */
function assetsBySource(tree: Tree, result: PreviewResult, contentRoot: string): Tree {
  const assets = result.page?.assets ?? [];
  const relative = (file: string): string =>
    path.relative(contentRoot, file).split(path.sep).join("/");
  const byReference = new Map(
    assets.map((a) => [canonicalReference(a.reference), relative(a.path)]),
  );
  const byStem = (url: string): string | undefined => {
    // Astro's copy of an image: `<base>/_astro/<stem>.<hash>.<ext>`.
    const name = /^\/docs\/_astro\/([^/]+)$/.exec(url)?.[1];
    if (name === undefined) return undefined;
    const matches = assets.filter((a) => {
      const stem = path.basename(a.path, path.extname(a.path));
      return new RegExp(`^${stem.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\.[\\w-]+\\.\\w+$`).test(
        name,
      );
    });
    expect(matches.length, `one source file for ${url}`).toBe(1);
    return relative(matches[0]?.path ?? "");
  };
  return walk(tree, (element) => {
    for (const attribute of ["src", "href"]) {
      const value = element.attrs[attribute];
      if (value === undefined) continue;
      const source = byReference.get(canonicalReference(value)) ?? byStem(value);
      if (source !== undefined) {
        element.attrs[attribute] =
          `asset:${source}${value.includes("#") ? value.slice(value.indexOf("#")) : ""}`;
      }
    }
  });
}

const headings = (tree: Tree): string[] => {
  const ids: string[] = [];
  walk(tree, (e) => {
    if (/^h[1-6]$/.test(e.tag)) ids.push(e.attrs["id"] ?? "");
  });
  return ids;
};

describe("the preview equals the published site", () => {
  for (const variant of VARIANTS) {
    describe(`build ${variant.name}`, () => {
      it("has the build in the picker, with the same pages as the site", async () => {
        const root = roots.get(variant.name) as string;
        const first = await previewOf(variant, pagesOf(root)[0] as string);
        expect(first.builds.map((b) => b.name)).toContain(variant.build);
        expect(first.build).toBe(variant.build);
        expect(first.builds.find((b) => b.editor)?.name).toBe("site");
        // A page has a preview exactly when Astro built it.
        for (const contentPath of pagesOf(root)) {
          const result = await previewOf(variant, contentPath);
          const route = result.page?.route;
          if (result.page === null) {
            expect(result.problems.length, contentPath).toBeGreaterThan(0);
          } else {
            expect(builtHtml(root, route as string), `${contentPath} at ${route}`).toBeDefined();
          }
        }
      });

      for (const contentPath of ["index.md", "Guides/My Setup.md", "Reference/Options.md"]) {
        it(`${contentPath}: the same elements, attributes, heading ids, and image attributes`, async () => {
          const root = roots.get(variant.name) as string;
          const result = await previewOf(variant, contentPath);
          const rendered = result.page;
          expect(rendered, JSON.stringify(result.problems)).not.toBeNull();
          if (rendered === null) return;
          const built = builtHtml(root, rendered.route);
          expect(built, rendered.route).toBeDefined();
          const contentRoot = result.contentRoot as string;

          await load(built as string);
          const astro = await treeOf(page, "main > article");
          const astroTitle = await page.locator("main > h1").textContent();
          const astroAvailability = await page
            .locator('ascribe-availability[scope="page"] ascribe-availability-target')
            .evaluateAll((targets) =>
              targets.map((t) => ({
                target: t.getAttribute("target"),
                dimension: t.getAttribute("dimension"),
                states: (t.getAttribute("states") ?? "").split(" "),
                versions: t.getAttribute("versions")?.split(" "),
                text: t.textContent,
              })),
            );

          await load(`<!doctype html><main><article>${rendered.html}</article></main>`);
          const preview = await treeOf(page, "main > article");

          expect(astro, "Astro built the article").toBeDefined();
          expect(preview, "the preview has content").toBeDefined();
          if (!astro || !preview) return;

          // Heading ids first: the clearest failure.
          expect(headings(preview)).toEqual(headings(astro));
          const previewTree = assetsBySource(preview, result, contentRoot);
          const astroTree = assetsBySource(astro, result, contentRoot);
          expect(previewTree).toEqual(astroTree);

          // The layout's title and page-level badges come from the same frontmatter.
          expect(rendered.title).toBe(astroTitle);
          expect(
            (rendered.frontmatter.available ?? []).map((t) => ({
              target: t.target,
              dimension: t.dimension,
              states: t.states,
              versions: t.versions,
              text: t.text,
            })),
          ).toEqual(astroAvailability);
        });
      }

      it("shows the fragment's image and the marker's width, by the same source file", async () => {
        const result = await previewOf(variant, "Guides/My Setup.md");
        const rendered = result.page;
        expect(rendered).not.toBeNull();
        const image = rendered?.assets.find((a) => a.path.endsWith("_fragments/requirements.png"));
        expect(image?.kind).toBe("image");
        expect(image?.servable).toBe(true);
        expect(rendered?.html).toContain('width="300"');
      });
    });
  }

  it("the builds differ where they should, in both", async () => {
    const setup = "Guides/My Setup.md";
    const text = async (variant: Variant): Promise<{ preview: string; astro: string }> => {
      const result = await previewOf(variant, setup);
      const root = roots.get(variant.name) as string;
      await load(builtHtml(root, result.page?.route as string) as string);
      const astro = (await page.locator("main > article").textContent()) ?? "";
      return { preview: result.page?.html ?? "", astro };
    };
    // One after another: they share one browser page.
    const site = await text(VARIANTS[0] as Variant);
    const all = await text(VARIANTS[1] as Variant);
    const cloudOnly = await text(VARIANTS[2] as Variant);
    // `site` selects the cloud arm; `all` keeps both arms as tabs.
    expect(site?.preview).not.toContain("Point the CLI at your own server");
    expect(site?.astro).not.toContain("Point the CLI at your own server");
    expect(all?.preview).toContain("Point the CLI at your own server");
    expect(all?.astro).toContain("Point the CLI at your own server");
    // `cloud-only` filters out the self-managed streaming section.
    expect(all?.preview).toContain("Streaming pushes changes");
    expect(cloudOnly?.preview).not.toContain("Streaming pushes changes");
    expect(cloudOnly?.astro).not.toContain("Streaming pushes changes");
  });
});
