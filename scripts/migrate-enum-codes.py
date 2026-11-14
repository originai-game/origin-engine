#!/usr/bin/env python3
"""Run from the repository root: the constants that became enums, as their variants.

og.Collider.shape and ShapeDesc.shape (ColliderShape), og.SoftBody.shape
(SoftBodyShape) and the `simulation` of og.Cloth, og.Garment, og.SoftBody and
og.Fluid3D (Simulation) took string enums; the SHAPE_*, SOFT_* and
SIMULATION_* constants are gone. In TypeScript and Markdown a constant
becomes its variant's string (and leaves the engine's import list; in
Markdown prose it is named as code); a collider's numeric `shape` in the
files named below becomes its name. Rust's
own og_physics::SHAPE_* codes stay. Generated files are not touched: run
`og api gen` after it.
"""
from pathlib import Path
import re
import subprocess

COLLIDER = ["box", "sphere", "capsule", "cylinder", "convex", "trimesh", "heightfield", "diamond", "hexagon"]
CONSTANTS = {f"SHAPE_{name.upper()}": name for name in COLLIDER}
CONSTANTS |= {"SOFT_SPHERE": "sphere", "SOFT_BOX": "box", "SOFT_MESH": "mesh"}
CONSTANTS |= {"SIMULATION_GAMEPLAY": "gameplay", "SIMULATION_VISUAL": "visual"}
# og.Joint.kind (JointType) and motor (JointMotor), og.Vehicle.drive
# (VehicleDrive) and model (VehicleModel).
CONSTANTS |= {f"JOINT_{name.upper()}": name for name in ["fixed", "revolute", "prismatic", "spring", "rope", "ball"]}
CONSTANTS |= {f"MOTOR_{name.upper()}": name for name in ["none", "velocity", "position"]}
CONSTANTS |= {f"DRIVE_{name.upper()}": name for name in ["rear", "front", "all"]}
CONSTANTS |= {"VEHICLE_ARCADE": "arcade", "VEHICLE_DRIVETRAIN": "drivetrain"}

# Where a collider's shape is written as its code.
NUMERIC = [
    "crates/og-api/guides/world.md",
    "tests/fixtures/motion-feet/src/main.ts",
    "packages/engine/tests/fixtures/saves/game.ts",
    "packages/engine/tests/support/yard-piers.ts",
    "crates/og-runtime/examples/flight_plan.rs",
    "crates/og-runtime/tests",
]
SKIP = ("node_modules/", "target/", "dist/", "packages/knowledge/wiki/", "packages/engine/types/",
        "packages/engine/schema/", "/generated/", "changelog/", "packages/engine/changelog.json",
        "scripts/migrate-enum-codes.py")

name = re.compile(r"\b(?:og\.)?(" + "|".join(CONSTANTS) + r")\b")
imports = re.compile(r"(import\s*(?:type\s*)?\{)([^}]*)(\}\s*from\s*\"@origingame/engine\")")
code = re.compile(r"(\"?\bshape\"?\s*:\s*)([0-8])\b")


def without_constants(match: re.Match) -> str:
    if not any(n.strip() in CONSTANTS for n in match[2].split(",")):
        return match[0]
    names = [n for n in match[2].split(",") if n.strip() and n.strip() not in CONSTANTS]
    multiline = "\n" in match[2]
    body = ",".join(names)
    if multiline and not body.rstrip(" ").endswith("\n"):
        body = body.rstrip() + ",\n"
    elif not multiline:
        body = " " + body.strip() + " "
    return match[1] + body + match[3]


def tracked(suffixes: tuple) -> list:
    out = subprocess.run(["git", "ls-files"], capture_output=True, text=True, check=True).stdout
    return [p for p in out.splitlines() if p.endswith(suffixes) and not any(s in p for s in SKIP)]


for file in tracked((".ts", ".tsx", ".md", ".rs")):
    path = Path(file)
    before = path.read_text()
    text = before
    if not file.endswith(".rs"):
        text = imports.sub(without_constants, text)
        # In prose a variant is named as code; in code it is the string.
        quote = "`" if file.endswith(".md") else '"'
        text = name.sub(lambda m: quote + CONSTANTS[m[1]] + quote, text)
    if any(file == n or file.startswith(n + "/") for n in NUMERIC):
        text = code.sub(lambda m: m[1] + '"' + COLLIDER[int(m[2])] + '"', text)
    if text != before:
        path.write_text(text)
        print(path)
