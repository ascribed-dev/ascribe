# @ascribed/cli

The `ascribe` command for [Ascribe](https://github.com/ascribed-dev/ascribe): it checks, builds, and formats a documentation project, and runs the language server editors use.

```sh
npm install --save-dev @ascribed/cli
npx ascribe check
npx ascribe build
npx ascribe fmt --check
```

The [command reference](https://github.com/ascribed-dev/ascribe/blob/main/docs/cli.md) covers every command, option, output, and exit code, and [Getting started](https://github.com/ascribed-dev/ascribe/blob/main/docs/getting-started.md) sets up a project.

## Platforms

The package is a small launcher, plus one optional dependency per platform that holds the native binary: macOS on Apple silicon, Linux (arm64 and x64, glibc 2.35 or later), and Windows (x64). Your package manager installs only your platform's. There's no install script. Intel Macs aren't supported.

If `ascribe` reports that it can't find its native package, your package manager left out optional dependencies (for example, `npm install --omit=optional`): reinstall with them. The error names the package and platform it looked for.

## Finding the binary from code

Integrations can find the native executable without running the launcher:

```ts
import { resolveBinary } from "@ascribed/cli/binary";

const executable = resolveBinary(); // the absolute path of this platform's ascribe
```

`resolveBinary({ platform, arch })` resolves another platform's, for packaging tests.

## Development

The platform packages are in `platforms/`. Their binaries aren't in the repository: a release stages each one before packing (`scripts/release/pack.mjs`). To try the launcher with a local build, stage it by hand:

```sh
cargo build -p tessera-cli
ASCRIBE_BIN_DARWIN_ARM64=target/debug/ascribe pnpm --filter @ascribed/cli stage-native darwin-arm64
```

The variables are `ASCRIBE_BIN_DARWIN_ARM64`, `ASCRIBE_BIN_LINUX_ARM64`, `ASCRIBE_BIN_LINUX_X64`, and `ASCRIBE_BIN_WIN32_X64`. Without a target, `stage-native` stages all four and needs all four variables.
