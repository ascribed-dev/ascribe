// The navigation lists every published page, and only those.
import { beforeAll, expect, test } from "vitest";
import { nav } from "../src/nav.ts";
import { publishedPages, requireBuild } from "./built.ts";

beforeAll(requireBuild);

const listed = nav.flatMap((group) => group.pages);

test("every published page is in src/nav.ts", async () => {
  const missing = (await publishedPages()).filter((page) => !listed.includes(page));
  expect(missing, "add these pages to src/nav.ts").toEqual([]);
});

test("src/nav.ts names only published pages", async () => {
  const pages = await publishedPages();
  const unknown = listed.filter((page) => !pages.includes(page));
  expect(unknown, "src/nav.ts names pages that don't exist").toEqual([]);
});

test("src/nav.ts names each page once", () => {
  const repeated = listed.filter((page, i) => listed.indexOf(page) !== i);
  expect(repeated).toEqual([]);
});
