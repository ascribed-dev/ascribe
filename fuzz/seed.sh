#!/bin/sh
# Seeds each target's corpus with the conformance cases: their pages for
# `parse` and `format`, their models for `model`. A file already in a corpus
# is left alone, so running it over the corpus the nightly workflow keeps
# only adds what's new. Run from anywhere.
set -eu
fuzz=$(cd "$(dirname "$0")" && pwd)
cases="$fuzz/../tests/conformance/cases"
mkdir -p "$fuzz/corpus/parse" "$fuzz/corpus/format" "$fuzz/corpus/model"
# Named by content, as libFuzzer names what it adds.
seed() {
  find "$cases" -name "$1" -type f | while read -r file; do
    name=$(git hash-object "$file")
    for target in $2; do
      [ -e "$fuzz/corpus/$target/$name" ] || cp "$file" "$fuzz/corpus/$target/$name"
    done
  done
}
seed '*.md' "parse format"
seed ascribe.toml model
