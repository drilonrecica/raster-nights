#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

project_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

mkdir -p "$temporary_dir/package/subdirectory"
printf 'release fixture\n' > "$temporary_dir/package/README.md"
printf '#!/usr/bin/env sh\nexit 0\n' > "$temporary_dir/package/subdirectory/program"
chmod 0755 "$temporary_dir/package/subdirectory/program"

SOURCE_DATE_EPOCH=946684800 python3 "$project_root/scripts/create-release-archive.py" \
    "$temporary_dir/package" "$temporary_dir/first.tar.gz"
SOURCE_DATE_EPOCH=946684800 python3 "$project_root/scripts/create-release-archive.py" \
    "$temporary_dir/package" "$temporary_dir/second.tar.gz"
cmp "$temporary_dir/first.tar.gz" "$temporary_dir/second.tar.gz"

tar -tzf "$temporary_dir/first.tar.gz" > "$temporary_dir/archive-contents"
grep -qx 'package/README.md' "$temporary_dir/archive-contents"
grep -qx 'package/subdirectory/program' "$temporary_dir/archive-contents"

version="0.1.0-rc.1"
arm_archive="$temporary_dir/raster-nights-v${version}-macos-arm64.tar.gz"
intel_archive="$temporary_dir/raster-nights-v${version}-macos-x86_64.tar.gz"
cp "$temporary_dir/first.tar.gz" "$arm_archive"
cp "$temporary_dir/second.tar.gz" "$intel_archive"
"$project_root/scripts/generate-checksums.sh" \
    "$temporary_dir/SHA256SUMS" "$arm_archive" "$intel_archive"
"$project_root/scripts/generate-homebrew-formula.py" \
    --version "$version" \
    --base-url "https://example.invalid/releases/v${version}/" \
    --checksums "$temporary_dir/SHA256SUMS" \
    --output "$temporary_dir/raster-nights.rb"

grep -q "version \"$version\"" "$temporary_dir/raster-nights.rb"
grep -q "raster-nights-v${version}-macos-arm64.tar.gz" \
    "$temporary_dir/raster-nights.rb"
grep -q "raster-nights-v${version}-macos-x86_64.tar.gz" \
    "$temporary_dir/raster-nights.rb"

echo "release tooling tests passed"
