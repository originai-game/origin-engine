// og.ui.scoreboard, og.ui.hudTimer, og.ui.scorebug, og.ui.comboCounter,
// og.ui.judgement, og.ui.resourceBar and og.ui.turnOrder. Read against
// Valorant's and Overwatch's tab scoreboards (team headers in their
// colour, tabular columns, the player's row lit, the dead dimmed), the
// round timers of fighting games and battle royales (tenths in the last
// ten seconds, a tick each second, red and pulsing at the end), EA FC's
// and NBA 2K's broadcast scorebugs, Devil May Cry's and Hi-Fi Rush's combo
// counters, the judgements of osu!, Project Sekai and Beat Saber,
// StarCraft II's and Civilization's resource bars (amount, a signed rate,
// supply at its cap) and the turn orders of Final Fantasy X and Baldur's
// Gate 3 (portraits in the order they act, the current one larger).

import type * as Api from "@origingame/engine";
import { a11y, absolute, anchorOf, box, chip, clamp01, countdown, easeBack, easeOut, edge, ellipseShape, fillDraw, formatNumber, frame, glow, groups, hudPlace, icon, inkOn, keyId, kit, leaf, lit, mix, msToTicks, now, paintRgba, pct, pop, pulse, px, reduced, role, sel, shapeOf, solid, sound, spring, stroke, teamColor, text, token, trackDraw, ts, u, vgrad, type Rgba, apply, type Props } from "./core.ts";

const WHITE: Rgba = { r: 1, g: 1, b: 1, a: 1 };

function pingLevel(ms: number): "good" | "fair" | "poor" {
  return ms > 200 ? "poor" : ms > 100 ? "fair" : "good";
}

export function scoreboard(id: string, desc: Api.ScoreboardDesc = {}): void {
  const f = frame();
  const tk = f.t;
  const width = desc.width || (f.phone ? f.viewport.width - f.viewport.safeArea.left - f.viewport.safeArea.right - tk.gutter * 2 : u(760));
  const cols = desc.columns ?? [];
  const rowH = u(f.phone ? 30 : 34);
  hudPlace(id, anchorOf(desc, "center"), () => {
    box(id, { direction: "column", gap: tk.sm, width: px(width), padding: { left: tk.md, right: tk.md, top: tk.md, bottom: tk.md }, part: sel("Scoreboard", "panel"), partClass: "panel" }, {
      ...chip(shapeOf("panel").radius, true),
    }, () => {
      if (desc.title || desc.subtitle) {
        kit.row("Head", { gap: tk.sm, cross: "end", width: pct(1) }, () => {
          text("Title", desc.title ?? "", { step: ts(20), font: "display", color: token("ink"), width: { kind: "grow" }, nowrap: true, ellipsis: true });
          if (desc.subtitle) text("Subtitle", desc.subtitle, { step: ts(13), color: token("inkMuted"), nowrap: true });
        });
      }
      (desc.teams ?? []).forEach((team, ti) => {
        const tc = teamColor(team.team ?? ti);
        box(kit.idi("Team", ti), { direction: "column", gap: u(2), width: pct(1), part: sel("Scoreboard", "team", ["team", team.team ?? ti]) }, undefined, () => {
          // The header: the team's colour band, its name and score, the column labels.
          box("Header", { direction: "row", gap: tk.sm, cross: "center", width: pct(1), height: px(rowH), padding: { left: tk.sm, right: tk.sm, top: 0, bottom: 0 }, part: sel("Scoreboard", "header") }, {
            shape: { kind: "rect", radii: [3, 3, 3, 3] },
            fills: [vgrad([[0, lit(tc, 0.85)], [1, lit(tc, 0.35)]], -90)],
          }, () => {
            text("Name", team.name ?? "", { step: ts(14), font: "display", color: lit(inkOn(tc)), width: { kind: "grow" }, nowrap: true, ellipsis: true });
            for (const c of cols) text(keyId(c.key ?? c.label ?? ""), c.label ?? "", { step: ts(11), font: "mono", color: lit(inkOn(tc), 0.85), nowrap: true, width: px(u(c.width || 56)), align: "end" });
            text("Score", team.score ?? "", { step: ts(20, true), font: "display", color: lit(inkOn(tc)), nowrap: true, width: px(u(56)), align: "end" });
          });
          (team.rows ?? []).forEach((row, ri) => {
            const lvl = pingLevel(row.ping ?? 0);
            box(keyId(row.key ?? row.name ?? String(ri)), {
              direction: "row", gap: tk.sm, cross: "center", width: pct(1), height: px(rowH), padding: { left: tk.sm, right: tk.sm, top: 0, bottom: 0 },
              part: sel("Scoreboard", "row", row.local ? "local" : null, row.dead ? "dead" : null),
            }, {
              shape: { kind: "rect", radii: [2, 2, 2, 2] },
              fills: [solid(row.local ? token("highlight", 0.22) : ri % 2 === 0 ? token("shadow", 0.35) : token("shadow", 0.2))],
              strokes: row.local ? [stroke(1, token("highlight", 0.8))] : [],
            }, () => {
              leaf("Team", { width: px(u(3)), height: pct(0.7) }, { fills: [solid(lit(tc))] });
              if (row.avatar) {
                kit.image("Avatar", { texture: row.avatar, width: px(rowH - u(6)), height: px(rowH - u(6)), label: "" });
              } else if (row.icon) icon("Icon", row.icon, rowH - u(10), token("ink"));
              text("Name", row.name ?? "", { step: ts(13), font: row.local ? "display" : "body", color: row.dead ? token("inkFaint") : token("ink"), width: { kind: "grow" }, nowrap: true, ellipsis: true });
              if (row.mvp) icon("Mvp", "star", u(14), lit(role("gold")), "MVP");
              if (row.dead) icon("Dead", "skull", u(14), token("inkFaint"), "Dead");
              (row.values ?? []).forEach((v, i) => {
                text(kit.idi("Value", i), v, { step: ts(13), font: "mono", color: row.dead ? token("inkMuted") : token("ink"), nowrap: true, width: px(u(cols[i]?.width || 56)), align: "end" });
              });
              if ((row.ping ?? 0) > 0) {
                const ink = lvl === "poor" ? role("danger") : lvl === "fair" ? role("warn") : role("success");
                const bars = lvl === "poor" ? 1 : lvl === "fair" ? 2 : 3;
                box("Ping", { direction: "row", gap: 1, cross: "end", width: px(u(56)), align: "end", part: sel("Scoreboard", "ping", lvl) }, undefined, () => {
                  for (let b = 0; b < 3; b++) leaf(kit.idi("Bar", b), { width: px(u(3)), height: px(u(4 + b * 3)) }, { fills: [solid(b < bars ? lit(ink) : token("inkFaint", 0.4))] });
                });
              } else leaf("PingSpace", { width: px(u(56)), height: px(1) });
            }, {
              opacity: row.dead ? 0.7 : 1,
              a11y: a11y("row", row.name ?? "", { value: (row.values ?? []).join(", "), selected: !!row.local }),
            });
          });
        }, { a11y: a11y("group", team.name ?? "") });
      });
    }, { a11y: a11y("table", desc.title || "Scoreboard") });
  });
}

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

