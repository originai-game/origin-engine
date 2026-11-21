// The development inspector's panel on the game's UI (crates/og-runtime/src/
// author/inspector): at each platform pump, the values the runtime built since
// the last (og_engine_inspector_panel) go to the TSX panel a development bundle
// registered (src/ui/inspector.tsx), mounted at the `system` layer while the
// inspector is open and unmounted when it closes; the panel's events go back
// through og_engine_inspector_event.

import type { World } from "@origingame/engine";
import { abi } from "../runtime/host.ts";
import { worldFor } from "../runtime/world.ts";
import { mount } from "./index.ts";
import { inspector } from "./rt.ts";
import { createSignal } from "./signals.ts";
import type { UiChild } from "./types.ts";

const panels = new Map<number, { set: (values: unknown) => void; unmount: () => void }>();

export function pumpInspector(engine: number): void {
  const Panel = inspector.panel;
  if (!Panel) return;
  const text = abi().og_engine_inspector_panel(engine);
  if (text === "") return;
  const open = panels.get(engine);
  if (text === "null") {
    open?.unmount();
    panels.delete(engine);
    return;
  }
  const values: unknown = JSON.parse(text);
  if (open) return open.set(values);
  const signal = createSignal(values, { equals: false });
  const send = (e: unknown) => { abi().og_engine_inspector_event(engine, JSON.stringify(e)); };
  const props = { values: signal, send } as never;
  const unmount = mount(worldFor(engine) as unknown as World, Panel as (props: never) => UiChild, props, { layer: "system" });
  panels.set(engine, { set: (v) => signal.set(v), unmount });
}
