#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0

set -euo pipefail

output="${1:-}"
shift || true

if [[ -z "$output" || "$#" -eq 0 ]]; then
    echo "usage: $0 <output-file> <archive>..." >&2
    exit 2
fi

if command -v sha256sum >/dev/null 2>&1; then
    checksum_command=(sha256sum)
elif command -v shasum >/dev/null 2>&1; then
    checksum_command=(shasum -a 256)
else
    echo "neither sha256sum nor shasum is available" >&2
    exit 1
fi

temporary_file="$(mktemp)"
trap 'rm -f "$temporary_file"' EXIT

for archive in "$@"; do
    if [[ ! -f "$archive" ]]; then
        echo "archive not found: $archive" >&2
        exit 1
    fi
    checksum="$("${checksum_command[@]}" "$archive" | awk '{print $1}')"
    printf '%s  %s\n' "$checksum" "$(basename "$archive")" >> "$temporary_file"
done

LC_ALL=C sort -k2 "$temporary_file" > "$output"