export function scorebug(id: string, desc: Api.ScorebugDesc = {}): void {
  const tk = frame().t;
  const h = u(36);
  const side = (sid: "Home" | "Away", team: Api.ScorebugTeam | undefined, k: number) => {
    const t = team ?? {};
    const c = t.color ? paintRgba(t.color, teamColor(k)) : teamColor(k);
    const e = edge(`${sid}~score`, t.score ?? 0);
    sound("success", e.changed && (t.score ?? 0) > e.prev);
    const flash = e.since < msToTicks(900) && (t.score ?? 0) > e.prev ? 1 - e.since / msToTicks(900) : 0;
    box(sid, { direction: "row", gap: 0, cross: "stretch", height: px(h), part: sel("Scorebug", "team", sid.toLowerCase()) }, undefined, () => {
      box("Block", { direction: "row", gap: u(6), cross: "center", padding: { left: u(8), right: u(8), top: 0, bottom: 0 }, height: pct(1) }, {
        fills: [vgrad([[0, lit(mix(c, WHITE, 0.15))], [1, lit(mix(c, { r: 0, g: 0, b: 0, a: 1 }, 0.2))]])],
      }, () => {
        if (t.logo) kit.image("Logo", { texture: t.logo, width: px(h - u(10)), height: px(h - u(10)), label: "" });
        text("Abbr", t.abbr ?? "", { step: ts(15), font: "display", color: lit(inkOn(c)), nowrap: true });
        const max = t.timeoutsMax ?? 0;
        if (max > 0) {
          kit.column("Timeouts", { gap: u(2) }, () => {
            for (let i = 0; i < max; i++) leaf(kit.idi("Timeout", i), { width: px(u(8)), height: px(u(2)), part: sel("Scorebug", "timeout", i >= (t.timeouts ?? 0) ? "used" : null) }, { fills: [solid(lit(inkOn(c), i < (t.timeouts ?? 0) ? 0.95 : 0.25))] });
          });
        }
      });
      box("ScoreBox", { width: px(u(46)), height: pct(1), align: "center", cross: "center", part: sel("Scorebug", "score") }, {
        fills: [solid(token("hud", 0.96)), ...(flash > 0 ? [solid(lit(c, 0.6 * flash))] : [])],
      }, () => {
        const s = (t.score ?? 0) > e.prev ? pop(e.since, 0.25, 420) : 1;
        text("Score", String(t.score ?? 0), { step: ts(22, true), font: "display", color: token("ink"), nowrap: true }, { transform: { scaleX: s, scaleY: s } });
        if (t.possession) leaf("Possession", { width: px(u(6)), height: px(u(6)), part: sel("Scorebug", "possession") }, { shape: ellipseShape(), fills: [solid(lit(role("gold")))] }, absolute({ end: u(3), top: u(3) }));
      });
    });
  };
  hudPlace(id, anchorOf(desc, "topLeft"), () => {
    kit.column(id, { gap: 0, cross: "start" }, () => {
      box("Bar", { direction: "row", gap: 0, cross: "center", part: sel("Scorebug", "frame"), partClass: "hud" }, {
        shape: { kind: "rect", radii: [3, 3, 3, 3] },
        fills: [solid(token("hud", 0.96))],
        effects: [{ kind: "dropShadow", offset: { x: 0, y: 2 }, blur: 8, spread: 0, color: token("shadow", 0.5) }],
        clip: "shape",
      }, () => {
        side("Home", desc.home, 0);
        side("Away", desc.away, 1);
        box("Clock", { direction: "row", gap: u(6), cross: "center", padding: { left: tk.sm, right: tk.sm, top: 0, bottom: 0 }, part: sel("Scorebug", "clock") }, undefined, () => {
          text("Period", desc.period ?? "", { step: ts(12), font: "display", color: token("inkMuted"), nowrap: true });
          text("Clock", desc.clock ?? "", { step: ts(17), font: "mono", color: token("ink"), nowrap: true });
          if (desc.shotClock) {
            box("Shot", { padding: { left: u(4), right: u(4), top: 0, bottom: 0 }, part: sel("Scorebug", "shot") }, { shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [solid(lit(role("warn")))] }, () => {
              text("Text", desc.shotClock ?? "", { step: ts(12), font: "mono", color: lit(inkOn(role("warn"))), nowrap: true });
            });
          }
        });
      });
      if (desc.situation) {
        const e = edge("situation", desc.situation.length);
        const x = reduced() ? 1 : easeOut(e.since / msToTicks(260));
        box("Situation", { padding: { left: tk.sm, right: tk.sm, top: u(2), bottom: u(2) }, part: sel("Scorebug", "situation") }, {
          shape: { kind: "rect", radii: [0, 0, 3, 3] },
          fills: [solid(token("primary", 0.92))],
        }, () => text("Text", desc.situation ?? "", { step: ts(12), font: "display", color: token("onPrimary"), nowrap: true }), { transform: { y: -(1 - x) * u(20) }, opacity: x });
      }
    });
    apply({ a11y: a11y("status", "Score", { value: `${desc.home?.abbr ?? "Home"} ${desc.home?.score ?? 0}, ${desc.away?.abbr ?? "Away"} ${desc.away?.score ?? 0}, ${desc.clock ?? ""} ${desc.period ?? ""}`.trim() }) });
  });
}

