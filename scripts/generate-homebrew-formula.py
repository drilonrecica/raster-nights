#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0

"""Generate the Raster Nights Homebrew formula from release checksums."""

from __future__ import annotations

import argparse
from pathlib import Path
from urllib.parse import urljoin


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--base-url", required=True)
    parser.add_argument("--checksums", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument(
        "--template",
        type=Path,
        default=Path(__file__).with_name("homebrew-formula.rb.in"),
    )
    return parser.parse_args()


def read_checksums(path: Path) -> dict[str, str]:
    checksums: dict[str, str] = {}
    for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        fields = line.split()
        if not fields:
            continue
        if len(fields) != 2 or len(fields[0]) != 64:
            raise SystemExit(f"{path}:{line_number}: invalid SHA-256 entry")
        checksums[fields[1].lstrip("*")] = fields[0].lower()
    return checksums


def main() -> None:
    args = parse_args()
    base_url = args.base_url.rstrip("/") + "/"
    names = {
        "ARM64": f"raster-nights-v{args.version}-macos-arm64.tar.gz",
        "X86_64": f"raster-nights-v{args.version}-macos-x86_64.tar.gz",
    }
    checksums = read_checksums(args.checksums)
    missing = [name for name in names.values() if name not in checksums]
    if missing:
        raise SystemExit(f"checksums are missing: {', '.join(missing)}")

    formula = args.template.read_text(encoding="utf-8")
    replacements = {"@VERSION@": args.version}
    for architecture, name in names.items():
        replacements[f"@{architecture}_URL@"] = urljoin(base_url, name)
        replacements[f"@{architecture}_SHA256@"] = checksums[name]
    for marker, value in replacements.items():
        formula = formula.replace(marker, value)
    if "@" in formula:
        raise SystemExit("unexpanded marker remains in formula")

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(formula, encoding="utf-8")


if __name__ == "__main__":
    main()
