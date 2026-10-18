#!/usr/bin/env bash
# Build "Sable.app" plus a disk image and a zip for GitHub Releases.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' crates/ui/Cargo.toml | head -1)"
if [[ -z "$version" ]]; then
  echo "Could not read version from crates/ui/Cargo.toml" >&2
  exit 1
fi

export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-12.0}"

echo "Building Sable ${version} for ${MACOSX_DEPLOYMENT_TARGET}+"
cargo build -p sable --release

app="$root/dist/Sable.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

cp "$root/target/release/sable" "$app/Contents/MacOS/Sable"
chmod +x "$app/Contents/MacOS/Sable"
printf 'APPL????' > "$app/Contents/PkgInfo"

iconset="$(mktemp -d)/Sable.iconset"
python3 "$root/scripts/make_icon.py" "$iconset"
iconutil -c icns "$iconset" -o "$app/Contents/Resources/Sable.icns"
rm -rf "$(dirname "$iconset")"

sed "s/VERSION/${version}/g" "$root/packaging/macos/Info.plist" > "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist" >/dev/null

# Ad-hoc signature. Gatekeeper still asks on first open of a downloaded build.
codesign --force --sign - "$app"
codesign --verify --verbose=2 "$app"

arch_name="$(uname -m)"
case "$arch_name" in
  arm64) arch_label="arm64" ;;
  x86_64) arch_label="x64" ;;
  *) arch_label="$arch_name" ;;
esac

stage="$(mktemp -d)"
cp -R "$app" "$stage/Sable.app"
ln -s /Applications "$stage/Applications"
dmg="$root/dist/Sable-${version}-macos-${arch_label}.dmg"
hdiutil create -volname "Sable" -srcfolder "$stage" -ov -format UDZO "$dmg"
rm -rf "$stage"

zip="$root/dist/Sable-${version}-macos-${arch_label}.zip"
rm -f "$zip"
ditto -c -k --keepParent "$app" "$zip"

echo
echo "Installable builds:"
ls -lh "$app" "$dmg" "$zip"
