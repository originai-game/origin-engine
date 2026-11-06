#!/usr/bin/env python3
"""Summarize measure.py's records; failures remain visible, never dropped."""
import argparse
import collections
import json
from pathlib import Path
import statistics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--export", type=Path, help="save measurements and trace totals for review")
args = parser.parse_args()
root = Path(__file__).resolve().parents[3]
work = root / ".amp/in/og-session-baseline"
groups = collections.defaultdict(list)
measurements = []
for path in sorted(work.glob("*/*/*/record.json")):
    record = json.loads(path.read_text())
    groups[(path.parts[-4], path.parts[-3], record["temperature"])].append(record)
    outcomes = []
    for line in path.with_name("stdout").read_text().splitlines():
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict):
            outcomes.append({k: v for k, v in value.items() if k in [
                "event", "playtest", "ok", "checks", "frameChecks", "unchecked", "failures",
                "notes", "ms", "tick", "hash", "hashes", "tier", "warnings", "reason",
                "tool", "version", "diagnostics", "passed", "failed", "total", "cells",
            ]})
    measurements.append({"record": str(path.relative_to(work)), **record, "outcomes": outcomes})

print("| Project | Command | Cache | n | Median ms | Range ms | Exit codes |")
print("| --- | --- | --- | ---: | ---: | ---: | --- |")
for (project, command, cache), records in sorted(groups.items()):
    times = [r["ms"] for r in records]
    codes = dict(collections.Counter(r["exit"] for r in records))
    print(f"| {project} | {command} | {cache} | {len(times)} | {statistics.median(times):.1f} | {min(times):.1f}–{max(times):.1f} | {codes} |")

print("\n## Trace phases (instrumented runs only)\n")
print("Durations overlap: asset loads belong to engine create/start, and capture may belong to a tick. Do not sum these rows.\n")
print("| Project | Cache | Phase | Median ms | n |")
print("| --- | --- | --- | ---: | ---: |")
phases = collections.defaultdict(list)
for path in sorted(work.glob("*/trace/*/trace.json")):
    project, _, cache = path.parts[-4:-1]
    cache = cache.split("-", 1)[1]
    totals = collections.defaultdict(float)
    for event in json.loads(path.read_text())["traceEvents"]:
        if event["ph"] != "X":
            continue
        category = event.get("cat")
        if category in ["prepare", "host"] or event["name"] == "cook":
            totals[event["name"]] += event["dur"] / 1000
        elif category in ["tick", "render", "load", "capture"]:
            totals[category] += event["dur"] / 1000
    for phase, duration in totals.items():
        phases[(project, cache, phase)].append(duration)
for (project, cache, phase), values in sorted(phases.items()):
    print(f"| {project} | {cache} | {phase} | {statistics.median(values):.3f} | {len(values)} |")

if args.export:
    args.export.write_text(json.dumps({
        "metadata": json.loads((work / "metadata.json").read_text()),
        "measurements": measurements,
        "trace_phases": [
            {"project": project, "cache": cache, "phase": phase, "ms": values}
            for (project, cache, phase), values in sorted(phases.items())
        ],
    }, indent=2) + "\n")
