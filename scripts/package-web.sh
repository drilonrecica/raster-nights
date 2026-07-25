#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
version="${1:-}"
output_dir="${2:-$project_root/release-artifacts}"

if [[ -z "$version" ]]; then
    echo "usage: $0 <version> [output-directory]" >&2
    exit 2
fi

for command in npm python3 wasm-pack; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "required command not found: $command" >&2
        exit 1
    fi
done

archive_stem="raster-nights-v${version}-web"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT
package_root="$temporary_dir/$archive_stem"

rm -rf "$project_root/website/public/wasm" "$project_root/website/dist"
wasm-pack build "$project_root/apps/web" \
    --target web \
    --no-pack \
    --release \
    --out-dir "$project_root/website/public/wasm"
npm ci --prefix "$project_root/website"
npm --prefix "$project_root/website" run build

mkdir -p "$package_root"
cp -R "$project_root/website/dist/." "$package_root/"
for file in \
    README.md \
    CHANGELOG.md \
    LICENSE \
    NOTICE \
    ASSET-LICENSES.md \
    DOCUMENT-LICENSES.md \
    TRADEMARKS.md; do
    install -m 0644 "$project_root/$file" "$package_root/$file"
done
install -m 0644 "$project_root/docs/releases/v${version}.md" \
    "$package_root/RELEASE-NOTES.md"
printf '%s\n' "$version" > "$package_root/VERSION"

python3 "$project_root/scripts/create-release-archive.py" \
    "$package_root" \
    "$output_dir/$archive_stem.tar.gz"

echo "$output_dir/$archive_stem.tar.gz"
