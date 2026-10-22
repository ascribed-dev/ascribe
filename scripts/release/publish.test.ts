// publish.ts's tag commands against a stand-in for npm, whose packages show
// when the test says: a canary's `next` moves on every package or on none.
import { expect, test, vi } from "vitest";
import { catchUp, moveTag, type Npm } from "./publish.ts";

vi.spyOn(process.stdout, "write").mockImplementation(() => true);

const names = ["@ascribed/cli-linux-arm64", "@ascribed/cli-linux-x64", "@ascribed/cli"];

/** npm, where each package's version shows `after[name]` milliseconds from the start. */
function fakeNpm(
  after: Record<string, number>,
  tags: Record<string, Record<string, string>> = {},
  deprecated: string[] = [],
): Npm & { tags: Record<string, Record<string, string>>; added: string[] } {
  let clock = 0;
  const added: string[] = [];
  return {
    tags,
    added,
    shows: (name) => clock >= (after[name] ?? 0),
    tagged: (name, tag) => tags[name]?.[tag],
    deprecated: (name, version) => deprecated.includes(`${name}@${version}`),
    addTag(name, version, tag) {
      if (clock < (after[name] ?? 0)) throw new Error(`${name}@${version} isn't on npm yet`);
      tags[name] = { ...tags[name], [tag]: version };
      added.push(name);
    },
    now: () => clock,
    sleep(ms) {
      clock += ms;
    },
  };
}

test("tags every package once all show", () => {
  const npm = fakeNpm({});
  expect(moveTag(names, "1.0.0-next.2", "next", npm, { wait: 60_000 })).toBe(true);
  expect(npm.added).toEqual(names);
});

test("waits for a package that shows late, and tags none before it does", () => {
  const npm = fakeNpm({ "@ascribed/cli-linux-arm64": 5 * 60_000 });
  expect(moveTag(names, "1.0.0-next.2", "next", npm, { wait: 15 * 60_000 })).toBe(true);
  expect(npm.now()).toBe(5 * 60_000);
  expect(npm.added).toEqual(names);
});

test("leaves every package's tag where it was when one shows after the wait", () => {
  const tags = Object.fromEntries(names.map((name) => [name, { next: "1.0.0-next.1" }]));
  const npm = fakeNpm({ "@ascribed/cli-linux-arm64": 17 * 60_000 }, tags);
  expect(moveTag(names, "1.0.0-next.2", "next", npm, { wait: 15 * 60_000 })).toBe(false);
  expect(npm.added).toEqual([]);
  for (const name of names) expect(npm.tags[name]?.next).toBe("1.0.0-next.1");
});

test("a later catch-up moves the tag once the late package shows", () => {
  const tags = Object.fromEntries(
    names.map((name) => [name, { next: "1.0.0-next.1", "next-pending": "1.0.0-next.2" }]),
  );
  const npm = fakeNpm({ "@ascribed/cli-linux-arm64": 17 * 60_000 }, tags);
  expect(moveTag(names, "1.0.0-next.2", "next", npm, { wait: 15 * 60_000 })).toBe(false);
  // Still missing: catch-up doesn't wait, and moves nothing.
  expect(catchUp(names, "next-pending", "next", npm)).toBeUndefined();
  npm.sleep(60 * 60_000);
  expect(catchUp(names, "next-pending", "next", npm)).toBe("1.0.0-next.2");
  for (const name of names) expect(npm.tags[name]?.next).toBe("1.0.0-next.2");
});

test("a re-run moves only the packages not yet tagged", () => {
  const tags: Record<string, Record<string, string>> = {
    "@ascribed/cli-linux-arm64": { next: "1.0.0-next.2", "next-pending": "1.0.0-next.2" },
    "@ascribed/cli-linux-x64": { next: "1.0.0-next.1", "next-pending": "1.0.0-next.2" },
    "@ascribed/cli": { next: "1.0.0-next.1", "next-pending": "1.0.0-next.2" },
  };
  const npm = fakeNpm({}, tags);
  expect(catchUp(names, "next-pending", "next", npm)).toBe("1.0.0-next.2");
  expect(npm.added).toEqual(["@ascribed/cli-linux-x64", "@ascribed/cli"]);
});

test("catch-up does nothing when the tag is already there", () => {
  const tags = Object.fromEntries(
    names.map((name) => [name, { next: "1.0.0-next.2", "next-pending": "1.0.0-next.2" }]),
  );
  const npm = fakeNpm({}, tags);
  expect(catchUp(names, "next-pending", "next", npm)).toBeUndefined();
  expect(npm.added).toEqual([]);
});

test("catch-up leaves a deprecated canary behind", () => {
  const tags = Object.fromEntries(
    names.map((name) => [name, { next: "1.0.0-next.1", "next-pending": "1.0.0-next.2" }]),
  );
  const npm = fakeNpm({}, tags, ["@ascribed/cli@1.0.0-next.2"]);
  expect(catchUp(names, "next-pending", "next", npm)).toBeUndefined();
  expect(npm.added).toEqual([]);
});
