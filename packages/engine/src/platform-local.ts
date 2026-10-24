// The local platform: og.platform's calls answered from a JSON document the
// host keeps (the native host's file, the browser's own storage). It is the
// native host's platform and the web shell's fallback when no Player
// answers. Store keys carry revisions and refuse a stale write with
// `conflict`; leaderboards keep each name's best and rank in the board's
// order; Origin Point and AI marks are recorded (a local balance, a list),
// never sent anywhere; telemetry is kept as the last 200 events.

/** One platform call, as og_platform_take lists it. */
export interface PlatformCall {
  readonly request: number;
  readonly method: string;
  readonly args: Record<string, unknown>;
  readonly tick: number;
}

/** An answer, as og_platform_answer takes it. */
export type PlatformAnswer = (
  | { readonly ok: true; readonly data: unknown }
  | { readonly ok: false; readonly error: { readonly code: string; readonly message: string; readonly retryable: boolean; readonly revision?: number } }
) & {
  /** The answering host's clock, UTC ms (a Player that sends its time); the engine's host clock fills it in otherwise. */
  readonly time?: number;
};

/** What answers an engine's platform calls. */
export interface PlatformHost {
  /** `player`, `local` or `none` (og.platform.info). */
  readonly host: string;
  readonly capabilities: readonly string[];
  /** The Player hello's UTC milliseconds, sampled when this host attaches. */
  readonly time?: number | undefined;
  /** Handle calls in order; answer every one but `telemetry`, now or later. */
  handle(calls: readonly PlatformCall[], answer: (request: number, result: PlatformAnswer) => void): void;
}

/** Every method the local platform answers. */
export const LOCAL_CAPABILITIES = ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "telemetry", "point", "ai.mark"] as const;

export interface LocalDocument {
  version: 1;
  player: string;
  store: Record<string, { value: unknown; revision: number }>;
  boards: Record<string, { order: "asc" | "desc"; entries: Array<{ name: string; score: number }> }>;
  points: { balance: number; grants: Array<{ reason: string; amount: number; tick: number }> };
  marks: Array<{ kind: string; label: string; model?: string; tick: number }>;
  events: Array<{ name: string; attrs: Record<string, unknown>; tick: number }>;
}

/** Where the document lives: a file natively, localStorage in a browser, memory in tests. */
export interface LocalPersistence {
  load(): string | null;
  save(text: string): void;
}

export function emptyDocument(player = "You"): LocalDocument {
  return { version: 1, player, store: {}, boards: {}, points: { balance: 0, grants: [] }, marks: [], events: [] };
}

function readDocument(text: string | null): LocalDocument {
  if (!text) return emptyDocument();
  try {
    const d = JSON.parse(text) as Partial<LocalDocument>;
    const e = emptyDocument(typeof d.player === "string" ? d.player : "You");
    return { ...e, ...d, points: { ...e.points, ...(d.points ?? {}) }, version: 1 } as LocalDocument;
  } catch {
    return emptyDocument();
  }
}

export const fail = (code: string, message: string): PlatformAnswer => ({
  ok: false,
  error: { code, message, retryable: ["offline", "timeout", "unavailable", "rate_limited"].includes(code) },
});

const okData = (data: unknown): PlatformAnswer => ({ ok: true, data });

/** A refused conditional write: `conflict`, with the revision the key is at now. */
export const conflict = (message: string, revision: number): PlatformAnswer => ({ ok: false, error: { code: "conflict", message, retryable: false, revision } });

/** Ranked entries of a board, best first. */
export function ranked(order: "asc" | "desc", entries: ReadonlyArray<{ name: string; score: number }>, me: string) {
  const sorted = [...entries].sort((a, b) => (order === "asc" ? a.score - b.score : b.score - a.score) || (a.name < b.name ? -1 : 1));
  return sorted.map((e, i) => ({ rank: i + 1, name: e.name, score: e.score, me: e.name === me }));
}

