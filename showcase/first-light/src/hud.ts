// The cosy HUD (assets/ui/hud.svelte): the clock, the day's tasks and the
// hotbar, built from the engine's components in the default style. H hides
// it; a slot pressed on the hotbar is the same choice as its key.
import { og, defineSystem } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Pick, Scene, SEASONS, TIMES, WEATHERS } from "./env.ts";
import { must } from "./util.ts";

const title = (word: string) => word.charAt(0).toUpperCase() + word.slice(1);
const two = (n: number) => (n < 10 ? `0${n}` : `${n}`);

/** The day's tasks: each is done once the player has seen what it names. */
const TASKS = [
  { id: "dawn", text: "Watch the sun come up over the lake" },
  { id: "weather", text: "See the weather turn" },
  { id: "season", text: "Follow the oak through a season" },
  { id: "night", text: "Stay until the village lights its lamps" },
] as const;

function slots(weather: number, season: number, time: number) {
  const slot = (id: string, icon: string, name: string, hotkey: string) => ({ id, icon, name, action: id, hotkey, cooldown: 0, cooldownMax: 0, cost: "", unaffordable: false, locked: false, active: false, count: 0 });
  return [
    slot("weather", weather === 2 ? "bolt" : "drop", `Weather: ${title(WEATHERS[weather]!)}`, "1"),
    slot("season", "leaf", `Season: ${title(SEASONS[season]!)}`, "2"),
    slot("time", "clock", `Time: ${title(TIMES[time]!)}`, "3"),
  ];
}

/** Registers the HUD's stores and mounts it (GameDesc.ui). */
export function mountHud(world: World): void {
  const scope = og.ui.scope(world, "hud");
  must(scope.mount("hud", "hud", {
    shown: scope.writable("shown", true),
    clock: scope.writable("clock", "06:54"),
    day: scope.writable("day", "Summer · Dawn"),
    sky: scope.writable("sky", "Clear"),
    quest: scope.writable("quest", "Morning rounds"),
    steps: scope.collection("steps", TASKS.map(t => ({ ...t, done: false })), { key: "id" }),
    slots: scope.collection("slots", slots(0, 1, 0), { key: "id" }),
  }));
}

/** Writes the HUD's stores from the scene, and turns a pressed slot into its choice. */
export const Hud = defineSystem("Hud", { reads: [Scene], writes: [Pick], after: ["Air"] }, (_q, _t, world) => {
  const s = must(world.resource(Scene));
  const scope = og.ui.scope(world, "hud");
  scope.writable("shown", true).set(s.hud === 1);
  const minutes = Math.floor(s.hour * 60 + 0.5) % 1440;
  scope.writable("clock", "").set(`${two(Math.floor(minutes / 60))}:${two(minutes % 60)}`);
  scope.writable("day", "").set(`${title(SEASONS[s.season]!)} · ${title(TIMES[s.time]!)}`);
  scope.writable("sky", "").set(title(WEATHERS[s.weather]!));
  // A row is patched only when it changed, so an unchanged HUD rebuilds nothing.
  const bar = scope.collection("slots", slots(0, 1, 0), { key: "id" });
  for (const slot of slots(s.weather, s.season, s.time)) {
    const row = bar.row(slot.id);
    if (row.ok && (row.value.name !== slot.name || row.value.icon !== slot.icon)) must(bar.patch(slot.id, { name: slot.name, icon: slot.icon }));
  }
  const done = [true, (s.seen & 1) !== 0, (s.seen & 2) !== 0, (s.seen & 4) !== 0];
  const steps = scope.collection("steps", TASKS.map(t => ({ ...t, done: false })), { key: "id" });
  for (const [i, task] of TASKS.entries()) {
    const row = steps.row(task.id);
    if (row.ok && row.value.done !== done[i]) must(steps.patch(task.id, { done: done[i]! }));
  }
  for (const e of og.ui.docEvents(world)) {
    const which = e.name === "use" ? ["weather", "season", "time"].indexOf(e.rowKey) : -1;
    if (which >= 0) must(og.input.intent(world, Pick, { which }));
  }
});
