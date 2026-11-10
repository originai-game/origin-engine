/** A field of a resource's packed layout, as the engine answers it. */
interface RecordField {
  readonly name: string;
  readonly kind: string;
  readonly offset: number;
  readonly size: number;
  readonly readonly: boolean;
  readonly variants?: readonly string[];
  /** A list's element kind (`f32`, `f64`, `u32`, `i32`, `handle`) and capacity in elements. */
  readonly element?: string;
  readonly capacity?: number;
}

// Read from globalThis, not the DOM lib's globals: kits import this module
// (through @origingame/engine) into games typechecked with lib
// ES2022 alone, which has no TextEncoder.
const utf8 = globalThis as unknown as {
  TextEncoder: new () => { encode(text: string): Uint8Array };
  TextDecoder: new () => { decode(bytes: Uint8Array): string };
};
const utf8in = new utf8.TextEncoder();
const utf8out = new utf8.TextDecoder();

const LANES: Readonly<Record<string, readonly string[]>> = {
  Vector2: ["x", "y"],
  Vector3: ["x", "y", "z"],
  Quaternion: ["x", "y", "z", "w"],
  Color: ["r", "g", "b"],
};

/** Bytes of one lane of a list's element kind. */
const LANE_SIZE: Readonly<Record<string, number>> = { f32: 4, f64: 8, u32: 4, i32: 4, handle: 8 };

function readLane(v: DataView, o: number, element: string): number {
  switch (element) {
    case "f32": return v.getFloat32(o, true);
    case "f64": return v.getFloat64(o, true);
    case "u32": return v.getUint32(o, true);
    case "i32": return v.getInt32(o, true);
    default: return v.getUint32(o, true) + v.getUint32(o + 4, true) * 4294967296;
  }
}

function writeLane(v: DataView, o: number, element: string, n: number): void {
  switch (element) {
    case "f32": return v.setFloat32(o, n, true);
    case "f64": return v.setFloat64(o, n, true);
    case "u32": return v.setUint32(o, n, true);
    case "i32": return v.setInt32(o, n, true);
    default:
      v.setUint32(o, n % 4294967296, true);
      v.setUint32(o + 4, Math.floor(n / 4294967296), true);
  }
}

/** A list element checked as the engine's JSON path checks it; throws what it refuses. */
function listElement(value: unknown, element: string, i: number): number {
  const n = number(value, `element ${i}`);
  const whole = (lo: number, hi: number, what: string) => {
    if (!Number.isInteger(n) || n < lo || n > hi) throw new Error(`element ${i}: ${n} is not ${what}`);
  };
  if (element === "u32") whole(0, 4294967295, "a u32");
  else if (element === "i32") whole(-2147483648, 2147483647, "an i32");
  else if (element === "handle") whole(0, 9007199254740991, "a handle");
  return n;
}

function decodeField(v: DataView, bytes: Uint8Array, f: RecordField): unknown {
  const o = f.offset;
  switch (f.kind) {
    case "list": {
      // A frozen copy: `push` throws instead of changing nothing.
      const element = f.element ?? "f32";
      const lane = LANE_SIZE[element] ?? 4;
      const count = Math.min(readLane(v, o, element), f.capacity ?? 0);
      const out: number[] = [];
      for (let i = 0; i < count; i++) out.push(readLane(v, o + lane * (i + 1), element));
      return Object.freeze(out);
    }
    case "f32": return v.getFloat32(o, true);
    case "f64": return v.getFloat64(o, true);
    case "u32": return v.getUint32(o, true);
    case "i32": return v.getInt32(o, true);
    case "bool": return bytes[o] !== 0;
    case "handle": return v.getUint32(o, true) + v.getUint32(o + 4, true) * 4294967296;
    case "enum": {
      const i = v.getUint32(o, true);
      return f.variants?.[i] ?? i;
    }
    case "str": {
      const lane = bytes.subarray(o, o + f.size);
      const end = lane.indexOf(0);
      return utf8out.decode(end < 0 ? lane : lane.subarray(0, end));
    }
    case "Matrix4": {
      const elements: number[] = [];
      for (let i = 0; i < 16; i++) elements.push(v.getFloat32(o + i * 4, true));
      return { elements };
    }
    default: {
      const out: Record<string, number> = {};
      (LANES[f.kind] ?? []).forEach((k, i) => (out[k] = v.getFloat32(o + i * 4, true)));
      return out;
    }
  }
}

function number(value: unknown, what: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) throw new Error(`${what} is a number, got ${JSON.stringify(value) ?? String(value)}`);
  return value;
}

