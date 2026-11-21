// The TSX path's contracts end to end (crates/og-ui-next/CONVENTIONS.md §JSX
// compile, §Ops): nsh-jsx compiles a component to the renderer's calls, the
// renderer writes the OGOP batch the tree applies, a changed signal sends only
// what changed, and an OGEV reply calls the node's handler with the world.

import { expect, test } from "bun:test";
import type { World } from "@origingame/engine";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { bundleGame, compileJsx } from "../src/node/build.ts";
import { mount, renderers } from "../src/ui/index.ts";
import { Renderer } from "../src/ui/rt.ts";
import { createSignal } from "../src/ui/signals.ts";

const SOURCE = `import { View, Text, Show, Switch, Match, packClass } from "@origingame/engine/ui";
export function Hud(props: { hp: number; on: boolean; press: (e: unknown) => void }) {
  return (
    <View class="p-4 bg-surface" onPress={props.press}>
      <Text class="text-fg">HP {props.hp}</Text>
      <Show when={props.on}><Text>on</Text></Show>
      <Switch fallback={<Text>low</Text>}>
        <Match when={props.hp > 3}><Text class={packClass("Hud", "hp", "high")}>high</Text></Match>
      </Switch>
    </View>
  );
}`;

type Op = (string | number)[];

/** The ops of an OGOP batch, read independently of the renderer. */
function decode(bytes: Uint8Array): Op[] {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  expect(new TextDecoder().decode(bytes.subarray(0, 4))).toBe("OGOP");
  let at = 4;
  const u8 = () => v.getUint8(at++);
  const u32 = () => { const n = v.getUint32(at, true); at += 4; return n; };
  const str = () => { const n = u32(); const s = new TextDecoder().decode(bytes.subarray(at, at + n)); at += n; return s; };
  const names = ["", "create", "createText", "setText", "setClass", "setStyle", "setProp", "listen", "insert", "remove", "release", "placeholder"];
  const ops: Op[] = [];
  for (let i = u32(); i > 0; i--) {
    const code = u8();
    const name = names[code]!;
    if (code === 1) ops.push([name, u32(), u8()]);
    else if (code === 2 || code === 3 || code === 4 || code === 5) ops.push([name, u32(), str()]);
    else if (code === 6) ops.push([name, u32(), str(), str()]);
    else if (code === 7) ops.push([name, u32(), u8(), u8()]);
    else if (code === 8) { const p = u32(), c = u32(), b = u32(); ops.push([name, p, c, b === 0xffffffff ? -1 : b]); }
    else if (code === 9) ops.push([name, u32(), u32()]);
    else ops.push([name, u32()]);
  }
  expect(at).toBe(bytes.length);
  return ops;
}

function reply(events: [id: number, code: number][]): Uint8Array {
  const out = new Uint8Array(8 + events.length * 21);
  const v = new DataView(out.buffer);
  out.set([0x4f, 0x47, 0x45, 0x56]);
  v.setUint32(4, events.length, true);
  events.forEach(([id, code], i) => {
    const at = 8 + i * 21;
    v.setUint32(at, id, true);
    v.setUint8(at + 4, code);
    v.setFloat32(at + 5, 12, true);
    v.setFloat32(at + 9, 34, true);
  });
  return out;
}

