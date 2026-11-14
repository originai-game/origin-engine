// og.platform and the rooms on the page (shell.ts): what answers the
// engine's platform calls (the browser's own storage by default, a Player
// bridge for instance) and what carries its rooms.

import { localPlatform, storagePersistence, type PlatformAnswer, type PlatformCall, type PlatformHost, type PlatformLink, type PlatformNotice } from "../../platform-local.ts";
import type { RoomProvider } from "../../provider.ts";

/**
 * What carries og.platform's rooms on the web (docs/design.md §2.8): the engine's room ops
 * (`{ op: "create" | "join" | "quick" | "leave" | "send", ... }`, og-net's protocol) in, its room events
 * (`joined`, `peer_joined`, `peer_left`, `message`, `left`, `error`) out, in order. The provider
 * contract's RoomProvider (`@origingame/engine/provider`).
 */
export type RoomHost = RoomProvider;

export class PlatformBridge {
  /** A message to the engine Worker. */
  readonly #post: (message: Record<string, unknown>) => void;
  #rooms: RoomHost | null = null;
  #platform: PlatformHost = localPlatform(storagePersistence(`og.platform:${globalThis.location?.pathname ?? "/"}`));
  #pause: (on: boolean) => void;

  /** `tick` is the tick the Worker last reached; `play` starts or stops the clock, the default pause. */
  constructor(post: (message: Record<string, unknown>) => void, tick: () => number | null, play: (on: boolean) => void) {
    this.#post = post;
    this.#pause = (on) => play(!on);
    const bridge = this;
    this.#link = {
      get tick() {
        return tick() ?? 0;
      },
      answer: (request: number, answer: PlatformAnswer) => post({ type: "platform-answer", request, answer }),
      notify: (notice: PlatformNotice) => bridge.notify(notice),
      pause: () => bridge.#pause(true),
      resume: () => bridge.#pause(false),
    };
  }

  /** What the platform host may do besides answering: notices, and pausing the game while an ad shows (onPlatformPause). */
  readonly #link: PlatformLink;

  /** What answers og.platform's calls now. */
  get platform(): PlatformHost {
    return this.#platform;
  }

  /** Shell.setRooms. */
  setRooms(host: RoomHost | null, link?: string): void {
    this.#rooms = host;
    this.#post({ type: "room-host", host: host ? host.host : "none", ...(link ? { join: link } : {}) });
    // After the engine knows its host: what the provider heard already (a room joined at connect) follows.
    host?.attach?.(this.#deliverRooms);
  }

  /** Room events to the engine, in order (og_room_deliver). */
  readonly #deliverRooms = (events: unknown[]): void => this.#post({ type: "room-events", events });

  /** The engine's room ops (a `room` message), to the room host. */
  rooms(ops: Record<string, unknown>[]): void {
    const deliver = this.#deliverRooms;
    if (this.#rooms) this.#rooms.handle(ops, deliver);
    // No room host: what asked for a room hears so (the engine refuses first while its host is none).
    else deliver(ops.filter((o) => o["op"] === "create" || o["op"] === "join" || o["op"] === "quick").map((o) => ({ type: "error", op: o["op"], code: "unavailable", message: "this page carries no rooms" })));
  }

  /** The engine's platform calls (a `platform` message), to the host. */
  calls(calls: PlatformCall[]): void {
    this.#platform.handle(calls, this.#link.answer, this.#link);
    this.#platform.poll?.(this.#link);
  }

  /** The Worker plays on: a host that answers later (an ad's answer after the resume) or sends notices on its own is asked. */
  beat(): void {
    this.#platform.poll?.(this.#link);
  }

  /** Shell.onPlatformPause. */
  onPause(pause: (on: boolean) => void): void {
    this.#pause = pause;
  }

  /** Shell.notify. */
  notify(notice: PlatformNotice): void {
    this.#post({ type: "platform-notice", notice });
  }

  /** Shell.setPlatform. */
  setPlatform(host: PlatformHost): void {
    this.#platform = host;
    this.#post({ type: "platform-host", host: host.host, capabilities: host.capabilities, time: host.time });
  }
}
