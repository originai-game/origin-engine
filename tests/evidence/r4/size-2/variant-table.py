# Per-crate table of one variant, before and after:
#   python3 variant-table.py BEFORE.twiggy.txt BEFORE.report.txt AFTER.twiggy.txt AFTER.report.txt
# twiggy.txt: tests/evidence/debt2/size/twiggy-crates.ts over DIR/named.wasm;
# report.txt: scripts/wasm-size-report.sh's output (its per-crate "brotli
# alone"). Crates under 20,000 B twiggy in both are summed into one row.
import re, sys
def tw(f):
    m = {}; tot = 0
    for l in open(f):
        t = re.match(r'^twiggy top.*: ([\d,]+) B', l)
        if t: tot = int(t.group(1).replace(',', ''))
        r = re.match(r'^\| (.+?) \| (\d+) \| ([\d,]+) \|', l)
        if r: m[r.group(1)] = int(r.group(3).replace(',', ''))
    return m, tot
def br(f):
    m = {}
    for l in open(f):
        r = re.match(r'^\| (.+?) \| [\d,]+ \| [\d,]+ \| [\d.]+ % \| ([\d,]+) \|', l)
        if r: m[r.group(1)] = int(r.group(2).replace(',', ''))
    return m
t0, T0 = tw(sys.argv[1]); b0 = br(sys.argv[2]); t1, T1 = tw(sys.argv[3]); b1 = br(sys.argv[4])
f = lambda n: f"{n:,}"
d = lambda n: "" if n == 0 else f" ({'+' if n > 0 else '−'}{abs(n):,})"
names = sorted(set(t0) | set(t1), key=lambda k: -max(t0.get(k, 0), t1.get(k, 0)))
print("| Crate | before twiggy B | before brotli alone | after twiggy B | after brotli alone |")
print("|---|---:|---:|---:|---:|")
rest = [0, 0, 0, 0]
for k in names:
    v = [t0.get(k, 0), b0.get(k, 0), t1.get(k, 0), b1.get(k, 0)]
    if max(v[0], v[2]) < 20000:
        rest = [a + b for a, b in zip(rest, v)]; continue
    print(f"| {k} | {f(v[0])} | {f(v[1])} | {f(v[2])}{d(v[2]-v[0])} | {f(v[3])}{d(v[3]-v[1])} |")
print(f"| (the rest, under 20,000 B) | {f(rest[0])} | {f(rest[1])} | {f(rest[2])}{d(rest[2]-rest[0])} | {f(rest[3])}{d(rest[3]-rest[1])} |")
print(f"| **module** | **{f(T0)}** | | **{f(T1)}**{d(T1-T0)} | |")
