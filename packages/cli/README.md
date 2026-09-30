# @ascribed/cli

The Ascribe command-line tool distributed through npm. The package contains a
small JavaScript launcher and declares one optional native package for each
supported target: macOS arm64/x64, Linux arm64/x64, and Windows x64.

The launcher resolves the matching optional package at runtime and forwards all
arguments and exit status to its `ascribe` executable. It has no `postinstall`
script. If optional dependencies were omitted, reinstall with optional
dependencies enabled; the error identifies the missing package and target.

Integrations can resolve the executable without spawning it:

```ts
import { resolveBinary } from "@ascribed/cli/binary";

const executable = resolveBinary(); // absolute path to the native ascribe binary
```

`resolveBinary({ platform, arch })` is also supported for packaging tests.

## Staging native binaries

Release or CI jobs stage cross-compiled binaries explicitly before packing. Set
the environment variable for the target and run:

```sh
ASCRIBE_BIN_LINUX_X64=target/x86_64-unknown-linux-gnu/release/ascribe \
  pnpm --filter @ascribed/cli stage-native linux-x64
pnpm --filter @ascribed/cli pack
```

The supported variables are `ASCRIBE_BIN_DARWIN_ARM64`,
`ASCRIBE_BIN_DARWIN_X64`, `ASCRIBE_BIN_LINUX_ARM64`,
`ASCRIBE_BIN_LINUX_X64`, and `ASCRIBE_BIN_WIN32_X64`. Staging without a target
stages all five and requires all five variables.
