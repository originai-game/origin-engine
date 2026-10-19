// The Stage overlay in the page's own document as the shell's Player
// (shell/player.ts, own-window peer; docs/web-export.md §Player protocol 2):
// a top-level page with no `__OG_GAME` that was told its own origin as the
// Stage host speaks Player protocol 2 to its own window by postMessage, and
// a stand-in overlay listening in that same window answers it as the
// platform's Stage overlay would (sdk/src/stage-page.ts: started, the scratch
// store's methods). Asserted are what reaches the overlay, what the shell
// takes from it (one hello, never its own echo), og.platform's answers with
// the overlay's `time`, the browser's storage left alone, foreign messages
// ignored, and the precedence: framed → parent, `__OG_GAME` or another
// origin → no postMessage Player at all.
import { expect, test } from "bun:test";
import { connectPlayer, type PlayerResult } from "../shell/player.ts";
import { playerPlatform } from "../shell/platform.ts";
import type { LocalPersistence, PlatformAnswer, PlatformCall } from "../src/platform-local.ts";

type Msg = Record<string, unknown>;
const g = globalThis as unknown as Record<string, unknown>;
const KEYS = ["location", "parent", "__OG_STAGE_HOST__", "__OG_GAME", "postMessage"] as const;
const SCRATCH = ["save.get", "save.read", "save.set", "store.get", "store.set", "store.list"];
let run = 0;
/** Puts the globals of the last page() back (each page() undoes the one before it). */
let restore: (() => void) | null = null;

/** A message event as the browser hands it to a window listener. */
function deliver(data: unknown, origin: string, source: unknown) {
  const e = new Event("message");
  Object.defineProperties(e, { data: { value: structuredClone(data) }, origin: { value: origin }, source: { value: source } });
  dispatchEvent(e);
}

/**
 * A window for one test: its own origin (so a shell left from an earlier test
 * hears nothing of this one), `parent` itself unless framed, and a
 * `postMessage` that delivers to this window only at its own origin.
 */
function page(opts: { stageHost?: (origin: string) => string | undefined; game?: unknown; framed?: boolean } = {}) {
  restore?.();
  const origin = `https://stage-${++run}.test`;
  const saved = KEYS.map((k) => [k, Object.getOwnPropertyDescriptor(g, k)] as const);
  const self: Msg[] = [];
  const toParent: Array<{ data: Msg; targetOrigin: string }> = [];
  const listeners: Array<(e: Event) => void> = [];
  g["location"] = { origin, pathname: "/s/ticket/" };
  g["parent"] = opts.framed ? { postMessage: (data: Msg, targetOrigin: string) => toParent.push({ data: structuredClone(data), targetOrigin }) } : globalThis;
  const host = opts.stageHost ? opts.stageHost(origin) : origin;
  if (host === undefined) delete g["__OG_STAGE_HOST__"];
  else g["__OG_STAGE_HOST__"] = host;
  if (opts.game === undefined) delete g["__OG_GAME"];
  else g["__OG_GAME"] = opts.game;
  // A post naming another origin (a shell left from an earlier test) is not delivered here.
  g["postMessage"] = (data: Msg, targetOrigin: string) => {
    if (targetOrigin !== origin) return;
    self.push(structuredClone(data));
    setTimeout(() => deliver(data, origin, globalThis), 0);
  };
  restore = () => {
    for (const l of listeners) removeEventListener("message", l);
    for (const [k, d] of saved) {
      if (d) Object.defineProperty(g, k, d);
      else delete g[k];
    }
  };
  /** Listens in this window as the overlay does. */
  const listen = (l: (e: MessageEvent) => void) => {
    const f = l as unknown as (e: Event) => void;
    listeners.push(f);
    addEventListener("message", f);
  };
  return { origin, self, toParent, listen };
}

/**
 * The stand-in Stage overlay: answers the game's hello with its own (started,
 * the listed capabilities) and its calls from memory, each result stamped with
 * its clock; it skips the Player messages it posted itself.
 */
