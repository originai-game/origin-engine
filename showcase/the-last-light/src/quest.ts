// What makes the walk read as a game: the HUD (assets/ui/hud.tsx) with the
// objective, how far the lighthouse is and the controls. `walk:path` walks
// the wanderer along the winding stretch of the cliff path for recordings,
// through the same move input a player gives, and stops where it ends.
import { defineResource, defineSystem, math, og, u32 } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { mount, useStore } from "@origingame/engine/ui";
import { Hud as HudView } from "../assets/ui/hud.tsx";
import { LIGHTHOUSE } from "./layout.ts";
import { must } from "./must.ts";

/** The stretch of the path walk:path takes: PATH[FIRST] to PATH[LAST], its bends past the crag and down the shelf. */
export const FIRST = 4, LAST = 8;

export const Quest = defineResource("Quest", {
  /** The path is walked for the player (walk:path), towards PATH[leg]. */
  auto: u32(), leg: u32(FIRST),
});

export function buildQuest(world: World): void {
  og.ui.configure(world, { font: must(og.load.font(world, "base/fonts/latin/NotoSans-Latin")) });
}

export const Play = defineSystem("Play", { reads: [og.Transform], writes: [Quest], after: ["Walk"] }, (_q, _time, world) => {
  const me = world.find("wanderer");
  if (me === null) return;
  const p = must(world.get(me, og.Transform)).position;
  const away = math.sqrt((p.x - LIGHTHOUSE.x) ** 2 + (p.z - LIGHTHOUSE.z) ** 2);
  must(og.ui.scope(world, "hud").writable("distance", "").set(`${Math.round(away)} m`));
});

export function mountHud(world: World): void {
  const distance = og.ui.scope(world, "hud").writable("distance", "");
  mount(world, () => {
    const d = useStore(distance);
    return HudView({ get distance() { return d(); } });
  }, {}, { layer: "hud" });
}

/** `walk:path`: walk the winding stretch of the path for the player; `walk:stop` hands it back. */
export const questCommands = {
  walk: (world: World, args: readonly string[]) => {
    const q = must(world.resource(Quest));
    q.auto = args[0] === "stop" ? 0 : 1;
    q.leg = FIRST;
  },
};