test("a TSX component's ops, its updates and its events", async () => {
  const compiled = compileJsx("Hud.tsx", SOURCE);
  if (!compiled.ok) throw new Error(compiled.reason);
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-"));
  try {
    const rt = fileURLToPath(new URL("../src/ui/rt.ts", import.meta.url));
    const ui = fileURLToPath(new URL("../src/ui/index.ts", import.meta.url));
    const code = compiled.code.replace('"@origingame/engine/ui/rt"', JSON.stringify(rt)).replace('"@origingame/engine/ui"', JSON.stringify(ui));
    writeFileSync(join(dir, "Hud.js"), code);
    const { Hud } = await import(join(dir, "Hud.js"));

    const batches: Op[][] = [];
    let next: Uint8Array = reply([]);
    // The runtime's worlds carry their id; the public World does not name it.
    const world = { id: 9001 } as unknown as World;
    renderers.set(9001, new Renderer((bytes) => { batches.push(decode(bytes)); const r = next; next = reply([]); return r; }, world));
    const hp = createSignal(3), on = createSignal(false);
    const pressed: unknown[] = [];
    mount(world, Hud, { get hp() { return hp(); }, get on() { return on(); }, press: (e: unknown) => pressed.push(e) }, { layer: "hud" });

    // The mount's one batch builds the whole tree and puts its root in 0.
    expect(batches).toHaveLength(1);
    const first = batches[0]!;
    expect(first).toContainEqual(["create", 2, 0]);
    expect(first).toContainEqual(["setClass", 2, "p-4 bg-surface"]);
    expect(first).toContainEqual(["listen", 2, 0, 1]);
    expect(first).toContainEqual(["createText", 4, "HP "]);
    expect(first).toContainEqual(["createText", 5, "3"]);
    expect(first.at(-1)).toEqual(["insert", 0, 1, -1]);
    // The mount's root fills the surface, so a component's `absolute inset-0` covers the screen.
    expect(first).toContainEqual(["setClass", 1, "absolute inset-0 flex-col"]);
    expect(first.some((op) => op[0] === "createText" && op[2] === "on")).toBe(false);
    expect(first.filter((op) => op[0] === "createText" && (op[2] === "low" || op[2] === "high")).map((op) => op[2])).toEqual(["low"]);

    // A changed signal sends only its text; the branch shows when `on` does.
    hp.set(4);
    on.set(true);
    renderers.get(9001)!.flush();
    const second = batches[1]!;
    expect(second).toContainEqual(["setText", 5, "4"]);
    // The Switch's case changes with hp: "high" is made, with its pack token, and "low" goes.
    expect(second.filter((op) => op[0] === "createText").map((op) => op[2]).sort()).toEqual(["high", "on"]);
    expect(second).toContainEqual(["setClass", expect.any(Number), "@Hud.hp.high"]);
    const made = (text: string) => second.find((op) => op[0] === "createText" && op[2] === text)!;
    const holder = (text: string) => second.find((op) => op[0] === "insert" && op[2] === made(text)[1])![1];
    expect(second).toContainEqual(["insert", 2, holder("high"), -1]);
    expect(second).toContainEqual(["insert", 2, holder("on"), -1]);
    expect(second.filter((op) => op[0] === "remove" && op[1] === 2)).toHaveLength(1);
    expect(second.filter((op) => op[0] === "release")).toHaveLength(1);
    expect(second.some((op) => op[0] === "create" && op[2] === 0)).toBe(false);

    // A press on the view (id 2) calls its handler with the world.
    next = reply([[2, 0]]);
    renderers.get(9001)!.flush();
    expect(pressed).toEqual([expect.objectContaining({ world, type: "press", x: 12, y: 34 })]);
  } finally {
    rmSync(dir, { recursive: true, force: true });
    renderers.delete(9001);
  }
});

test("a native bundle compiles the engine's TSX components with the game's", async () => {
  const dir = mkdtempSync(join(tmpdir(), "og-tsx-native-"));
  try {
    writeFileSync(join(dir, "project.ogt"), "type project\nversion 1\n---\n");
    writeFileSync(join(dir, "main.ts"), `export { Spinner } from "@origingame/engine/ui/components";\n`);
    const out = join(dir, "game.js");
    const b = await bundleGame(join(dir, "main.ts"), out, "native");
    if (!b.ok) throw new Error(b.reason);
    const code = readFileSync(out, "utf8");
    // Loaded as an external module, Bun's own JSX transform would have kept
    // the compile-time Show/For imports, which the engine does not export.
    expect(code).not.toContain("@origingame/engine/ui/components");
    expect(code).toMatch(/from ["'][^"']*ui\/rt(\.ts)?["']/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
