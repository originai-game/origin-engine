// og.platform on the native host: a game's calls answered by the local
// platform (store with revisions and conflicts, leaderboards with top and
// around, telemetry, Origin Point, AI marks, settings), answers applied at
// the start of the next tick with og.PlatformAnswered, refusals for
// malformed calls, `unavailable` without a host, settings that survive a
// restart through the store file, and `dump_platform` listing every call.
// The web shell's Player bridge is covered by tests/web/player.ts.
import { beforeAll, expect, test } from "bun:test";
import { mkdirSync, readFileSync, rmSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { og, defineResource, defineSystem, f64, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { loadNativeHost } from "../src/node/index.ts";
import { nativePlatform } from "../src/node/platform.ts";
import { emptyDocument, localPlatform, memoryPersistence } from "../src/platform-local.ts";
import { createEngine, type Engine, type GameToken } from "../src/runtime.ts";

beforeAll(() => {
  loadNativeHost();
});

const work = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..", "target", "evidence", "platform");

function start(game: unknown): Engine {
  const r = createEngine(game as GameToken);
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

type Res = { request: number; method: string; ok: boolean; data: Record<string, unknown>; error?: { code: string; retryable: boolean; revision: number }; tick: number } | null;
const result = (w: World, r: number) => og.platform.result(w, r) as unknown as Res;

/** Requests made by the Caller system on tick 1, and answers it saw. */
const Calls = defineResource("PtCalls", { set: u32(), get: u32(), submit: u32(), top: u32(), around: u32(), point: u32(), mark: u32(), answered: u32(), answeredAt: f64() });

const Caller = defineSystem("PtCaller", { writes: [Calls], reads: [og.PlatformAnswered] }, (_q, time, world) => {
  const c = must(world.resource(Calls));
  for (const e of world.events(og.PlatformAnswered)) {
    c.answered += 1;
    c.answeredAt = time.tick;
    void e;
  }
  if (time.tick !== 1) return;
  c.set = must(og.platform.storeSet(world, { key: "progress", value: { level: 3, stars: [1, 1, 0] } }));
  c.get = must(og.platform.storeGet(world, "progress"));
  c.submit = must(og.platform.leaderboardSubmit(world, { board: "speedrun", score: 61.5, order: "asc" as never }));
  c.top = og.platform.leaderboardTop(world, { board: "speedrun", count: 3 });
  c.around = og.platform.leaderboardAround(world, { board: "speedrun", count: 3 });
  must(og.platform.telemetry(world, "level.completed", { level: 3, time: 61.5 }));
  c.point = must(og.platform.point(world, { reason: "level.completed", amount: 5 }));
  c.mark = must(og.platform.aiMark(world, { kind: "level", label: "Level 3 layout", model: "maker" }));
});

test("og.platform: a game's calls are answered by the local platform at the next tick's start", () => {
  // The board already has 30 runs by others: 50 s, 51 s, … 79 s (61.5 s ranks 13th).
  const doc = emptyDocument("You");
  doc.boards["speedrun"] = { order: "asc", entries: Array.from({ length: 30 }, (_, i) => ({ name: `p${i}`, score: 50 + i })) };
  const local = localPlatform(memoryPersistence(doc));
  const engine = start(og.game({ resources: [Calls], systems: [Caller] }));
  engine.attachPlatform(local);
  const w = engine.world;
  const info = og.platform.info(w);
  expect([info.host, info.pending]).toEqual(["local", 0]);
  for (const m of ["store.get", "leaderboard.around", "ai.mark"]) expect(info.capabilities.includes(m)).toBeTrue();
  engine.advance(2); // tick 0, tick 1 (the calls), then pumped
  const c = must(w.resource(Calls));
  expect(c.answered).toBe(0);
  expect((og.platform.info(w) as { pending: number }).pending).toBe(7);
  expect(result(w, c.get)).toBeNull();
  engine.advance(1); // tick 2 applies them: results now, PlatformAnswered read by systems on tick 3
  expect(c.answered).toBe(0);
  expect(result(w, c.get)).not.toBeNull();
  engine.advance(1);
  const after = must(w.resource(Calls));
  expect(after.answered).toBe(7);
  expect(after.answeredAt).toBe(3);

  const set = result(w, c.set)!;
  expect([set.method, set.ok, set.data, set.tick]).toEqual(["store.set", true, { key: "progress", revision: 1 }, 2]);
  expect(result(w, c.get)!.data).toEqual({ key: "progress", value: { level: 3, stars: [1, 1, 0] }, revision: 1 });
  expect(result(w, c.submit)!.data).toEqual({ board: "speedrun", rank: 13, best: 61.5 });
  const top = result(w, c.top)!.data as { order: string; entries: Array<{ rank: number; name: string; me: boolean }>; me: { rank: number } };
  expect(top.order).toBe("asc");
  expect(top.entries.map((e) => e.name)).toEqual(["p0", "p1", "p2"]);
  expect(top.me.rank).toBe(13);
  const around = result(w, c.around)!.data as { entries: Array<{ rank: number; me: boolean }> };
  expect(around.entries.map((e) => e.rank)).toEqual([12, 13, 14]);
  expect(around.entries[1]!.me).toBeTrue();
  expect(result(w, c.point)!.data).toEqual({ granted: 5, balance: 5 });
  expect(result(w, c.mark)!.data).toEqual({ marked: true });
  expect(local.document().events.map((e) => e.name)).toEqual(["level.completed"]);
  expect(local.document().marks[0]).toEqual({ kind: "level", label: "Level 3 layout", model: "maker", tick: 1 });

  // A write at a stale revision is a conflict; at the current one it lands.
  // A write at a stale revision, or a create of a key already written, is a
  // conflict naming the revision the key is at; at the current one it lands.
  const stale = must(og.platform.storeSet(w, { key: "progress", value: {}, revision: 7 }));
  const fresh = must(og.platform.storeSet(w, { key: "progress", value: { level: 4 }, revision: 1 }));
  const again = must(og.platform.storeSet(w, { key: "progress", value: {}, create: true }));
  const born = must(og.platform.storeSet(w, { key: "fresh", value: 1, create: true }));
  engine.advance(1); // answered before the tick, applied at its start
  const s1 = result(w, stale)!;
  expect([s1.ok, s1.error?.code, s1.error?.retryable, s1.error?.revision]).toEqual([false, "conflict", false, 1]);
  const f1 = result(w, fresh)!;
  expect([f1.ok, f1.data["revision"]]).toEqual([true, 2]);
  expect([result(w, again)!.ok, result(w, again)!.error?.revision]).toEqual([false, 2]);
  expect(result(w, born)!.data).toEqual({ key: "fresh", revision: 1 });

  // dump_platform lists every call with its state and answer.
  engine.probe.commands("0:dump_platform:platform.json");
  engine.advance(1);
  const out = engine.takeOutputs().find((o) => o.file === "platform.json")!;
  const dump = JSON.parse(out.data) as { host: string; calls: Array<{ method: string; state: string; tick: number }> };
  expect(dump.host).toBe("local");
  expect(dump.calls.map((x) => `${x.method}:${x.state}`)).toEqual([
    "store.set:answered", "store.get:answered", "leaderboard.submit:answered", "leaderboard.top:answered", "leaderboard.around:answered",
    "telemetry:sent", "point:answered", "ai.mark:answered", "store.set:answered", "store.set:answered", "store.set:answered", "store.set:answered",
  ]);
});

test("og.platform: malformed calls are refused naming the fix, and without a host every call is unavailable", () => {
  const engine = start(og.game({}));
  const w = engine.world;
  expect(og.platform.storeGet(w, "has space").ok).toBeFalse();
  const big = og.platform.storeSet(w, { key: "k", value: "x".repeat(70_000) });
  expect(big.ok ? "" : big.reason).toContain("at most 65536");
  expect(og.platform.telemetry(w, "bad name!").ok).toBeFalse();
  expect(og.platform.telemetry(w, "ok", { nested: { no: 1 } }).ok).toBeFalse();
  expect(og.platform.leaderboardSubmit(w, { score: Number.NaN }).ok).toBeFalse();
  expect(og.platform.aiMark(w, { kind: "image", label: "" }).ok).toBeFalse();
  expect(og.platform.aiMark(w, { kind: "Image!", label: "a picture" }).ok).toBeFalse();
  expect(og.platform.telemetry(w, "ok", { BadKey: 1 }).ok).toBeFalse();
  expect(og.platform.point(w, { reason: "has space" }).ok).toBeFalse();
  expect(og.platform.saveSettings(w, [1, 2]).ok).toBeFalse();
  const r = must(og.platform.storeGet(w, "anything"));
  engine.advance(1); // answered before tick 0, applied at its start
  const none = result(w, r)!;
  expect([none.ok, none.error?.code, none.error?.retryable]).toEqual([false, "unavailable", true]);
  const none2 = og.platform.info(w) as { host: string; capabilities: readonly string[] };
  expect([none2.host, none2.capabilities.length]).toEqual(["none", 0]);
});

test("og.platform: settings saved to the store file are there for the next run", () => {
  rmSync(work, { recursive: true, force: true });
  mkdirSync(work, { recursive: true });
  const file = join(work, "platform.json");
  const first = start(og.game({}));
  first.attachPlatform(nativePlatform(file));
  must(og.platform.saveSettings(first.world, { volume: 0.4, language: "ja" }));
  first.advance(1);
  first.destroy();
  // (the save was answered, and so written, before tick 0)
  expect(JSON.parse(readFileSync(file, "utf8")).store["og.settings"].value).toEqual({ volume: 0.4, language: "ja" });

  const second = start(og.game({}));
  second.attachPlatform(nativePlatform(file));
  const load = og.platform.loadSettings(second.world);
  second.advance(1);
  expect(result(second.world, load)!.data).toEqual({ key: "og.settings", value: { volume: 0.4, language: "ja" }, revision: 1 });
});
