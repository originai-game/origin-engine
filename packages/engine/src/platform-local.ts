// The local platform: og.platform's calls answered from a JSON document the
// host keeps (the native host's file, the browser's own storage). It is the
// native host's platform and the web shell's fallback when no Player
// answers. Store keys carry revisions and refuse a stale write with
// `conflict`; leaderboards keep each name's best and rank in the board's
// order; Origin Point and AI marks are recorded (a local balance, a list),
// never sent anywhere; telemetry is kept as the last 200 events.
//
// Purchases, ads, achievements, the player, friends, invites and share
// (docs/platform-asks.md #19) are the platform's. Natively, in playtests and
// on a dev page without a Player, the local platform fakes them
// deterministically (LocalFake): outcomes come from per-method queues, never
// from chance, and what they leave behind (purchases, achievements) is kept
// in the document, so a relaunch on the same store sees it. Without a fake
// (a published page with no Player) they answer `unavailable`: a page never
// grants a purchase nobody paid for.

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

/** Unasked host input (og_platform_notify): applied at the next tick's start as og.PlatformNotice. */
export interface PlatformNotice {
  readonly kind: "purchase" | "entitlements" | "invite";
  readonly data: unknown;
  readonly time?: number;
}

/** What a platform host may do to the engine it answers, besides answering. */
export interface PlatformLink {
  /** The engine's tick now (read between ticks). */
  readonly tick: number;
  /** Answer one call (as `handle`'s `answer`). */
  answer(request: number, result: PlatformAnswer): void;
  /** Send a notice. */
  notify(notice: PlatformNotice): void;
  /** Pause the game through the host's lifecycle (no ticks, no audio), as while an ad shows. */
  pause(): void;
  /** Resume it. */
  resume(): void;
}

/** What answers an engine's platform calls. */
export interface PlatformHost {
  /** `player`, `local` or `none` (og.platform.info). */
  readonly host: string;
  readonly capabilities: readonly string[];
  /** The Player hello's UTC milliseconds, sampled when this host attaches. */
  readonly time?: number | undefined;
  /** Handle calls in order; answer every one but `telemetry`, now or later. */
  handle(calls: readonly PlatformCall[], answer: (request: number, result: PlatformAnswer) => void, link?: PlatformLink): void;
  /** Called on every pump, with calls or without: a host that answers later or sends notices on its own. */
  poll?(link: PlatformLink): void;
}

/** The store, leaderboards, telemetry, Origin Point and AI marks: what the local platform always answers. */
export const LOCAL_CAPABILITIES = ["store.get", "store.set", "store.list", "leaderboard.submit", "leaderboard.top", "leaderboard.around", "telemetry", "point", "ai.mark"] as const;

/** Purchases, ads, achievements, the player, friends, invites and share (docs/platform-asks.md #19): the platform's, faked locally only with a LocalFake. */
export const SERVICE_METHODS = [
  "purchase.products",
  "purchase.buy",
  "purchase.entitlements",
  "purchase.consume",
  "ad.ready",
  "ad.show",
  "achievement.unlock",
  "achievement.progress",
  "achievement.list",
  "player.get",
  "friends.list",
  "invite.send",
  "invite.list",
  "share",
] as const;

/** Every error code a platform answer may carry. */
export const ERROR_CODES = ["offline", "timeout", "invalid", "unauthorized", "not_found", "conflict", "too_large", "rate_limited", "unavailable", "failed", "cancelled", "no_fill"] as const;

/** A product the fake sells. */
export interface FakeProduct {
  readonly id: string;
  /** Default `consumable`. */
  readonly kind?: "consumable" | "durable" | "subscription";
  readonly title?: string;
  readonly description?: string;
  /** Default `{ amount: 0.99, currency: "USD" }`. */
  readonly price?: { readonly amount: number; readonly currency: string };
}

/**
 * The local platform's fake of the platform services, for `og run --platform-fake <file.json>`
 * and PlaytestDesc.platform: every outcome scripted, none by chance.
 */