const MILESTONES = [10, 25, 50, 100];

export function comboCounter(id: string, desc: Api.ComboDesc = {}): void {
  const count = desc.count ?? 0;
  const t = now();
  let comboProps: Props = {};
  hudPlace(id, anchorOf(desc, "right"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      const e = edge("count", count);
      // A break: the last count shakes and falls off.
      const brokeAt = kit.memory("brokeAt", -1e9);
      const lastLive = kit.memory("last", 0);
      if (e.changed && count < 2 && e.prev >= 2) {
        kit.remember("brokeAt", t);
        kit.remember("last", e.prev);
      }
      const ms = (desc.milestones ?? []).length > 0 ? desc.milestones! : MILESTONES;
      const hitMilestone = e.changed && count > e.prev && (ms.includes(count) || (count > 100 && count % 100 === 0));
      if (hitMilestone) kit.remember("mileAt", t);
      sound("reward", hitMilestone);
      const mileAt = kit.memory("mileAt", -1e9);
      const mile = t - mileAt < msToTicks(700);
      const broken = count < 2 && t - brokeAt < msToTicks(600);
      const state = broken ? "broken" : count < 2 ? "idle" : mile ? "milestone" : "live";
      if (count >= 2 || broken) {
        const shown = broken ? lastLive : count;
        const grow = 1 + Math.min(0.35, Math.log10(Math.max(1, shown)) * 0.15);
        const s = broken ? 1 : (mile ? pop(t - mileAt, 0.45, 520) : pop(e.since, 0.22, 260)) * grow;
        const bx = broken ? (t - brokeAt) / msToTicks(600) : 0;
        const ink = mile ? role("gold") : shown >= 50 ? role("danger") : shown >= 25 ? role("warn") : WHITE;
        text("Count", String(shown), { step: ts(48, true), font: "display", color: lit(ink), nowrap: true, outline: 1.6 }, {
          transform: broken ? { x: reduced() ? 0 : Math.sin(bx * 40) * u(6) * (1 - bx), y: reduced() ? 0 : bx * bx * u(40), rotate: reduced() ? 0 : bx * 0.3 } : { scaleX: s, scaleY: s, rotate: reduced() ? 0 : -0.06 },
          opacity: broken ? 1 - bx : 1,
        });
        kit.row("Under", { gap: u(6), cross: "center" }, () => {
          text("Label", desc.label ?? "Combo", { step: ts(14), font: "display", color: token("ink"), nowrap: true, outline: 1 });
          if ((desc.multiplier ?? 0) > 1) {
            box("Multiplier", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, part: sel("Combo", "multiplier") }, {
              shape: { kind: "rect", radii: [999, 999, 999, 999] },
              fills: [solid(lit(role("gold")))],
            }, () => text("Text", `×${formatNumber(desc.multiplier ?? 1, { decimals: Number.isInteger(desc.multiplier) ? 0 : 1 })}`, { step: ts(13), font: "display", color: lit(inkOn(role("gold"))), nowrap: true }));
          }
        });
        if (desc.decay !== null && desc.decay !== undefined && !broken) {
          box("Decay", { width: px(u(96)), height: px(u(4)), part: sel("Combo", "decay") }, trackDraw(2), () => {
            leaf("Fill", { width: pct(clamp01(desc.decay ?? 0)), height: pct(1) }, fillDraw(clamp01(desc.decay ?? 0) < 0.25 ? role("danger") : ink, 2));
          });
        }
        if (mile && !reduced()) {
          const x = (t - mileAt) / msToTicks(700);
          leaf("Burst", { width: px(u(140)), height: px(u(140)), part: sel("Combo", "burst") }, {
            shape: { kind: "polygon", sides: 12, inner: 0.55 },
            fills: [solid(lit(role("gold"), 0.5 * (1 - x)))],
          }, absolute({ centerX: true, top: -u(30) }, { transform: { scaleX: 0.5 + easeOut(x), scaleY: 0.5 + easeOut(x), rotate: x * 0.8 }, z: -1 }));
        }
      }
      comboProps = {
        a11y: a11y("status", desc.label ?? "Combo", { value: count >= 2 ? String(count) : "" }),
        ...groups({ combo: state }, { combo: ["idle", "live", "milestone", "broken"] }),
      };
    });
    apply(comboProps);
  });
}

