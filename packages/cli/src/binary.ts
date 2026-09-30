import { createRequire } from "node:module";
import { arch, platform } from "node:process";

const require = createRequire(import.meta.url);

const packages: Record<string, string> = {
  "darwin-arm64": "@ascribed/cli-darwin-arm64",
  "linux-arm64": "@ascribed/cli-linux-arm64",
  "linux-x64": "@ascribed/cli-linux-x64",
  "win32-x64": "@ascribed/cli-win32-x64",
};

export interface BinaryPlatform {
  platform?: string;
  arch?: string;
}

export function nativePackageName(
  currentPlatform: string = platform,
  currentArch: string = arch,
): string {
  const name = packages[`${currentPlatform}-${currentArch}`];
  if (name === undefined) {
    throw new Error(
      `@ascribed/cli does not support ${currentPlatform}/${currentArch}. ` +
        "Supported platforms are darwin arm64, linux arm64/x64, and win32 x64.",
    );
  }
  return name;
}

/**
 * Resolve the installed native executable for the current machine.
 *
 * The optional `platform` and `arch` arguments are intended for packaging
 * tests; callers normally use `resolveBinary()` with no arguments.
 */
export function resolveBinary(options: BinaryPlatform = {}): string {
  const currentPlatform = options.platform ?? platform;
  const currentArch = options.arch ?? arch;
  const packageName = nativePackageName(currentPlatform, currentArch);
  const executable = currentPlatform === "win32" ? "ascribe.exe" : "ascribe";
  try {
    return require.resolve(`${packageName}/bin/${executable}`);
  } catch (error) {
    const reason = error instanceof Error ? ` (${error.message})` : "";
    throw new Error(
      `@ascribed/cli could not find its native package ${packageName}${reason}. ` +
        `Reinstall @ascribed/cli for ${currentPlatform}/${currentArch}, and ensure optional ` +
        "dependencies are enabled. The package manager must not omit optional dependencies.",
      { cause: error },
    );
  }
}

/** Backwards-compatible name for resolveBinary. */
export const nativeBinaryPath = (currentPlatform?: string, currentArch?: string): string =>
  resolveBinary(
    currentPlatform === undefined && currentArch === undefined
      ? {}
      : {
          ...(currentPlatform === undefined ? {} : { platform: currentPlatform }),
          ...(currentArch === undefined ? {} : { arch: currentArch }),
        },
  );
