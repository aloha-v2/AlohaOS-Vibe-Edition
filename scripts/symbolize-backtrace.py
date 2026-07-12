#!/usr/bin/env python3
"""Resolve AlohaOS panic addresses with llvm-addr2line or addr2line."""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys

ADDRESS = re.compile(r"(?:backtrace: #\d+\s+|FRAME:\s+)(0x[0-9a-fA-F]+)")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("log", help="serial log containing backtrace addresses")
    parser.add_argument(
        "--kernel",
        default="target/x86_64-unknown-none/release/kernel",
        help="matching unstripped kernel ELF",
    )
    args = parser.parse_args()

    tool = shutil.which("llvm-addr2line") or shutil.which("addr2line")
    if tool is None:
        print("llvm-addr2line or addr2line is required", file=sys.stderr)
        return 2

    with open(args.log, "r", encoding="utf-8", errors="replace") as handle:
        addresses = ADDRESS.findall(handle.read())
    if not addresses:
        print("no backtrace addresses found", file=sys.stderr)
        return 1

    command = [tool, "-e", args.kernel, "-f", "-C", *addresses]
    return subprocess.run(command, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
