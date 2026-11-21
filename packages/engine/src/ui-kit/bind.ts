// og.ui's host calls beyond the core ones in runtime.ts (IDL module `ui`):
// the frame's queries, the virtual cursor, the actions and glyphs.

import type * as Api from "@origingame/engine";
import { boundHost } from "../runtime.ts";

type HostAbi = NonNullable<ReturnType<typeof boundHost>>["abi"];

function abi(): HostAbi {
  const host = boundHost();
  if (!host) throw new Error("no engine host is loaded: start the game through @origingame/engine's web or node loader");
  return host.abi;
}

const parse = <T>(text: string): T => JSON.parse(text) as T;
const json = (value: unknown): string | undefined => (value === undefined ? undefined : JSON.stringify(value));
const worldId = (world: Api.World): number => (world as unknown as { id: number }).id;

/**
 * The frame's theme and viewport, parsed only when their text changes: the
 * same object while the frame's style and surface stay the same (the
 * components read them many times a tick), and a fresh one the tick they
 * change, whichever engine is building.
 */
const lastParsed = new Map<string, { text: string; value: unknown }>();
function parsedOnce<T>(key: string, text: string): T {
  const last = lastParsed.get(key);
  if (last && last.text === text) return last.value as T;
  const value = parse<T>(text);
  lastParsed.set(key, { text, value });
  return value;
}

export const widgets = {
  frameViewport: (): Api.Viewport => parsedOnce("viewport", abi().og_ui_frame_viewport()),
  frameTheme: (): Api.UiTheme => parsedOnce("theme", abi().og_ui_frame_theme()),
  framePointer: (): Api.UiPointer => parse(abi().og_ui_frame_pointer()),
  hasIcon: (name: string): boolean => abi().og_ui_has_icon(name),
  seconds: (): number => abi().og_ui_seconds(),
  // A pad's virtual cursor.
  setVirtualCursor: (desc: Api.VirtualCursorDesc | null): void =>
    abi().og_ui_set_virtual_cursor(JSON.stringify(desc ?? null)),
  virtualCursor: (): Api.VirtualCursor | null => parse(abi().og_ui_virtual_cursor()),
  tick: (): number => abi().og_ui_tick(),
  previewViewport: (world: Api.World, viewport: Api.Viewport | null): void =>
    abi().og_ui_preview_viewport(worldId(world), JSON.stringify(viewport ?? null)),
};
