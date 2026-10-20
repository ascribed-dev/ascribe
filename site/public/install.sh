#!/bin/sh
# Installs ascribe, the command line tool of Ascribe (https://ascribed-dev.com),
# from a release on GitHub, without Node.
#
#   curl -fsSL https://ascribed-dev.com/install.sh | sh
#   curl -fsSL https://ascribed-dev.com/install.sh | sh -s -- --version 0.3.0
#
# Options:
#   --version <version>  the release to install, such as 0.3.0 (default: the latest)
#   --to <folder>        where to put `ascribe` (default: $HOME/.local/bin)
#   --help               print this and stop
#
# It works out the platform, downloads that release's archive and its
# SHA256SUMS over HTTPS from the release's own address, checks the archive's
# hash, and puts the binary in the folder. It stops, installing nothing, when
# the hash doesn't match or there's no tool to check it with. It never asks for
# sudo, reads no variable but HOME and PATH, and runs nothing it downloaded except the
# installed binary's `--version`.
#
# This file is served from Ascribe's repository, site/public/install.sh, and
# changes only through it.

set -eu

repo="ascribed-dev/ascribe"
# The oldest glibc the Linux binaries run on: `GLIBC` in
# .github/workflows/release-build.yml (scripts/docs-site/facts.test.ts).
glibc_floor="2.28"
docs="https://ascribed-dev.com/go/install/"

say() { printf '%s\n' "$*"; }
fail() {
  printf 'ascribe install: %s\n' "$*" >&2
  exit 1
}

version=""
to=""
while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      [ $# -ge 2 ] || fail "--version needs a version, such as 0.3.0"
      version=${2#v}
      shift 2
      ;;
    --to)
      [ $# -ge 2 ] || fail "--to needs a folder"
      to=$2
      shift 2
      ;;
    -h | --help)
      say "Installs ascribe from a release on GitHub. $docs"
      say ""
      say "Options:"
      say "  --version <version>  the release to install, such as 0.3.0 (default: the latest)"
      say "  --to <folder>        where to put ascribe (default: \$HOME/.local/bin)"
      exit 0
      ;;
    *) fail "unknown option \`$1\`. Options: --version <version>, --to <folder>, --help" ;;
  esac
done

if [ -z "$to" ]; then
  [ -n "${HOME:-}" ] || fail "HOME isn't set; say where to install with --to <folder>"
  to="$HOME/.local/bin"
fi

# The platform, named as a release names its archives.
os=$(uname -s)
arch=$(uname -m)
case "$os" in
  Linux)
    case "$arch" in
      x86_64 | amd64) target=linux-x64 ;;
      aarch64 | arm64) target=linux-arm64 ;;
      *) target="" ;;
    esac
    ;;
  Darwin)
    # A shell under Rosetta says x86_64 on Apple silicon; the hardware doesn't.
    if [ "$arch" = arm64 ] || [ "$(sysctl -n hw.optional.arm64 2> /dev/null || true)" = 1 ]; then
      target=darwin-arm64
    else
      target=""
    fi
    ;;
  *) target="" ;;
esac
if [ -z "$target" ]; then
  fail "there's no ascribe for $os on $arch here. Releases have it for Linux (x64 and arm64), macOS on Apple silicon, and Windows (x64); on Windows, see $docs"
fi

