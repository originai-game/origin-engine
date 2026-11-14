// og.platform and the rooms in the engine Worker: both are carried by the
// page, so their calls and ops cross to the shell as messages.

import { boundHost, type Engine } from "../../runtime.ts";
import type { PlatformHost } from "../../platform-local.ts";
import type { PlatformCounter } from "../settle.ts";
import type { WorkerScope } from "./messages.ts";

declare const self: WorkerScope;

/**
 * Rooms (docs/design.md §2.8): the engine's room ops (og_room_take) go to the
 * page as `room` messages, which the shell's room host carries (the Player's
 * room.*, docs/platform-asks.md #16); the page's `room-events` come back
 * through og_room_deliver, in order, and `room-host` names what carries them.
 * Ops are taken after every frame and after events are delivered, so
 * lockstep's inputs leave on the frame that made them.
 */
export function pumpRoom(engine: Engine): void {
  const host = boundHost();
  if (!host) return;
  const ops = JSON.parse(host.abi.og_room_take(engine.id)) as unknown[];
  if (ops.length > 0) self.postMessage({ type: "room", ops });
}

/** og.platform's host in the Worker: the calls go to the page (counted), whose answers come back as `platform-answer`. */
export function toPage(host: string, capabilities: readonly string[], counter: PlatformCounter): PlatformHost {
  return {
    host,
    capabilities,
    handle: (calls) => {
      counter.sent(calls);
      self.postMessage({ type: "platform", calls });
    },
  };
}
