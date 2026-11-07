#!/usr/bin/env python3
"""Prepare a `kind vfx` effect pack for the asset library (its contract:
docs/vfx-packs.md).

    python3 scripts/vfx-pack.py manifest <pack> --spec <spec.json> [--revision SHA]
    python3 scripts/vfx-pack.py preview <pack> [--og target/debug/og] [--cover <effect.ogt>]

`manifest` writes the pack's assets.json (every installed file with its
sha256 and provenance, every top-level effect with its timing) and the
catalog fields of extension.json (minEngine, tags, keywords) from a spec:

    {
      "minEngine": "0.8.0",
      "style": "pixel", "dim": "2d", "keywords": ["pixel"],
      "files": [                      # first match wins, globs over pack paths
        {"glob": "*.json", "licence": "custom", "licenceText": "...",
         "source": {"url": "https://..."}, "credit": "...", "derived": "copied"}
      ],
      "effects": {"<path.ogt>": {"title": "...", "category": "fire", "element": "fire",
                                  "keywords": ["..."]}}
    }

`preview` installs the pack into a scratch game, plays every effect in one
`og run` on this machine's GPU and writes, per effect, preview/<effect>.png
(six 960x540 frames in time order, the impact frame among them, 3 x 2),
preview/<effect>.cost.json (the effect's peak GPU milliseconds over an
empty frame: the 90th percentile of its frames from the run's trace, and the render dump's draws and particle
slots at the six frames) and preview/cover.png. Timing (duration, loop,
impact) comes from a first headless run that watches each effect's events,
and is written back into assets.json's effects.
"""

import argparse
import fnmatch
import hashlib
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path

W, H = 960, 540
TICK = 60
GAP = 0.6          # seconds of empty scene between effects (the GPU baseline)
LOOP_SHOW = 2.0    # seconds a looping effect is shown
WARM = 5           # ticks after a spawn its cost is not counted (one-off uploads)
FRACTIONS = [0.04, 0.15, 0.3, 0.5, 0.7, 0.9]


def ogt_type(path):
    for line in path.read_text(errors="replace").splitlines():
        if line.startswith("type "):
            return line.split()[1]
    return None


def effects_of(pack):
    """The top-level effects: `type vfx` files no other vfx names, and every vfx2d."""
    found = {}
    for p in sorted(pack.rglob("*.ogt")):
        rel = p.relative_to(pack).as_posix()
        if rel.startswith("preview/"):
            continue
        t = ogt_type(p)
        if t in ("vfx", "vfx2d"):
            found[rel] = t
    return found


def installed_files(pack):
    out = []
    for p in sorted(pack.rglob("*")):
        rel = p.relative_to(pack).as_posix()
        if p.is_symlink():
            sys.exit(f"{rel}: symlinks are refused")
        if not p.is_file() or rel == "assets.json" or rel.startswith("preview/"):
            continue
        if any(part.startswith(".") for part in rel.split("/")):
            continue
        out.append(rel)
    return out


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def manifest(args):
    pack = Path(args.pack)
    spec = json.loads(Path(args.spec).read_text())
    ext = json.loads((pack / "extension.json").read_text())
    effects = effects_of(pack)
    meta = spec.get("effects", {})
    cats = sorted({m.get("category") for m in meta.values() if m.get("category")})
    elems = sorted({m.get("element") for m in meta.values() if m.get("element")})
    ext["minEngine"] = spec["minEngine"]
    ext["tags"] = {"style": spec["style"], "dim": spec["dim"], "category": cats, "element": elems}
    ext["keywords"] = sorted({k.lower() for k in spec.get("keywords", [])})
    (pack / "extension.json").write_text(json.dumps(ext, indent=2) + "\n")

    files = []
    for rel in installed_files(pack):
        rule = next((r for r in spec["files"] if fnmatch.fnmatch(rel, r["glob"])), None)
        if rule is None:
            sys.exit(f"{rel}: no provenance rule in the spec")
        entry = {"path": rel, "sha256": sha256(pack / rel)}
        entry.update({k: v for k, v in rule.items() if k != "glob"})
        files.append(entry)
    old = {}
    if (pack / "assets.json").exists():
        old = {e["path"]: e for e in json.loads((pack / "assets.json").read_text()).get("effects", [])}
    out_effects = []
    for rel, t in effects.items():
        m = meta.get(rel, {})
        e = {
            "path": rel,
            "type": t,
            "title": m.get("title") or Path(rel).stem.replace("-", " ").replace("_", " ").title(),
            # Timing is measured by `preview`; kept from an earlier run.
            "loop": old.get(rel, {}).get("loop", False),
            "durationSeconds": old.get(rel, {}).get("durationSeconds", 0),
            "impact": old.get(rel, {}).get("impact", False),
            "category": m.get("category", ""),
            "keywords": sorted({k.lower() for k in m.get("keywords", [])}),
        }
        # Optional in the contract: omitted, never empty.
        if m.get("element"):
            e["element"] = m["element"]
        out_effects.append(e)
    doc = {
        "format": 1,
        "engine": {"version": spec["minEngine"], "revision": args.revision or git_head()},
        "files": files,
        "effects": out_effects,
    }
    (pack / "assets.json").write_text(json.dumps(doc, indent=2) + "\n")
    print(f"{pack}: {len(files)} files, {len(out_effects)} effects")


