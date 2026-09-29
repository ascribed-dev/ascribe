import { describe, expect, it } from "vitest";
import { compareVersions, formatVersion, parseVersion } from "../../src/version.js";

const v = (text: string) => {
  const parsed = parseVersion(text);
  if (!parsed) throw new Error(`no version in ${text}`);
  return parsed;
};

describe("parseVersion", () => {
  it("finds the version in --version output", () => {
    expect(v("ascribe 0.3.1\n").parts).toEqual([0, 3, 1]);
    expect(v("ascribe 10.20.30 (abc123)").parts).toEqual([10, 20, 30]);
  });

  it("reads a pre-release tag", () => {
    expect(v("ascribe 1.0.0-rc.1").prerelease).toBe("rc.1");
    expect(v("ascribe 1.0.0").prerelease).toBeUndefined();
  });

  it("returns undefined without a three-part version", () => {
    expect(parseVersion("ascribe")).toBeUndefined();
    expect(parseVersion("1.2")).toBeUndefined();
  });
});

describe("compareVersions", () => {
  it("compares numerically, not as text", () => {
    expect(compareVersions(v("0.10.0"), v("0.9.0"))).toBeGreaterThan(0);
    expect(compareVersions(v("1.0.0"), v("0.99.99"))).toBeGreaterThan(0);
    expect(compareVersions(v("1.2.3"), v("1.2.3"))).toBe(0);
    expect(compareVersions(v("1.2.3"), v("1.2.4"))).toBeLessThan(0);
  });

  it("puts a pre-release before its release", () => {
    expect(compareVersions(v("1.0.0-beta"), v("1.0.0"))).toBeLessThan(0);
    expect(compareVersions(v("1.0.0"), v("1.0.0-beta"))).toBeGreaterThan(0);
    expect(compareVersions(v("1.0.0-alpha"), v("1.0.0-beta"))).toBeLessThan(0);
  });
});

describe("formatVersion", () => {
  it("writes a version back as it was written", () => {
    expect(formatVersion(v("0.3.1"))).toBe("0.3.1");
    expect(formatVersion(v("0.3.1-beta.2"))).toBe("0.3.1-beta.2");
  });
});