/** Writes `value` into a field's bytes, refusing (throwing) what the engine's JSON path refuses. */
function encodeField(v: DataView, bytes: Uint8Array, f: RecordField, value: unknown): void {
  const o = f.offset;
  switch (f.kind) {
    case "list": {
      // The whole list: an array, at most the capacity, every element checked before a byte moves.
      const element = f.element ?? "f32";
      const capacity = f.capacity ?? 0;
      const items = value === null || value === undefined ? [] : value;
      if (!Array.isArray(items)) throw new Error(`a list is written whole, as an array, got ${JSON.stringify(value)}`);
      if (items.length > capacity) {
        const kind = element === "handle" ? "Asset" : element;
        throw new Error(`${items.length} elements and the field holds ${capacity}: remove ${items.length - capacity}, or declare the field list(${kind}(), ${items.length}) or larger`);
      }
      const lanes = items.map((x, i) => listElement(x, element, i));
      const lane = LANE_SIZE[element] ?? 4;
      bytes.fill(0, o, o + f.size);
      writeLane(v, o, element, lanes.length);
      lanes.forEach((n, i) => writeLane(v, o + lane * (i + 1), element, n));
      return;
    }
    case "f32": return v.setFloat32(o, number(value, "the value"), true);
    case "f64": return v.setFloat64(o, number(value, "the value"), true);
    case "u32": {
      const n = number(value, "the value");
      if (!Number.isInteger(n) || n < 0 || n > 4294967295) throw new Error(`${n} is not a u32`);
      return v.setUint32(o, n, true);
    }
    case "i32": {
      const n = number(value, "the value");
      if (!Number.isInteger(n) || n < -2147483648 || n > 2147483647) throw new Error(`${n} is not an i32`);
      return v.setInt32(o, n, true);
    }
    case "bool": {
      if (value !== null && value !== undefined && typeof value !== "boolean") throw new Error(`expected true or false, got ${JSON.stringify(value)}`);
      bytes[o] = value === true ? 1 : 0;
      return;
    }
    case "handle": {
      const n = number(value, "a handle");
      if (!Number.isInteger(n) || n < 0 || n >= 9007199254740992) throw new Error(`${n} is not a handle`);
      v.setUint32(o, n % 4294967296, true);
      v.setUint32(o + 4, Math.floor(n / 4294967296), true);
      return;
    }
    case "enum": {
      const variants = f.variants ?? [];
      let i: number;
      if (value === null || value === undefined) i = 0;
      else if (typeof value === "string") {
        i = variants.indexOf(value);
        if (i < 0) throw new Error(`'${value}' is not one of ${JSON.stringify(variants)}`);
      } else {
        i = number(value, "an enum index");
        if (!Number.isInteger(i) || i < 0 || i >= variants.length) throw new Error(`enum index ${i}, but there are ${variants.length} variants`);
      }
      return v.setUint32(o, i, true);
    }
    case "str": {
      if (value !== null && value !== undefined && typeof value !== "string") throw new Error(`expected a string, got ${JSON.stringify(value)}`);
      const text = (value ?? "") as string;
      const encoded = utf8in.encode(text);
      if (encoded.length > f.size) {
        const shown = [...text].slice(0, 24).join("");
        const more = shown.length < text.length ? "..." : "";
        const fix = encoded.length <= 4096
          ? `shorten the text, or declare the field str(${encoded.length}) or larger`
          : "shorten the text: a string field holds at most 4096 bytes";
        throw new Error(`'${shown}${more}' is ${encoded.length} bytes of UTF-8 and the field holds ${f.size}: ${fix}`);
      }
      if (encoded.includes(0)) throw new Error("a string field cannot hold a NUL character (\\0)");
      bytes.fill(0, o, o + f.size);
      bytes.set(encoded, o);
      return;
    }
    case "Matrix4": {
      const elements = (value as { elements?: unknown } | null)?.elements;
      if (!Array.isArray(elements) || elements.length !== 16) throw new Error(`expected { elements: [16 numbers] }, got ${JSON.stringify(value)}`);
      const lanes = elements.map((e) => number(e, "an element"));
      lanes.forEach((n, i) => v.setFloat32(o + i * 4, n, true));
      return;
    }
    default: {
      const keys = LANES[f.kind] ?? [];
      if (typeof value !== "object" || value === null) throw new Error(`expected { ${keys.join(", ")} }, got ${JSON.stringify(value)}`);
      const lanes = keys.map((k) => number((value as Record<string, unknown>)[k] ?? null, k));
      lanes.forEach((n, i) => v.setFloat32(o + i * 4, n, true));
    }
  }
}

export type { RecordField };
export { decodeField, encodeField };
