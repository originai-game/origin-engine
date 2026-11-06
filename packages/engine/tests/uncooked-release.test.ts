// og.load.release on a real renderer (tests/fixtures/uncooked-release): the
// memory a 1024 px image and a 131,072-triangle model took when loaded
// uncooked is given back when they are released, on the GPU (textures,
// meshes) and in the counted heap, as `dump_memory` reports it before the
// loads, while they are drawn, and after the release.
//
// Ways it could fail: release marks the handles failed but leaves the
// texels and vertices with the renderer (the GPU figures stay up); it frees
// the GPU copies but keeps the scene's CPU copies (the heap stays up); a
// released texture or mesh still named by the sprite and the model breaks
// the frame (og run fails, or warns).
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[], o?: { cwd?: string }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };
const text = (b: Uint8Array) => new TextDecoder().decode(b);
const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const fixture = join(repo, "packages/engine/tests/fixtures/uncooked-release");

interface Memory {
  heap: { live: number } | null;
  gpu: { textures: number; meshes: number; total: number } | null;
}

test("og.load.release gives back what an uncooked image and model took on the GPU and the heap", () => {
  const out = mkdtempSync(join(tmpdir(), "og-uncooked-release-"));
  try {
    // Ticks 15 (the warm-up drawn, before the loads at 20), 45 (drawn) and
    // 60 (released at 50). The warm-up (a small image and grid loaded the
    // same way, drawn from about tick 2 and never released) puts what the
    // renderer makes on first use, such as the pipelines a sprite and a lit
    // model draw with, into the base: the dumps then differ by the loads.
    const script = "0.25:dump_memory:base.json;0.75:dump_memory:loaded.json;1:dump_memory:released.json;1:screenshot:released.png;1.1:quit";
    const r = Bun.spawnSync([ogBin(), "run", "--headless", fixture, "--commands", script, "--out", out, "--json"], { cwd: repo });
    if (r.exitCode !== 0) throw new Error(`og run exited ${r.exitCode}: ${text(r.stderr)}`);
    const run = JSON.parse(text(r.stdout).trim().split("\n").pop()!) as { tier: string; warnings: string[] };
    expect(run.warnings).toEqual([]);
    const [base, loaded, released] = ["base", "loaded", "released"].map((f) => JSON.parse(readFileSync(join(out, `${f}.json`), "utf8")) as Memory);
    // The heap: 4 MiB of texels and the grid's vertices and indices, held by the scene, then given back.
    const heap = (m: Memory) => m.heap!.live;
    expect(heap(loaded) - heap(base)).toBeGreaterThan(8_000_000);
    expect(heap(loaded) - heap(released)).toBeGreaterThan(0.95 * (heap(loaded) - heap(base)));
    if (run.tier === "none") throw new Error("missing GPU adapter: release checks require GPU allocation counters");
    const gpu = (m: Memory, k: "textures" | "meshes") => m.gpu![k];
    // The texture with its mips (at least its 4 MiB of level 0), and the grid's 66,049 vertices.
    expect(gpu(loaded, "textures") - gpu(base, "textures")).toBeGreaterThan(4 * 1024 * 1024);
    expect(gpu(loaded, "meshes") - gpu(base, "meshes")).toBeGreaterThan(2_000_000);
    // Released: both back to where they were before the loads, give or take a texel.
    expect(Math.abs(gpu(released, "textures") - gpu(base, "textures"))).toBeLessThan(1024);
    expect(Math.abs(gpu(released, "meshes") - gpu(base, "meshes"))).toBeLessThan(1024);
  } finally {
    rmSync(out, { recursive: true, force: true });
  }
});
