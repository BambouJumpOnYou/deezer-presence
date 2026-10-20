#!/bin/sh
# Builds "dist/Deezer Presence.app".
# --universal: Apple Silicon + Intel binary. --zip: also writes dist/Deezer-Presence-<version>-macos.zip.
set -eu

UNIVERSAL=0
ZIP=0
for arg in "$@"; do
    case "$arg" in
        --universal) UNIVERSAL=1 ;;
        --zip) ZIP=1 ;;
        *) echo "unknown option: $arg" >&2; exit 1 ;;
    esac
done

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP="$ROOT/dist/Deezer Presence.app"
BIN=deezer-presence
BUNDLE_ID=io.github.bamboujumponyou.deezer-presence
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)

if [ "$UNIVERSAL" = 1 ]; then
    for target in aarch64-apple-darwin x86_64-apple-darwin; do
        cargo build --release --target "$target" --manifest-path "$ROOT/Cargo.toml"
    done
    mkdir -p "$ROOT/target/universal"
    lipo -create -output "$ROOT/target/universal/$BIN" \
        "$ROOT/target/aarch64-apple-darwin/release/$BIN" \
        "$ROOT/target/x86_64-apple-darwin/release/$BIN"
    BUILT="$ROOT/target/universal/$BIN"
else
    cargo build --release --manifest-path "$ROOT/Cargo.toml"
    BUILT="$ROOT/target/release/$BIN"
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BUILT" "$APP/Contents/MacOS/$BIN"

ICONSET=$(mktemp -d)/AppIcon.iconset
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
    sips -z $size $size "$ROOT/assets/AppIcon.png" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z $double $double "$ROOT/assets/AppIcon.png" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$(dirname "$ICONSET")"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Deezer Presence</string>
    <key>CFBundleDisplayName</key>
    <string>Deezer Presence</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleExecutable</key>
    <string>$BIN</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

codesign --force --deep --sign - "$APP"
echo "built $APP"

if [ "$ZIP" = 1 ]; then
    ARCHIVE="$ROOT/dist/Deezer-Presence-$VERSION-macos.zip"
    rm -f "$ARCHIVE"
    ditto -c -k --keepParent "$APP" "$ARCHIVE"
    echo "zipped $ARCHIVE"
fi