function tierInk(tier: number): Rgba {
  return [role("gold"), role("success"), role("info"), role("warn")][tier] ?? role("danger");
}

export function judgement(id: string, desc: Api.JudgementDesc = {}): void {
  const t = now();
  const ev = desc.last ?? null;
  let judgeProps: Props = {};
  hudPlace(id, anchorOf(desc, "center"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      const age = ev ? t - (ev.at ?? 0) : 1e9;
      const life = msToTicks(600);
      const tier = ev?.tier ?? 0;
      const miss = tier >= 4;
      const state = !ev || age >= life || age < 0 ? "none" : tier === 0 ? "best" : miss ? "miss" : "good";
      if (state !== "none" && ev) {
        const x = age / life;
        const ink = tierInk(tier);
        const s = reduced() ? 1 : x < 0.15 ? 1.35 - 0.35 * easeBack(x / 0.15) : 1;
        const y = reduced() ? 0 : miss ? u(18) * easeOut(x) : -u(10) * easeOut(x);
        const grad: Api.Gradient | undefined = tier === 0 ? {
          type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 },
          stops: [{ offset: 0, color: lit(mix(ink, WHITE, 0.6)) }, { offset: 0.55, color: lit(ink) }, { offset: 1, color: lit(mix(ink, { r: 0.6, g: 0.3, b: 0, a: 1 }, 0.5)) }],
        } : undefined;
        text("Word", ev.text ?? "", { step: ts(30, true), font: "display", color: lit(miss ? mix(ink, { r: 0.5, g: 0.5, b: 0.5, a: 1 }, 0.4) : ink), nowrap: true, outline: 1.4, ...(grad ? { gradient: grad } : {}) }, {
          transform: { scaleX: s, scaleY: s, y },
          opacity: x < 0.7 ? 1 : 1 - (x - 0.7) / 0.3,
        });
        const off = ev.offsetMs ?? 0;
        if (off !== 0 && !miss) {
          text("Timing", `${off < 0 ? desc.earlyLabel ?? "Early" : desc.lateLabel ?? "Late"} ${Math.abs(Math.round(off))}ms`, {
            step: ts(12), font: "mono", color: lit(off < 0 ? role("info") : role("warn")), nowrap: true, outline: 1,
          }, { transform: { y }, opacity: x < 0.7 ? 1 : 1 - (x - 0.7) / 0.3 });
        }
      }
      judgeProps = {
        a11y: a11y("status", state === "none" ? "" : ev?.text ?? ""),
        ...groups({ judge: state }, { judge: ["none", "best", "good", "miss"] }),
      };
    });
    apply(judgeProps);
  }, { offsetY: desc.offsetY ?? 0 });
}

