#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

artifact_dir="${1:-}"
version="${2:-}"

if [[ -z "$artifact_dir" || -z "$version" ]]; then
    echo "usage: $0 <artifact-directory> <version>" >&2
    exit 2
fi

expected=(
    "raster-nights-v${version}-linux-x86_64.tar.gz"
    "raster-nights-v${version}-macos-x86_64.tar.gz"
    "raster-nights-v${version}-macos-arm64.tar.gz"
    "raster-nights-v${version}-web.tar.gz"
)

temporary_dir="$(mktemp -d)"
trap 'rm -rf "$temporary_dir"' EXIT

for filename in "${expected[@]}"; do
    archive="$artifact_dir/$filename"
    [[ -f "$archive" ]] || {
        echo "missing release archive: $filename" >&2
        exit 1
    }
    tar -xzf "$archive" -C "$temporary_dir"
    package_root="$temporary_dir/${filename%.tar.gz}"
    for required in \
        README.md \
        CHANGELOG.md \
        LICENSE \
        NOTICE \
        ASSET-LICENSES.md \
        DOCUMENT-LICENSES.md \
        TRADEMARKS.md \
        RELEASE-NOTES.md \
        VERSION; do
        [[ -f "$package_root/$required" ]] || {
            echo "$filename is missing $required" >&2
            exit 1
        }
    done
    [[ "$(<"$package_root/VERSION")" == "$version" ]] || {
        echo "$filename contains the wrong version" >&2
        exit 1
    }
done

linux_root="$temporary_dir/raster-nights-v${version}-linux-x86_64"
[[ -x "$linux_root/raster-nights" ]] || {
    echo "Linux archive is missing an executable raster-nights binary" >&2
    exit 1
}

web_root="$temporary_dir/raster-nights-v${version}-web"
[[ -f "$web_root/index.html" ]] || {
    echo "web archive is missing index.html" >&2
    exit 1
}
find "$web_root" -type f -name '*.wasm' -print -quit | grep -q . || {
    echo "web archive is missing its WebAssembly module" >&2
    exit 1
}

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)
        "$linux_root/raster-nights" validate-content >/dev/null
        ;;
esac

echo "release artifacts validated"