function overlay(p: ReturnType<typeof page>, capabilities: readonly string[] = SCRATCH) {
  const heard: Msg[] = [];
  const store = new Map<string, { value: unknown; revision: number }>();
  let save: { data: unknown; revision: number } = { data: null, revision: 0 };
  let clock = 1_800_000_000_000;
  const post = (m: Msg) => (globalThis as unknown as Window).postMessage({ og: "player", protocol: 2, ...m }, p.origin);
  const answer = (method: string, a: Msg): Msg => {
    switch (method) {
      case "store.get": {
        const s = store.get(String(a["key"]));
        return { ok: true, data: { key: a["key"], value: s?.value ?? null, revision: s?.revision ?? 0 } };
      }
      case "store.set": {
        const s = store.get(String(a["key"]));
        const revision = (s?.revision ?? 0) + 1;
        store.set(String(a["key"]), { value: a["value"], revision });
        return { ok: true, data: { key: a["key"], revision } };
      }
      case "store.list":
        return { ok: true, data: { keys: [...store].map(([key, s]) => ({ key, revision: s.revision })) } };
      case "save.get":
        return { ok: true, data: save.data };
      case "save.read":
        return { ok: true, data: { data: save.data, revision: save.revision } };
      case "save.set":
        save = { data: a["data"], revision: save.revision + 1 };
        return { ok: true, data: { revision: save.revision } };
      default:
        return { ok: false, error: { code: "unavailable", message: `no ${method}`, retryable: false } };
    }
  };
  p.listen((e) => {
    if ((e.source as unknown) !== globalThis || e.origin !== p.origin) return;
    const m = e.data as Msg;
    if (m?.["og"] !== "player" || m["protocol"] !== 2) return;
    if (m["kind"] === "hello" && Array.isArray(m["capabilities"])) return; // its own
    if (m["kind"] === "result" || m["kind"] === "start" || m["kind"] === "gesture" || m["kind"] === "pause" || m["kind"] === "resume") return; // its own
    heard.push(m);
    if (m["kind"] === "hello") post({ kind: "hello", locale: "en", region: "global", capabilities, started: true, paused: false });
    if (m["kind"] === "call") {
      const method = String(m["method"]);
      if (capabilities.includes(method)) post({ kind: "result", id: m["id"], ...answer(method, (m["args"] ?? {}) as Msg), time: (clock += 1000) });
      else post({ kind: "result", id: m["id"], ok: false, error: { code: "unavailable", message: `no ${method}`, retryable: false } });
    }
  });
  return { heard, get clock() { return clock; } };
}

const until = async (ok: () => boolean, ms = 2000) => {
  for (const end = Date.now() + ms; !ok() && Date.now() < end; ) await new Promise((r) => setTimeout(r, 5));
  return ok();
};

/** A persistence that records every write: og.platform's browser storage. */
function watched(): LocalPersistence & { saves: number } {
  let text: string | null = null;
  const p = { saves: 0, load: () => text, save: (t: string) => void ((text = t), p.saves++) };
  return p;
}

async function platform(player: Parameters<typeof playerPlatform>[0], local: LocalPersistence, calls: Array<[string, Msg]>): Promise<PlatformAnswer[]> {
  const host = playerPlatform(player, local);
  const got = new Map<number, PlatformAnswer>();
  const list: PlatformCall[] = calls.map(([method, args], i) => ({ request: i + 1, method, args, tick: 0 }));
  host.handle(list, (r, a) => got.set(r, a));
  await until(() => got.size === list.length);
  return list.map((c) => got.get(c.request)!);
}

test("the Stage overlay in the page's own document is its Player: one hello, play started, never the shell's own echo", async () => {
  const p = page();
  const o = overlay(p);
  const hellos: Array<{ started: boolean; paused: boolean; capabilities: readonly string[] }> = [];
  const player = connectPlayer({ hello: (m) => hellos.push(m) });
  expect(player.peer).toBe("self");
  expect(await until(() => player.hosted)).toBe(true);
  await new Promise((r) => setTimeout(r, 600)); // past any repeated hello
  expect(hellos).toEqual([{ started: true, paused: false, capabilities: SCRATCH }]);
  expect(player.started).toBe(true);
  expect(o.heard.filter((m) => m["kind"] === "hello")).toEqual([{ og: "player", protocol: 2, kind: "hello", substrate: "origin-engine" }]);
  // Readiness reaches the overlay in the same window.
  player.ready();
  expect(await until(() => o.heard.some((m) => m["kind"] === "ready"))).toBe(true);
  expect(p.toParent).toEqual([]);
});

