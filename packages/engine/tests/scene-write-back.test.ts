// save_scene beyond the boot scene (docs/design.md §3.4): the fixture
// tests/fixtures/write-back boots scenes/main.ogt (which sets the resource
// Score), instantiates scenes/room.ogt under a root it spawns, and spawns a
// bolt with a spark under it. A run tunes a field of each scene, a resource
// field the text lacks and one it sets, then `save_scene:spawned`; a second
// run cooks the written texts, reloads and saves again. The ways this
// could fail, each asserted below: the instantiated scene is not written
// (only GameDesc.scene was); a tuned resource is reported as skipped (scene
// text had no resources); the spawned entities are missing, or mixed into
// the authored text above the mark; the instance's root, which the game
// makes again when it instantiates, is written; the reloaded world is not
// the saved one; saving it again changes a line.
import { expect, test } from "bun:test";
import { cpSync, mkdirSync, mkdtempSync, readFileSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare module "node:fs" {
  export function symlinkSync(target: string, path: string): void;
}
declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");

function run(dir: string, commands: string, out: string): string {
  const r = Bun.spawnSync([ogBin(), "run", ".", "--headless", "--no-render", "--no-check", "--commands", commands, "--out", out], { cwd: dir });
  const text = new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
  if (r.exitCode !== 0) throw new Error(text);
  return text;
}

test("save_scene writes every scene, the resources and the entities spawned at run time, and the reload is exact", () => {
  const dir = mkdtempSync(join(tmpdir(), "og-write-back-"));
  cpSync(join(pkg, "tests", "fixtures", "write-back"), dir, { recursive: true });
  mkdirSync(join(dir, "node_modules", "@origingame"), { recursive: true });
  symlinkSync(pkg, join(dir, "node_modules", "@origingame", "engine"));
  const main = () => readFileSync(join(dir, "assets", "scenes", "main.ogt"), "utf8");
  const room = () => readFileSync(join(dir, "assets", "scenes", "room.ogt"), "utf8");
  const authoredRoom = room();

  const log = run(
    dir,
    "0.1:set:post.og.Transform.position=-0.8234567,0.5912345,0.5;0.1:set:lamp.og.Transform.position=1,2.5,0;0.1:set:Score.lives=3;0.1:set:Score.best=7.1234567;0.2:save_scene:spawned",
    join(dir, "out1"),
  );
  expect(log).not.toContain("not written");
  // The instantiated scene, in its own file: only the lamp's position moved.
  expect(room()).toBe(authoredRoom.replace("position 0 2 0", "position 1 2.5 0"));
  const text = main();
  // The resource: its line rewritten with its comment, the tuned field added.
  expect(text).toContain("resource Score {\n  best 7.1234565 # the record\n  lives 3\n}");
  expect(text).toContain("    position -0.8234567 0.5912345 0.5 # where it stands");
  // The spawned entities under the mark, nested as they were; the root instantiate made under hall (the game makes it again) is not written.
  const [authored, section] = text.split("# ── spawned at run time");
  expect(section).toBeDefined();
  expect(authored).not.toContain("bolt");
  expect(section).toContain('entity "bolt" {');
  expect(section).toMatch(/entity "bolt" \{[\s\S]*Charge \{\n    amount 0.75\n  \}\n  entity "spark" \{/);
  expect(section).toContain('entity "hall" {');
  expect(section).not.toContain('entity "" {');

  // Cook, reload, save again: nothing moves, and the world is the saved one.
  const [first, firstRoom] = [main(), room()];
  const again = run(dir, "0.2:save_scene:spawned;0.2:dump_entity:bolt:bolt.json;0.2:dump_entity:lamp:lamp.json;0.2:get:Score:score.json", join(dir, "out2"));
  expect(again).toContain("assets/scenes/main.ogt (0 line(s) changed), assets/scenes/room.ogt (0 line(s) changed)");
  expect(main()).toBe(first);
  expect(room()).toBe(firstRoom);
  const read = (f: string) => JSON.parse(readFileSync(join(dir, "out2", f), "utf8"));
  expect(read("score.json").value).toEqual({ best: 7.1234565, lives: 3 });
  expect(read("bolt.json").components.Charge).toEqual({ amount: 0.75 });
  expect(read("bolt.json").components["og.Transform"].position).toEqual([0.125, 2.3333333, -4]);
  expect(read("lamp.json").components["og.Transform"].position).toEqual([1, 2.5, 0]);
}, 120_000);
