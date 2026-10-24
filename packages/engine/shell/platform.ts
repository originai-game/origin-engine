// og.platform on the web: the shell's answer to the engine Worker's
// platform calls. With a Player framing the page, each call goes to it over
// Player protocol 2 (./player.ts, origin-checked postMessage; the Player
// owns every credential; or the Stage overlay in the page's own document),
// as its hello's capabilities allow; an answer's `time` (the Player's clock)
// passes on to og.platform; without one, or
// where the Player answers no such method, the browser's own storage answers
// (../src/platform-local.ts over localStorage). The mapping:
// docs/web-export.md §Platform services.
//
//   store.get/set/list   store.get / store.set / store.list, each key with its
//                        own revision: og.platform's revision N sends
//                        revision N, `create` sends revision 0 (only a key
//                        never written), revision 0 sends none (the last
//                        write wins); a conflict carries the key's revision.
//                        A Player with save.* but no store.* (older ones)
//                        keeps the store in the save's `store` object, read
//                        with save.read (or save.get) and written back at the
//                        save's revision; neither: the browser's storage.
//   leaderboard.submit   leaderboard.submit { score, board, order } → { board, rank, best }
//   leaderboard.top      leaderboard.top { n, board } → { board, sort, top, me }
//   leaderboard.around   leaderboard.around { n, board } → { board, sort, entries, me }
//                        (older Players: leaderboard.top { n: 100 }, cut around me)
//   telemetry            an `event` named game.custom.<name>
//   point                point.award { reason, amount } → { granted, balance }
//   ai.mark              ai.mark { kind, label, model? } → { marked }
//   rooms                room.create { name?, capacity? }, room.join { room, name? },
//                        room.leave {}, room.send { to?, channel?, data? | bytes? }
//                        (docs/platform-asks.md #16): a result only acknowledges
//                        (a failed one becomes the engine's `error` event); every
//                        room event arrives as a pushed `room` message in the
//                        engine's own shape (joined, peer_joined, peer_left,
//                        message, left), in order. A Player without room.join
//                        carries no rooms: roomInfo's host is `none`.

import { answerLocally, fail, localPlatform, type LocalDocument, type LocalPersistence, type PlatformAnswer, type PlatformCall, type PlatformHost } from "../src/platform-local.ts";
export { storagePersistence } from "../src/platform-local.ts";
import type { PlayerResult } from "./player.ts";
import type { RoomHost } from "../src/web/shell.ts";

/** The part of the Player link this needs (connectPlayer's answer). */
export interface PlayerLink {
  readonly framed: boolean;
  readonly hosted: boolean;
  readonly capabilities: readonly string[];
  readonly time?: number | undefined;
  request(method: string, args?: Record<string, unknown>): Promise<PlayerResult>;
  event(name: string, attrs?: Record<string, string | number | boolean | null>): void;
}

type Save = { store?: Record<string, { value: unknown; revision: number }> } & Record<string, unknown>;

function asSave(data: unknown): Save {
  return data && typeof data === "object" && !Array.isArray(data) ? (data as Save) : {};
}

/** The Player's `time` on an answer, when it stamped one: og.platform's clock is then the Player's. */
const stamp = (r: PlayerResult | undefined): { time?: number } => (typeof r?.time === "number" ? { time: r.time } : {});

/** The Player's answer as og.platform's (a conflict keeps the revision it names; the Player's `time` passes on). */
const fromPlayer = (r: PlayerResult, data: (d: unknown) => unknown): PlatformAnswer =>
  r.ok
    ? { ok: true, data: data(r.data), ...stamp(r) }
    : { ok: false, error: { code: r.error.code, message: r.error.message, retryable: r.error.retryable, ...(typeof r.error.revision === "number" ? { revision: r.error.revision } : {}) }, ...stamp(r) };

const STORE = ["store.get", "store.set", "store.list"];
type Entry = { rank: number; name: string; score: number; me?: boolean };

/** og.platform's leaderboard answer: `order` from the Player's `sort`. */
const board = (a: Record<string, unknown>, d: unknown, list: "top" | "entries") => {
  const t = (d ?? {}) as { board?: string; sort?: string; top?: Entry[]; entries?: Entry[]; me?: { rank: number } | null };
  const me = t.me ?? null;
  const entries = (t[list] ?? []).map((e) => ({ rank: e.rank, name: e.name, score: e.score, me: e.me ?? (!!me && e.rank === me.rank) }));
  return { board: t.board ?? a["board"], order: t.sort === "asc" ? "asc" : "desc", entries, me };
};

