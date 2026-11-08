import { beforeAll, expect, test } from "bun:test";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, type UiReadable, type UiScope, type UiWritable } from "@origingame/engine";
import { beginBundle, createEngine, type GameToken } from "../src/runtime.ts";
import { loadNativeHost } from "../src/node/index.ts";
import { cookProject } from "./support/og-bin.ts";

declare const Bun: { spawnSync(cmd: string[]): { exitCode: number; stdout: Uint8Array; stderr: Uint8Array } };

beforeAll(() => { loadNativeHost(); });

test("named store mounts render and restore keyed documents on both hosts", () => {
  const root = mkdtempSync(join(tmpdir(), "og-ui-mounts-"));
  try {
    mkdirSync(join(root, "assets"));
    writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "Mounts"\nid "mounts"\nentry "main.ts"\n');
    writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
    writeFileSync(join(root, "assets/mount.svelte"), `<meta name="component" content="mount">
<column><text id="Title">{$title}</text><text id="Sensor">{$sensor}</text>
{#each $rows as row (row.id)}<text id="Label">{row.label}</text>{/each}
</column>`);
    cookProject(root, root);
    const run = (host: string): unknown => {
      const result = Bun.spawnSync(["bun", fileURLToPath(new URL("./support/ui-mount-round.ts", import.meta.url)), host, root]);
      if (result.exitCode !== 0) throw new Error(new TextDecoder().decode(result.stderr));
      return JSON.parse(new TextDecoder().decode(result.stdout));
    };
    const node = run("node");
    expect(node).toEqual({ restored: true, starts: 4, stops: 4, labels: [] });
    expect(run("web")).toEqual(node);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test("focus and hover stay outside simulation while recorded confirmation replays across hosts", () => {
  const root = mkdtempSync(join(tmpdir(), "og-ui-checkpoint-"));
  writeFileSync(join(root, "project.ogt"), 'type project\nversion 1\n---\nname "UI checkpoint"\nid "ui-checkpoint"\nentry "main.ts"\n');
  writeFileSync(join(root, "main.ts"), 'import { og } from "@origingame/engine"; export default og.game({});\n');
  mkdirSync(join(root, "assets"));
  writeFileSync(join(root, "assets", "pause.svelte"), `<meta name="screen" content="boundaryPause" data-layer="menu" data-pause="true">
<column id="Root">
  <text id="Title" i18n="pause.title">Paused</text>
</column>
`);
  cookProject(root, root);
  const run = (host: string) => {
    const result = Bun.spawnSync(["bun", fileURLToPath(new URL("./support/ui-checkpoint-round.ts", import.meta.url)), host, root]);
    if (result.exitCode !== 0) throw new Error(new TextDecoder().decode(result.stderr));
    return JSON.parse(new TextDecoder().decode(result.stdout));
  };
  try {
    const node = run("node");
    const web = run("web");
    expect(web).toEqual(node);
    expect(node.withUi.hovered).toBe(true);
    expect(node.withUi.focused).toBe(true);
    expect(node.replay.hovered).toBe(false);
    expect(node.replay.focused).toBe(false);
    expect(node.withUi.hashes).toEqual(node.replay.hashes);
    expect(node.replay.confirms).toBe(3);
    expect(node.withUi.scale).toBe(0);
    expect(node.replay.scale).toBe(0);
    expect(node.noIntent.scale).toBe(1);
    expect(node.withUi.intentsRecorded.map((intent: { name: string }) => intent.name)).toEqual(["CheckpointConfirm", "og.PauseIntent"]);
    expect(node.noIntent.hashes).toEqual(node.noUi.hashes);
    expect(node.noIntent.confirms).toBe(0);
    expect(node.noIntent.hashes.slice(0, 3)).toEqual(node.replay.hashes.slice(0, 3));
    expect(node.noIntent.hashes.at(-1)).not.toBe(node.replay.hashes.at(-1));
  } finally { rmSync(root, { recursive: true, force: true }); }
});

function engine(desc: Parameters<typeof og.game>[0] = { systems: [] }) {
  const created = createEngine(og.game(desc) as unknown as GameToken);
  if (!created.ok) throw new Error(created.reason);
  return created.value;
}

test("named stores subscribe synchronously, stop at the last subscriber, and retain values on registration", () => {
  const e = engine();
  try {
    const scope = og.ui.scope(e.world, "inventory");
    const gold = scope.writable("gold", 3);
    const values: number[] = [];
    const stop = gold.subscribe(value => values.push(value));
    expect(values).toEqual([3]);
    expect(gold.update(value => value + 4).ok).toBe(true);
    expect(gold.set(7).ok).toBe(true);
    expect(values).toEqual([3, 7]);
    expect(scope.get(scope.writable("gold", 999))).toBe(7);
    stop();
    stop();
    gold.set(11);
    expect(values).toEqual([3, 7]);

    let starts = 0, stops = 0;
    const readable = scope.readable("sensor", 0, set => {
      starts++;
      set(23);
      return () => { stops++; };
    });
    expect(starts).toBe(0);
    const a: number[] = [], b: number[] = [];
    const stopA = readable.subscribe(value => a.push(value));
    const stopB = readable.subscribe(value => b.push(value));
    expect(a).toEqual([23]);
    expect(b).toEqual([23]);
    expect(starts).toBe(1);
    stopA();
    expect(stops).toBe(0);
    stopB();
    expect(stops).toBe(1);
    expect(scope.get(readable)).toBe(23);
    expect([starts, stops]).toEqual([2, 2]);
    const object = scope.writable("object", { value: 7 });
    let objectNotifications = 0;
    const stopObject = object.subscribe(() => { objectNotifications++; });
    object.set({ value: 7 });
    expect(objectNotifications).toBe(2);
    stopObject();
    expect(scope.takeErrors()).toEqual([]);
  } finally { e.destroy(); }
});

test("ui registration runs after start, reloads callbacks without start, and restores older snapshots by ID", () => {
  beginBundle();
  let scope!: UiScope;
  let kept!: UiWritable<number>;
  let computed!: UiReadable<number>;
  let added!: UiWritable<number>;
  let starts = 0, setups = 0;
  const e = engine({
    start(world) { starts++; world.spawn({ name: "from-start" }); },
    ui(world) {
      setups++;
      expect(world.find("from-start")).not.toBeNull();
      scope = og.ui.scope(world, "reload");
      kept = scope.writable("kept", 3);
      scope.writable("removed", 11);
      computed = scope.derived("computed", kept, value => value * 2);
    },
  });
  try {
    expect([starts, setups]).toEqual([1, 1]);
    kept.set(13);
    const saved = e.world.snapshot();
    if (!saved.ok) throw new Error(saved.reason);
    const observed: number[] = [];
    const stop = computed.subscribe(value => observed.push(value));
    expect(e.advance(3).ok).toBe(true);
    expect([starts, setups]).toEqual([1, 1]);
    beginBundle();
    og.game({
      start() { throw new Error("reload must not run gameplay start"); },
      ui(world) {
        setups++;
        scope = og.ui.scope(world, "reload");
        kept = scope.writable("kept", 999);
        added = scope.writable("added", 7);
        computed = scope.derived("computed", kept, value => value * 5);
      },
    });
    expect(e.probe.reload()).not.toBeNull();
    expect([starts, setups]).toEqual([1, 2]);
    expect(scope.get(kept)).toBe(13);
    expect(scope.get(computed)).toBe(65);
    expect(observed.at(-1)).toBe(65);
    kept.set(19);
    added.set(97);
    expect<unknown>(e.world.restore(saved.value)).toEqual({ ok: true, value: null });
    expect(scope.get(kept)).toBe(13);
    expect(scope.get(computed)).toBe(65);
    expect(scope.get(added)).toBe(7);
    expect(scope.takeErrors()).toEqual([]);
    stop();
  } finally { e.destroy(); }
});

test("ui registration reports duplicate IDs and refuses gameplay writes before they happen", () => {
  const duplicate = createEngine(og.game({ ui(world) {
    const scope = og.ui.scope(world, "duplicate");
    scope.writable("gold", 1);
    scope.writable("gold", 2);
  } }) as unknown as GameToken);
  expect(duplicate.ok).toBe(false);
  if (!duplicate.ok) expect(duplicate.reason).toContain("duplicate ID");

  let refused = false;
  const created = createEngine(og.game({ ui(world) {
    refused = !world.spawn({ name: "forbidden" }).ok;
  } }) as unknown as GameToken, { deferStart: true });
  if (!created.ok) throw new Error(created.reason);
  const e = created.value;
  try {
    const advanced = e.advance(1);
    expect(advanced.ok).toBe(false);
    if (!advanced.ok) expect(advanced.reason).toContain("GameDesc.ui");
    expect(refused).toBe(true);
    expect(e.world.find("forbidden")).toBeNull();
  } finally { e.destroy(); }
});

test("presentation stores checkpoint independently of the simulation hash", () => {
  const first = engine();
  const second = engine();
  const incompatible = engine();
  try {
    const scope = og.ui.scope(first.world, "checkpoint");
    const count = scope.writable("count", 3);
    const before = first.probe.snapshot();
    expect(before).not.toBeNull();
    expect(count.set(19).ok).toBe(true);
    const after = first.probe.snapshot();
    expect(after?.hash).toBe(before?.hash);
    expect(after?.presentation).not.toBe(before?.presentation);
    const saved = first.world.snapshot();
    if (!saved.ok) throw new Error(saved.reason);
    const wrongScope = og.ui.scope(incompatible.world, "checkpoint");
    const wrongType = wrongScope.readable("count", "different type");
    incompatible.world.spawn({ name: "keep-on-refusal" });
    const unchanged = incompatible.probe.snapshot();
    const refused = incompatible.world.restore(saved.value);
    expect(refused.ok).toBe(false);
    if (!refused.ok) expect(refused.reason).toContain("different store type");
    expect(incompatible.probe.snapshot()?.hash).toBe(unchanged?.hash);
    expect(wrongScope.get(wrongType)).toBe("different type");
    const other = og.ui.scope(second.world, "checkpoint");
    const restored = other.writable("count", 99);
    expect(second.world.restore(saved.value).ok).toBe(true);
    expect(other.get(restored)).toBe(19);
    expect(count.set(31).ok).toBe(true);
    if (!after) throw new Error("missing checkpoint");
    expect(first.probe.restore(after.id)).not.toBeNull();
    expect(scope.get(count)).toBe(19);
    expect(first.probe.snapshot()?.hash).toBe(before?.hash);
  } finally { first.destroy(); second.destroy(); incompatible.destroy(); }
});

test("derived stores are lazy, settle an asymmetric diamond once, and replace callbacks by ID", () => {
  const e = engine();
  try {
    const scope = og.ui.scope(e.world, "hud");
    const count = scope.writable("count", 2);
    let leftRuns = 0, rightRuns = 0;
    const left = scope.derived("z-left", count, value => { leftRuns++; return value * 3; }, 0);
    const right = scope.derived("y-right", count, value => { rightRuns++; return value + 7; }, 0);
    const sum = scope.derived("a-sum", [left, right] as const, ([a, b]) => a + b);
    expect([leftRuns, rightRuns]).toEqual([0, 0]);
    const values: number[] = [];
    const stop = sum.subscribe(value => values.push(value));
    expect(values).toEqual([15]);
    count.set(5);
    expect(values).toEqual([15, 27]);
    expect([leftRuns, rightRuns]).toEqual([2, 2]);
    scope.derived("z-left", count, value => value * 4, 0);
    expect(values).toEqual([15, 27, 32]);
    expect(scope.get(count)).toBe(5);
    stop();
    count.set(9);
    expect(scope.get(sum)).toBe(52);
    expect(scope.takeErrors()).toEqual([]);
  } finally { e.destroy(); }
});

test("collections preserve keyed edits through restore and refuse duplicate keys without mutation", () => {
  const e = engine();
  try {
    const scope = og.ui.scope(e.world, "shop");
    const rows = scope.collection("items", [{ id: "b", count: 7 }, { id: "a", count: 13 }]);
    expect(rows.keys()).toEqual(["b", "a"]);
    expect(rows.insert({ id: "b", count: 99 }).ok).toBe(false);
    expect(scope.get(rows)).toEqual([{ id: "b", count: 7 }, { id: "a", count: 13 }]);
    expect(rows.move("a", "b").ok).toBe(true);
    expect(rows.patch("b", { count: 17 }).ok).toBe(true);
    const observed: number[] = [];
    const total = scope.derived("total", rows, values => values.reduce((n, row) => n + row.count, 0));
    const stop = total.subscribe(value => observed.push(value));
    const save = e.world.snapshot();
    if (!save.ok) throw new Error(save.reason);
    rows.remove("a");
    rows.patch("b", { count: 21 });
    expect(e.world.restore(save.value).ok).toBe(true);
    expect(observed).toEqual([30, 17, 21, 30]);
    expect(rows.keys()).toEqual(["a", "b"]);
    expect(scope.get(rows)).toEqual([{ id: "a", count: 13 }, { id: "b", count: 17 }]);
    stop();
    const dormantRuns: number[] = [];
    scope.derived("dormant", rows, values => { dormantRuns.push(values.length); return values.length; });
    expect(e.world.restore(save.value).ok).toBe(true);
    expect(dormantRuns).toEqual([]);
    const errors = scope.takeErrors();
    expect(errors.length).toBe(1);
    expect(errors[0]).toContain("shop");
    expect(errors[0]).toContain("items");
    expect(errors[0]).toContain("duplicate key");
  } finally { e.destroy(); }
});

test("undeclared reads and reentrant writes report scope and store without committing derived results", () => {
  const e = engine();
  try {
    const scope = og.ui.scope(e.world, "errors");
    const a = scope.writable("a", 3), b = scope.writable("b", 11);
    const undeclared = scope.derived("undeclared", a, value => value + scope.get(b), -1);
    expect(scope.get(undeclared)).toBe(-1);
    const reentrant = scope.derived("reentrant", a, value => { a.set(99); return value * 2; }, -2);
    expect(scope.get(reentrant)).toBe(-2);
    expect(scope.get(a)).toBe(3);
    const typed = scope.derived("typed", a, value => value * 7);
    expect(scope.get(typed)).toBe(21);
    scope.derived("typed", a, value => String(value));
    expect(scope.get(typed)).toBe(21);
    const errors = scope.takeErrors().join("\n");
    expect(errors).toContain("errors");
    expect(errors).toContain("undeclared");
    expect(errors).toContain("reentrant");
    expect(errors).toContain("different derived store type");
  } finally { e.destroy(); }
});
