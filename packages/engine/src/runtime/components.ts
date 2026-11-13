import { SHARED_BYTES, abi, words } from "./host.ts";
import { type RecordField, decodeComponentField, encodeComponentField } from "./record-bytes.ts";
import { type Result, parse } from "./result.ts";
import { rt } from "./state.ts";

/** What `og_engine_component_id` answers for a name the world has not registered as a component. */
const NO_COMPONENT = 4294967295;

/** Component numbers by engine and name (`og_engine_component_id`), good for one layout epoch. */
const componentIds = new Map<number, Map<string, number>>();

/** A component's number in an engine's world; NO_COMPONENT while the world has not registered it. */
export function componentId(engine: number, name: string): number {
  let ids = componentIds.get(engine);
  if (!ids) componentIds.set(engine, (ids = new Map()));
  let id = ids.get(name);
  if (id === undefined) {
    id = abi().og_engine_component_id(engine, name);
    // An unregistered name is asked for again: a later use may register it.
    if (id !== NO_COMPONENT) ids.set(name, id);
  }
  return id;
}

// ── components as packed records ──────────────────────────────────────────
//
// world.get and world.set cross as the component's packed bytes in the
// shared buffer, read and written here field by field
// (runtime/record-bytes.ts), with the component named by its number. Only
// what the bytes answer exactly as the engine's JSON does goes this way:
// every refusal, and every value JSON would carry differently, is left to
// og_world_get and og_world_set, so a Result reads the same either way.

/** A component's packed layout in one engine (`og_engine_component_layout`). */
interface ComponentRecord {
  readonly id: number;
  readonly epoch: number;
  /** In the order world.get answers them: by name. */
  readonly fields: readonly RecordField[];
  readonly byName: ReadonlyMap<string, RecordField>;
  /** The word after the record in the shared buffer: where a write's ranges start. */
  readonly rangesAt: number;
}

/** Per engine, per component: its record, or null for one only JSON reads (an ease field). */
const componentRecords = new Map<number, Map<string, ComponentRecord | null>>();

/** Layouts were declared again (a new engine, a reload): numbers and offsets may have moved. */
export function forgetComponents(): void {
  componentIds.clear();
  componentRecords.clear();
}

export function componentRecord(engine: number, name: string): ComponentRecord | null {
  let byName = componentRecords.get(engine);
  if (!byName) componentRecords.set(engine, (byName = new Map()));
  const known = byName.get(name);
  if (known !== undefined) return known;
  type Layout = { epoch: number; id: number; size: number; fields: RecordField[] };
  const answer = parse<Result<Layout>>(abi().og_engine_component_layout(engine, name));
  // An unregistered name is asked for again: a later use may register it.
  if (!answer.ok) return null;
  const { epoch, id, size, fields } = answer.value;
  const rangesAt = Math.ceil(size / 4);
  const read = fields.every((f) => f.kind !== "ease" && f.name !== "__proto__") && (rangesAt + 1 + fields.length * 2) * 4 <= SHARED_BYTES;
  const record = read ? { id, epoch, fields, byName: new Map(fields.map((f) => [f.name, f])), rangesAt } : null;
  byName.set(name, record);
  return record;
}

/** The component the engine wrote to the shared buffer, as og_world_get's value; undefined when only JSON reads it. */
export function decodeComponent(c: ComponentRecord): Record<string, unknown> | undefined {
  words();
  const out: Record<string, unknown> = {};
  const fields = c.fields;
  for (let i = 0; i < fields.length; i++) {
    const f = fields[i]!;
    const value = decodeComponentField(rt.sharedView, rt.sharedBytes, f);
    if (value === undefined) return undefined;
    out[f.name] = value;
  }
  return out;
}

/** Writes the fields `value` names to the shared buffer, and their ranges after the record; false when og_world_set must take the value. */
export function encodeComponent(c: ComponentRecord, value: Record<string, unknown>): boolean {
  const w = words();
  let at = c.rangesAt + 1;
  for (const key in value) {
    const x = value[key];
    // JSON leaves an undefined value out.
    if (x === undefined) continue;
    const f = c.byName.get(key);
    if (!f || !encodeComponentField(rt.sharedView, rt.sharedBytes, f, x)) return false;
    w[at++] = f.offset;
    w[at++] = f.size;
  }
  w[c.rangesAt] = (at - c.rangesAt - 1) / 2;
  return true;
}
