import { describe, expect, it, vi } from "vitest";
import { nativeBinaryPath, nativePackageName, resolveBinary } from "../src/binary.js";

describe("platform selection", () => {
  it.each([
    ["darwin", "arm64", "@ascribed/cli-darwin-arm64"],
    ["linux", "arm64", "@ascribed/cli-linux-arm64"],
    ["linux", "x64", "@ascribed/cli-linux-x64"],
    ["win32", "x64", "@ascribed/cli-win32-x64"],
  ])("maps %s/%s to %s", (platform, arch, expected) => {
    expect(nativePackageName(platform, arch)).toBe(expected);
  });

  it("rejects Intel Macs, which aren't supported", () => {
    expect(() => nativePackageName("darwin", "x64")).toThrow(
      /does not support darwin\/x64.*Supported platforms are darwin arm64,/,
    );
  });

  it("rejects unsupported targets with an actionable message", () => {
    expect(() => nativePackageName("freebsd", "x64")).toThrow(
      /does not support freebsd\/x64.*Supported platforms/,
    );
  });

  it("reports the optional package when it is not installed", async () => {
    // A built checkout stages the host's binary, so resolution is made to fail
    // here rather than depending on what is installed.
    vi.resetModules();
    vi.doMock("node:module", () => ({
      createRequire: () => ({
        resolve: (request: string) => {
          throw Object.assign(new Error(`Cannot find module '${request}'`), {
            code: "MODULE_NOT_FOUND",
          });
        },
      }),
    }));
    try {
      const { nativeBinaryPath: missing } = await import("../src/binary.js");
      expect(() => missing("linux", "x64")).toThrow(
        /could not find its native package @ascribed\/cli-linux-x64/,
      );
      expect(() => missing("linux", "x64")).toThrow(/optional dependencies/);
    } finally {
      vi.doUnmock("node:module");
      vi.resetModules();
    }
  });

  it("exposes resolveBinary as the integration API", () => {
    expect(resolveBinary).toBeTypeOf("function");
    expect(nativeBinaryPath).toBeTypeOf("function");
  });
});
