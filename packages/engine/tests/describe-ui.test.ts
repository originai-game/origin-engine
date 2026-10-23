// describe's occlusion counts the UI (docs/design.md §3.4): templates/game's
// tick-1 frame at 800×450, where the HUD panel (Menu, an opaque background
// from x 24 to 252) covers all but a strip of the crate hanging at the top
// left. The ways this could fail, each asserted below: the HUD is ignored
// (the crate reads as unoccluded with its whole drawn box visible, as
// before); the UI is counted where it draws nothing opaque (the entities
// clear of the panel gain occlusion); the box or the point lies under the
// panel, so `pick` there would answer the HUD's pixel.
import { expect, test } from "bun:test";
import { mkdtempSync, readFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const repo = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");

type Box = { x: number; y: number; width: number; height: number };
type Described = {
  entities: { id: string; box: Box; drawn: Box; point: { x: number; y: number }; occlusion: number }[];
  ui: { id: string; opaque: boolean; box: Box }[];
};

test("a HUD panel over an entity counts in its occlusion: templates/game at tick 1, 800×450", () => {
  const out = mkdtempSync(join(tmpdir(), "og-describe-ui-"));
  const r = Bun.spawnSync(
    [ogBin(), "run", ".", "--headless", "--no-check", "--size", "800x450", "--commands", "0.016:pause;0.016:describe:d.json", "--out", out, "--json"],
    { cwd: join(repo, "templates", "game") },
  );
  const stdout = new TextDecoder().decode(r.stdout);
  if (r.exitCode !== 0) throw new Error(new TextDecoder().decode(r.stderr) + stdout);
  const run = JSON.parse(stdout.trim().split("\n").pop()!) as { tier: string };
  // The surface takes --size only with a renderer; without one the frame is 1280×720 and clear of the panel.
  if (run.tier === "none") return;
  const d = JSON.parse(readFileSync(join(out, "d.json"), "utf8")) as Described;
  const menu = d.ui.find((u) => u.id === "Menu")!;
  expect(menu.opaque).toBe(true);
  const right = menu.box.x + menu.box.width;
  const crate = d.entities.find((e) => e.id === "crate")!;
  // Drawn alone it starts left of the panel's edge; what shows starts at the edge.
  expect(crate.drawn.x).toBeLessThan(right - 40);
  expect(crate.box.x).toBeGreaterThanOrEqual(right - 1);
  expect(crate.point.x).toBeGreaterThanOrEqual(right);
  expect(crate.occlusion).toBeGreaterThan(0.6);
  // Entities clear of the panel are as the geometry leaves them.
  for (const id of ["trickster", "dropship"]) expect(d.entities.find((e) => e.id === id)!.occlusion).toBe(0);
}, 120_000);
