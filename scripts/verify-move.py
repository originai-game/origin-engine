#!/usr/bin/env python3
"""Checks that a file split moved code verbatim (AGENTS.md §Streams and landing).

    scripts/verify-move.py <old-path-at-HEAD>[,<old>...] <new-file>...

Compares the non-blank lines of the old files (read from git, default
revision HEAD, or --rev REV) with those of the new files as multisets,
ignoring leading whitespace and the lines a split may add or change: `mod`
and `use` declarations, visibility prefixes, `impl` and `#[cfg(test)]`
headers, closing braces and module doc comments. Prints the lines only one
side has and exits 1 when there are any.
"""
import collections
import re
import subprocess
import sys

VIS = re.compile(r"^pub(\((crate|super|in [\w:]+)\))?\s+")
SKIP = re.compile(r"^(mod |use |impl\b.*\{$|#\[cfg\(test\)\]$|\}$|\};$|//!|#!\[)")


def norm(line: str):
    s = line.strip()
    if not s:
        return None
    s = VIS.sub("", s)
    if SKIP.match(s) or s.startswith(("mod ", "use ")):
        return None
    return s


def bag(texts):
    c = collections.Counter()
    for t in texts:
        in_use = False
        for line in t.splitlines():
            s = line.strip()
            # A multi-line `use a::{ ... };` is skipped whole.
            if in_use:
                in_use = not s.endswith(";")
                continue
            if VIS.sub("", s).startswith("use ") and not s.endswith(";"):
                in_use = True
                continue
            n = norm(line)
            if n is not None:
                c[n] += 1
    return c


def main():
    args = sys.argv[1:]
    rev = "HEAD"
    if args[:1] == ["--rev"]:
        rev, args = args[1], args[2:]
    if len(args) < 2:
        sys.exit(__doc__)
    olds = [subprocess.run(["git", "show", f"{rev}:{p}"], check=True, capture_output=True, text=True).stdout for p in args[0].split(",")]
    news = [open(p, encoding="utf-8").read() for p in args[1:]]
    a, b = bag(olds), bag(news)
    lost, added = a - b, b - a
    for name, d in (("only in the old file", lost), ("only in the new files", added)):
        for line, n in sorted(d.items()):
            print(f"{name} ({n}x): {line}")
    print(f"old {sum(a.values())} lines, new {sum(b.values())} lines, lost {sum(lost.values())}, added {sum(added.values())}")
    sys.exit(1 if lost or added else 0)


if __name__ == "__main__":
    main()
