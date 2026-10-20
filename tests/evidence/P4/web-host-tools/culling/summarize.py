#!/usr/bin/env python3
"""Medians of interleave.sh's rounds: Renderer::render (cpu=) and frame
(frame=, render plus the GPU wait) per instance count, path and scene, and
the median of the per-round gpu/cpu ratios."""
import re, sys, statistics as st
from collections import defaultdict
runs = defaultdict(dict)  # (file, round, side, scene) -> variant -> (cpu, frame, instances)
loads = []
for path in sys.argv[1:]:
    for line in open(path):
        m = re.match(r"round=(\d+) load=(.*)", line)
        if m:
            loads.append(float(m.group(2).split()[0])); continue
        m = re.match(r"round=(\d+) side=(\d+) variant=(\w+) (city-\w+): .*instances=(\d+).* frame=([\d.]+)ms cpu=([\d.]+)ms", line)
        if not m: continue
        r, side, v, scene, inst, frame, cpu = m.groups()
        runs[(path, int(r), int(side), scene)][v] = (float(cpu), float(frame), int(inst) - 1)
table = defaultdict(lambda: defaultdict(list))
for (p, r, side, scene), vs in runs.items():
    if "cpu" not in vs or "gpu" not in vs: continue
    inst = vs["cpu"][2]
    t = table[(scene, inst)]
    for k, i in (("render", 0), ("frame", 1)):
        t[f"cpu_{k}"].append(vs["cpu"][i]); t[f"gpu_{k}"].append(vs["gpu"][i])
        t[f"ratio_{k}"].append(vs["gpu"][i] / vs["cpu"][i])
print(f"rounds' load averages (1 min): {min(loads):.0f} to {max(loads):.0f}, median {st.median(loads):.0f}")
print("| Scene | Instances | n | render cpu | render gpu | gpu/cpu | frame cpu | frame gpu | gpu/cpu |")
print("|---|---|---|---|---|---|---|---|---|")
for (scene, inst) in sorted(table, key=lambda k: (k[0] != "city-street", k[1])):
    t = table[(scene, inst)]
    md = lambda k: st.median(t[k])
    print(f"| {scene} | {inst:,} | {len(t['ratio_render'])} | {md('cpu_render'):.3f} | {md('gpu_render'):.3f} | {md('ratio_render'):.2f} | {md('cpu_frame'):.3f} | {md('gpu_frame'):.3f} | {md('ratio_frame'):.2f} |")
