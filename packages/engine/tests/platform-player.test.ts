// og.platform through a Player (shell/platform.ts): a stand-in Player that
// answers Player protocol 2 as origingame/docs/player-protocol.md "Calls"
// says (a keyed store whose revision 0 means "never written", the save with
// its revision, leaderboards with sort and around, point.award, ai.mark)
// receives the shell's calls; asserted are the calls on the wire and the
// answers og.platform gets. An older Player with save.* alone keeps the store
// in the save, and a Player with neither leaves it to the browser's storage.
import { expect, test } from "bun:test";
import { playerPlatform, playerRooms, type PlayerLink } from "../shell/platform.ts";
import type { PlayerResult } from "../shell/player.ts";
import { memoryPersistence, type PlatformAnswer, type PlatformCall } from "../src/platform-local.ts";

type Wire = { method: string; args?: Record<string, unknown> };
const err = (code: string, message: string, revision?: number): PlayerResult => ({ ok: false, error: { code, message, retryable: false, ...(revision === undefined ? {} : { revision }) } });

/** A Player answering protocol 2's calls from memory, listing `capabilities` in its hello. */
function standIn(capabilities: readonly string[]): PlayerLink & { wire: Wire[]; events: string[] } {
  const store = new Map<string, { value: unknown; revision: number }>();
  let save: { data: unknown; revision: number } = { data: null, revision: 0 };
  const wire: Wire[] = [];
  const events: string[] = [];
  const board = [{ name: "Ada", score: 90 }, { name: "Bo", score: 70 }, { name: "Me", score: 60 }, { name: "Cy", score: 50 }];
  const ranked = () => board.map((e, i) => ({ rank: i + 1, name: e.name, score: e.score, me: e.name === "Me" }));
  const answer = (method: string, a: Record<string, unknown>): PlayerResult => {
    switch (method) {
      case "store.get": {
        const s = store.get(String(a["key"]));
        return { ok: true, data: { key: a["key"], value: s?.value ?? null, revision: s?.revision ?? 0 } };
      }
      case "store.set": {
        const key = String(a["key"]);
        const have = store.get(key)?.revision ?? 0;
        if (typeof a["revision"] === "number" && a["revision"] !== have) return err("conflict", `${key} is at ${have}`, have);
        store.set(key, { value: a["value"], revision: have + 1 });
        return { ok: true, data: { key, revision: have + 1 } };
      }
      case "store.list":
        return { ok: true, data: { keys: [...store.keys()].sort().map((key) => ({ key, revision: store.get(key)!.revision })) } };
      case "save.read":
        return { ok: true, data: { data: save.data, revision: save.revision } };
      case "save.set":
        if (typeof a["revision"] === "number" && a["revision"] !== save.revision) return err("conflict", "the save moved", save.revision);
        save = { data: a["data"], revision: save.revision + 1 };
        return { ok: true, data: { ok: true, revision: save.revision } };
      case "leaderboard.submit":
        return { ok: true, data: { ok: true, rank: 3, board: a["board"], best: 64 } };
      case "leaderboard.top":
        return { ok: true, data: { board: a["board"], sort: "desc", top: ranked().slice(0, Number(a["n"])).map(({ me: _, ...e }) => e), me: { rank: 3, name: "Me", score: 60 } } };
      case "leaderboard.around":
        return { ok: true, data: { board: a["board"], sort: "desc", entries: ranked().slice(1, 1 + Number(a["n"])), me: { rank: 3, name: "Me", score: 60 } } };
      case "point.award":
        return { ok: true, data: { granted: 0, balance: null } };
      case "ai.mark":
        return { ok: true, data: { marked: true } };
      default:
        return err("unavailable", `no ${method}`);
    }
  };
  return {
    framed: true,
    hosted: true,
    capabilities,
    wire,
    events,
    async request(method, args) {
      wire.push({ method, ...(args ? { args } : {}) });
      return capabilities.includes(method) ? answer(method, args ?? {}) : err("unavailable", `no ${method}`);
    },
    event(name) {
      events.push(name);
    },
  };
}

/** Runs the calls through the shell's bridge and answers them in request order. */
async function run(player: PlayerLink, calls: Array<[string, Record<string, unknown>]>): Promise<PlatformAnswer[]> {
  const host = playerPlatform(player, memoryPersistence());
  const got = new Map<number, PlatformAnswer>();
  const list: PlatformCall[] = calls.map(([method, args], i) => ({ request: i + 1, method, args, tick: 0 }));
  host.handle(list, (r, a) => got.set(r, a));
  for (let i = 0; i < 50 && got.size < list.filter((c) => c.method !== "telemetry").length; i++) await new Promise((r) => setTimeout(r, 1));
  return list.filter((c) => c.method !== "telemetry").map((c) => got.get(c.request)!);
}

