#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
target="${1:-}"
version="${2:-}"
output_dir="${3:-$project_root/release-artifacts}"

if [[ -z "$target" || -z "$version" ]]; then
    echo "usage: $0 <rust-target> <version> [output-directory]" >&2
    exit 2
fi

case "$target" in
    x86_64-unknown-linux-gnu) platform="linux-x86_64" ;;
    x86_64-apple-darwin) platform="macos-x86_64" ;;
    aarch64-apple-darwin) platform="macos-arm64" ;;
    *)
        echo "unsupported release target: $target" >&2
        exit 2
        ;;
esac

for command in cargo python3; do
    if ! command -v "$command" >/dev/null 2>&1; then
        echo "required command not found: $command" >&2
        exit 1
    fi
done

archive_stem="raster-nights-v${version}-${platform}"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT
package_root="$temporary_dir/$archive_stem"

cargo build \
    --locked \
    --release \
    --package raster-nights \
    --target "$target"

mkdir -p "$package_root"
install -m 0755 "$project_root/target/$target/release/raster-nights" \
    "$package_root/raster-nights"
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
