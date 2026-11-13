#!/usr/bin/env bash
set -euo pipefail

target=$1
arch=$2
version=$(python3 -c 'import tomllib; print(tomllib.load(open("Cargo.toml", "rb"))["package"]["version"])')
binary="target/$target/release/examples/showcase"
output="temivu-showcase-macos-$arch.dmg"

work=$(mktemp -d)
stage="$work/image"
app="$stage/Temivu Showcase.app"
resources="$app/Contents/Resources"
mkdir -p "$app/Contents/MacOS" "$work/Temivu.iconset" "$work/mounted"
install -m 755 "$binary" "$app/Contents/MacOS/showcase"
lipo "$app/Contents/MacOS/showcase" -verify_arch "$arch"
python3 examples/showcase/notices.py "$resources/licenses" "$target"

for size in 16 32 128 256; do
  sips -z "$size" "$size" branding/temivu-mark-512.png \
    --out "$work/Temivu.iconset/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" branding/temivu-mark-512.png \
    --out "$work/Temivu.iconset/icon_${size}x${size}@2x.png" >/dev/null
done
cp branding/temivu-mark-512.png "$work/Temivu.iconset/icon_512x512.png"
iconutil -c icns "$work/Temivu.iconset" -o "$resources/Temivu.icns"

plist="$app/Contents/Info.plist"
plutil -create xml1 "$plist"
plutil -insert CFBundleIdentifier -string io.github.kappyra.temivu.showcase "$plist"
plutil -insert CFBundleName -string "Temivu Showcase" "$plist"
plutil -insert CFBundleExecutable -string showcase "$plist"
plutil -insert CFBundlePackageType -string APPL "$plist"
plutil -insert CFBundleShortVersionString -string "$version" "$plist"
plutil -insert CFBundleVersion -string "$version" "$plist"
plutil -insert CFBundleIconFile -string Temivu.icns "$plist"
plutil -insert LSMinimumSystemVersion -string "$MACOSX_DEPLOYMENT_TARGET" "$plist"
plutil -insert NSHighResolutionCapable -bool YES "$plist"
plutil -lint "$plist"

codesign --force --sign - --timestamp=none "$app"
ln -s /Applications "$stage/Applications"
hdiutil create -fs HFS+ -format UDZO -volname "Temivu Showcase" -srcfolder "$stage" "$output"
hdiutil verify "$output"
hdiutil attach "$output" -readonly -nobrowse -mountpoint "$work/mounted"
trap 'hdiutil detach "$work/mounted"' EXIT
mounted="$work/mounted/Temivu Showcase.app"
codesign --verify --strict "$mounted"
test -f "$mounted/Contents/Resources/licenses/third_party/forui/LICENSE-MIT"
test "$(readlink "$work/mounted/Applications")" = /Applications
hdiutil detach "$work/mounted"
trap - EXIT
