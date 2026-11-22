// A resource's byte path refuses what the engine's JSON path refuses, in
// the same words (og-runtime's codec/field.rs): a handle written to a u32
// field (a clip from og.load.animation into `u32()`) names the handle field
// types that hold one, a number that is no handle does not.
import { expect, test } from "bun:test";
import { encodeField } from "../src/runtime/record-bytes.ts";

const write = (kind: string, value: unknown, element?: string): string => {
  const bytes = new Uint8Array(16);
  const field = { name: "clip", kind, offset: 0, size: 16, readonly: false, ...(element ? { element, capacity: 2 } : {}) };
  try {
    encodeField(new DataView(bytes.buffer), bytes, field, value);
    return "accepted";
  } catch (e) {
    return (e as Error).message;
  }
};

test("a handle in a u32 field or list names ClipAsset and the other handle fields", () => {
  for (const message of [write("u32", 4294967298), write("list", [4294967298], "u32")]) {
    expect(message).toContain("4294967298 is not a u32: it is a handle (generation 1, index 2)");
    expect(message).toContain("ClipAsset()");
    expect(message).toContain("Entity()");
  }
  for (const value of [-1, 1.5, 1e300]) {
    const message = write("u32", value);
    expect(message).toContain("is not a u32");
    expect(message).not.toContain("handle");
  }
  expect(write("u32", 4294967295)).toBe("accepted");
});