export function resourceBar(id: string, desc: Api.ResourceBarDesc = {}): void {
  const tk = frame().t;
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { direction: "row", gap: tk.lg, cross: "center", padding: { left: tk.md, right: tk.md, top: u(5), bottom: u(5) }, part: sel("ResourceBar", "frame"), partClass: "hud" }, chip(), () => {
      for (const r of desc.items ?? []) {
        const cap = r.capacity ?? null;
        const amount = r.amount ?? 0;
        const capped = cap !== null && amount >= cap;
        const near = cap !== null && !capped && amount >= cap * 0.85;
        const fmt: Api.NumberFormatDesc = r.format ?? { notation: "compact" };
        box(keyId(r.key ?? r.name ?? ""), { direction: "row", gap: u(6), cross: "center", part: sel("ResourceBar", "item", capped ? "capped" : near ? "near" : null) }, undefined, () => {
          if (r.icon) icon("Icon", r.icon, u(20), r.icon === "coin" || r.icon === "gold" ? lit(role("gold")) : token("ink"));
          // The amount counts to its value on the ui spring.
          const shown = spring("amount", amount);
          const e = edge("amount", amount);
          const s = e.since < msToTicks(300) && amount > e.prev ? pop(e.since, 0.1, 300) : 1;
          const ink = capped ? lit(role("danger")) : near ? lit(role("warn")) : token("ink");
          text("Amount", cap !== null ? `${formatNumber(Math.round(shown), fmt)}/${formatNumber(cap, fmt)}` : formatNumber(Math.abs(amount - shown) < 0.5 ? amount : Math.round(shown), fmt), {
            step: ts(15), font: "mono", color: ink, nowrap: true, outline: 1,
          }, { transform: { scaleX: s, scaleY: s }, ...(capped ? { opacity: 0.6 + 0.4 * pulse(1.5) } : {}) });
          const rate = r.rate ?? 0;
          if (desc.rates !== false && rate !== 0) {
            // The direction as a drawn mark (no font need carry ▲▼), then the amount a second.
            const rateInk = lit(rate > 0 ? role("success") : role("danger"));
            box("Rate", { direction: "row", gap: u(2), cross: "center", part: sel("ResourceBar", "rate", rate > 0 ? "up" : "down") }, undefined, () => {
              leaf("Mark", { width: px(u(7)), height: px(u(6)) }, { shape: { kind: "polygon", sides: 3, rotation: rate > 0 ? 0 : Math.PI }, fills: [solid(rateInk)] });
              text("Text", `${formatNumber(Math.abs(rate), { ...fmt, decimals: Math.abs(rate) < 100 && !Number.isInteger(rate) ? 1 : 0 })}${r.rateSuffix ?? "/s"}`, {
                step: ts(11), font: "mono", color: rateInk, nowrap: true, outline: 0.8,
              });
            });
          }
        }, { a11y: a11y("text", r.name ?? r.key ?? "", { value: `${formatNumber(amount, fmt)}${cap !== null ? ` of ${formatNumber(cap, fmt)}` : ""}${rate(r)}` }) });
      }
    }, { a11y: a11y("list", "Resources") });
  });
}

