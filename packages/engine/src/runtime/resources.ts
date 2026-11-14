import type { Token } from "./declare.ts";
import { abi } from "./host.ts";
import { type RecordField, decodeField, encodeField } from "./record-bytes.ts";
import { type Result, parse } from "./result.ts";
import { rt } from "./state.ts";
import { checkAccess } from "./views.ts";
import { WorldImpl } from "./world.ts";

// ── resources as live values ──────────────────────────────────────────────
//
// A resource handle is a typed view of the resource's packed record: the
// runtime keeps one copy of each record's bytes per world, shared by every
// handle of it, and a field read or write is a DataView access at the
// field's offset, no JSON. The copy is fetched again (one engine call
// answering the bytes) only when an engine call or a dispatch happened since
// it was read, the only times the engine can change a resource; writes mark
// it dirty and are flushed in one call before the next engine call, at the
// end of the system or command that made them, and at once outside game
// code. So two handles agree, a read after an engine call sees the engine's
// value, and the engine sees every write before it can read the resource:
// the world (and its hash) is what writing field by field gave.

interface RecordLayout {
  readonly epoch: number;
  readonly size: number;
  readonly fields: readonly RecordField[];
}

/** One world's copy of one resource's bytes. */
interface RecordCopy {
  readonly world: number;
  readonly name: string;
  layout: RecordLayout;
  /** The packed record then the layout epoch (u32), as the engine answered and is written back. */
  bytes: Uint8Array;
  view: DataView;
  /** `engineCalls` when the bytes were the engine's; -1: never. */
  at: number;
  dirty: boolean;
  handle: object | null;
}

/** Per world, per resource: its copy, or null for one read as JSON. */
const records = new Map<number, Map<string, RecordCopy | null>>();
export const dirtyRecords: RecordCopy[] = [];

export function forgetRecords(world: number): void {
  records.delete(world);
  for (let i = dirtyRecords.length - 1; i >= 0; i--) if (dirtyRecords[i]!.world === world) dirtyRecords.splice(i, 1);
}

function writeRecord(r: RecordCopy): void {
  r.dirty = false;
  const answer = parse<Result<void>>(rt.host!.abi.og_engine_write_resource_bytes(r.world, r.name, r.bytes));
  rt.engineCalls++;
  if (!answer.ok) {
    r.at = -1;
    throw new Error(`${r.name}: ${answer.reason}`);
  }
  r.at = rt.engineCalls;
}

/** Writes every dirty record back, oldest first; throws the first refusal after writing the rest. */
export function flushRecords(): void {
  let first: unknown = null;
  for (const r of dirtyRecords.splice(0)) {
    try {
      writeRecord(r);
    } catch (e) {
      first ??= e;
    }
  }
  if (first !== null) throw first;
}

function recordLayout(world: number, name: string): RecordLayout | null {
  const answer = parse<Result<RecordLayout>>(rt.host!.abi.og_engine_resource_layout(world, name));
  return answer.ok ? answer.value : null;
}

/** The world's copy of a resource with a byte layout, or null for one read as JSON (og.Time, og.Rng). */
export function recordOf(world: number, name: string): RecordCopy | null {
  let byName = records.get(world);
  if (!byName) records.set(world, (byName = new Map()));
  const known = byName.get(name);
  if (known !== undefined) return known;
  const layout = recordLayout(world, name);
  // An unregistered name is asked again next time (a reload may declare it).
  if (!layout) {
    if (name === "og.Time" || name === "og.Rng") byName.set(name, null);
    return null;
  }
  const bytes = new Uint8Array(layout.size + 4);
  const r: RecordCopy = { world, name, layout, bytes, view: new DataView(bytes.buffer), at: -1, dirty: false, handle: null };
  byName.set(name, r);
  return r;
}

/** Brings the copy up to the engine's value when an engine call happened since; false when the engine refused. */
export function refresh(r: RecordCopy): boolean {
  if (r.dirty || r.at === rt.engineCalls) return true;
  const fetched = rt.host!.abi.og_engine_read_resource(r.world, r.name);
  if (fetched.length === 0) return false;
  const epoch = new DataView(fetched.buffer, fetched.byteOffset, fetched.byteLength).getUint32(fetched.length - 4, true);
  if (epoch !== r.layout.epoch || fetched.length !== r.layout.size + 4) {
    // A reload or a new engine declared the layouts again.
    const layout = recordLayout(r.world, r.name);
    if (!layout || fetched.length !== layout.size + 4) return false;
    r.layout = layout;
    // The next world.resource hands out a handle with the new fields.
    r.handle = null;
  }
  // Into the copy's own buffer: no allocation from one read to the next, and
  // what is written back is memory the runtime made (a host's external
  // buffer crosses back slower).
  if (r.bytes.length !== fetched.length) {
    r.bytes = new Uint8Array(fetched.length);
    r.view = new DataView(r.bytes.buffer);
  }
  r.bytes.set(fetched);
  r.at = rt.engineCalls;
  return true;
}