def git_head():
    root = Path(__file__).resolve().parent.parent
    return subprocess.run(["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True).stdout.strip()


MAIN_TS = r"""
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}
const DIM = "__DIM__";
const shown: number[] = [];
const log: { name: string; tick: number }[] = [];
const starts: { path: string; tick: number; duration: number; loop: boolean }[] = [];

function build(world: World): void {
  const cam = must(world.resource(og.Camera));
  cam.background = { r: 0.035, g: 0.04, b: 0.055 };
  const rs = must(world.resource(og.RenderSettings));
  rs.bloom = true;
  if (DIM === "2d") {
    cam.orthographic = true;
    cam.height = 10;
    cam.position = { x: 0, y: 0, z: 10 };
    return;
  }
  rs.environmentIntensity = 0.15;
  const box = must(og.render.boxGeometry(world, {}));
  const ground = must(world.spawn({ position: { x: 0, y: -0.25, z: 0 } }));
  must(world.set(ground, og.Transform, { scale: { x: 30, y: 0.5, z: 30 } }));
  must(world.insert(ground, og.Mesh, { geometry: box, material: must(og.render.standardMaterial(world, { color: { r: 0.12, g: 0.12, b: 0.13 }, roughness: 0.85 })) }));
  const moon = must(world.spawn({ quaternion: og.transform.lookAt({ x: 0, y: 0, z: 0 }, { x: -0.3, y: -0.6, z: -0.6 }) }));
  must(world.insert(moon, og.DirectionalLight, { color: { r: 0.55, g: 0.6, b: 0.9 }, intensity: 0.25, castShadow: true }));
  cam.position = { x: 0, y: 2.2, z: 5.2 };
  cam.quaternion = og.transform.lookAt(cam.position, { x: 0, y: 0.8, z: 0 });
  cam.fov = 50;
}

function clear(world: World): void {
  for (const e of shown.splice(0)) world.despawn(e);
}

function play(world: World, path: string): void {
  clear(world);
  if (path.endsWith(".ogt") && __VFX2D__.includes(path)) {
    const info = must(og.load.flipbook(world, path));
    const clip = info.clips[0]!;
    // Whole screen pixels per art pixel, the frame filling at most 80% of
    // the view (Camera.height 10 m over __H__ px).
    const k = Math.max(1, Math.floor(Math.min((0.8 * __H__) / info.frameHeight, (0.8 * __W__) / info.frameWidth)));
    const m = 10 / __H__;
    const e = must(world.spawn({ position: { x: 0, y: 0, z: 0 } }));
    must(world.insert(e, og.Sprite, { texture: info.texture, width: info.frameWidth * k * m, height: info.frameHeight * k * m }));
    must(world.insert(e, og.Flipbook, { sheet: info.sheet, clip: clip.clip }));
    shown.push(e);
    starts.push({ path, tick: world.tick, duration: clip.duration, loop: clip.mode !== "once" });
    return;
  }
  const asset = must(og.load.vfx(world, path));
  const info = must(og.vfx.info(world, asset));
  shown.push(must(og.vfx.spawn(world, { asset, position: { x: 0, y: 0, z: 0 } })));
  starts.push({ path, tick: world.tick, duration: info.duration, loop: info.loop });
}

export default og.game({
  tick: __TICK__,
  seed: 1,
  systems: [
    defineSystem("Watch", { reads: [og.VfxEvent, og.FlipbookEvent], writes: [] }, (_q, _t, world) => {
      for (const e of world.events(og.VfxEvent)) log.push({ name: e.name, tick: world.tick });
      for (const e of world.events(og.FlipbookEvent)) log.push({ name: e.name, tick: world.tick });
    }),
  ],
  start: build,
  // Pixel art keeps its pixels: no temporal antialiasing to soften them.
  render: DIM === "2d" ? { antialias: "off" } : {},
  commands: {
    play: (world, args) => play(world, String(args[0])),
    clear: (world) => clear(world),
  },
  facts: () => ({ starts, log }),
});
"""


def scratch_game(pack, og, dim, vfx2d):
    root = Path(tempfile.mkdtemp(prefix="vfx-preview-"))
    game = root / "game"
    engine = Path(__file__).resolve().parent.parent
    subprocess.run(["bun", str(engine / "packages/create/bin/create.ts"), str(game)], check=True, capture_output=True)
    (game / "assets").mkdir(exist_ok=True)
    main = MAIN_TS.replace("__DIM__", dim).replace("__TICK__", str(TICK)).replace("__W__", str(W)).replace("__H__", str(H))
    main = main.replace("__VFX2D__", json.dumps(vfx2d))
    (game / "src/main.ts").write_text(main)
    name = json.loads((pack / "extension.json").read_text())["name"]
    r = subprocess.run([og, "vfx", "add", str(pack), str(game), "--any-engine"], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stdout + r.stderr)
    r = subprocess.run([og, "cook", str(game)], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stdout + r.stderr)
    return root, game, name


def run(og, game, commands, extra, seconds):
    cmd = [og, "run", str(game), "--size", f"{W}x{H}", "--json", "--no-check",
           "--commands", ";".join(commands + [f"{seconds:.3f}:quit"])] + extra
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=1800)
    last = [l for l in r.stdout.splitlines() if l.startswith("{")]
    if r.returncode != 0 or not last:
        sys.exit(f"og run failed ({r.returncode}):\n{r.stdout[-3000:]}\n{r.stderr[-3000:]}")
    return json.loads(last[-1])


def gpu_frames(trace_path):
    """Per rendered frame: (tick, total GPU ms, {pass: ms})."""
    d = json.loads(Path(trace_path).read_text())
    ev = d["traceEvents"] if isinstance(d, dict) else d
    renders = sorted((e for e in ev if e.get("ph") == "X" and e.get("cat") == "render"
                      and e["name"].startswith("frame ")), key=lambda e: e["ts"])
    ticks = [e["args"]["tick"] for e in renders]
    frames = {}
    for e in ev:
        if e.get("cat") != "gpu" or e.get("ph") != "X":
            continue
        f = e["args"]["frame"]
        slot = frames.setdefault(f, {"total": 0.0, "passes": {}})
        if e["name"].startswith("gpu frame"):
            slot["total"] = e["dur"] / 1000.0
        else:
            slot["passes"][e["name"]] = slot["passes"].get(e["name"], 0.0) + e["dur"] / 1000.0
    out = []
    for f, slot in sorted(frames.items()):
        if f < len(ticks):
            out.append((ticks[f], slot["total"], slot["passes"]))
    return out


def device_name():
    if platform.system() == "Darwin":
        r = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True)
        return (r.stdout.strip() + " GPU (Metal)").strip()
    return platform.processor() or "unknown"


