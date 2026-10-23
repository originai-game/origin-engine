// What Brasswick keeps in og.platform (og-platform-sdk §2, og-game §17):
//
// - `og.settings`: the settings, read on the first tick and applied when
//   they answer, saved when the settings screen closes (a pair, §3).
// - `progress`: a record of the levels unlocked and each one's best gears,
//   time and stars, read on the first tick, written at the revision read
//   when a level is finished (a conflict is read again, merged by the best
//   of each, and written again).
// - the `vault` slot: the whole world (world.snapshot) at the last lamp lit,
//   through saveSlot/loadSlot (a few store keys and a manifest); its meta
//   (the level, the lamp, the gears) is what the title's Continue shows.
//   Continue restores it; the tick after a restore re-reads what lives
//   outside the world (the requests in flight, the settings, the progress).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Game, GEARS, Keys, LEVELS, Prefs } from "./components.ts";
import { say } from "./feel.ts";
import { settingsOf, takeSettings } from "./prefs.ts";
import { must } from "./level.ts";

type GameV = Value<typeof Game>;

interface Progress {
  v: 1;
  unlocked: number;
  best: number[];
  stars: number[];
  time: number[];
  score: number[];
}

export function progressOf(g: GameV): Progress {
  return { v: 1, unlocked: g.unlocked, best: [g.best0, g.best1, g.best2], stars: [g.stars0, g.stars1, g.stars2], time: [g.time0, g.time1, g.time2], score: [g.top0, g.top1, g.top2] };
}

/** Merges a stored record into the run's: the best of each (a record from another device never loses progress). */
function merge(g: GameV, r: Partial<Progress>): void {
  const n = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : 0);
  g.unlocked = Math.min(LEVELS, Math.max(g.unlocked, n(r.unlocked)));
  const best = r.best ?? [];
  const stars = r.stars ?? [];
  const time = r.time ?? [];
  const score = r.score ?? [];
  g.top0 = Math.max(g.top0, n(score[0]));
  g.top1 = Math.max(g.top1, n(score[1]));
  g.top2 = Math.max(g.top2, n(score[2]));
  g.best0 = Math.max(g.best0, Math.min(GEARS[0], n(best[0])));
  g.best1 = Math.max(g.best1, Math.min(GEARS[1], n(best[1])));
  g.best2 = Math.max(g.best2, Math.min(GEARS[2], n(best[2])));
  g.stars0 = Math.max(g.stars0, Math.min(3, n(stars[0])));
  g.stars1 = Math.max(g.stars1, Math.min(3, n(stars[1])));
  g.stars2 = Math.max(g.stars2, Math.min(3, n(stars[2])));
  const t = (mine: number, theirs: number) => (mine === 0 ? theirs : theirs === 0 ? mine : Math.min(mine, theirs));
  g.time0 = t(g.time0, n(time[0]));
  g.time1 = t(g.time1, n(time[1]));
  g.time2 = t(g.time2, n(time[2]));
}

function askAll(world: World, g: GameV): void {
  g.settingsRequest = og.platform.loadSettings(world);
  const p = og.platform.storeGet(world, Keys.progress);
  g.progressRequest = p.ok ? p.value : 0;
  const s = og.platform.loadSlot(world, { slot: Keys.slot, metaOnly: true });
  g.slotRequest = s.ok ? s.value : 0;
}

/** Continue: the vault slot's world, restored when it answers. */
export function askContinue(world: World, g: GameV): void {
  const r = og.platform.loadSlot(world, { slot: Keys.slot });
  g.loadRequest = r.ok ? r.value : 0;
}

/**
 * First in the tick: the answers that arrived, applied before the Ui system builds the screens (FINDINGS.md #16), so
 * the title the shell draws behind the play gate already shows the saved language and Continue
 * (og-platform-sdk §3: the shell runs one more tick per answer the first tick asked for, and the
 * frame is that tick's UI).
 */