const ALL = ["store.get", "store.set", "store.list", "save.read", "save.set", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark"];

test("a Player's keyed store: revision N, create and the last write wins map onto protocol 2's revisions", async () => {
  const p = standIn(ALL);
  const a = await run(p, [
    ["store.set", { key: "og.settings", value: { volume: 1 }, revision: 0 }],
    ["store.set", { key: "og.settings", value: { volume: 0.5 }, revision: 0 }],
    ["store.set", { key: "progress", value: 1, create: true }],
    ["store.set", { key: "progress", value: 2, create: true }],
    ["store.set", { key: "progress", value: 3, revision: 1 }],
    ["store.set", { key: "progress", value: 4, revision: 1 }],
    ["store.get", { key: "progress" }],
    ["store.list", {}],
  ]);
  // saveSettings (revision 0) sends no revision: the second save lands too.
  expect(p.wire.slice(0, 2).map((w) => w.args)).toEqual([{ key: "og.settings", value: { volume: 1 } }, { key: "og.settings", value: { volume: 0.5 } }]);
  expect(a[1]).toEqual({ ok: true, data: { key: "og.settings", revision: 2 } });
  // create sends revision 0: only a key never written.
  expect(p.wire[2]!.args).toEqual({ key: "progress", value: 1, revision: 0 });
  expect(a[2]!.ok).toBeTrue();
  expect(a[3]).toEqual({ ok: false, error: { code: "conflict", message: "progress is at 1", retryable: false, revision: 1 } });
  expect(a[4]).toEqual({ ok: true, data: { key: "progress", revision: 2 } });
  expect(a[5]).toEqual({ ok: false, error: { code: "conflict", message: "progress is at 2", retryable: false, revision: 2 } });
  expect(a[6]).toEqual({ ok: true, data: { key: "progress", value: 3, revision: 2 } });
  expect(a[7]).toEqual({ ok: true, data: { keys: [{ key: "og.settings", revision: 2 }, { key: "progress", revision: 2 }] } });
  expect(p.wire.some((w) => w.method.startsWith("save."))).toBeFalse();
});

test("leaderboards, Origin Point and AI marks go to the Player's own methods", async () => {
  const p = standIn(ALL);
  const a = await run(p, [
    ["leaderboard.submit", { board: "main", score: 64, order: "desc" }],
    ["leaderboard.top", { board: "main", count: 2 }],
    ["leaderboard.around", { board: "main", count: 3 }],
    ["telemetry", { name: "boss.defeated", attrs: { boss: 1 } }],
    ["point", { reason: "level.completed", amount: 5 }],
    ["ai.mark", { kind: "level", label: "Level 3 layout" }],
  ]);
  expect(p.wire.map((w) => w.method)).toEqual(["leaderboard.submit", "leaderboard.top", "leaderboard.around", "point.award", "ai.mark"]);
  expect(p.wire[1]!.args).toEqual({ n: 2, board: "main" });
  expect(p.wire[2]!.args).toEqual({ n: 3, board: "main" });
  expect(a[0]).toEqual({ ok: true, data: { board: "main", rank: 3, best: 64 } });
  expect(a[1]!.ok && (a[1]!.data as { order: string; entries: unknown[] })).toEqual({ board: "main", order: "desc", entries: [{ rank: 1, name: "Ada", score: 90, me: false }, { rank: 2, name: "Bo", score: 70, me: false }], me: { rank: 3, name: "Me", score: 60 } } as never);
  expect((a[2]!.ok && (a[2]!.data as { entries: Array<{ name: string; me: boolean }> }).entries.map((e) => `${e.name}${e.me ? "*" : ""}`))).toEqual(["Bo", "Me*", "Cy"]);
  expect(a[3]).toEqual({ ok: true, data: { granted: 0, balance: null } });
  expect(a[4]).toEqual({ ok: true, data: { marked: true } });
  expect(p.events).toEqual(["game.custom.boss.defeated"]);
});

test("an older Player keeps the store in the save at its revision; with neither, the browser's storage answers", async () => {
  const old = standIn(["save.read", "save.set", "leaderboard.top"]);
  const a = await run(old, [
    ["store.set", { key: "best", value: 10 }],
    ["store.get", { key: "best" }],
    ["leaderboard.around", { board: "main", count: 1 }],
    ["point", { reason: "x" }],
  ]);
  expect(old.wire.map((w) => w.method)).toEqual(["save.read", "save.set", "save.read", "leaderboard.top"]);
  expect(old.wire[1]!.args).toEqual({ data: { store: { best: { value: 10, revision: 1 } } }, revision: 0 });
  expect(a[1]).toEqual({ ok: true, data: { key: "best", value: 10, revision: 1 } });
  expect((a[2]!.ok && (a[2]!.data as { entries: Array<{ name: string }> }).entries.map((e) => e.name))).toEqual(["Me"]);
  expect([a[3]!.ok, !a[3]!.ok && a[3]!.error.code]).toEqual([false, "unavailable"]);
  const none = standIn([]);
  const b = await run(none, [["store.set", { key: "k", value: 1 }], ["store.get", { key: "k" }]]);
  expect(none.wire).toEqual([]);
  expect(b[1]).toEqual({ ok: true, data: { key: "k", value: 1, revision: 1 } });
});

test("a Player's rooms (platform-asks #16): ops map to room.*, a refusal is an error event, pushed events pass through", async () => {
  expect(playerRooms(standIn(ALL))).toBeNull();
  const rooms = ["room.create", "room.join", "room.leave", "room.send"];
  const p = standIn([...ALL, ...rooms]);
  const host = playerRooms(p)!;
  expect(host.host).toBe("player");
  const got: unknown[] = [];
  host.handle([{ op: "join", room: "duel" }, { op: "send", channel: "og.lockstep", data: { t: 1 } }], (e) => got.push(...e));
  for (let i = 0; i < 20 && p.wire.length < 2; i++) await new Promise((r) => setTimeout(r, 1));
  expect(p.wire).toEqual([{ method: "room.join", args: { room: "duel" } }, { method: "room.send", args: { channel: "og.lockstep", data: { t: 1 } } }]);
  // The stand-in answers no room.* call: each becomes the engine's error event, named by its op.
  for (let i = 0; i < 20 && got.length < 2; i++) await new Promise((r) => setTimeout(r, 1));
  expect(got).toEqual([
    { type: "error", op: "join", code: "unavailable", message: "no room.join" },
    { type: "error", op: "send", code: "unavailable", message: "no room.send" },
  ]);
  host.push({ type: "peer_joined", peer: 1, name: "" });
  expect(got[2]).toEqual({ type: "peer_joined", peer: 1, name: "" });
});