/** The handle of a resource read through its packed bytes, shared by every `world.resource` of it. */
export function recordHandle(world: WorldImpl, r: RecordCopy): object {
  if (r.handle) return r.handle;
  const id = world.id;
  const name = r.name;
  const out: Record<string, unknown> = {};
  const fields = r.layout.fields;
  for (let i = 0; i < fields.length; i++) {
    const key = fields[i]!.name;
    // The field by name in the copy's current layout (a reload may move it).
    const field = (): RecordField => {
      const f = r.layout.fields[i];
      return f && f.name === key ? f : (r.layout.fields.find((x) => x.name === key) ?? fields[i]!);
    };
    let lastValue: unknown;
    const get = (): unknown => {
      // Parameterized timing is serialized by the shared Rust parser/codec.
      // abi() flushes pending packed writes before reading the same record.
      if (field().kind === "ease") {
        if (!checkAccess("read", name)) return lastValue;
        const answer = parse<Result<Record<string, unknown>>>(abi().og_world_resource(id, name));
        if (!answer.ok) throw new Error(`${name}: ${answer.reason}`);
        return (lastValue = answer.value[key]);
      }
      if (r.at !== rt.engineCalls && !r.dirty) {
        // Undeclared in a publish build: the engine refuses and logs it once;
        // the handle answers what it last read and the game keeps running.
        if (!checkAccess("read", name)) return lastValue;
        if (!refresh(r)) {
          const answer = parse<Result<Record<string, unknown>>>(rt.host!.abi.og_world_resource(id, name));
          if (!answer.ok) throw new Error(`${name}: ${answer.reason}`);
          const value = answer.value[key];
          // A list reads as a frozen copy on this path too.
          return (lastValue = Array.isArray(value) ? Object.freeze(value) : value);
        }
      }
      return (lastValue = decodeField(r.view, r.bytes, field()));
    };
    const set = (value: unknown): void => {
      // Undeclared in a publish build: the engine refuses and logs it once; the game keeps running.
      if (!checkAccess("write", name)) return void abi().og_engine_write_resource(id, name, JSON.stringify({ [key]: value }));
      if (field().kind === "ease" || !refresh(r)) {
        const answer = parse<Result<void>>(abi().og_engine_write_resource(id, name, JSON.stringify({ [key]: value })));
        if (!answer.ok) throw new Error(`${name}.${key}: ${answer.reason}`);
        return;
      }
      try {
        // Every check comes before the first byte is written: a refused value leaves the copy as it was.
        encodeField(r.view, r.bytes, field(), value);
      } catch (e) {
        throw new Error(`${name}.${key}: ${(e as Error).message}`);
      }
      if (rt.inGame === 0) return writeRecord(r);
      if (!r.dirty) {
        r.dirty = true;
        dirtyRecords.push(r);
      }
    };
    Object.defineProperty(out, key, fields[i]!.readonly ? { enumerable: true, get } : { enumerable: true, get, set });
  }
  methods(id, name, out);
  return (r.handle = Object.seal(out));
}

