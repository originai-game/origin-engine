import re, sys
def parse(path):
    rows = {}
    for line in open(path):
        if not line.startswith("n="): continue
        parts = line.strip().split(" | ")
        n = int(re.match(r"n=(\d+)", parts[0]).group(1))
        d = {}
        for p in parts[1:]:
            name, rest = p.split(" ", 1)
            vals = dict(kv.split("=") for kv in rest.split())
            d[name] = (float(vals["p50"]), float(vals["mean"]))
        rows[n] = d
    return rows
def cell(v): return f"{v[0]:.0f} / {v[1]:.0f}"
def table(prefix, threads, variant="g1024"):
    data = {t: parse(f"out/{prefix}-t{t}.txt") for t in threads}
    ns = sorted(next(iter(data.values())).keys())
    hdr = "| bounds | serial | " + " | ".join(f"lanes, {t} threads" for t in threads) + " |"
    print(hdr); print("|---:|---:|" + "---:|" * len(threads))
    for n in ns:
        s = data[threads[-1]][n]["serial"]
        # serial: the median over the runs of the per-run figures
        ser = sorted(data[t][n]["serial"] for t in threads)
        serv = (sorted(x[0] for x in ser)[len(ser)//2], sorted(x[1] for x in ser)[len(ser)//2])
        cells = []
        for t in threads:
            v = data[t][n][variant]
            if variant == "g1024" and n < 2048: cells.append("(serial)")
            else: cells.append(f"{cell(v)} ({v[1]/data[t][n]['serial'][1]:.2f}×)")
        print(f"| {n:,} | {cell(serv)} | " + " | ".join(cells) + " |")
args = sys.argv[1:]
table(args[0], [int(x) for x in args[1].split(",")], args[2] if len(args) > 2 else "g1024")