/** The Player-backed platform, falling back to `local` (the browser's storage). */
export function playerPlatform(player: PlayerLink, local: LocalPersistence): PlatformHost {
  const fallback = localPlatform(local);
  const has = (m: string) => player.capabilities.includes(m);
  const keyed = STORE.every(has);
  const saved = (has("save.read") || has("save.get")) && has("save.set");
  const capabilities = [
    ...STORE,
    "telemetry",
    ...(has("leaderboard.submit") ? ["leaderboard.submit"] : []),
    ...(has("leaderboard.top") ? ["leaderboard.top"] : []),
    ...(has("leaderboard.around") || has("leaderboard.top") ? ["leaderboard.around"] : []),
    ...(has("point.award") ? ["point"] : []),
    ...(has("ai.mark") ? ["ai.mark"] : []),
  ];
  let queue: Promise<void> = Promise.resolve();

  const viaLocal = (call: PlatformCall): PlatformAnswer | null => {
    let out: PlatformAnswer | null = null;
    fallback.handle([call], (_r, a) => (out = a));
    return out;
  };

  /** The keyed store, through the Player's store.* methods. */
  const keyedStore = async (call: PlatformCall): Promise<PlatformAnswer | null> => {
    const a = call.args;
    if (call.method === "store.get") return fromPlayer(await player.request("store.get", { key: a["key"] }), (d) => d);
    if (call.method === "store.list") return fromPlayer(await player.request("store.list"), (d) => ({ keys: ((d ?? {}) as { keys?: unknown[] }).keys ?? [] }));
    const revision = a["create"] === true ? 0 : Number(a["revision"] ?? 0) > 0 ? Number(a["revision"]) : undefined;
    const r = await player.request("store.set", { key: a["key"], value: a["value"] ?? null, ...(revision === undefined ? {} : { revision }) });
    return r.ok || r.error.code !== "unavailable" ? fromPlayer(r, (d) => d) : viaLocal(call);
  };

  /** The store in the save's `store` object (a Player with save.* and no store.*). */
  const savedStore = async (call: PlatformCall): Promise<PlatformAnswer | null> => {
    const read = has("save.read");
    const got = await player.request(read ? "save.read" : "save.get");
    if (!got.ok) return got.error.code === "unavailable" ? viaLocal(call) : fromPlayer(got, () => null);
    const at = read ? ((got.data ?? {}) as { data?: unknown; revision?: number | null }) : { data: got.data, revision: undefined };
    const save = asSave(at.data);
    const doc: LocalDocument = { version: 1, player: "", store: { ...(save.store ?? {}) }, boards: {}, points: { balance: 0, grants: [] }, marks: [], events: [] };
    const answer = answerLocally(doc, call);
    if (call.method !== "store.set" || !answer?.ok) return answer && { ...answer, ...stamp(got) };
    // Written back at the revision it was read at: a write from another device in between is a conflict.
    const revision = typeof at.revision === "number" ? at.revision : undefined;
    const put = await player.request("save.set", { data: { ...save, store: doc.store }, ...(revision === undefined ? {} : { revision }) });
    if (!put.ok) return fromPlayer(put, () => null);
    const queued = (put.data as { queued?: boolean } | null)?.queued === true;
    return { ok: true, data: { ...(answer.data as object), ...(queued ? { queued: true } : {}) }, ...stamp(put.time === undefined ? got : put) };
  };

  const one = async (call: PlatformCall): Promise<PlatformAnswer | null> => {
    const a = call.args;
    if (STORE.includes(call.method)) return keyed ? keyedStore(call) : saved ? savedStore(call) : viaLocal(call);
    const n = Math.max(1, Math.min(100, Number(a["count"] ?? 10)));
    switch (call.method) {
      case "telemetry":
        player.event(`game.custom.${String(a["name"])}`, a["attrs"] as Record<string, string | number | boolean | null>);
        return null;
      case "leaderboard.submit":
        if (!has("leaderboard.submit")) return fail("unavailable", "this Player answers no leaderboard.submit here");
        return fromPlayer(await player.request("leaderboard.submit", { score: a["score"], board: a["board"], order: a["order"] }), (d) => {
          const r = (d ?? {}) as { board?: string; rank?: number | null; best?: number | null; queued?: boolean };
          return { board: r.board ?? a["board"], rank: r.rank ?? null, best: r.best ?? null, ...(r.queued ? { queued: true } : {}) };
        });
      case "leaderboard.top":
        if (!has("leaderboard.top")) return fail("unavailable", "this Player answers no leaderboard.top here");
        return fromPlayer(await player.request("leaderboard.top", { n, board: a["board"] }), (d) => board(a, d, "top"));
      case "leaderboard.around": {
        if (has("leaderboard.around")) return fromPlayer(await player.request("leaderboard.around", { n, board: a["board"] }), (d) => board(a, d, "entries"));
        if (!has("leaderboard.top")) return fail("unavailable", "this Player answers no leaderboard.around here");
        // An older Player: the top 100, cut around the player.
        return fromPlayer(await player.request("leaderboard.top", { n: 100, board: a["board"] }), (d) => {
          const b = board(a, d, "top");
          const at = b.entries.findIndex((e) => e.me);
          const from = b.me && at >= 0 ? Math.max(0, Math.min(b.entries.length - n, at - Math.floor(n / 2))) : 0;
          return { ...b, entries: b.entries.slice(from, from + n) };
        });
      }
      case "point":
        if (!has("point.award")) return fail("unavailable", "this Player answers no point.award here");
        return fromPlayer(await player.request("point.award", { reason: a["reason"], amount: a["amount"] ?? 1 }), (d) => {
          const r = (d ?? {}) as { granted?: number; balance?: number | null };
          return { granted: r.granted ?? 0, balance: r.balance ?? null };
        });
      case "ai.mark":
        if (!has("ai.mark")) return fail("unavailable", "this Player answers no ai.mark here");
        return fromPlayer(await player.request("ai.mark", { kind: a["kind"], label: a["label"], ...(typeof a["model"] === "string" ? { model: a["model"] } : {}) }), () => ({ marked: true }));
      default:
        return fail("unavailable", `the Player has no ${call.method} method in protocol 2`);
    }
  };

  return {
    host: "player",
    capabilities,
    time: player.time,
    handle(calls, answer) {
      // In order: a store write through the save reads it first, so calls never interleave.
      for (const call of calls) {
        queue = queue.then(async () => {
          let r: PlatformAnswer | null;
          try {
            r = await one(call);
          } catch (e) {
            r = fail("failed", e instanceof Error ? e.message : String(e));
          }
          if (r) answer(call.request, r);
        });
      }
    },
  };
}

