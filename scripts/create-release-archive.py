#!/usr/bin/env python3
# SPDX-License-Identifier: MPL-2.0

"""Create a deterministic gzip-compressed tar archive from one directory."""

from __future__ import annotations

import argparse
import gzip
import os
from pathlib import Path
import tarfile


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path, help="directory to archive")
    parser.add_argument("output", type=Path, help="output .tar.gz path")
    return parser.parse_args()


def normalized_info(tar: tarfile.TarFile, path: Path, arcname: str, epoch: int) -> tarfile.TarInfo:
    info = tar.gettarinfo(str(path), arcname)
    info.uid = 0
    info.gid = 0
    info.uname = "root"
    info.gname = "root"
    info.mtime = epoch
    if info.isdir():
        info.mode = 0o755
    elif info.isfile():
        info.mode = 0o755 if os.access(path, os.X_OK) else 0o644
    return info


def main() -> None:
    args = parse_args()
    source = args.source.resolve()
    output = args.output.resolve()
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH", "0"))

    if not source.is_dir():
        raise SystemExit(f"source directory does not exist: {source}")
    if output.suffixes[-2:] != [".tar", ".gz"]:
        raise SystemExit("output filename must end in .tar.gz")

    entries = [source, *sorted(source.rglob("*"), key=lambda path: path.as_posix())]
    if any(path.is_symlink() for path in entries):
        raise SystemExit("release archives may not contain symbolic links")

    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=epoch) as compressed:
            with tarfile.open(fileobj=compressed, mode="w", format=tarfile.GNU_FORMAT) as archive:
                for path in entries:
                    arcname = path.relative_to(source.parent).as_posix()
                    info = normalized_info(archive, path, arcname, epoch)
                    if info.isfile():
                        with path.open("rb") as file:
                            archive.addfile(info, file)
                    else:
                        archive.addfile(info)


if __name__ == "__main__":
    main()
