#!/usr/bin/env python3
"""Prepare a `kind vfx` effect pack for the asset library (its contract:
docs/vfx-packs.md).

    python3 scripts/vfx-pack.py manifest <pack> --spec <spec.json> [--revision SHA]
    python3 scripts/vfx-pack.py preview <pack> [--og target/debug/og] [--cover <effect.ogt>] [--only <a.ogt,b.ogt>]

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

`preview` installs the pack into a scratch game and has `og vfx preview`
(crates/og-cli/src/commands/vfx_preview.rs, which owns the stage, the
camera, the shots and the timing) play each effect there, then writes, per
effect, preview/<effect>.png (six 960x540 frames in time order, the impact
frame among them, 3 x 2, the camera placed by the effect's measured core),
preview/<effect>.cost.json (its peak GPU milliseconds over an empty frame,
its draws, particle slots and measured bounds) and preview/cover.png, and
its timing back into assets.json's effects:

1. the shots, `og vfx preview --no-cost` for CHUNK effects at a time, each
   call holding one GPU slot (scripts/lock.sh's og_gpu_acquire);
2. the cost, `og vfx preview --cost-only` per effect, holding every slot
   (og_gpu_acquire_all) for that call only.

The script takes its locks itself, each for one call, so a pack never holds
the GPU across a series and other lanes' GPU work runs between its
effects. Run it without a lock of your own.
"""

import argparse
import fnmatch
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

W, H = 960, 540
CHUNK = 6          # effects per `og vfx preview --no-cost` call (one GPU slot each)
LOCK_SH = Path(__file__).resolve().parent / "lock.sh"


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


def scratch_game(pack, og):
    root = Path(tempfile.mkdtemp(prefix="vfx-preview-"))
    game = root / "game"
    engine = Path(__file__).resolve().parent.parent
    subprocess.run(["bun", str(engine / "packages/create/bin/create.ts"), str(game)], check=True, capture_output=True)
    (game / "assets").mkdir(exist_ok=True)
    name = json.loads((pack / "extension.json").read_text())["name"]
    r = subprocess.run([og, "vfx", "add", str(pack), str(game), "--any-engine"], capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(r.stdout + r.stderr)
    return root, game, name


def og_preview(og, game, out, ids, flags, lock):
    """One `og vfx preview` of `ids`, holding one GPU slot (`""`,
    og_gpu_acquire) or every slot (`"_all"`, og_gpu_acquire_all) for this
    call only: scripts/lock.sh's own functions, taken by the shell that runs
    og and released by its exit trap when og ends. Its error text, or None."""
    suffix, label = lock
    cmd = ["bash", "-c", f'. "$0" && og_gpu_acquire{suffix} "$1" && shift && "$@"', str(LOCK_SH), label,
           og, "vfx", "preview", *ids, "--project", str(game), "--out", str(out), "--size", f"{W}x{H}", "--json", *flags]
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=3600)
    return None if r.returncode == 0 else (r.stdout[-2000:] + r.stderr[-2000:])


