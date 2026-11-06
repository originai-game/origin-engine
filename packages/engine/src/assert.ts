// og.assert (IDL `assert`): the game's assertions, recorded into the run's
// report beside the world, never in it. A call that passes after its first
// is a Map lookup here, with no call into the engine: the engine hears of a
// site's first call in a report, each violation, and its first firing.
//
// The build (src/node/build.ts) writes each `og.assert.<kind>(` call in the
// game's own files as `og.assert.<kind>.call("<file>:<line>", ...)`, so the
// member reads its place from `this`, and lists every such site with its
// name in `globalThis.__og_assert_sites`, so a site that never runs is in
// the report too (and a sometimes there is uncovered).

import type { HostAbi } from "./generated/abi.ts";
import { boundHost, currentCall, isPublishBuild } from "./runtime.ts";

type Kind = "always" | "sometimes" | "reachable" | "unreachable" | "each";

/** A site the build found: its kind, its name when the call names it with a literal, its place. */
interface Declared {
  readonly kind: Kind;
  readonly name: string;
  readonly location: string;
}

/** One engine's report, as far as this runtime knows it. */
interface Report {
  epoch: number;
  tick: number;
  /** The sites (and each's buckets) the engine has heard of in this report. */
  readonly seen: Set<string>;
  /** Those it has heard fire. */
  readonly fired: Set<string>;
  /** How many of the build's sites it was told of. */
  declared: number;
}

// By the engine's world object: a destroyed engine's id may be taken again by a new one.
const reports = new WeakMap<object, Report>();

function abi(): HostAbi<Uint32Array, unknown> | null {
  return boundHost()?.abi ?? null;
}

function declaredSites(): readonly Declared[] {
  return ((globalThis as { __og_assert_sites?: Declared[] }).__og_assert_sites ?? []) as readonly Declared[];
}

function send(engine: number, record: unknown): number {
  let text: string;
  try {
    text = JSON.stringify(record);
  } catch {
    // Details that do not serialise (a cycle) are left out, never the call.
    text = JSON.stringify({ ...(record as object), details: "(details do not serialise)" });
  }
  return abi()!.og_assert_record(engine, text);
}

/** The engine's report, its epoch refreshed once a tick (the probe's take starts a new one between ticks). */
function reportOf(engine: number, world: object, tick: number): Report {
  let r = reports.get(world);
  if (!r) {
    r = { epoch: -1, tick: -1, seen: new Set(), fired: new Set(), declared: 0 };
    reports.set(world, r);
  }
  if (r.tick !== tick) {
    r.tick = tick;
    const epoch = abi()!.og_assert_record(engine, "null");
    if (epoch !== r.epoch) {
      r.epoch = epoch;
      r.seen.clear();
      r.fired.clear();
    }
  }
  const sites = declaredSites();
  for (; r.declared < sites.length; r.declared++) {
    const s = sites[r.declared]!;
    send(engine, { ...s, condition: true, key: "", details: null, tick, declared: true });
  }
  return r;
}

function assert(kind: Kind, location: unknown, name: string, condition: boolean, key: string, details: unknown): void {
  const call = currentCall();
  if (call === null || isPublishBuild() !== false || abi() === null) return;
  const r = reportOf(call.engine, call.world, call.tick);
  const site = `${kind}\u0000${typeof location === "string" ? location : ""}\u0000${name}\u0000${key}`;
  const held = !!condition;
  const violated = (kind === "always" && !held) || kind === "unreachable";
  const first = !r.seen.has(site);
  const fires = held && kind !== "always" && kind !== "unreachable" && !r.fired.has(site);
  if (!violated && !first && !fires) return;
  r.seen.add(site);
  if (fires) r.fired.add(site);
  send(call.engine, {
    kind,
    name,
    location: typeof location === "string" ? location : "",
    condition: held,
    key,
    details: details === undefined ? null : details,
    tick: call.tick,
    declared: false,
  });
}

const keyText = (key: unknown): string => (typeof key === "string" ? key : JSON.stringify(key) ?? String(key));

export const assertModule = Object.freeze({
  always(this: unknown, condition: boolean, name: string, details?: unknown) {
    assert("always", this, name, condition, "", details);
  },
  sometimes(this: unknown, condition: boolean, name: string, details?: unknown) {
    assert("sometimes", this, name, condition, "", details);
  },
  reachable(this: unknown, name: string, details?: unknown) {
    assert("reachable", this, name, true, "", details);
  },
  unreachable(this: unknown, name: string, details?: unknown) {
    assert("unreachable", this, name, true, "", details);
  },
  each(this: unknown, key: unknown, condition: boolean, name: string) {
    assert("each", this, name, condition, keyText(key), null);
  },
  record: (world: { readonly id: number }, record: unknown) =>
    abi()?.og_assert_record(world.id, record === null || record === undefined ? "null" : JSON.stringify(record)) ?? 0,
});
