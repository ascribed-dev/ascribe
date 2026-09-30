import { describe, expect, it } from "vitest";
import { nativeBinaryPath, nativePackageName, resolveBinary } from "../src/binary.js";

describe("platform selection", () => {
  it.each([
    ["darwin", "arm64", "@ascribed/cli-darwin-arm64"],
    ["darwin", "x64", "@ascribed/cli-darwin-x64"],
    ["linux", "arm64", "@ascribed/cli-linux-arm64"],
    ["linux", "x64", "@ascribed/cli-linux-x64"],
    ["win32", "x64", "@ascribed/cli-win32-x64"],
  ])("maps %s/%s to %s", (platform, arch, expected) => {
    expect(nativePackageName(platform, arch)).toBe(expected);
  });

  it("rejects unsupported targets with an actionable message", () => {
    expect(() => nativePackageName("freebsd", "x64")).toThrow(
      /does not support freebsd\/x64.*Supported platforms/,
    );
  });

  it("reports the optional package when it is not installed", () => {
    expect(() => nativeBinaryPath("linux", "x64")).toThrow(
      /could not find its native package @ascribed\/cli-linux-x64/,
    );
    expect(() => nativeBinaryPath("linux", "x64")).toThrow(/optional dependencies/);
  });

  it("exposes resolveBinary as the integration API", () => {
    expect(resolveBinary).toBeTypeOf("function");
    expect(nativeBinaryPath).toBeTypeOf("function");
  });
});
