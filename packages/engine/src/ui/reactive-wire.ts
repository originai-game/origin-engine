// Binary value tags shared with og-reactive/src/reactive/wire.rs. Values are copied
// at the boundary; functions, promises, accessors and cyclic objects are errors.
import type { Result } from "../runtime.ts";

const LIMIT = 16 * 1024 * 1024;
const utf8 = new TextEncoder();
const text = new TextDecoder("utf-8", { fatal: true });

export function encodeUiPatch(value: unknown): Result<Uint8Array> {
  let bytes = new Uint8Array(256);
  let view = new DataView(bytes.buffer);
  let at = 0;
  const ancestors = new Set<object>();
  const reserve = (n: number): void => {
    if (n > LIMIT - at) throw new Error("UI patch exceeds 16 MiB");
    if (at + n <= bytes.length) return;
    const grown = new Uint8Array(Math.min(LIMIT, Math.max(at + n, bytes.length * 2)));
    grown.set(bytes.subarray(0, at));
    bytes = grown;
    view = new DataView(bytes.buffer);
  };
  const tag = (n: number): void => { reserve(1); bytes[at++] = n; };
  const count = (n: number): void => {
    reserve(4);
    view.setUint32(at, n, true);
    at += 4;
  };
  const string = (s: string): void => {
    count(0);
    const lengthAt = at - 4;
    // Keys, ids and ops are short ASCII: copied here, since on an
    // interpreter (Hermes) a call to encodeInto per string costs more than
    // the bytes. Anything else, from its first byte, is encodeInto's.
    if (s.length <= 64) {
      reserve(s.length);
      let i = 0;
      for (; i < s.length; i++) {
        const c = s.charCodeAt(i);
        if (c > 0x7f) break;
        bytes[at + i] = c;
      }
      if (i === s.length) {
        at += i;
        view.setUint32(lengthAt, i, true);
        return;
      }
    }
    // UTF-8 needs at most three bytes per UTF-16 unit. Clamp the reserve,
    // not the input: near-limit ASCII can still fit in the remaining bytes.
    reserve(Math.min(s.length * 3, LIMIT - at));
    const encoded = utf8.encodeInto(s, bytes.subarray(at));
    if (encoded.read !== s.length) throw new Error("UI patch exceeds 16 MiB");
    at += encoded.written;
    view.setUint32(lengthAt, encoded.written, true);
  };
  const write = (value: unknown, depth: number): string | null => {
    if (depth > 64) return "UI patch nesting exceeds 64";
    if (value === null) tag(0);
    else if (typeof value === "boolean") tag(value ? 2 : 1);
    else if (typeof value === "number") {
      if (!Number.isFinite(value)) return "UI patch number must be finite";
      tag(3);
      reserve(8);
      view.setFloat64(at, value, true);
      at += 8;
    } else if (typeof value === "string") {
      tag(4);
      string(value);
    } else if (typeof value === "object") {
      if (ancestors.has(value)) return "UI patch value contains a cycle";
      ancestors.add(value);
      if (Array.isArray(value)) {
        tag(5);
        count(value.length);
        for (const item of value) {
          const error = write(item, depth + 1);
          if (error) return error;
        }
      } else {
        const prototype = Object.getPrototypeOf(value);
        if (prototype !== null && prototype !== Object.prototype) return "UI patch values must be plain records";
        const descriptors = Object.getOwnPropertyDescriptors(value);
        const keys = Object.keys(descriptors).sort();
        tag(6);
        count(keys.length);
        for (const key of keys) {
          const descriptor = descriptors[key];
          if (!descriptor || !("value" in descriptor)) return "UI patch values cannot have accessors";
          string(key);
          const error = write(descriptor.value, depth + 1);
          if (error) return error;
        }
      }
      ancestors.delete(value);
    } else return `UI patch cannot carry ${typeof value}`;
    return null;
  };
  try {
    const reason = write(value, 0);
    return reason ? { ok: false, reason } : { ok: true, value: bytes.subarray(0, at) };
  } catch (error) {
    return { ok: false, reason: `UI patch value could not be read: ${String(error)}` };
  }
}

export function decodeUiPatch(bytes: Uint8Array): Result<unknown> {
  if (bytes.length > LIMIT) return { ok: false, reason: "UI patch exceeds 16 MiB" };
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let at = 0;
  const take = (n: number): number => {
    if (n > bytes.length - at) throw new Error("truncated UI patch");
    const start = at;
    at += n;
    return start;
  };
  const count = (): number => {
    const n = view.getUint32(take(4), true);
    if (n > bytes.length - at) throw new Error("truncated UI patch sequence");
    return n;
  };
  const string = (): string => {
    const n = count();
    const start = take(n);
    return text.decode(bytes.subarray(start, start + n));
  };
  const read = (depth: number): unknown => {
    if (depth > 64) throw new Error("UI patch nesting exceeds 64");
    switch (bytes[take(1)]) {
      case 0: return null;
      case 1: return false;
      case 2: return true;
      case 3: {
        const n = view.getFloat64(take(8), true);
        if (!Number.isFinite(n)) throw new Error("UI patch number must be finite");
        return n;
      }
      case 4: return string();
      case 5: {
        const n = count();
        const values: unknown[] = [];
        for (let i = 0; i < n; i++) values.push(read(depth + 1));
        return values;
      }
      case 6: {
        const n = count();
        const record: Record<string, unknown> = {};
        for (let i = 0; i < n; i++) {
          const key = string();
          if (Object.hasOwn(record, key)) throw new Error("duplicate UI patch field");
          Object.defineProperty(record, key, { value: read(depth + 1), enumerable: true });
        }
        return record;
      }
      default: throw new Error("unknown UI patch value tag");
    }
  };
  try {
    const value = read(0);
    return at === bytes.length ? { ok: true, value } : { ok: false, reason: "trailing UI patch bytes" };
  } catch (error) {
    return { ok: false, reason: String(error) };
  }
}
