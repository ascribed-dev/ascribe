#!/bin/sh
# Prints a crash's signature from a `cargo fuzz run` log: what failed (the
# panic's file, or libFuzzer's kind of error, such as a timeout), then each
# frame of the stack in this repository's code, by function and file. Line
# numbers are left out, so an unrelated edit above the crash doesn't make it
# a new one. The nightly workflow (.github/workflows/fuzz.yml) keys its issue
# by this, hashed.
set -eu
log=$1
{
  # `panicked at crates/ascribe-fmt/src/lib.rs:12:5:`, without the line.
  grep -o 'panicked at [^ ]*' "$log" | head -n 1 | sed -E 's/:[0-9]+:[0-9]+:?$//' || true
  # `==123== ERROR: libFuzzer: timeout after 10 seconds`, without the numbers.
  grep -o 'ERROR: libFuzzer: [a-z -]*' "$log" | head -n 1 || true
  # `#19 0x55f1 in format::_::run /home/runner/work/ascribe/ascribe/fuzz/fuzz_targets/format.rs:33:5`
  grep -E '^ *#[0-9]+ 0x[0-9a-f]+ in ' "$log" |
    sed -nE 's#^ *\#[0-9]+ 0x[0-9a-f]+ in (.*) [^ ]*/((crates|fuzz_targets)/[^:]*):[0-9]+(:[0-9]+)?$#\1 \2#p'
} | awk '!seen[$0]++'
