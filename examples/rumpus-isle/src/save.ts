// What Rumpus Isle keeps in og.platform (og-platform-sdk §2, §3):
//
// - `og.settings`: the settings, read on the first tick and applied when
//   they answer, saved when the settings screen closes (a pair).
// - `records`: the isle's bests, most berries in one Berry Rush and the
//   longest on the Crumble Tiles, read on the first tick and written at the
//   revision read when a round sets one (a conflict is read again, merged by
//   the best of each, and written again).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, Keys, Prefs } from "./components.ts";
import { settingsOf, takeSettings } from "./prefs.ts";
import { must } from "./util.ts";

type GameV = Value<typeof Game>;

interface Records {
  v: 1;
  berries: number;
  crumble: number;
}

function merge(g: GameV, r: Partial<Records>): void {
  const n = (v: unknown) => (typeof v === "number" && Number.isFinite(v) && v >= 0 ? v : 0);
  g.bestBerries = Math.max(g.bestBerries, Math.min(999, Math.floor(n(r.berries))));
  g.bestCrumble = Math.max(g.bestCrumble, Math.min(999, n(r.crumble)));
}

function askRecords(world: World, g: GameV): void {
  const r = og.platform.storeGet(world, Keys.records);
  g.recordsRequest = r.ok ? r.value : 0;
  if (!r.ok) g.recordsRead = true;
}

/**
 * First in the tick: the answers that arrived, applied before the screens are built, so the
 * title the shell draws behind the play gate already shows the saved language (og-platform-sdk §3).
 */
export const Answers = defineSystem("Answers", { writes: [Game, Prefs, og.Audio] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  if (time.tick === 0) {
    g.settingsRequest = og.platform.loadSettings(world);
    askRecords(world, g);
  }
  if (g.settingsRequest !== 0) {
    const a = og.platform.result(world, g.settingsRequest);
    if (a !== null) {
      g.settingsRequest = 0;
      const saved = a.ok ? (a.data as { value?: unknown } | null)?.value : null;
      if (saved !== null && typeof saved === "object") takeSettings(world, p, saved as Record<string, unknown>);
    }
  }
  if (g.recordsRequest !== 0) {
    const a = og.platform.result(world, g.recordsRequest);
    if (a !== null) {
      g.recordsRequest = 0;
      g.recordsRead = true;
      if (a.ok) {
        const d = a.data as { value?: unknown; revision?: number } | null;
        g.recordsRevision = d?.revision ?? 0;
        if (d?.value !== null && typeof d?.value === "object") merge(g, d.value as Partial<Records>);
      }
    }
  }
  if (g.saveRequest !== 0) {
    const a = og.platform.result(world, g.saveRequest);
    if (a !== null) {
      g.saveRequest = 0;
      if (a.ok) {
        g.recordsRevision = (a.data as { revision?: number } | null)?.revision ?? g.recordsRevision + 1;
        g.saves += 1;
      } else if (a.error?.code === "conflict") {
        // Another device wrote first: read it, merge (the best of each), and write again.
        askRecords(world, g);
        g.writeRecords = true;
      }
    }
  }
});

/** Last in the tick: the writes the tick asked for. */
export const Saves = defineSystem("Saves", { reads: [Prefs], writes: [Game] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  if (g.saveSettings) {
    g.saveSettings = false;
    og.platform.saveSettings(world, settingsOf(must(world.resource(Prefs))));
  }
  if (g.writeRecords && g.saveRequest === 0 && g.recordsRequest === 0) {
    g.writeRecords = false;
    const value: Records = { v: 1, berries: g.bestBerries, crumble: g.bestCrumble };
    const r = og.platform.storeSet(world, { key: Keys.records, value, ...(g.recordsRevision >= 0 ? { revision: g.recordsRevision } : {}) });
    if (r.ok) g.saveRequest = r.value;
  }
});