export interface LocalFake {
  /** What the game sells; omitted: any product id is a consumable at 0.99 USD. */
  readonly products?: readonly FakeProduct[];
  /**
   * Per method, the outcomes of its calls in order, then the default (success): `ok`, an error
   * code (`cancelled`, `no_fill`, `unavailable`, ...), `pending` for purchase.buy (settled
   * completed by a `purchase` notice at the next pump), `completed`, `skipped` or `shown` for
   * ad.show, `not_ready` for ad.ready. `{ "purchase.buy": ["cancelled", "ok"] }`.
   */
  readonly outcomes?: Readonly<Record<string, readonly string[]>>;
  /** player.get's answer; default `{ player: "local-player", name: <the store's player>, guest: false }`. */
  readonly player?: { readonly player?: string; readonly name?: string; readonly guest?: boolean };
  /** friends.list's friends. */
  readonly friends?: ReadonlyArray<{ readonly player: string; readonly name: string; readonly online?: boolean; readonly playing?: boolean }>;
  /** invite.list's invites. */
  readonly invites?: ReadonlyArray<{ readonly invite: string; readonly from: { readonly player: string; readonly name: string }; readonly payload?: unknown; readonly opened?: boolean }>;
  /** Notices sent once the engine reaches `tick` (between ticks: applied at the next one's start). */
  readonly notices?: ReadonlyArray<PlatformNotice & { readonly tick: number }>;
  /** Methods this fake host lacks: left out of its capabilities and answered `unavailable`. */
  readonly unavailable?: readonly string[];
  /** Pumps an ad holds the game paused before its answer (default 2: a host frame). */
  readonly adPumps?: number;
}

export interface LocalDocument {
  version: 1;
  player: string;
  store: Record<string, { value: unknown; revision: number }>;
  boards: Record<string, { order: "asc" | "desc"; entries: Array<{ name: string; score: number }> }>;
  points: { balance: number; grants: Array<{ reason: string; amount: number; tick: number }> };
  marks: Array<{ kind: string; label: string; model?: string; tick: number }>;
  events: Array<{ name: string; attrs: Record<string, unknown>; tick: number }>;
  /** The fake's: purchases made (consumed or not), with the next id's number. */
  purchases?: Array<{ purchase: string; product: string; kind: string; quantity: number; state: "completed" | "pending"; consumed: boolean; tick: number }>;
  achievements?: Record<string, { percent: number; unlocked: boolean; tick: number | null }>;
  shares?: Array<{ text: string; title: string; path: string; image: boolean; tick: number }>;
  invitesSent?: Array<{ invite: string; to: string | null; message: string; payload: unknown; tick: number }>;
  ads?: Array<{ kind: string; placement: string; outcome: string; tick: number }>;
  seq?: number;
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

const SERVICES: ReadonlySet<string> = new Set(SERVICE_METHODS);

/** What a fake call leaves to do later: an ad's answer while the game is paused, a pending purchase to settle. */
type Later = { kind: "ad"; request: number; result: PlatformAnswer; pumps: number; fresh: boolean } | { kind: "settle"; purchase: string; fresh: boolean };

type FakeCheckpoint = { used: [string, number][]; later: Later[]; sent: number[] };

/** What a local platform holds, to put back with its `restore`: the persisted document and the fake's state. */
export type LocalCheckpoint = { text: string | null; fake: FakeCheckpoint | null };

/** The fake's state for one host: the outcome queues' positions, what waits for a later pump, the notices sent. */
class Fake {
  readonly #fake: LocalFake;
  readonly #used = new Map<string, number>();
  readonly later: Later[] = [];
  readonly #sent = new Set<number>();
  constructor(fake: LocalFake) {
    this.#fake = fake;
  }

