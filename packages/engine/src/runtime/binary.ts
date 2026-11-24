// OGB1 binary wire codec for TypedArrays and raw Uint8Array payloads crossing
// the host boundary (Self::Bytes). Keeps small metadata as compact JSON while
// moving numeric buffers as 8-byte-aligned raw little-endian slices, enabling
// zero-copy TypedArray views on decode and avoiding JSON.stringify / Array.from
// and base64 overhead on both Web and Node hosts.

const MAGIC0 = 0x4f; // 'O'
const MAGIC1 = 0x47; // 'G'
const MAGIC2 = 0x42; // 'B'
const MAGIC3 = 0x31; // '1'

const TAG_U8 = 0;
const TAG_I32 = 1;
const TAG_U32 = 2;
const TAG_F32 = 3;
const TAG_F64 = 4;

const IS_LE = new Uint8Array(new Uint32Array([1]).buffer)[0] === 1;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

type TypedView = Uint8Array | Int32Array | Uint32Array | Float32Array | Float64Array;

interface BlobSpec {
  tag: number;
  count: number;
  bytes: Uint8Array;
}

const align8 = (n: number): number => (n + 7) & ~7;

function tagOf(v: unknown): number {
  if (v instanceof Uint8Array) return TAG_U8;
  if (v instanceof Int32Array) return TAG_I32;
  if (v instanceof Uint32Array) return TAG_U32;
  if (v instanceof Float32Array) return TAG_F32;
  if (v instanceof Float64Array) return TAG_F64;
  return -1;
}

function viewBytesLE(view: TypedView, tag: number): Uint8Array {
  if (IS_LE || tag === TAG_U8) {
    return new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
  }
  const out = new Uint8Array(view.byteLength);
  const dv = new DataView(out.buffer);
  if (tag === TAG_I32) {
    for (let i = 0; i < view.length; i++) dv.setInt32(i * 4, view[i]!, true);
  } else if (tag === TAG_U32) {
    for (let i = 0; i < view.length; i++) dv.setUint32(i * 4, view[i]!, true);
  } else if (tag === TAG_F32) {
    for (let i = 0; i < view.length; i++) dv.setFloat32(i * 4, view[i]!, true);
  } else if (tag === TAG_F64) {
    for (let i = 0; i < view.length; i++) dv.setFloat64(i * 8, view[i]!, true);
  }
  return out;
}

/** Encode any structured argument (including nested TypedArrays) into an OGB1 packet. */
export function bin(value: unknown): Uint8Array {
  const blobs: BlobSpec[] = [];
  const text = JSON.stringify(value, (_key, v: unknown) => {
    const tag = tagOf(v);
    if (tag >= 0) {
      const view = v as TypedView;
      const idx = blobs.length;
      blobs.push({ tag, count: view.length, bytes: viewBytesLE(view, tag) });
      return { $b: idx };
    }
    return v;
  }) ?? "null";
  const jsonBytes = encoder.encode(text);
  const headerEnd = 16 + blobs.length * 16;
  let cursor = align8(headerEnd + jsonBytes.length);
  const offsets: number[] = new Array(blobs.length);
  for (let i = 0; i < blobs.length; i++) {
    offsets[i] = cursor;
    cursor = align8(cursor + blobs[i]!.bytes.byteLength);
  }
  const out = new Uint8Array(cursor);
  const dv = new DataView(out.buffer);
  out[0] = MAGIC0;
  out[1] = MAGIC1;
  out[2] = MAGIC2;
  out[3] = MAGIC3;
  dv.setUint32(4, 0, true);
  dv.setUint32(8, jsonBytes.byteLength, true);
  dv.setUint32(12, blobs.length, true);
  for (let i = 0; i < blobs.length; i++) {
    const b = blobs[i]!;
    const base = 16 + i * 16;
    dv.setUint32(base, b.tag, true);
    dv.setUint32(base + 4, b.count, true);
    dv.setUint32(base + 8, offsets[i]!, true);
    dv.setUint32(base + 12, b.bytes.byteLength, true);
  }
  out.set(jsonBytes, headerEnd);
  for (let i = 0; i < blobs.length; i++) {
    out.set(blobs[i]!.bytes, offsets[i]!);
  }
  return out;
}