def preview(args):
    from PIL import Image

    pack = Path(args.pack).resolve()
    og = str(Path(args.og).resolve())
    ext = json.loads((pack / "extension.json").read_text())
    assets = json.loads((pack / "assets.json").read_text())
    dim = ext.get("tags", {}).get("dim", "3d")
    effects = [e["path"] for e in assets["effects"]]
    vfx2d_ids = [f"vfx/{ext['name']}/{e['path']}" for e in assets["effects"] if e["type"] == "vfx2d"]
    root, game, name = scratch_game(pack, og, dim, vfx2d_ids)
    ids = [f"vfx/{name}/{p}" for p in effects]
    try:
        # Pass 1, headless: each effect's length, loop and impact tick.
        t, cmds = 0.5, []
        for i in ids:
            cmds.append(f"{t:.3f}:play:{i}")
            t += 4.0
        cmds.append(f"{t:.3f}:clear")
        facts = run(og, game, cmds, ["--headless"], t + 0.2)["facts"]
        timing = {}
        for s in facts["starts"]:
            end = s["tick"] + (LOOP_SHOW if s["loop"] else min(max(s["duration"], 0.2), 3.9)) * TICK
            impact = next((l["tick"] for l in facts["log"] if l["name"] == "impact" and s["tick"] <= l["tick"] <= s["tick"] + 4 * TICK), None)
            timing[s["path"]] = {
                "duration": s["duration"], "loop": s["loop"],
                "impact": None if impact is None else (impact - s["tick"]) / TICK,
                "show": (end - s["tick"]) / TICK,
            }
        missing = [i for i in ids if i not in timing]
        if missing and not args.skip_failed:
            sys.exit(f"never started: {missing}")
        if missing:
            # Reviewing a pack that is not finished: sheets for the rest.
            print(f"never started, skipped: {missing}", file=sys.stderr)
            keep = [(i, rel) for i, rel in zip(ids, effects) if i not in missing]
            ids, effects = [k[0] for k in keep], [k[1] for k in keep]

        # Pass 2, on the GPU: shots, render dumps and the trace.
        t, cmds, plan = 0.5, [], {}
        for i in ids:
            tm = timing[i]
            start = t + GAP
            times = [f * tm["show"] for f in FRACTIONS]
            if tm["impact"] is not None:
                k = min(range(len(times)), key=lambda j: abs(times[j] - tm["impact"]))
                times[k] = tm["impact"] + 1 / TICK
                times.sort()
            cmds.append(f"{start:.3f}:play:{i}")
            shots = []
            for n, dt in enumerate(times):
                at = start + max(dt, 1 / TICK)
                stem = f"{len(plan):03d}-{n}"
                cmds.append(f"{at:.3f}:screenshot:{stem}.png")
                cmds.append(f"{at:.3f}:dump_render:{stem}.json")
                shots.append(stem)
            end = start + tm["show"]
            cmds.append(f"{end:.3f}:clear")
            plan[i] = {"start": start, "end": end, "shots": shots, "gap": (t, start)}
            t = end
        trace = root / "trace.json"
        run(og, game, cmds, ["--trace", str(trace), "--out", str(root / "out")], t + 0.3)
        frames = gpu_frames(trace)

        out = pack / "preview"
        if out.exists():
            shutil.rmtree(out)
        out.mkdir()
        gaps = [f for f in frames if any(p["gap"][0] * TICK + 6 <= f[0] < p["gap"][1] * TICK for p in plan.values())]
        base_total = statistics.median([f[1] for f in gaps]) if gaps else 0.0
        base_main = statistics.median([f[2].get("main", 0.0) for f in gaps]) if gaps else 0.0
        device = device_name()
        rows = []
        for i, rel in zip(ids, effects):
            p = plan[i]
            sheet = Image.new("RGB", (W * 3, H * 2))
            draws = particles = 0
            for n, stem in enumerate(p["shots"]):
                im = Image.open(root / "out" / f"{stem}.png").convert("RGB")
                sheet.paste(im, ((n % 3) * W, (n // 3) * H))
                dump = json.loads((root / "out" / f"{stem}.json").read_text())
                draws = max(draws, dump.get("draws", 0))
                pt = dump.get("particles", {})
                particles = max(particles, pt.get("capacity", 0) + pt.get("cpu", 0))
            target = out / (rel[:-4] + ".png")
            target.parent.mkdir(parents=True, exist_ok=True)
            sheet.save(target, optimize=True)
            # From the fifth tick: the first frames upload the effect's
            # textures and make its pipelines, once, not per frame.
            window = [f for f in frames if p["start"] * TICK + WARM <= f[0] <= p["end"] * TICK]
            # The 90th percentile over the effect's frames: its heavy moment
            # without one frame's scheduling noise.
            def p90(xs, default):
                xs = sorted(xs)
                return xs[min(len(xs) - 1, int(0.9 * len(xs)))] if xs else default
            cost = {
                "format": 1,
                "engine": assets["engine"],
                "device": device,
                "peakFrameMs": {
                    "total": round(max(0.0, p90([f[1] for f in window], base_total) - base_total), 3),
                    "sim": round(p90([f[2].get("particles", 0.0) for f in window], 0.0), 3),
                    # The main pass draws the blended surfaces, sprites and
                    # particles after the opaque scene: its growth is theirs.
                    "transparent": round(max(0.0, p90([f[2].get("main", 0.0) for f in window], base_main) - base_main), 3),
                },
                "peakParticles": particles,
                "draws": draws,
            }
            (out / (rel[:-4] + ".cost.json")).write_text(json.dumps(cost, indent=2) + "\n")
            rows.append((rel, cost))
        if f"vfx/{name}/{args.cover}" not in timing:
            args.cover = None
        cover = args.cover or next((e["path"] for e in assets["effects"] if timing[f"vfx/{name}/{e['path']}"]["impact"] is not None), effects[0])
        cp = plan[f"vfx/{name}/{cover}"]
        tm = timing[f"vfx/{name}/{cover}"]
        pick = cp["shots"][2] if tm["impact"] is None else cp["shots"][min(range(6), key=lambda n: abs(FRACTIONS[n] * tm["show"] - tm["impact"]))]
        Image.open(root / "out" / f"{pick}.png").convert("RGB").save(out / "cover.png", optimize=True)

        for e in assets["effects"]:
            tm = timing.get(f"vfx/{name}/{e['path']}")
            if tm is None:
                continue
            e["loop"] = tm["loop"]
            e["durationSeconds"] = round(tm["duration"], 3)
            e["impact"] = tm["impact"] is not None
        (pack / "assets.json").write_text(json.dumps(assets, indent=2) + "\n")
        print(f"baseline GPU frame {base_total:.3f} ms (main {base_main:.3f} ms) on {device}")
        for rel, c in rows:
            m = c["peakFrameMs"]
            print(f"{rel}\t+{m['total']:.3f} ms\tsim {m['sim']:.3f}\ttransparent +{m['transparent']:.3f}\tparticles {c['peakParticles']}\tdraws {c['draws']}")
    finally:
        if not args.keep:
            shutil.rmtree(root, ignore_errors=True)
        else:
            print(f"scratch kept: {root}")


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("manifest")
    m.add_argument("pack")
    m.add_argument("--spec", required=True)
    m.add_argument("--revision")
    p = sub.add_parser("preview")
    p.add_argument("pack")
    p.add_argument("--og", default=str(Path(__file__).resolve().parent.parent / "target/debug/og"))
    p.add_argument("--cover")
    p.add_argument("--keep", action="store_true")
    p.add_argument("--skip-failed", action="store_true",
                   help="render the effects that load and list the rest, instead of refusing")
    args = ap.parse_args()
    manifest(args) if args.cmd == "manifest" else preview(args)


if __name__ == "__main__":
    main()
