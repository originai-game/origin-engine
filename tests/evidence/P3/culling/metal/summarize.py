# Summarise the interleaved og-gfx rounds: per scene and variant the median and
# minimum of cpu= (ms of Renderer::render per frame), then, within each round,
# the ratio of one variant to another, and the median of those ratios.
#   python3 summarize.py rounds-*.txt
import re, sys, statistics as st
d, rounds = {}, {}
for i, path in enumerate(sys.argv[1:]):
    for l in open(path):
        m = re.search(r"round=(\d+) variant=(\w+) ([a-z-]+):", l)
        if not m:
            continue
        cpu = float(re.search(r" cpu=([0-9.]+)ms", l).group(1))
        r, v, s = (i, int(m.group(1))), m.group(2), m.group(3)
        d.setdefault((s, v), []).append(cpu)
        rounds.setdefault((s, r), {})[v] = cpu
for (s, v), xs in sorted(d.items()):
    print(f"{s:12} {v:7} n={len(xs):3} median {st.median(xs):.2f} min {min(xs):.2f} ms")
for s in ("city-street", "city-aerial"):
    rs = [x for (sc, _), x in rounds.items() if sc == s and len(x) == 3]
    for a, b in (("gpu", "before"), ("cpu", "before"), ("gpu", "cpu")):
        print(f"{s:12} {a}/{b} median of {len(rs)} paired ratios {st.median(x[a] / x[b] for x in rs):.2f}")