/**
 * A host that holds calls until `ready` names the real one: a framed page's
 * first calls (the first tick, behind the gate) wait for the Player's hello
 * instead of reaching the browser's storage.
 */
export function deferredPlatform(ready: Promise<PlatformHost>, capabilities: readonly string[]): PlatformHost {
  let real: PlatformHost | null = null;
  const held: Array<[readonly PlatformCall[], (request: number, result: PlatformAnswer) => void]> = [];
  void ready.then((h) => {
    real = h;
    for (const [calls, answer] of held.splice(0)) h.handle(calls, answer);
  });
  return {
    host: "player",
    capabilities,
    get time() {
      return real?.time;
    },
    handle(calls, answer) {
      if (real) real.handle(calls, answer);
      else held.push([calls, answer]);
    },
  };
}

/** How long a framed page waits for a Player's hello before the browser's storage answers. */
export const HELLO_WAIT_MS = 1500;

/** The room ops the Player answers, by the engine's op name. */
const ROOM_OPS: Record<string, string> = { create: "room.create", join: "room.join", leave: "room.leave", send: "room.send" };

/**
 * The Player's rooms (docs/platform-asks.md #16), or null for a Player that has none. Ops go
 * to it in order without waiting (a send never waits for a join's answer: the Player queues
 * them); `push` takes the events it pushes. A refused op becomes an `error` event naming it.
 */
export function playerRooms(player: PlayerLink): (RoomHost & { push(event: Record<string, unknown>): void }) | null {
  if (!player.capabilities.includes("room.join")) return null;
  let deliver: ((events: unknown[]) => void) | null = null;
  return {
    host: "player",
    handle(ops, to) {
      deliver = to;
      for (const op of ops) {
        const name = String(op["op"]);
        const method = ROOM_OPS[name];
        const { op: _op, ...args } = op;
        if (!method || !player.capabilities.includes(method)) {
          to([{ type: "error", op: name, code: "unavailable", message: `this Player answers no ${method ?? name}` }]);
          continue;
        }
        void player.request(method, args).then(
          (r) => {
            if (!r.ok) deliver?.([{ type: "error", op: name, code: r.error.code, message: r.error.message }]);
          },
          (e: unknown) => deliver?.([{ type: "error", op: name, code: "offline", message: e instanceof Error ? e.message : String(e) }]),
        );
      }
    },
    push(event) {
      deliver?.([event]);
    },
  };
}