def device_name():
    if platform.system() == "Darwin":
        r = subprocess.run(["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True)
        return (r.stdout.strip() + " GPU (Metal)").strip()
    return platform.processor() or "unknown"


def preview(args):
    from PIL import Image

    pack = Path(args.pack).resolve()
    og = str(Path(args.og).resolve())
    assets = json.loads((pack / "assets.json").read_text())
    effects = [e["path"] for e in assets["effects"]]
    if args.only:
        # Re-render these effects alone; the pack's other sheets stay.
        only = set(args.only.split(","))
        if only - set(effects):
            sys.exit(f"not effects of the pack: {sorted(only - set(effects))}")
        effects = [e for e in effects if e in only]
    root, game, name = scratch_game(pack, og)
    work = root / "out"
    ids = {rel: f"vfx/{name}/{rel}" for rel in effects}
    try:
        # The shots, a few effects per call; a call that fails is taken apart
        # to find the effect that does not play.
        failed = {}
        for k in range(0, len(effects), CHUNK):
            chunk = effects[k:k + CHUNK]
            label = f"vfx-pack preview {name}: shots {k + 1}-{min(k + CHUNK, len(effects))} of {len(effects)}"
            if og_preview(og, game, work, [ids[r] for r in chunk], ["--no-cost"], ("", label)) is not None:
                for rel in chunk:
                    err = og_preview(og, game, work, [ids[rel]], ["--no-cost"], ("", f"vfx-pack preview {name}/{rel}"))
                    if err is not None:
                        failed[rel] = err
        if failed and not args.skip_failed:
            sys.exit("\n".join(f"{rel}: {err}" for rel, err in failed.items()))
        if failed:
            # Reviewing a pack that is not finished: sheets for the rest.
            print(f"do not play, skipped: {sorted(failed)}", file=sys.stderr)
            effects = [rel for rel in effects if rel not in failed]

        device = device_name()
        out = pack / "preview"
        if out.exists() and not args.only:
            shutil.rmtree(out)
        out.mkdir(exist_ok=True)
        rows, records = [], {}
        for rel in effects:
            # Every GPU slot for one effect's timed run at a time.
            err = og_preview(og, game, work, [ids[rel]], ["--cost-only"], ("_all", f"vfx-pack cost {name}/{rel}"))
            if err is not None:
                sys.exit(f"{rel}: {err}")
            record = json.loads((work / (ids[rel][:-4] + ".json")).read_text())
            records[rel] = record
            target = out / (rel[:-4] + ".png")
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(record["sheet"], target)
            cost = {
                "format": 1,
                "engine": assets["engine"],
                "device": device,
                "peakFrameMs": record["cost"]["peakFrameMs"],
                "peakParticles": record["peakParticles"],
                "draws": record["draws"],
                # Measured by the preview: null for an effect that draws
                # nothing over black (and for a vfx2d).
                "bounds": record["bounds"],
            }
            (out / (rel[:-4] + ".cost.json")).write_text(json.dumps(cost, indent=2) + "\n")
            rows.append((rel, cost))
            print(f"{rel}\tbaseline {record['cost']['baselineMs']:.3f} ms\t+{cost['peakFrameMs']['total']:.3f} ms", flush=True)

        # The cover: one frame of the named effect (else the first with an
        # impact, else the first), its impact's when it has one.
        if args.cover not in records:
            args.cover = None
        if args.only and args.cover is None:
            # The cover is one of the pack's effects, maybe not re-rendered.
            cover = None
        else:
            cover = args.cover or next((r for r in effects if records[r]["impactSeconds"] is not None), effects[0] if effects else None)
        if cover is not None:
            record = records[cover]
            n = 2
            if record["impactSeconds"] is not None:
                fractions = [0.04, 0.15, 0.3, 0.5, 0.7, 0.9]
                n = min(range(6), key=lambda n: abs(fractions[n] * record["shownSeconds"] - record["impactSeconds"]))
            sheet = Image.open(record["sheet"]).convert("RGB")
            sheet.crop(((n % 3) * W, (n // 3) * H, (n % 3 + 1) * W, (n // 3 + 1) * H)).save(out / "cover.png", optimize=True)

        for e in assets["effects"]:
            record = records.get(e["path"])
            if record is None:
                continue
            e["loop"] = record["loop"]
            e["durationSeconds"] = record["durationSeconds"]
            e["impact"] = record["impactSeconds"] is not None
        (pack / "assets.json").write_text(json.dumps(assets, indent=2) + "\n")
        print(f"on {device}, each effect over its own run's empty frames")
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
    p.add_argument("--only", help="comma-separated effect paths to re-render; the other sheets stay")
    p.add_argument("--skip-failed", action="store_true",
                   help="render the effects that load and list the rest, instead of refusing")
    args = ap.parse_args()
    manifest(args) if args.cmd == "manifest" else preview(args)


if __name__ == "__main__":
    main()
