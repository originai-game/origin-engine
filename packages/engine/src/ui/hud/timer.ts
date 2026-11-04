// og.ui.hudTimer: a round, match or countdown timer, read against the round
// timers of fighting games and battle royales (tenths in the last ten
// seconds, a tick each second, red and pulsing at the end). Split out of
// scores.ts unchanged when the genre HUDs moved into kits (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { a11y, absolute, anchorOf, box, chip, clamp01, countdown, edge, frame, groups, hudPlace, kit, leaf, lit, msToTicks, pct, pop, pulse, px, role, sel, solid, sound, text, token, ts, u, type Rgba } from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

export function hudTimer(id: string, desc: Api.HudTimerDesc = {}): void {
  const tk = frame().t;
  const secs = Math.max(0, desc.seconds ?? 0);
  const mode: Api.TimerMode = desc.mode ?? "countdown";
  const warnAt = desc.warnAt ?? 10;
  const down = mode === "countdown";
  const done = down && secs <= 0;
  const state = desc.paused ? "paused" : done ? "done" : down && secs <= warnAt / 3 ? "critical" : down && secs <= warnAt ? "warn" : "normal";
  const ink = state === "critical" || state === "done" ? role("danger") : state === "warn" ? role("warn") : WHITE;
  const textNow = countdown(secs, desc.tenths !== false && down && secs < 10);
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { direction: "column", gap: 0, cross: "center", padding: { left: tk.md, right: tk.md, top: u(4), bottom: u(4) }, part: sel("HudTimer", "frame", state), partClass: "hud" }, chip(), () => {
      // A tick each of the last seconds, rising in pitch.
      const whole = Math.ceil(secs);
      const e = edge("sec", whole);
      sound("counterTick", e.changed && down && !desc.paused && whole <= Math.min(5, warnAt) && whole > 0, Math.max(0, 6 - whole));
      if (desc.label) text("Label", desc.label, { step: ts(11), font: "display", color: token("inkMuted"), nowrap: true });
      kit.row("Body", { gap: tk.sm, cross: "center" }, () => {
        if ((desc.total ?? 0) > 0 && down) {
          box("Ring", { width: px(u(22)), height: px(u(22)), part: sel("HudTimer", "ring") }, undefined, () => {
            leaf("Track", { width: pct(1), height: pct(1) }, { shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.PI * 2, thickness: u(3), cap: "butt" }, fills: [solid(token("shadow", 0.6))] }, absolute({ start: 0, top: 0 }));
            const left = clamp01(secs / (desc.total ?? 1));
            leaf("Fill", { width: pct(1), height: pct(1) }, { shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.max(0.0001, left) * Math.PI * 2, thickness: u(3), cap: "butt" }, fills: [solid(lit(ink))] }, absolute({ start: 0, top: 0 }));
          });
        }
        const beat = state === "critical" && e.since < msToTicks(250) ? pop(e.since, 0.14, 250) : 1;
        text("Time", textNow, { step: ts(26, true), font: "display", color: lit(ink), nowrap: true, outline: 1.2 }, {
          transform: { scaleX: beat, scaleY: beat },
          opacity: desc.paused ? 0.45 + 0.55 * pulse(1) : 1,
        });
      });
    }, {
      a11y: a11y("timer", desc.label || "Timer", { value: textNow }),
      ...groups({ time: state }, { time: ["normal", "warn", "critical", "paused", "done"] }),
    });
  });
}
