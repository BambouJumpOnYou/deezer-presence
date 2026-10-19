#!/bin/sh
# Builds mediaremote-adapter into vendor/ (needs cmake).
set -eu

REPO=https://github.com/ungive/mediaremote-adapter.git
COMMIT=0e82bf1

ROOT=$(cd "$(dirname "$0")/.." && pwd)
SRC="$ROOT/vendor/src/mediaremote-adapter"
OUT="$ROOT/vendor/mediaremote-adapter"

[ -d "$SRC/.git" ] || git clone "$REPO" "$SRC"
git -C "$SRC" fetch --quiet origin
git -C "$SRC" checkout --quiet "$COMMIT"

cmake -S "$SRC" -B "$SRC/build" >/dev/null
cmake --build "$SRC/build"

rm -rf "$OUT"
mkdir -p "$OUT"
cp "$SRC/bin/mediaremote-adapter.pl" "$OUT/"
cp -R "$SRC/build/MediaRemoteAdapter.framework" "$OUT/"
cp "$SRC/LICENSE" "$OUT/LICENSE"
echo "adapter ready in $OUT"
