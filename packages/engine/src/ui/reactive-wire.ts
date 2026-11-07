// Binary value tags shared with og-ui/src/reactive/wire.rs. Values are copied
// at the boundary; functions, promises, accessors and cyclic objects are errors.
import type { Result } from "../runtime.ts";

const LIMIT = 16 * 1024 * 1024;
const utf8 = new TextEncoder();
const text = new TextDecoder("utf-8", { fatal: true });

export function encodeUiPatch(value: unknown): Result<Uint8Array> {
  const bytes: number[] = [];
  const ancestors = new Set<object>();
  const count = (n: number): void => {
    bytes.push(n & 255, (n >>> 8) & 255, (n >>> 16) & 255, (n >>> 24) & 255);
  };
  const string = (s: string): void => {
    const encoded = utf8.encode(s);
    count(encoded.length);
    for (const byte of encoded) bytes.push(byte);
  };
  const write = (value: unknown, depth: number): string | null => {
    if (depth > 64) return "UI patch nesting exceeds 64";
    if (value === null) bytes.push(0);
    else if (typeof value === "boolean") bytes.push(value ? 2 : 1);
    else if (typeof value === "number") {
      if (!Number.isFinite(value)) return "UI patch number must be finite";
      bytes.push(3);
      const buffer = new ArrayBuffer(8);
      new DataView(buffer).setFloat64(0, value, true);
      for (const byte of new Uint8Array(buffer)) bytes.push(byte);
    } else if (typeof value === "string") {
      bytes.push(4);
      string(value);
    } else if (typeof value === "object") {
      if (ancestors.has(value)) return "UI patch value contains a cycle";
      ancestors.add(value);
      if (Array.isArray(value)) {
        bytes.push(5);
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
        bytes.push(6);
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
    return bytes.length > LIMIT ? "UI patch exceeds 16 MiB" : null;
  };
  try {
    const reason = write(value, 0);
    return reason ? { ok: false, reason } : { ok: true, value: Uint8Array.from(bytes) };
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
