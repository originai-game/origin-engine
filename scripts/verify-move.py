#!/usr/bin/env python3
"""Checks that a file split moved code verbatim (AGENTS.md §Streams and landing).

    scripts/verify-move.py [--rev REV] [RENAMES] <old-path>[,<old>...] <new-file>...
    scripts/verify-move.py [--rev REV] [RENAMES] --in-place <file>...

The first form compares the non-blank lines of the old files (read from
git at REV, default HEAD) with those of the new files as multisets,
ignoring leading whitespace and the lines a split may add or change: `mod`
and `use` declarations, visibility prefixes, `impl` and `#[cfg(test)]`
headers, closing braces and module doc comments. Prints the lines only one
side has and exits 1 when there are any.

The second form checks a regrouping that edits files where they are: each
file at REV against the file now, in order, token by token, so rustfmt
rewrapping a line (whitespace, the trailing comma of a broken list) is not
a difference. Prints each differing span with its lines on both sides and
exits 1 when there are any.

RENAMES rewrite the old side before it is compared, so a commit that only
renames paths checks as verbatim modulo them:

    --rename OLD=NEW    repeatable, e.g. --rename self.theme=self.env.theme
    --renames FILE      one OLD=NEW per line; blank lines and # comments skipped

OLD matches as a literal, not inside a longer identifier, so `self.tick`
does not match `self.ticks`; with --in-place it matches as a token
sequence, so a path rustfmt broke across lines is renamed too. All renames
apply in one pass, longest OLD first, so NEW is never renamed again.
"""
import collections
import difflib
import re
import subprocess
import sys

VIS = re.compile(r"^pub(\((crate|super|in [\w:]+)\))?\s+")
SKIP = re.compile(r"^(mod |use |impl\b.*\{$|#\[cfg\(test\)\]$|\}$|\};$|//!|#!\[)")
TOKEN = re.compile(r'"(?:\\.|[^"\\])*"|\w+|\S')


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


def renamer(pairs):
    """One function applying every rename in a single pass."""
    if not pairs:
        return lambda text: text
    table = dict(pairs)
    parts = []
    for old in sorted(table, key=len, reverse=True):
        left = r"(?<!\w)" if re.match(r"\w", old) else ""
        right = r"(?!\w)" if re.search(r"\w$", old) else ""
        parts.append(left + re.escape(old) + right)
    pattern = re.compile("|".join(parts))
    return lambda text: pattern.sub(lambda m: table[m.group(0)], text)


def tokens(text):
    """Tokens with their line numbers, a comma before a closer dropped."""
    out = []
    for n, line in enumerate(text.splitlines(), 1):
        for m in TOKEN.finditer(line):
            t = m.group(0)
            if t in ")]}" and out and out[-1][0] == ",":
                out.pop()
            out.append((t, n))
    return out


def span(toks, lo, hi):
    """The lines tokens lo..hi cover; an insertion shows the line it is at."""
    if not toks:
        return 0, 0
    if hi > lo:
        return toks[lo][1], toks[hi - 1][1]
    at = toks[min(lo, len(toks) - 1)][1]
    return at, at


def token_renamer(pairs):
    """The renames over a token list, so a path rustfmt broke across lines
    (`self\\n    .kept`) is renamed too."""
    rules = collections.defaultdict(list)
    for old, new in pairs:
        o = [t for t, _ in tokens(old)]
        rules[o[0]].append((o, [t for t, _ in tokens(new)]))
    for r in rules.values():
        r.sort(key=lambda on: -len(on[0]))

    def apply(toks):
        out, i = [], 0
        while i < len(toks):
            for o, n in rules.get(toks[i][0], ()):
                if [t for t, _ in toks[i:i + len(o)]] == o:
                    out += [(t, toks[i][1]) for t in n]
                    i += len(o)
                    break
            else:
                out.append(toks[i])
                i += 1
        return out

    return apply


def in_place(rev, pairs, paths):
    rename = token_renamer(pairs)
    differ = 0
    for path in paths:
        old = show(rev, path)
        new = open(path, encoding="utf-8").read()
        a, b = rename(tokens(old)), tokens(new)
        old_lines, new_lines = old.splitlines(), new.splitlines()
        sm = difflib.SequenceMatcher(None, [t for t, _ in a], [t for t, _ in b], autojunk=False)
        for op, i1, i2, j1, j2 in sm.get_opcodes():
            if op == "equal":
                continue
            differ += 1
            (oa, ob), (na, nb) = span(a, i1, i2), span(b, j1, j2)
            print(f"{path}: {op} old {oa}-{ob} / new {na}-{nb}")
            for line in old_lines[oa - 1:ob]:
                print(f"  - {line.strip()}")
            for line in new_lines[na - 1:nb]:
                print(f"  + {line.strip()}")
    print(f"{len(paths)} files, {differ} differing spans")
    return differ


def show(rev, path):
    return subprocess.run(["git", "show", f"{rev}:{path}"], check=True, capture_output=True, text=True).stdout


def main():
    args = sys.argv[1:]
    rev, pairs, inplace = "HEAD", [], False
    while args[:1] and args[0].startswith("--"):
        flag = args.pop(0)
        if flag == "--rev":
            rev = args.pop(0)
        elif flag == "--rename":
            pairs.append(args.pop(0))
        elif flag == "--renames":
            with open(args.pop(0), encoding="utf-8") as f:
                pairs += [s for s in (l.strip() for l in f) if s and not s.startswith("#")]
        elif flag == "--in-place":
            inplace = True
        else:
            sys.exit(f"unknown option {flag}\n{__doc__}")
    for p in pairs:
        if "=" not in p or not p.split("=", 1)[0]:
            sys.exit(f"a rename is OLD=NEW: {p!r}")
    pairs = [p.split("=", 1) for p in pairs]
    if inplace:
        if not args:
            sys.exit(__doc__)
        sys.exit(1 if in_place(rev, pairs, args) else 0)
    rename = renamer(pairs)
    if len(args) < 2:
        sys.exit(__doc__)
    olds = [rename(show(rev, p)) for p in args[0].split(",")]
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
