#!/bin/sh
# Runs the `ascribe` binary the VS Code extension would, with the arguments
# given: the one `ASCRIBE_PATH` names, then the project's
# `node_modules/.bin/ascribe`, then `ascribe` on the path. The project's is
# looked for from `CLAUDE_PROJECT_DIR`, or the working directory, up to the
# repository's root.
#
# Written by the `ascribe` binary (crates/ascribe-cli/src/agents/plugin.rs).

if [ -n "${ASCRIBE_PATH:-}" ]; then
  if [ -f "$ASCRIBE_PATH" ] && [ -x "$ASCRIBE_PATH" ]; then
    exec "$ASCRIBE_PATH" "$@"
  fi
  echo "ascribe: ASCRIBE_PATH is \"$ASCRIBE_PATH\", but that isn't a binary that runs. Fix it, or unset it to use the project's binary or the one on your path." >&2
  exit 127
fi

dir=${CLAUDE_PROJECT_DIR:-$PWD}
while :; do
  if [ -f "$dir/node_modules/.bin/ascribe" ] && [ -x "$dir/node_modules/.bin/ascribe" ]; then
    exec "$dir/node_modules/.bin/ascribe" "$@"
  fi
  if [ -e "$dir/.git" ]; then
    break
  fi
  parent=$(dirname "$dir")
  if [ "$parent" = "$dir" ]; then
    break
  fi
  dir=$parent
done

if command -v ascribe >/dev/null 2>&1; then
  exec ascribe "$@"
fi
echo "ascribe: couldn't find the Ascribe binary. Install it in the project (npm install --save-dev @ascribed/cli), put it on your path, or set ASCRIBE_PATH to it." >&2
exit 127
