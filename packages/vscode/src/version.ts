/** A parsed version: `major.minor.patch`, with an optional pre-release tag. */
export interface Version {
  readonly parts: readonly [number, number, number];
  readonly prerelease: string | undefined;
}

/**
 * Finds the first `major.minor.patch` (with an optional `-pre-release`) in a
 * string, such as the output of `tessera --version` (`tessera 0.3.1`).
 * Returns `undefined` when there isn't one.
 */
export function parseVersion(text: string): Version | undefined {
  const match = /(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?/.exec(text);
  if (!match) return undefined;
  const [, major, minor, patch, prerelease] = match;
  return {
    parts: [Number(major), Number(minor), Number(patch)],
    prerelease,
  };
}

/**
 * Compares two versions as semantic versioning does: numerically by part, and
 * a pre-release sorts before its release. Returns a negative number when `a`
 * is older than `b`, zero when equal, and a positive number when newer.
 */
export function compareVersions(a: Version, b: Version): number {
  for (let i = 0; i < 3; i++) {
    const difference = (a.parts[i] ?? 0) - (b.parts[i] ?? 0);
    if (difference !== 0) return difference;
  }
  if (a.prerelease === b.prerelease) return 0;
  if (a.prerelease === undefined) return 1;
  if (b.prerelease === undefined) return -1;
  return a.prerelease < b.prerelease ? -1 : 1;
}

/** Formats a version the way it's written: `1.2.3` or `1.2.3-beta.1`. */
export function formatVersion(version: Version): string {
  const core = version.parts.join(".");
  return version.prerelease === undefined ? core : `${core}-${version.prerelease}`;
}
