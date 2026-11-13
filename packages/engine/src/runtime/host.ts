import bindings from "../../types/bindings.json" with { type: "json" };
import { type HostAbi, SYMBOLS, checkEntities } from "../generated/abi.ts";
import { forgetComponents } from "./components.ts";
import { dispatch } from "./engine.ts";
import { dirtyRecords, flushRecords } from "./resources.ts";
import { refused } from "./result.ts";
import { rt } from "./state.ts";

// ── host binding ──────────────────────────────────────────────────────────

/** A typed array of the column kinds' element types. */
export type ColumnArray = Float32Array | Float64Array | Uint32Array | Int32Array | Uint8Array;

/** How a loader hands the runtime its host. */
export interface HostBinding {
  /** The exported binding symbols. */
  readonly abi: HostAbi<Uint32Array, unknown>;
  /** The host's name, for messages: `web` or `node`. */
  readonly name: string;
  /**
   * Typed arrays over what `og_engine_view_column` answered, one per
   * field: `kinds[i]` is the field's column kind, `count` the entity count.
   * `null` when the host refused.
   */
  columns(raw: unknown, kinds: readonly ColumnSpec[], count: number): ColumnArray[] | null;
  /**
   * Where the shared buffer (`og_host_shared`) is now: the buffer holding
   * it and its byte offset there. Asked again when the views made over it
   * were detached (a grown wasm memory).
   */
  shared(): { readonly buffer: ArrayBufferLike; readonly offset: number };
  /** Whether arrays made earlier still see engine memory (wasm memory growth detaches them). */
  fresh?(array: ColumnArray): boolean;
}

/** A column kind as the binding table declares it. */
export interface ColumnSpec {
  readonly name: string;
  readonly array: "Float32Array" | "Float64Array" | "Uint32Array" | "Int32Array" | "Uint8Array";
  readonly stride: number;
  /** The stride is each field's capacity (`str(32)`), not the kind's. */
  readonly capacity?: boolean;
  /** The list factory (`list(u32(), 8)`): no kind of its own, a list field's kind is its element's. */
  readonly list?: boolean;
}

export const COLUMNS: ReadonlyMap<string, ColumnSpec> = new Map(
  (bindings.columns as ColumnSpec[]).map((c) => [c.name, c]),
);

export const ARRAYS = {
  Float32Array,
  Float64Array,
  Uint32Array,
  Int32Array,
  Uint8Array,
} as const;

const queue: Array<(abi: HostAbi<Uint32Array, unknown>) => void> = [];
const uiRegistrationReads = new Set([
  "og_engine_ui_patch", "og_engine_resource_layout", "og_engine_read_resource",
  "og_engine_component_id", "og_engine_component_layout", "og_engine_read_component",
  "og_world_resource", "og_world_get", "og_world_has", "og_world_find",
  "og_world_name_of", "og_world_alive", "og_world_get_tick",
]);
const abiReturns = new Map(bindings.calls.filter(call => "symbol" in call)
  .map(call => [call.symbol, call.returns]));

function registrationBoundary(abi: HostAbi<Uint32Array, unknown>): HostAbi<Uint32Array, unknown> {
  return new Proxy(abi, {
    get(target, property, receiver) {
      if (!rt.uiRegistrationErrors || typeof property !== "string" || uiRegistrationReads.has(property)) {
        return Reflect.get(target, property, receiver);
      }
      return () => {
        const reason = `GameDesc.ui may only register presentation data and read the world: ${property} is not allowed`;
        rt.uiRegistrationErrors?.push(reason);
        if (abiReturns.get(property)?.startsWith("Result<")) return JSON.stringify(refused(reason));
        // Void/property APIs have no Result to return. The dispatcher catches
        // this and returns the registration failure to both hosts as a value.
        throw new Error(reason);
      };
    },
  });
}

/** Bind the runtime to a host and flush the declarations made so far. */
export function bindHost(binding: HostBinding): void {
  const missing = SYMBOLS.filter((s) => typeof (binding.abi as unknown as Record<string, unknown>)[s] !== "function");
  if (missing.length > 0) {
    throw new Error(`the ${binding.name} host lacks ${missing.length} binding symbol(s): ${missing.join(", ")}; rebuild it from the same api/<module>.ogt`);
  }
  // Entity arguments are checked here, so null or undefined is refused
  // naming the call on both hosts alike (generated/abi.ts).
  rt.host = { ...binding, abi: registrationBoundary(checkEntities(binding.abi)) };
  rt.publishBuild = null;
  // Another host's shared buffer and component numbers are not this one's.
  rt.sharedWords = new Uint32Array(0);
  forgetComponents();
  binding.abi.og_host_set_dispatcher(dispatch);
  for (const declare of queue.splice(0)) declare(binding.abi);
}

/** The bound host, for loaders. */
export function boundHost(): HostBinding | null {
  return rt.host;
}


export function abi(): HostAbi<Uint32Array, unknown> {
  if (!rt.host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  // A resource handle's writes reach the engine before anything else does.
  if (dirtyRecords.length > 0) flushRecords();
  rt.engineCalls++;
  return rt.host.abi;
}

// ── the shared buffer ─────────────────────────────────────────────────────

/** The shared buffer's size in bytes (og-runtime's shared.rs). */
export const SHARED_BYTES = 65536;
/** Where a system call's archetypes start in it, in words: after the time scale's four doubles. */
export const FRAME_QUERY_WORD = 8;



/** The buffer as words (and its other views with it), made again once a grown wasm memory detached them. */
export function words(): Uint32Array {
  if (rt.sharedWords.length === 0) {
    const { buffer, offset } = rt.host!.shared();
    rt.sharedWords = new Uint32Array(buffer, offset, SHARED_BYTES / 4);
    rt.sharedFloats = new Float64Array(buffer, offset, SHARED_BYTES / 8);
    rt.sharedBytes = new Uint8Array(buffer, offset, SHARED_BYTES);
    rt.sharedView = new DataView(buffer, offset, SHARED_BYTES);
  }
  return rt.sharedWords;
}

/** Run now if a host is bound, else when one is. */
export function declare(call: (abi: HostAbi<Uint32Array, unknown>) => void): void {
  if (rt.host) call(rt.host.abi);
  else queue.push(call);
}