if [ "$os" = Linux ]; then
  libc=$(getconf GNU_LIBC_VERSION 2> /dev/null || true)
  found=${libc#glibc }
  if [ -z "$libc" ] || [ "$found" = "$libc" ]; then
    fail "ascribe runs on Linux with glibc $glibc_floor or later, and this system has no glibc (Alpine and other musl systems aren't supported)"
  fi
  # Compare major.minor as numbers.
  need_major=${glibc_floor%%.*}
  need_minor=${glibc_floor#*.}
  have_major=${found%%.*}
  have_minor=${found#*.}
  have_minor=${have_minor%%.*}
  if [ "$have_major" -lt "$need_major" ] ||
    { [ "$have_major" -eq "$need_major" ] && [ "$have_minor" -lt "$need_minor" ]; }; then
    fail "ascribe runs on Linux with glibc $glibc_floor or later, and this system has glibc $found"
  fi
fi

# Downloads over HTTPS only, failing on an HTTP error.
if command -v curl > /dev/null 2>&1; then
  fetch() { curl --proto '=https' --tlsv1.2 -fsSL "$1" -o "$2"; }
elif command -v wget > /dev/null 2>&1; then
  fetch() { wget --https-only -q "$1" -O "$2"; }
else
  fail "neither curl nor wget is installed, and one is needed to download ascribe"
fi

# A SHA-256 hash, with whichever tool the system has.
if command -v sha256sum > /dev/null 2>&1; then
  hash_of() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum > /dev/null 2>&1; then
  hash_of() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
  fail "no sha256sum or shasum to check the download with, so nothing was installed"
fi

work=$(mktemp -d 2> /dev/null || mktemp -d -t ascribe)
trap 'rm -rf "$work"' EXIT INT TERM

if [ -z "$version" ]; then
  # The latest release's SHA256SUMS names its version. GitHub's API isn't
  # asked, since it limits how often a machine may ask without signing in.
  fetch "https://github.com/$repo/releases/latest/download/SHA256SUMS" "$work/SHA256SUMS" ||
    fail "couldn't download the latest release's SHA256SUMS from GitHub; name a release with --version"
  version=$(sed -n "s|.*[ /]ascribe-\(.*\)-$target\.tar\.gz\$|\1|p" "$work/SHA256SUMS" | head -n 1)
  [ -n "$version" ] || fail "the latest release's SHA256SUMS names no archive for $target"
fi
case "$version" in
  *[!0-9A-Za-z.+-]* | "") fail "\`$version\` isn't a version, such as 0.3.0" ;;
esac

name="ascribe-$version-$target"
base="https://github.com/$repo/releases/download/v$version"
say "Downloading ascribe $version for $target"
if [ ! -f "$work/SHA256SUMS" ]; then
  fetch "$base/SHA256SUMS" "$work/SHA256SUMS" ||
    fail "couldn't download $base/SHA256SUMS. Is $version a release? They're listed at https://github.com/$repo/releases"
fi
fetch "$base/$name.tar.gz" "$work/$name.tar.gz" || fail "couldn't download $base/$name.tar.gz"

# SHA256SUMS names each file by its path in the release's folders, such as
# `github/ascribe-0.3.0-linux-x64.tar.gz`.
expected=$(awk -v file="$name.tar.gz" '$2 == file || $2 ~ ("/" file "$") { print $1; exit }' "$work/SHA256SUMS")
[ -n "$expected" ] || fail "$name.tar.gz isn't in the release's SHA256SUMS, so nothing was installed"
actual=$(hash_of "$work/$name.tar.gz")
if [ "$actual" != "$expected" ]; then
  fail "$name.tar.gz doesn't match the release's SHA256SUMS (expected $expected, got $actual), so nothing was installed"
fi

tar -xzf "$work/$name.tar.gz" -C "$work"
[ -f "$work/$name/ascribe" ] || fail "$name.tar.gz has no ascribe in it, so nothing was installed"

mkdir -p "$to" || fail "couldn't make $to; choose another folder with --to"
# Copied beside the old one and then renamed, so a running ascribe isn't overwritten in place.
cp "$work/$name/ascribe" "$to/.ascribe.new" || fail "couldn't write to $to; choose another folder with --to"
chmod 755 "$to/.ascribe.new"
mv -f "$to/.ascribe.new" "$to/ascribe"

say "Installed $("$to/ascribe" --version) in $to/ascribe"
case ":${PATH:-}:" in
  *":$to:"*) ;;
  *)
    say ""
    say "$to isn't on your PATH, so \`ascribe\` won't be found there yet. Add it in your shell's profile:"
    say ""
    say "  export PATH=\"$to:\$PATH\""
    ;;
esac