/** Answers one call against the document (mutating it). */
export function answerLocally(doc: LocalDocument, call: PlatformCall): PlatformAnswer | null {
  const a = call.args;
  switch (call.method) {
    case "store.get": {
      const key = String(a["key"]);
      const slot = doc.store[key];
      return okData({ key, value: slot?.value ?? null, revision: slot?.revision ?? 0 });
    }
    case "store.set": {
      const key = String(a["key"]);
      const want = Number(a["revision"] ?? 0);
      const slot = doc.store[key];
      const have = slot?.revision ?? 0;
      // `create`: only a key never written (the Player's revision 0).
      if (a["create"] === true ? have !== 0 : want > 0 && want !== have) return conflict(`'${key}' is at revision ${have}${a["create"] === true ? ", not new" : `, not ${want}`}: read it again and merge`, have);
      doc.store[key] = { value: a["value"] ?? null, revision: have + 1 };
      return okData({ key, revision: have + 1 });
    }
    case "store.list":
      return okData({ keys: Object.keys(doc.store).sort().map((key) => ({ key, revision: doc.store[key]!.revision })) });
    case "leaderboard.submit": {
      const name = String(a["board"] ?? "main");
      const board = (doc.boards[name] ??= { order: a["order"] === "asc" ? "asc" : "desc", entries: [] });
      const score = Number(a["score"]);
      const mine = board.entries.find((e) => e.name === doc.player);
      const better = (x: number, y: number) => (board.order === "asc" ? x < y : x > y);
      if (!mine) board.entries.push({ name: doc.player, score });
      else if (better(score, mine.score)) mine.score = score;
      const me = ranked(board.order, board.entries, doc.player).find((e) => e.me)!;
      return okData({ board: name, rank: me.rank, best: me.score });
    }
    case "leaderboard.top":
    case "leaderboard.around": {
      const name = String(a["board"] ?? "main");
      const board = doc.boards[name];
      const count = Math.max(1, Math.min(100, Number(a["count"] ?? 10)));
      const all = board ? ranked(board.order, board.entries, doc.player) : [];
      const me = all.find((e) => e.me) ?? null;
      let entries = all.slice(0, count);
      if (call.method === "leaderboard.around" && me) {
        const start = Math.max(0, Math.min(all.length - count, me.rank - 1 - Math.floor(count / 2)));
        entries = all.slice(start, start + count);
      }
      return okData({ board: name, order: board?.order ?? "desc", entries, me });
    }
    case "telemetry":
      doc.events.push({ name: String(a["name"]), attrs: (a["attrs"] as Record<string, unknown>) ?? {}, tick: call.tick });
      if (doc.events.length > 200) doc.events.splice(0, doc.events.length - 200);
      return null;
    case "point": {
      const amount = Math.max(1, Math.min(1000, Number(a["amount"] ?? 1)));
      doc.points.balance += amount;
      doc.points.grants.push({ reason: String(a["reason"]), amount, tick: call.tick });
      return okData({ granted: amount, balance: doc.points.balance });
    }
    case "ai.mark":
      doc.marks.push({ kind: String(a["kind"]), label: String(a["label"]), ...(typeof a["model"] === "string" ? { model: a["model"] } : {}), tick: call.tick });
      return okData({ marked: true });
    default:
      return fail("invalid", `unknown platform method '${call.method}'`);
  }
}

/** The local platform over a persistence: every call answered at once, the document saved after each batch. */
export function localPlatform(persistence: LocalPersistence = memoryPersistence()): PlatformHost & { document(): LocalDocument } {
  let doc = readDocument(persistence.load());
  return {
    host: "local",
    capabilities: LOCAL_CAPABILITIES,
    document: () => doc,
    handle(calls, answer) {
      doc = readDocument(persistence.load());
      const answers: Array<[number, PlatformAnswer]> = [];
      for (const c of calls) {
        const r = answerLocally(doc, c);
        if (r) answers.push([c.request, r]);
      }
      persistence.save(JSON.stringify(doc, null, 2));
      for (const [request, r] of answers) answer(request, r);
    },
  };
}

/** A document kept in memory only. */
export function memoryPersistence(initial: LocalDocument | null = null): LocalPersistence {
  let text: string | null = initial ? JSON.stringify(initial) : null;
  return { load: () => text, save: (t) => void (text = t) };
}

/** No platform at all: every call is `unavailable`. */
export const noPlatform: PlatformHost = {
  host: "none",
  capabilities: [],
  handle(calls, answer) {
    for (const c of calls) if (c.method !== "telemetry") answer(c.request, fail("unavailable", "no platform host is attached to this engine (the shell attaches the Player or local storage; og run a local store)"));
  },
};

/** The document in the browser's localStorage under `key` (memory only where storage is refused). */
export function storagePersistence(key: string): LocalPersistence {
  const store = (() => {
    try {
      return (globalThis as { localStorage?: { getItem(k: string): string | null; setItem(k: string, v: string): void } }).localStorage ?? null;
    } catch {
      return null;
    }
  })();
  let memory: string | null = null;
  return {
    load: () => {
      try {
        return store ? store.getItem(key) : memory;
      } catch {
        return memory;
      }
    },
    save: (text) => {
      memory = text;
      try {
        store?.setItem(key, text);
      } catch {
        /* storage full or refused: memory keeps it for this page */
      }
    },
  };
}
