// og.platform on the web: the shell's answer to the engine Worker's
// platform calls. With a Player framing the page, each call goes to it over
// Player protocol 2 (./player.ts, origin-checked postMessage; the Player
// owns every credential); without one, or where the Player has no such
// method, the browser's own storage answers (../src/platform-local.ts over
// localStorage). The mapping, and what protocol 2 lacks, are in
// docs/web-export.md §Platform services and docs/platform-asks.md.
//
//   store.get/set/list   save.get / save.set: the store lives in the save's
//                        `store` object, each key with its own revision; a
//                        conditional write checks the key's revision here
//                        (protocol 2's save.get carries no revision of its
//                        own). Without save.* (a verification run), or
//                        `unavailable`, the browser's storage.
//   leaderboard.submit   leaderboard.submit { score, board, order }
//   leaderboard.top      leaderboard.top { n, board }
//   leaderboard.around   leaderboard.top { n: 100 }, cut around `me`
//   telemetry            an `event` named game.custom.<name>
//   point, ai.mark       `unavailable`: protocol 2 has no such methods

import { answerLocally, fail, localPlatform, type LocalDocument, type LocalPersistence, type PlatformAnswer, type PlatformCall, type PlatformHost } from "../src/platform-local.ts";
export { storagePersistence } from "../src/platform-local.ts";
import type { PlayerResult } from "./player.ts";

/** The part of the Player link this needs (connectPlayer's answer). */
export interface PlayerLink {
  readonly framed: boolean;
  readonly hosted: boolean;
  readonly capabilities: readonly string[];
  request(method: string, args?: Record<string, unknown>): Promise<PlayerResult>;
  event(name: string, attrs?: Record<string, string | number | boolean | null>): void;
}

type Save = { store?: Record<string, { value: unknown; revision: number }> } & Record<string, unknown>;

function asSave(data: unknown): Save {
  return data && typeof data === "object" && !Array.isArray(data) ? (data as Save) : {};
}

/** The Player's answer as og.platform's. */
const fromPlayer = (r: PlayerResult, data: (d: unknown) => unknown): PlatformAnswer => (r.ok ? { ok: true, data: data(r.data) } : { ok: false, error: { code: r.error.code, message: r.error.message, retryable: r.error.retryable } });

const STORE = ["store.get", "store.set", "store.list"];

/** The Player-backed platform, falling back to `local` (the browser's storage). */
export function playerPlatform(player: PlayerLink, local: LocalPersistence): PlatformHost {
  const fallback = localPlatform(local);
  const has = (m: string) => player.capabilities.includes(m);
  const capabilities = [...STORE, "telemetry", ...(has("leaderboard.submit") ? ["leaderboard.submit"] : []), ...(has("leaderboard.top") ? ["leaderboard.top", "leaderboard.around"] : [])];
  let queue: Promise<void> = Promise.resolve();

  const viaLocal = (call: PlatformCall): PlatformAnswer | null => {
    let out: PlatformAnswer | null = null;
    fallback.handle([call], (_r, a) => (out = a));
    return out;
  };

  const one = async (call: PlatformCall): Promise<PlatformAnswer | null> => {
    const a = call.args;
    if (STORE.includes(call.method)) {
      if (!has("save.get") || !has("save.set")) return viaLocal(call);
      const got = await player.request("save.get");
      if (!got.ok) return got.error.code === "unavailable" ? viaLocal(call) : fromPlayer(got, () => null);
      const save = asSave(got.data);
      const doc: LocalDocument = { version: 1, player: "", store: { ...(save.store ?? {}) }, boards: {}, points: { balance: 0, grants: [] }, marks: [], events: [] };
      const answer = answerLocally(doc, call);
      if (call.method !== "store.set" || !answer?.ok) return answer;
      const put = await player.request("save.set", { data: { ...save, store: doc.store } });
      if (!put.ok) return fromPlayer(put, () => null);
      const queued = (put.data as { queued?: boolean } | null)?.queued === true;
      return { ok: true, data: { ...(answer.data as object), ...(queued ? { queued: true } : {}) } };
    }
    switch (call.method) {
      case "telemetry":
        player.event(`game.custom.${String(a["name"])}`, a["attrs"] as Record<string, string | number | boolean | null>);
        return null;
      case "leaderboard.submit":
        if (!has("leaderboard.submit")) return fail("unavailable", "this Player answers no leaderboard.submit here");
        return fromPlayer(await player.request("leaderboard.submit", { score: a["score"], board: a["board"], order: a["order"] }), (d) => {
          const r = (d ?? {}) as { rank?: number | null; queued?: boolean };
          return { board: a["board"], rank: r.rank ?? null, best: null, ...(r.queued ? { queued: true } : {}) };
        });
      case "leaderboard.top":
      case "leaderboard.around": {
        if (!has("leaderboard.top")) return fail("unavailable", "this Player answers no leaderboard.top here");
        const count = Number(a["count"] ?? 10);
        const around = call.method === "leaderboard.around";
        return fromPlayer(await player.request("leaderboard.top", { n: around ? 100 : count, board: a["board"] }), (d) => {
          const t = (d ?? {}) as { board?: string; sort?: string; top?: Array<{ rank: number; name: string; score: number }>; me?: { rank: number } | null };
          const me = t.me ?? null;
          let entries = (t.top ?? []).map((e) => ({ ...e, me: !!me && e.rank === me.rank }));
          if (around && me) {
            const at = entries.findIndex((e) => e.me);
            const start = Math.max(0, Math.min(entries.length - count, (at < 0 ? entries.length : at) - Math.floor(count / 2)));
            entries = entries.slice(start, start + count);
          } else entries = entries.slice(0, count);
          return { board: t.board ?? a["board"], order: t.sort === "asc" ? "asc" : "desc", entries, me };
        });
      }
      default:
        return fail("unavailable", `the Player has no ${call.method} method in protocol 2 (docs/platform-asks.md)`);
    }
  };

  return {
    host: "player",
    capabilities,
    handle(calls, answer) {
      // In order: a store write reads the save first, so calls never interleave.
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
    handle(calls, answer) {
      if (real) real.handle(calls, answer);
      else held.push([calls, answer]);
    },
  };
}

/** How long a framed page waits for a Player's hello before the browser's storage answers. */
export const HELLO_WAIT_MS = 1500;