function rate(r: Api.ResourceItem): string {
  const v = r.rate ?? 0;
  return v === 0 ? "" : `, ${v > 0 ? "+" : ""}${v}${r.rateSuffix ?? "/s"}`;
}

export function turnOrder(id: string, desc: Api.TurnOrderDesc = {}): void {
  const tk = frame().t;
  const size = u(desc.size || 52);
  const units = (desc.units ?? []).slice(0, desc.max || 8);
  const row = (desc.direction ?? "row") === "row";
  hudPlace(id, anchorOf(desc, "top"), () => {
    box(id, { direction: row ? "row" : "column", gap: tk.sm, cross: row ? "end" : "start", part: sel("TurnOrder", "frame") }, undefined, () => {
      units.forEach((un, i) => {
        const cur = !!un.current;
        const s = cur ? Math.round(size * 1.3) : size;
        const enemy = (un.team ?? 0) !== 0;
        const ink = enemy ? (un.team === 1 ? role("danger") : teamColor(un.team ?? 1)) : role("primary");
        // The order slides on layout: each unit keeps its key, and the frame animates its place.
        box(keyId(un.key ?? un.name ?? String(i)), {
          direction: "column", gap: u(3), cross: "center",
          part: sel("TurnOrder", "unit", cur ? "current" : null, ["team", un.team ?? 0], un.dead ? "dead" : null),
        }, undefined, () => {
          box("Portrait", { width: px(s), height: px(s), align: "center", cross: "center", part: sel("TurnOrder", "portrait") }, {
            // Enemies are notched at the corner, so a side is not told by colour alone.
            shape: { kind: "rect", radii: enemy ? [s * 0.3, 4, 4, 4] : [4, 4, 4, 4], corner: enemy ? "bevel" : "round" },
            fills: [vgrad([[0, lit(mix(ink, WHITE, 0.2), 0.9)], [1, lit(mix(ink, { r: 0, g: 0, b: 0, a: 1 }, 0.55))]])],
            strokes: [stroke(cur ? 3 : 1.5, cur ? lit(role("highlight")) : lit(ink)), stroke(1, token("shadow", 0.9), "outside")],
            effects: cur ? [glow(8 + 4 * pulse(1), lit(role("highlight"), 0.55))] : [],
            clip: "shape",
          }, () => {
            if (un.portrait) {
              kit.image("Art", { texture: un.portrait, width: px(s), height: px(s), label: "" });
              apply(absolute({ start: 0, top: 0 }));
            } else text("Initials", (un.name ?? "?").slice(0, 2).toUpperCase(), { step: ts(cur ? 20 : 15, true), font: "display", color: token("ink"), nowrap: true, outline: 1 });
            if (un.dead) {
              leaf("Cross", { width: px(s), height: px(s) }, {
                shape: { kind: "path", points: [{ x: s * 0.2, y: s * 0.2 }, { x: s * 0.8, y: s * 0.8 }], width: 3, cap: "round" },
                fills: [solid(lit(role("danger")))],
              }, absolute({ start: 0, top: 0 }));
            }
            if (un.delayed) box("Delay", { width: px(u(14)), height: px(u(14)), part: sel("TurnOrder", "delay") }, undefined, () => icon("Mark", "arrow-down", u(14), lit(role("warn")), "Delayed"), absolute({ end: 1, top: 1 }));
          });
          if (un.atb !== null && un.atb !== undefined) {
            box("Atb", { width: px(s), height: px(u(4)), part: sel("TurnOrder", "atb") }, trackDraw(2), () => {
              leaf("Fill", { width: pct(clamp01(un.atb ?? 0)), height: pct(1) }, fillDraw((un.atb ?? 0) >= 1 ? role("gold") : role("info"), 2));
            });
          }
          if (cur) {
            box("Now", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, part: sel("TurnOrder", "now") }, { shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [solid(token("highlight"))] }, () => {
              text("Text", desc.nowLabel ?? "Now", { step: ts(11), font: "display", color: token("inkInverse"), nowrap: true });
            });
          }
        }, {
          opacity: un.dead ? 0.45 : 1,
          a11y: a11y("image", un.name ?? "", { value: cur ? desc.nowLabel ?? "Now" : `${i + 1}` }),
        });
      });
    }, { a11y: a11y("list", "Turn order") });
  });
}