export const Answers = defineSystem("Answers", { writes: [Game, Prefs, og.Audio] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  if (g.restored) {
    // The world came back from the vault slot at the end of the last tick: what it holds of the
    // session is the save's (requests in flight, the settings as they were), so ask again. Nothing
    // else marks the tick after a restore (FINDINGS.md #8): Saves sets Game.restored around the snapshot.
    g.restored = false;
    g.loads += 1;
    g.progressRequest = g.writeRequest = g.settingsRequest = g.slotRequest = g.loadRequest = g.saveRequest = 0;
    g.writeNow = g.settingsNow = g.saveNow = g.rewrite = false;
    g.paused = g.settingsOpen = g.levelsOpen = false;
    askAll(world, g);
    og.ui.toast(world, say(world, "toast.welcome"), { tone: "primary" });
  } else if (time.tick === 0) askAll(world, g);

  if (g.settingsRequest !== 0) {
    const a = og.platform.result(world, g.settingsRequest);
    if (a !== null) {
      g.settingsRequest = 0;
      const saved = a.ok ? (a.data as { value?: unknown } | null)?.value : null;
      if (saved !== null && typeof saved === "object") takeSettings(world, p, saved as Record<string, unknown>);
    }
  }
  if (g.progressRequest !== 0) {
    const a = og.platform.result(world, g.progressRequest);
    if (a !== null) {
      g.progressRequest = 0;
      if (a.ok) {
        const d = a.data as { value?: unknown; revision?: number } | null;
        g.revision = d?.revision ?? 0;
        if (d?.value !== null && typeof d?.value === "object") merge(g, d.value as Partial<Progress>);
        // A write refused as stale is merged above and written again.
        if (g.rewrite) {
          g.rewrite = false;
          g.writeNow = true;
        }
      }
    }
  }
  if (g.slotRequest !== 0) {
    const a = og.platform.result(world, g.slotRequest);
    if (a !== null) {
      g.slotRequest = 0;
      const meta = a.ok ? ((a.data as { meta?: unknown } | null)?.meta as { level?: number; gears?: number } | null) : null;
      g.hasSlot = meta !== null && meta !== undefined && typeof meta.level === "number";
      if (g.hasSlot) {
        g.slotLevel = meta!.level!;
        g.slotGears = meta!.gears ?? 0;
      }
    }
  }
  if (g.loadRequest !== 0) {
    const a = og.platform.result(world, g.loadRequest);
    if (a !== null) {
      g.loadRequest = 0;
      const text = a.ok ? (a.data as { text?: string | null } | null)?.text : null;
      const back = typeof text === "string" ? world.restore(text) : { ok: false as const, reason: a.error?.message ?? "the vault slot is empty" };
      if (!back.ok) og.ui.toast(world, say(world, "toast.noSave"), { tone: "danger" });
    }
  }
  if (g.saveRequest !== 0) {
    const a = og.platform.result(world, g.saveRequest);
    if (a !== null) {
      g.saveRequest = 0;
      if (a.ok) {
        g.saves += 1;
        g.hasSlot = true;
        g.slotLevel = g.level;
        g.slotGears = g.gears;
      } else og.ui.toast(world, say(world, "toast.saveFailed"), { tone: "danger" });
    }
  }
  if (g.writeRequest !== 0) {
    const a = og.platform.result(world, g.writeRequest);
    if (a !== null) {
      g.writeRequest = 0;
      if (a.ok) g.revision = (a.data as { revision?: number } | null)?.revision ?? g.revision;
      else if (a.error?.code === "conflict") {
        // Another device wrote first: read it again, merge, write again (above).
        g.conflicts += 1;
        g.rewrite = true;
        const r = og.platform.storeGet(world, Keys.progress);
        g.progressRequest = r.ok ? r.value : 0;
      }
    }
  }
});

/** Late in the tick, after the lamps and the exits asked: the vault slot, the progress record and the settings written. */
export const Saves = defineSystem("Saves", { writes: [Game, Prefs] }, (_q, _time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  // A lamp was lit: the world as it stands, in the vault slot (one request for every key).
  if (g.saveNow && g.saveRequest === 0) {
    g.saveNow = false;
    g.restored = true;
    const s = world.snapshot();
    g.restored = false;
    if (s.ok) {
      const r = og.platform.saveSlot(world, { slot: Keys.slot, text: s.value, meta: { level: g.level, lamp: g.checkpoint, gears: g.gears } });
      if (r.ok) g.saveRequest = r.value;
    }
  }
  if (g.writeNow && g.writeRequest === 0 && g.progressRequest === 0) {
    g.writeNow = false;
    const r = og.platform.storeSet(world, g.revision === 0 ? { key: Keys.progress, value: progressOf(g), create: true } : { key: Keys.progress, value: progressOf(g), revision: g.revision });
    if (r.ok) g.writeRequest = r.value;
  }
  if (g.settingsNow) {
    g.settingsNow = false;
    og.platform.saveSettings(world, settingsOf(p));
  }
});
