// og.ui.frameCursor answers the `cursor-*` class of the TSX element under
// the pointer (or its nearest ancestor's): `og run` moves the pointer onto
// a `cursor-pointer` button's label, and a system that logs each change
// of frameCursor sees "default" before and "pointer" after.
//
// Ways it could fail: frameCursor reads a frame nothing writes (the old
// og_ui path, always "default"); the label, which names no cursor, hides
// its button's; or `auto` leaks out instead of the arrow.
import { expect, test } from "bun:test";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { ogBin } from "./support/og-bin.ts";

declare const Bun: {
  spawnSync(cmd: string[], o: { cwd: string; env?: Record<string, string | undefined> }): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array };
};

const pkg = join(dirname(fileURLToPath(import.meta.url)), "..");
const repo = join(pkg, "..", "..");
const work = join(repo, "target", "evidence", "ui-cursor");
const game = join(work, "game");

const PROJECT = `type project
version 1
---
name "UI cursor"
id "ui-cursor"
entry "src/main.ts"
create {
  form "2d"
}
tick 60
seed 1
`;

const MAIN = `import { defineSystem, og } from "@origingame/engine";
import { mount } from "@origingame/engine/ui";
import { Menu } from "../assets/ui/menu.tsx";

let last = "";
const Watch = defineSystem("Watch", { reads: [] }, () => {
  const c = og.ui.frameCursor();
  if (c !== last) console.log("CURSOR " + (last = c));
});

export default og.game({
  systems: [Watch],
  start(world) {
    mount(world, Menu, {});
  },
});
`;

const MENU = `import { Text, View } from "@origingame/engine/ui";

export function Menu() {
  return (
    <View class="absolute inset-0 items-center justify-center">
      <View class="px-6 py-3 bg-accent cursor-pointer" onPress={() => {}}>
        <Text>Play</Text>
      </View>
    </View>
  );
}
`;

test("frameCursor is the cursor-* of the TSX element under the pointer", () => {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(join(game, "src"), { recursive: true });
  mkdirSync(join(game, "assets", "ui"), { recursive: true });
  writeFileSync(join(game, "project.ogt"), PROJECT);
  writeFileSync(join(game, "assets", "ui", "menu.tsx"), MENU);
  writeFileSync(join(game, "src", "main.ts"), MAIN);
  const script = ["1.0:move_to_text:=Play", "1.5:quit"].join(";");
  const r = Bun.spawnSync([ogBin(), "run", game, "--no-check", "--commands", script, "--out", work, "--json"], { cwd: repo, env: { ...process.env, OG_ENGINE_PACKAGE: pkg } });
  const out = new TextDecoder().decode(r.stdout) + new TextDecoder().decode(r.stderr);
  if (r.exitCode !== 0) throw new Error(out);
  const seen = out.split("\n").filter((l) => l.startsWith("CURSOR ")).map((l) => l.slice(7));
  expect(seen).toEqual(["default", "pointer"]);
}, 120_000);