  /** Its state, to put back with `restore` (a fork of a run). */
  checkpoint(): FakeCheckpoint {
    return { used: [...this.#used], later: JSON.parse(JSON.stringify(this.later)) as Later[], sent: [...this.#sent] };
  }

  restore(c: FakeCheckpoint): void {
    this.#used.clear();
    for (const [k, v] of c.used) this.#used.set(k, v);
    this.later.splice(0, this.later.length, ...(JSON.parse(JSON.stringify(c.later)) as Later[]));
    this.#sent.clear();
    for (const n of c.sent) this.#sent.add(n);
  }

  get capabilities(): string[] {
    const off = new Set(this.#fake.unavailable ?? []);
    return [...LOCAL_CAPABILITIES, ...SERVICE_METHODS].filter((m) => !off.has(m));
  }

  /** The next scripted outcome of `method`, or undefined (the default). */
  #next(method: string): string | undefined {
    const queue = this.#fake.outcomes?.[method];
    const at = this.#used.get(method) ?? 0;
    this.#used.set(method, at + 1);
    return queue?.[at];
  }

  #product(id: string): Required<Omit<FakeProduct, "price">> & { price: { amount: number; currency: string; display: string } } | null {
    const list = this.#fake.products;
    const p = list ? list.find((x) => x.id === id) : { id };
    if (!p) return null;
    const price = p.price ?? { amount: 0.99, currency: "USD" };
    return { id: p.id, kind: p.kind ?? "consumable", title: p.title ?? p.id, description: p.description ?? "", price: { ...price, display: `${price.amount.toFixed(2)} ${price.currency}` } };
  }

  /** Answers one service call against `doc` (null: answered later, by `poll`). */
  answer(doc: LocalDocument, call: PlatformCall, link: PlatformLink | undefined): PlatformAnswer | null {
    const a = call.args;
    if ((this.#fake.unavailable ?? []).includes(call.method)) return fail("unavailable", `this host has no ${call.method} (the fake's unavailable list)`);
    const outcome = this.#next(call.method);
    if (outcome && (ERROR_CODES as readonly string[]).includes(outcome)) return fail(outcome, `${call.method}: ${outcome} (the fake's scripted outcome)`);
    const purchases = (doc.purchases ??= []);
    const id = (prefix: string) => `${prefix}-${(doc.seq = (doc.seq ?? 0) + 1)}`;
    switch (call.method) {
      case "purchase.products": {
        const ids = (a["ids"] as string[] | undefined) ?? [];
        const all = ids.length > 0 ? ids : (this.#fake.products ?? []).map((p) => p.id);
        return okData({ products: all.map((x) => this.#product(x)).filter((p) => p !== null) });
      }
      case "purchase.buy": {
        const product = this.#product(String(a["product"]));
        if (!product) return fail("not_found", `no product '${String(a["product"])}' in this game's store`);
        if (product.kind !== "consumable" && purchases.some((p) => p.product === product.id && !p.consumed)) return fail("conflict", `'${product.id}' is owned already`);
        const quantity = product.kind === "consumable" ? Number(a["quantity"] ?? 1) : 1;
        const state = outcome === "pending" ? "pending" : "completed";
        const purchase = id("purchase");
        purchases.push({ purchase, product: product.id, kind: product.kind, quantity, state, consumed: false, tick: call.tick });
        if (state === "pending") this.later.push({ kind: "settle", purchase, fresh: true });
        return okData({ purchase, product: product.id, quantity, state });
      }
      case "purchase.entitlements":
        return okData({ entitlements: entitlementsOf(doc) });
      case "purchase.consume": {
        const p = purchases.find((x) => x.purchase === a["purchase"] && x.state === "completed" && !x.consumed && x.kind === "consumable");
        if (!p) return fail("not_found", `no unconsumed consumable purchase '${String(a["purchase"])}'`);
        p.consumed = true;
        return okData({ purchase: p.purchase, product: p.product, consumed: true });
      }
      case "ad.ready":
        return okData({ kind: a["kind"], placement: a["placement"], ready: outcome !== "not_ready" });
      case "ad.show": {
        const kind = String(a["kind"]);
        const shown = outcome === "skipped" || outcome === "completed" || outcome === "shown" ? outcome : kind === "interstitial" ? "shown" : "completed";
        (doc.ads ??= []).push({ kind, placement: String(a["placement"]), outcome: shown, tick: call.tick });
        const result = okData({ kind, placement: a["placement"], outcome: shown });
        // The game is paused while the ad shows; the answer comes after it resumed.
        if (!link) return result;
        link.pause();
        this.later.push({ kind: "ad", request: call.request, result, pumps: Math.max(1, this.#fake.adPumps ?? 2), fresh: true });
        return null;
      }
      case "achievement.unlock":
      case "achievement.progress": {
        const key = String(a["id"]);
        const list = (doc.achievements ??= {});
        const was = list[key] ?? { percent: 0, unlocked: false, tick: null };
        const percent = call.method === "achievement.unlock" ? 100 : Math.max(was.percent, Number(a["percent"] ?? 0));
        const unlocked = was.unlocked || percent >= 100;
        list[key] = { percent, unlocked, tick: unlocked ? (was.tick ?? call.tick) : null };
        return okData({ id: key, percent, unlocked, new: unlocked && !was.unlocked });
      }
      case "achievement.list":
        return okData({ achievements: Object.keys(doc.achievements ?? {}).sort().map((k) => ({ id: k, percent: doc.achievements![k]!.percent, unlocked: doc.achievements![k]!.unlocked, time: null })) });
      case "player.get":
        return okData({ player: this.#fake.player?.player ?? "local-player", name: this.#fake.player?.name ?? doc.player, guest: this.#fake.player?.guest ?? false });
      case "friends.list":
        if (this.#fake.player?.guest) return fail("unauthorized", "a guest has no friends list");
        return okData({ friends: (this.#fake.friends ?? []).slice(0, Number(a["count"] ?? 50)).map((f) => ({ player: f.player, name: f.name, online: f.online ?? false, playing: f.playing ?? false })) });
      case "invite.send": {
        const invite = id("invite");
        const to = typeof a["to"] === "string" ? a["to"] : null;
        (doc.invitesSent ??= []).push({ invite, to, message: String(a["message"] ?? ""), payload: a["payload"] ?? null, tick: call.tick });
        return okData({ invite, to });
      }
      case "invite.list":
        return okData({ invites: (this.#fake.invites ?? []).map((i) => ({ invite: i.invite, from: i.from, payload: i.payload ?? null, time: null, opened: i.opened ?? false })) });
      case "share":
        (doc.shares ??= []).push({ text: String(a["text"]), title: String(a["title"] ?? ""), path: String(a["path"] ?? ""), image: a["image"] === true, tick: call.tick });
        return okData({ shared: true, target: null });
      default:
        return fail("invalid", `unknown platform method '${call.method}'`);
    }
  }

  /** What waits: ads released after their pumps (resuming the game first), pending purchases settled, notices due. */
  poll(doc: LocalDocument, link: PlatformLink): boolean {
    let changed = false;
    for (const l of this.later.splice(0)) {
      // Nothing is released in the pump that made it: the game runs at least one frame paused.
      if (l.fresh) {
        l.fresh = false;
        this.later.push(l);
        continue;
      }
      if (l.kind === "ad") {
        if (--l.pumps > 0) {
          this.later.push(l);
          continue;
        }
        link.resume();
        link.answer(l.request, l.result);
      } else {
        const p = doc.purchases?.find((x) => x.purchase === l.purchase);
        if (!p) continue;
        p.state = "completed";
        changed = true;
        link.notify({ kind: "purchase", data: { purchase: p.purchase, product: p.product, quantity: p.quantity, state: "completed" } });
      }
    }
    (this.#fake.notices ?? []).forEach((n, i) => {
      if (this.#sent.has(i) || link.tick < n.tick) return;
      this.#sent.add(i);
      link.notify({ kind: n.kind, data: n.data, ...(n.time === undefined ? {} : { time: n.time }) });
    });
    return changed;
  }
}

/** The entitlements a document holds: owned durables and subscriptions, unconsumed consumables. */
function entitlementsOf(doc: LocalDocument) {
  return (doc.purchases ?? []).filter((p) => p.state === "completed" && !p.consumed).map((p) => ({ purchase: p.purchase, product: p.product, kind: p.kind, quantity: p.quantity, expires: null }));
}

/** The local platform over a persistence: every call answered at once (an ad after a paused frame), the document saved after each batch. With `fake`, it fakes the platform services; without, they are `unavailable`. */
export function localPlatform(persistence: LocalPersistence = memoryPersistence(), fake?: LocalFake): PlatformHost & { document(): LocalDocument; checkpoint(): LocalCheckpoint; restore(c: LocalCheckpoint): void } {
  let doc = readDocument(persistence.load());
  const faked = fake ? new Fake(fake) : null;
  return {
    host: "local",
    capabilities: faked ? faked.capabilities : LOCAL_CAPABILITIES,
    document: () => doc,
    // Every call is answered within its batch, so nothing is in flight between ticks: the
    // document and the fake's state are the whole platform (og run --forks).
    checkpoint: () => ({ text: persistence.load(), fake: faked?.checkpoint() ?? null }),
    restore(c) {
      if (c.text !== null) persistence.save(c.text);
      doc = readDocument(persistence.load());
      if (faked && c.fake) faked.restore(c.fake);
    },
    handle(calls, answer, link) {
      doc = readDocument(persistence.load());
      const answers: Array<[number, PlatformAnswer]> = [];
      for (const c of calls) {
        const r = SERVICES.has(c.method)
          ? faked
            ? faked.answer(doc, c, link)
            : fail("unavailable", `no platform answers ${c.method} here: a page without a Player sells, shows and shares nothing`)
          : answerLocally(doc, c);
        if (r) answers.push([c.request, r]);
      }
      persistence.save(JSON.stringify(doc, null, 2));
      for (const [request, r] of answers) answer(request, r);
    },
    ...(faked
      ? {
          poll(link: PlatformLink) {
            if (faked.later.length === 0 && !(fake!.notices ?? []).length) return;
            doc = readDocument(persistence.load());
            if (faked.poll(doc, link)) persistence.save(JSON.stringify(doc, null, 2));
          },
        }
      : {}),
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
