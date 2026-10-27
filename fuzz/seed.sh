#!/bin/sh
# Seeds each target's corpus with the conformance cases: their pages for
# `parse`, `format`, and `frontmatter`, their models for `model`, and the
# OpenAPI specs their `api/` folders hold for `openapi`. A file already in a
# corpus is left alone, so running it over the corpus the nightly workflow
# keeps only adds what's new. `example` takes a two-part input (a schema and
# an example) that no single case file matches, so it starts from nothing.
# Run from anywhere.
set -eu
fuzz=$(cd "$(dirname "$0")" && pwd)
cases="$fuzz/../tests/conformance/cases"
mkdir -p "$fuzz/corpus/parse" "$fuzz/corpus/format" "$fuzz/corpus/model" \
  "$fuzz/corpus/frontmatter" "$fuzz/corpus/openapi" "$fuzz/corpus/example"
# Named by content, as libFuzzer names what it adds. `$1` is a `find`
# expression (a `-name`, optionally scoped with `-path`), `$2` the targets.
seed() {
  # shellcheck disable=SC2086
  find "$cases" $1 -type f | while read -r file; do
    name=$(git hash-object "$file")
    for target in $2; do
      [ -e "$fuzz/corpus/$target/$name" ] || cp "$file" "$fuzz/corpus/$target/$name"
    done
  done
}
seed "-name *.md" "parse format frontmatter"
seed "-name ascribe.toml" model
# The specs a `[apis.<name>] file` names live in a case's `api/` folder.
seed "-path */api/* -name *.yaml" openapi
seed "-path */api/* -name *.json" openapi