/** The methods a built-in resource's handle carries beside its fields. */
function methods(id: number, name: string, out: Record<string, unknown>): void {
  if (name === "og.Input") {
    Object.assign(out, {
      down: (key: string) => abi().og_input_down(id, key),
      justPressed: (key: string) => abi().og_input_just_pressed(id, key),
      justReleased: (key: string) => abi().og_input_just_released(id, key),
      axis: (negative: string, positive: string) => abi().og_input_axis(id, negative, positive),
      axis2: (left: string, right: string, down: string, up: string) => parse(abi().og_input_axis2(id, left, right, down, up)),
      buttonDown: (button: string) => abi().og_input_button_down(id, button),
      buttonJustPressed: (button: string) => abi().og_input_button_just_pressed(id, button),
      buttonJustReleased: (button: string) => abi().og_input_button_just_released(id, button),
    });
  }
  if (name === "og.Pointers") {
    Object.assign(out, {
      down: (pointer: number) => abi().og_pointers_down(id, pointer),
      justPressed: (pointer: number) => abi().og_pointers_just_pressed(id, pointer),
      justReleased: (pointer: number) => abi().og_pointers_just_released(id, pointer),
      position: (pointer: number) => parse(abi().og_pointers_position(id, pointer)),
      start: (pointer: number) => parse(abi().og_pointers_start(id, pointer)),
      count: () => abi().og_pointers_count(id),
    });
  }
  if (name === "og.Gamepad") {
    Object.assign(out, {
      connected: (pad?: number) => abi().og_gamepad_connected(id, pad),
      down: (button: string, pad?: number) => abi().og_gamepad_down(id, button, pad),
      justPressed: (button: string, pad?: number) => abi().og_gamepad_just_pressed(id, button, pad),
      justReleased: (button: string, pad?: number) => abi().og_gamepad_just_released(id, button, pad),
      axis: (axis: string, pad?: number) => abi().og_gamepad_axis(id, axis, pad),
    });
  }
  if (name === "og.Camera") {
    Object.assign(out, {
      lookAt: (target: unknown) => abi().og_camera_look_at(id, JSON.stringify(target)),
      worldToScreen: (point: unknown) => parse(abi().og_camera_world_to_screen(id, JSON.stringify(point))),
      screenToRay: (x: number, y: number) => parse(abi().og_camera_screen_to_ray(id, x, y)),
    });
  }
}

/** A resource read as JSON (og.Time, og.Rng): computed by the engine, read only. */
export function jsonResource(world: WorldImpl, token: Token, value: Record<string, unknown>): object {
  const id = world.id;
  if (token.name === "og.Rng") {
    return Object.freeze({
      get seed() { return parse<{ ok: boolean; value?: { seed: number } }>(abi().og_world_resource(id, "og.Rng")).value?.seed ?? 0; },
      get state() { return parse<{ ok: boolean; value?: { state: number } }>(abi().og_world_resource(id, "og.Rng")).value?.state ?? 0; },
      // A draw moves the stream: it needs og.Rng among the system's writes. A fork reads the seed only.
      next: () => (checkAccess("write", "og.Rng"), abi().og_rng_next(id)),
      range: (low: number, high: number) => (checkAccess("write", "og.Rng"), abi().og_rng_range(id, low, high)),
      integer: (low: number, high: number) => (checkAccess("write", "og.Rng"), abi().og_rng_integer(id, low, high)),
      chance: (p: number) => (checkAccess("write", "og.Rng"), abi().og_rng_chance(id, p)),
      weighted: (weights: readonly number[]) =>
        (checkAccess("write", "og.Rng"), abi().og_rng_weighted(id, JSON.stringify(weights))),
      permutation: (n: number) => (checkAccess("write", "og.Rng"), parse<number[]>(abi().og_rng_permutation(id, n))),
      fork: (label: string) => (checkAccess("read", "og.Rng"), rngStream(abi().og_rng_fork(id, label))),
    });
  }
  // Every getter reads through to the store, kept while no other engine
  // call or dispatch happened since (`engineCalls`).
  const name = token.name;
  let last = value;
  let at = rt.engineCalls;
  const read = (): Record<string, unknown> => {
    if (at === rt.engineCalls) return last;
    if (name !== "og.Time" && !checkAccess("read", name)) return last;
    const answer = parse<Result<Record<string, unknown>>>(rt.host!.abi.og_world_resource(id, name));
    if (answer.ok) {
      last = answer.value;
      at = rt.engineCalls;
    }
    return last;
  };
  const out: Record<string, unknown> = {};
  for (const key of Object.keys(value)) Object.defineProperty(out, key, { enumerable: true, get: () => read()[key] });
  methods(id, name, out);
  return Object.seal(out);
}

function rngStream(handle: number): object {
  return Object.freeze({
    get seed() { return abi().og_rng_stream_get_seed(handle); },
    next: () => abi().og_rng_stream_next(handle),
    range: (low: number, high: number) => abi().og_rng_stream_range(handle, low, high),
    integer: (low: number, high: number) => abi().og_rng_stream_integer(handle, low, high),
    chance: (p: number) => abi().og_rng_stream_chance(handle, p),
    weighted: (weights: readonly number[]) => abi().og_rng_stream_weighted(handle, JSON.stringify(weights)),
    permutation: (n: number) => parse<number[]>(abi().og_rng_stream_permutation(handle, n)),
    fork: (label: string) => rngStream(abi().og_rng_stream_fork(handle, label)),
  });
}
