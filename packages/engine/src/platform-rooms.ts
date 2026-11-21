// og.platform's room messages (IDL `platform`, `custom`): bytes cross as
// base64 under the key og-runtime's net.rs reads (BYTES_KEY).

import type { Custom } from "./generated/calls.ts";
import { abi } from "./runtime/host.ts";
import { fromBase64, parse } from "./runtime/result.ts";

export const platformCustom: Custom["platform"] = {
  roomSend: (world, data, to) =>
    parse(abi().og_platform_room_send(world.id, JSON.stringify(data instanceof Uint8Array ? { "$og.bytes": toBase64(data) } : (data ?? null)), to)),
  roomMessage: (world, message) => {
    const m = parse<{ bytes: string | null } | null>(abi().og_platform_room_message(world.id, message));
    return m && { ...m, bytes: typeof m.bytes === "string" ? fromBase64(m.bytes) : null };
  },
};

function toBase64(bytes: Uint8Array): string {
  let bin = "";
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]!);
  return btoa(bin);
}