test("og.platform's store is the overlay's, its time passes on, and the browser's storage is never written", async () => {
  const p = page();
  const o = overlay(p);
  const player = connectPlayer({});
  expect(await until(() => player.hosted)).toBe(true);
  const local = watched();
  const a = await platform(player, local, [
    ["store.set", { key: "best", value: 555 }],
    ["store.get", { key: "best" }],
    ["store.list", {}],
  ]);
  expect(a).toEqual([
    { ok: true, data: { key: "best", revision: 1 }, time: 1_800_000_001_000 },
    { ok: true, data: { key: "best", value: 555, revision: 1 }, time: 1_800_000_002_000 },
    { ok: true, data: { keys: [{ key: "best", revision: 1 }] }, time: 1_800_000_003_000 },
  ]);
  expect(o.heard.filter((m) => m["kind"] === "call").map((m) => [m["method"], m["args"] ?? null])).toEqual([
    ["store.set", { key: "best", value: 555 }],
    ["store.get", { key: "best" }],
    ["store.list", null],
  ]);
  expect(local.saves).toBe(0);
});

test("an overlay with the save alone: the store in the save, each answer with the time of the call that settled it", async () => {
  const p = page();
  overlay(p, ["save.read", "save.set"]);
  const player = connectPlayer({});
  expect(await until(() => player.hosted)).toBe(true);
  const local = watched();
  const a = await platform(player, local, [
    ["store.set", { key: "best", value: 7 }],
    ["store.get", { key: "best" }],
  ]);
  // store.set: save.read (1 000) then save.set (2 000); store.get: save.read (3 000).
  expect(a).toEqual([
    { ok: true, data: { key: "best", revision: 1 }, time: 1_800_000_002_000 },
    { ok: true, data: { key: "best", value: 7, revision: 1 }, time: 1_800_000_003_000 },
  ]);
  expect(local.saves).toBe(0);
});

test("an overlay whose listener comes up after the shell still hears a hello", async () => {
  const p = page();
  const player = connectPlayer({});
  await new Promise((r) => setTimeout(r, 400));
  expect(player.hosted).toBe(false);
  const o = overlay(p);
  expect(await until(() => player.hosted, 1500)).toBe(true);
  expect(player.capabilities).toEqual(SCRATCH);
  expect(o.heard.some((m) => m["kind"] === "hello")).toBe(true);
});

test("a hello or result from another origin or another window is not the Player's", async () => {
  const p = page();
  const player = connectPlayer({});
  const forged = { og: "player", protocol: 2, kind: "hello", locale: "en", region: "global", capabilities: SCRATCH, started: true, paused: false };
  deliver(forged, "https://elsewhere.test", globalThis);
  deliver(forged, p.origin, { postMessage() {} });
  const call = player.request("store.get", { key: "best" }, 300);
  deliver({ og: "player", protocol: 2, kind: "result", id: "oe-1", ok: true, data: { value: 1 }, time: 1 }, "https://elsewhere.test", globalThis);
  deliver({ og: "player", protocol: 2, kind: "result", id: "oe-1", ok: true, data: { value: 1 }, time: 1 }, p.origin, { postMessage() {} });
  const r: PlayerResult = await call;
  expect(r.ok).toBe(false);
  expect(r.ok ? null : r.error.code).toBe("timeout");
  expect(player.hosted).toBe(false);
  expect(player.started).toBe(false);
  // Its own posts named its own origin and went to its own window.
  expect(p.self.map((m) => m["kind"])).toContain("call");
});

test("precedence: framed speaks to the parent; __OG_GAME, another Stage origin or none leave no postMessage Player", async () => {
  {
    const p = page({ framed: true });
    const player = connectPlayer({});
    expect(player.peer).toBe("parent");
    expect(p.toParent).toEqual([{ data: { og: "player", protocol: 2, kind: "hello", substrate: "origin-engine" }, targetOrigin: p.origin }]);
    expect(p.self).toEqual([]);
  }
  for (const opts of [{ game: { id: "g", releaseId: "r", protocol: 2 } }, { stageHost: () => "https://stage.elsewhere.test" }, { stageHost: () => undefined }]) {
    const p = page(opts);
    const o = overlay(p);
    const player = connectPlayer({});
    expect(player.peer).toBe(null);
    const r = await player.request("store.get", { key: "best" });
    expect(r.ok ? null : r.error.code).toBe("unavailable");
    await new Promise((r) => setTimeout(r, 300));
    expect(p.self).toEqual([]);
    expect(o.heard).toEqual([]);
    expect(player.hosted).toBe(false);
  }
  restore?.();
  restore = null;
});
