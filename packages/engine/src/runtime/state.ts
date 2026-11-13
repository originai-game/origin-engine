import type { HostBinding } from "./host.ts";
import type { Running } from "./views.ts";

// ── state ─────────────────────────────────────────────────────────────────

/**
 * What the runtime remembers between calls, in one object: every part of
 * the runtime reads and writes this copy.
 */
export const rt = {
  /** The host a loader bound (bindHost), or null before one did. */
  host: null as HostBinding | null,
  /** What GameDesc.ui was refused, while its registration runs; null outside one. */
  uiRegistrationErrors: null as string[] | null,
  /**
   * Counts engine calls made through `abi()` and dispatches (game code entered
   * or left): the engine changes a resource only inside one of them, so a
   * resource handle's read made while the count holds is still the store's
   * value (recordHandle).
   */
  engineCalls: 0,
  /** The shared buffer as words, doubles, bytes and a view (words()); empty until first asked for and after a host change. */
  sharedWords: new Uint32Array(0) as Uint32Array,
  sharedFloats: new Float64Array(0) as Float64Array,
  sharedBytes: new Uint8Array(0) as Uint8Array,
  sharedView: new DataView(new ArrayBuffer(0)) as DataView,
  /**
   * Moves on whenever the engine may have written the buffer again (a call
   * into game code, a query's archetypes asked for), and when a system call
   * ends: what a system's query finds there is its own only while this holds.
   */
  sharedStamp: 0,
  /** The layout epoch the cached component numbers were read in. */
  componentIdsEpoch: 0,
  /** The root of the engine being created: GameDesc.start (and the boot scene's hooks) spawn before its id is known. */
  creatingRoot: undefined as string | undefined,
  /** Moves on whenever views may have gone stale: after every call into game code, and at every structural change outside systems. */
  epoch: 0,
  /** The system, run condition or hook game code is running in now, or null outside one. */
  running: null as Running | null,
  /** The engine and tick of the call into game code in progress (og.assert's), or null between calls. */
  calling: null as { readonly engine: number; readonly tick: number; readonly world: object } | null,
  /** Whether the bound host is a publish build (the probe's hello says), learnt when the first engine is made. */
  publishBuild: null as boolean | null,
  /** Calls into game code in progress (systems, commands, facts, start): writes wait for their end. */
  inGame: 0,
};
