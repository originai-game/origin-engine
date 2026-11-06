#!/usr/bin/env python3
"""Measure real og commands on disposable copies; never clear shared caches.

Run from the repository: python3 tests/evidence/og-session/measure.py
Use --only NAME to resume a command, --repeat N for cold/warm pairs.
Use --bounded for a two-screen UI matrix, retaining all viewport variants.
GPU commands acquire one machine GPU slot per invocation, outside the timer.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[3]
WORK = ROOT / ".amp/in/og-session-baseline"
OG = ROOT / "target/debug/og"
PACKAGE = ROOT / "packages/engine"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def invoke(command, directory, record):
    env = dict(os.environ, OG_ENGINE_PACKAGE=str(PACKAGE))
    record["load_at_start"] = os.getloadavg()
    start = time.perf_counter()
    with (directory / "stdout").open("w") as stdout, (directory / "stderr").open("w") as stderr:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    record.update(ms=(time.perf_counter() - start) * 1000, exit=result.returncode)
    record["load_after"] = os.getloadavg()
    record["files"] = {
        str(p.relative_to(directory)): {"bytes": p.stat().st_size, "sha256": digest(p)}
        for p in directory.rglob("*") if p.is_file() and p.name != "record.json"
    }
    if command[1] == "test":
        test_out = Path(command[2]) / "og-out/test"
        record["test_outputs"] = {
            str(p.relative_to(test_out)): {"bytes": p.stat().st_size, "sha256": digest(p)}
            for p in test_out.rglob("*") if p.is_file() and p.suffix in [".json", ".png"]
        }
    (directory / "record.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps({k: v for k, v in record.items() if k not in ["files", "test_outputs"]}), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--only")
    parser.add_argument("--project", choices=["game", "meridian-loop"])
    parser.add_argument("--repeat", type=int, default=5)
    parser.add_argument("--bounded", action="store_true")
    parser.add_argument("--locked", type=Path, help="argfile used by the lock wrapper")
    args = parser.parse_args()
    if (WORK / "stop-after-current").exists():
        raise SystemExit("measurement stopped at a command boundary")
    if args.locked:
        data = json.loads(args.locked.read_text())
        invoke(data["command"], Path(data["directory"]), data["record"])
        return
    exclude = Path(subprocess.check_output(["git", "rev-parse", "--git-path", "info/exclude"], cwd=ROOT, text=True).strip())
    if "/.amp/in/" not in exclude.read_text().splitlines():
        raise SystemExit("Add /.amp/in/ to the repository-local exclude before running")
    WORK.mkdir(parents=True, exist_ok=True)
    # Example tsconfigs address ../../packages; preserve that layout without
    # changing compiler options or the source being measured.
    if not (WORK / "packages").exists():
        (WORK / "packages").symlink_to(ROOT / "packages", target_is_directory=True)
    projects = {"game": "templates/game", "meridian-loop": "examples/meridian-loop"}
    metadata = {
        "revision": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "og_sha256": digest(OG),
        "addon_sha256": digest(PACKAGE / "native/origin-engine.darwin-arm64.node"),
        "bun": subprocess.check_output(["bun", "--version"], text=True).strip(),
        "profile": (PACKAGE / "native/build-profile").read_text().strip(),
        "cpus": os.cpu_count(), "load": os.getloadavg(),
    }
    (WORK / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    for project, source in projects.items():
        if args.project and project != args.project:
            continue
        fixture = WORK / "fixtures" / (project + ("-bounded" if args.bounded else ""))
        if not fixture.exists():
            shutil.copytree(ROOT / source, fixture, ignore=shutil.ignore_patterns(".og", "cooked", "og-out", "dist", "node_modules"))
            if args.bounded:
                second = 'screen "started" "0.1:key_down:Enter;0.2:key_up:Enter"' if project == "game" else 'screen "settings" "0.1:open:settings"'
                with (fixture / "project.ogt").open("a") as manifest:
                    manifest.write('\nui {\n  screen "title" "0:resume"\n  ' + second + '\n}\n')
        installed = fixture / "node_modules/@origingame/engine"
        if not installed.exists():
            installed.parent.mkdir(parents=True, exist_ok=True)
            installed.symlink_to(PACKAGE, target_is_directory=True)
        images = sorted(p for p in (ROOT / source / "references").rglob("*.png") if not p.name.endswith(".sheet.png"))
        image = str(images[0]) if images else "missing-reference.png"
        # The short run is identical on both projects, with a real capture and dump.
        script = "0.2:dump_world:world.json;0.2:screenshot:frame.png;0.3:quit"
        cases = {
            "cook": (["cook", str(fixture), "--json"], False),
            "check": (["check", str(fixture), "--json"], False),
            "run": (["run", str(fixture), "--headless", "--commands", script, "--json"], True),
            "test": (["test", str(fixture), "--json"], True),
            "ui-check": (["ui", "check", str(fixture), "--json"], True),
            "ui-preview": (["ui", "preview", str(fixture), "--json"], True),
            "build": (["build", str(fixture), "--json"], False),
            "compare": (["compare", image, image, "--json"], False),
            "frame": (["frame", "measure", image, "--metric", "contrast,highpass,mean,ramp", "--json"], False),
            "api": (["api", "find", "camera"], False),
            "trace": (["run", str(fixture), "--headless", "--commands", script, "--json"], True),
        }
        for name, (command, gpu) in cases.items():
            if args.bounded and name not in ["ui-check", "ui-preview"]:
                continue
            if args.only and args.only != name:
                continue
            for repeat in range(args.repeat):
                for temperature in ["cold", "warm"]:
                    directory = WORK / project / (name + ("-bounded" if args.bounded else "")) / f"{repeat}-{temperature}"
                    if (directory / "record.json").exists():
                        continue
                    directory.mkdir(parents=True, exist_ok=True)
                    if temperature == "cold":
                        for generated in [".og", "cooked", "og-out", "dist"]:
                            shutil.rmtree(fixture / generated, ignore_errors=True)
                    call = [str(OG), *command]
                    if name in ["run", "ui-check", "ui-preview", "build", "trace"]:
                        call += ["--out", str(directory / "outputs")]
                    if name == "trace":
                        call += ["--trace", str(directory / "trace.json")]
                    if name == "compare":
                        call += ["--sheet", str(directory / "comparison.png")]
                    record = {"project": source, "command": call, "temperature": temperature, "repeat": repeat, "bounded": args.bounded, "gpu_slots": 1 if gpu else 0, "load_before": os.getloadavg()}
                    if gpu:
                        job = directory / "job.json"
                        job.write_text(json.dumps({"command": call, "directory": str(directory), "record": record}))
                        subprocess.run(["bash", "-c", 'source scripts/lock.sh; og_gpu_acquire og-session-baseline; "$@"', "measure", sys.executable, str(Path(__file__).resolve()), "--locked", str(job)], cwd=ROOT, check=True)
                    else:
                        invoke(call, directory, record)


if __name__ == "__main__":
    main()