/** Encode an optional structured argument, leaving `undefined` as `undefined`. */
export const binOpt = (value: unknown): Uint8Array | undefined =>
  value === undefined ? undefined : bin(value);

function makeView(packet: Uint8Array, tag: number, count: number, offset: number, byteLen: number): TypedView {
  if (tag === TAG_U8) {
    return new Uint8Array(packet.buffer, packet.byteOffset + offset, count);
  }
  const stride = tag === TAG_F64 ? 8 : 4;
  const absOff = packet.byteOffset + offset;
  if (IS_LE && absOff % stride === 0) {
    if (tag === TAG_I32) return new Int32Array(packet.buffer, absOff, count);
    if (tag === TAG_U32) return new Uint32Array(packet.buffer, absOff, count);
    if (tag === TAG_F32) return new Float32Array(packet.buffer, absOff, count);
    if (tag === TAG_F64) return new Float64Array(packet.buffer, absOff, count);
  }
  // Unaligned slice or big-endian host: copy cleanly into an aligned buffer.
  if (IS_LE) {
    const copy = packet.slice(offset, offset + byteLen);
    if (tag === TAG_I32) return new Int32Array(copy.buffer, 0, count);
    if (tag === TAG_U32) return new Uint32Array(copy.buffer, 0, count);
    if (tag === TAG_F32) return new Float32Array(copy.buffer, 0, count);
    return new Float64Array(copy.buffer, 0, count);
  }
  const dv = new DataView(packet.buffer, absOff, byteLen);
  if (tag === TAG_I32) {
    const a = new Int32Array(count);
    for (let i = 0; i < count; i++) a[i] = dv.getInt32(i * 4, true);
    return a;
  }
  if (tag === TAG_U32) {
    const a = new Uint32Array(count);
    for (let i = 0; i < count; i++) a[i] = dv.getUint32(i * 4, true);
    return a;
  }
  if (tag === TAG_F32) {
    const a = new Float32Array(count);
    for (let i = 0; i < count; i++) a[i] = dv.getFloat32(i * 4, true);
    return a;
  }
  const a = new Float64Array(count);
  for (let i = 0; i < count; i++) a[i] = dv.getFloat64(i * 8, true);
  return a;
}

function resolveBlobs(v: unknown, views: TypedView[]): unknown {
  if (v === null || typeof v !== "object") return v;
  if (Array.isArray(v)) {
    for (let i = 0; i < v.length; i++) v[i] = resolveBlobs(v[i], views);
    return v;
  }
  const o = v as Record<string, unknown>;
  if ("$b" in o) {
    const keys = Object.keys(o);
    if (keys.length === 1 && typeof o.$b === "number") {
      const view = views[o.$b];
      if (view !== undefined) return view;
    }
  }
  for (const k of Object.keys(o)) {
    o[k] = resolveBlobs(o[k], views);
  }
  return o;
}

/** Decode an OGB1 binary packet returned by a host binding with zero-copy TypedArray views. */
export function binDecode(packet: Uint8Array): unknown {
  if (
    packet.byteLength < 16 ||
    packet[0] !== MAGIC0 ||
    packet[1] !== MAGIC1 ||
    packet[2] !== MAGIC2 ||
    packet[3] !== MAGIC3
  ) {
    return JSON.parse(decoder.decode(packet)) as unknown;
  }
  const dv = new DataView(packet.buffer, packet.byteOffset, packet.byteLength);
  const jsonLen = dv.getUint32(8, true);
  const blobCount = dv.getUint32(12, true);
  const jsonStart = 16 + blobCount * 16;
  const views: TypedView[] = new Array(blobCount);
  for (let i = 0; i < blobCount; i++) {
    const base = 16 + i * 16;
    const tag = dv.getUint32(base, true);
    const count = dv.getUint32(base + 4, true);
    const offset = dv.getUint32(base + 8, true);
    const byteLen = dv.getUint32(base + 12, true);
    views[i] = makeView(packet, tag, count, offset, byteLen);
  }
  const jsonText = decoder.decode(packet.subarray(jsonStart, jsonStart + jsonLen));
  const parsed = JSON.parse(jsonText) as unknown;
  return blobCount === 0 ? parsed : resolveBlobs(parsed, views);
}
