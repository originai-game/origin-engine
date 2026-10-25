import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import assert from "node:assert/strict";
import { ogBin } from "../../../../packages/engine/tests/support/og-bin.ts";
assert.equal(readFileSync("packages/engine/native/build-profile", "utf8").trim(), "release");
const out = resolve(process.argv[2]);
const games = [
  ["hollowbell", "0.1:play;0.5:key_down:KeyD;1.5:key_down:Space;1.8:key_up:Space;2:key_down:KeyX;2.05:key_up:KeyX;3:key_down:KeyX;3.05:key_up:KeyX;4:key_down:Space;4.4:key_up:Space;6:key_up:KeyD;6:key_down:KeyA;8:key_down:KeyX;8.05:key_up:KeyX;9.5:key_up:KeyA"],
  ["kestrel-stair", "0.1:play:0:quick;0.5:key_down:KeyW;2:key_down:KeyJ;2.05:key_up:KeyJ;3:key_down:KeyD;5:key_up:KeyD;5:key_down:Space;5.2:key_up:Space;7:key_down:KeyJ;7.05:key_up:KeyJ;9.5:key_up:KeyW"],
  ["lantern-vale", "0.1:run;0.5:key_down:KeyW;3:key_down:KeyD;5:key_up:KeyD;6:key_up:KeyW;6:warp:0;6.2:key_down:KeyE;8:key_up:KeyE;8.2:key_down:KeyS;9.9:key_up:KeyS"],
];
const rows = [];
for (let round = 0; round < 5; round++) for (const [game, script] of games) {
  const dir = `${out}/${round}/${game}`;
  mkdirSync(dir, { recursive: true });
  const args = [ogBin(), "run", `examples/${game}`, "--headless", "--json", "--ticks", "600", "--size", "1280x720", "--commands", script, "--out", dir];
  writeFileSync(`${dir}/command.json`, JSON.stringify(args));
  const child = Bun.spawn(args, { env: { ...process.env, OG_ENGINE_PACKAGE: resolve("packages/engine") }, stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, status] = await Promise.all([new Response(child.stdout).text(), new Response(child.stderr).text(), child.exited]);
  writeFileSync(`${dir}/stdout.txt`, stdout);
  writeFileSync(`${dir}/stderr.txt`, stderr);
  writeFileSync(`${dir}/exit.txt`, `${status}\n`);
  assert.equal(status, 0, `${game} failed`);
  const result = JSON.parse(stdout.trim().split("\n").at(-1));
  assert(!result.warnings?.length && !result.unchecked?.length);
  rows.push({ game, round, result });
  writeFileSync(`${out}/runs.json`, JSON.stringify(rows, null, 2) + "\n");
}
