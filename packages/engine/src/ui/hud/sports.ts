// og.ui.matchStats, og.ui.goalTimeline, og.ui.playerPlate and
// og.ui.substitution: the sports rows of the genre sheets (B-sports-hud,
// B-sports-scoreboard). Read against EA FC's and NBA 2K's broadcast
// graphics: match facts as two sides' values at the ends of one bar split
// at their shares, each side in its team colour; the
// scorers set against a centre rule; the controlled player's plate with a
// segmented stamina bar; a substitution's lower third with a down mark for
// the player going off and an up mark for the one coming on. The pack
// dresses them: nothing here draws a glow edge or a bevel of a particular
// look.

import type * as Api from "@origingame/engine";
import { a11y, absolute, anchorOf, box, chip, clamp01, easeOut, edge, frame, groups, hudPlace, hudRoom, icon, keyId, kit, leaf, lit, mix, msToTicks, paintRgba, pct, px, reduced, role, sel, solid, stepSize, stroke, teamColor, text, token, ts, u, ut, type Rgba } from "./core.ts";

const BLACK: Rgba = { r: 0, g: 0, b: 0, a: 1 };

function sideColor(team: Api.ScorebugTeam | undefined, k: number): Rgba {
  return team?.color ? paintRgba(team.color, teamColor(k)) : teamColor(k);
}

/** A value as printed: its text, else the number without a trailing .0. */
function shown(text: string | undefined, v: number): string {
  return text || (Number.isInteger(v) ? String(v) : v.toFixed(1));
}

/** A crest at `side` px, or nothing. */
function crest(id: string, logo: Api.Texture | null | undefined, side: number): void {
  if (logo) kit.image(id, { texture: logo, width: px(side), height: px(side), label: "" });
}

// ── match stats ───────────────────────────────────────────────────────

export function matchStats(id: string, desc: Api.MatchStatsDesc = {}): void {
  const tk = frame().t;
  const stats = desc.stats ?? [];
  const home = sideColor(desc.home, 0);
  const away = sideColor(desc.away, 1);
  const anchor = anchorOf(desc, null);
  hudPlace(id, anchor, (plate) => {
    const valStep = ts(plate ? 17 : 20, true);
    // Both value columns as wide as the widest value, so every row's bar starts and ends at the same x.
    const valW = Math.ceil(stepSize(valStep).size * 2.6);
    const barH = Math.max(4, u(plate ? 6 : 6));
    const crestSide = Math.round(stepSize(valStep).line * 1.25);
    // A plate's bar takes what its row leaves of the room: the band, or half of it when the other corner of the same
    // edge holds something (a scorebug top-start), so the two never meet. Never longer than the design's.
    const other = anchor === "topRight" ? "topLeft" : anchor === "topLeft" ? "topRight" : anchor === "bottomRight" ? "bottomLeft" : anchor === "bottomLeft" ? "bottomRight" : null;
    const shared = other !== null && kit.memory(`~hudStack/${other}/tick`, -10) >= kit.tick() - 1 && kit.memory(`~hudStack/${other}/total`, 0) > 0;
    const room = shared ? (hudRoom() - tk.gutter) / 2 : hudRoom();
    const base = 2 * valW + 2 * tk.sm + 2 * tk.md;
    const logos = (desc.home?.logo ? 1 : 0) + (desc.away?.logo ? 1 : 0);
    // The crests go first when the row is tight (a phone at 200 %): the values and the bar are what the plate says.
    const crests = plate && logos > 0 && room - base - logos * (crestSide + tk.sm) >= u(80);
    const plateBar = Math.max(u(48), Math.min(u(170), room - base - (crests ? logos * (crestSide + tk.sm) : 0)));
    box(id, {
      direction: "column", gap: plate ? u(4) : tk.sm, cross: plate ? "center" : "stretch",
      // A plate fits its row (the bar a fixed length, the values and crests growing with the text); a board fills its parent.
      width: plate ? null : pct(1),
      padding: plate ? { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm } : { left: 0, right: 0, top: 0, bottom: 0 },
      part: sel("MatchStats", "frame"), partClass: "hud",
    }, plate ? chip() : undefined, () => {
      if (desc.title) {
        text("Title", desc.title, { step: ts(plate ? 11 : 14, true), font: "display", color: token(plate ? "inkMuted" : "ink"), nowrap: true, ...(plate ? {} : { width: pct(1) }) });
      }
      if (desc.header && !plate) {
        kit.row("Header", { gap: tk.sm, cross: "center", width: pct(1), part: sel("MatchStats", "header") }, () => {
          text("Home", desc.home?.name || desc.home?.abbr || "", { step: ts(12, true), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: true, width: { kind: "grow" } });
          text("Away", desc.away?.name || desc.away?.abbr || "", { step: ts(12, true), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: true, align: "end", width: { kind: "grow" } });
        });
      }
      for (const st of stats) {
        const h = Math.max(0, st.home ?? 0);
        const a = Math.max(0, st.away ?? 0);
        const share = h + a > 0 ? h / (h + a) : 0.5;
        const worse = !!st.worse;
        // The bar is always both teams' colours (EA FC's split); the leader's numeral takes its colour, lightened, and
        // a stat where more is worse (fouls) tints neither numeral: leading it is nothing to light up.
        const lead = worse || h === a ? 0 : h > a ? 1 : -1;
        const valueInk = (c: Rgba | null) => (c ? lit(mix(c, { r: 1, g: 1, b: 1, a: 1 }, 0.35)) : token("ink"));
        const homeInk = lead > 0 ? home : null;
        const awayInk = lead < 0 ? away : null;
        kit.row(keyId(st.key ?? st.label ?? ""), { gap: tk.sm, cross: "end", width: plate ? null : pct(1), part: sel("MatchStats", "row", worse ? "worse" : null) }, () => {
          if (crests) crest("HomeCrest", desc.home?.logo, crestSide);
          text("Home", shown(st.homeText, h), { step: valStep, font: "display", color: valueInk(homeInk), nowrap: true, width: px(valW) });
          kit.column("Mid", { gap: u(4), cross: "stretch", width: plate ? px(plateBar) : { kind: "grow" }, padding: { left: 0, right: 0, top: 0, bottom: Math.round((stepSize(valStep).line - barH) / 2) } }, () => {
            if (!plate && st.label) text("Label", st.label, { step: ts(11, true), font: "display", color: token("inkMuted"), nowrap: true, ellipsis: true, align: "center", width: pct(1) });
            // The home colour from the start to its share, the away colour from the end, a 2 px gap where they meet.
            kit.row("Bar", { gap: 2, width: pct(1), height: px(barH), part: sel("MatchStats", "bar") }, () => {
              const r = barH / 2;
              leaf("Home", { width: pct(share), height: pct(1), part: sel("MatchStats", "fill", "home") }, { shape: { kind: "rect", radii: [r, 0, 0, r] }, fills: [solid(lit(home))] });
              leaf("Away", { width: { kind: "grow" }, height: pct(1), part: sel("MatchStats", "fill", "away") }, { shape: { kind: "rect", radii: [0, r, r, 0] }, fills: [solid(lit(away))] });
              // A tick at the bar's middle, standing over it (B-sports-scoreboard): how far the split is from even reads at a glance.
              const tickH = barH + Math.max(4, u(6));
              leaf("Centre", { width: px(Math.max(1, u(2))), height: px(tickH), part: sel("MatchStats", "centre") }, { fills: [solid(token("ink", 0.7))] }, absolute({ centerX: true, top: barH - tickH }));
            });
          });
          text("Away", shown(st.awayText, a), { step: valStep, font: "display", color: valueInk(awayInk), nowrap: true, align: "end", width: px(valW) });
          if (crests) crest("AwayCrest", desc.away?.logo, crestSide);
        });
        kit.semantics(a11y("text", st.label ?? "", { value: `${shown(st.homeText, h)} to ${shown(st.awayText, a)}` }));
      }
    }, { a11y: a11y("table", desc.title || "Match stats") });
  });
}

// ── goal timeline ─────────────────────────────────────────────────────

/** An event's mark: its icon, else a ball (a light disc with a dark ring). */
function mark(ev: Api.MatchEvent, side: number): void {
  if (ev.icon && kit.hasIcon(ev.icon)) {
    icon("Mark", ev.icon, side, token("ink"));
    return;
  }
  leaf("Mark", { width: px(side), height: px(side), part: sel("GoalTimeline", "mark") }, {
    shape: { kind: "ellipse" },
    fills: [solid(token("ink"))],
    strokes: [stroke(Math.max(1, side / 7), lit(BLACK, 0.55), "inside")],
  });
}

export function goalTimeline(id: string, desc: Api.GoalTimelineDesc = {}): void {
  const tk = frame().t;
  const events = desc.events ?? [];
  const nameStep = ts(13, true);
  const markSide = Math.round(stepSize(nameStep).size * 0.95);
  const minW = Math.ceil(stepSize(nameStep).size * 2.6);
  hudPlace(id, anchorOf(desc, null), (inAnchor) => {
    box(id, {
      direction: "row", gap: tk.lg, cross: "stretch", width: inAnchor ? px(ut(560)) : pct(1),
      padding: inAnchor ? { left: tk.md, right: tk.md, top: tk.sm, bottom: tk.sm } : { left: 0, right: 0, top: 0, bottom: 0 },
      part: sel("GoalTimeline", "frame"), partClass: "hud",
    }, inAnchor ? chip() : undefined, () => {
      for (const [sid, team] of [["Home", 0], ["Away", 1]] as const) {
        if (team === 1) leaf("Rule", { width: px(1), height: { kind: "grow" }, part: sel("GoalTimeline", "rule") }, { fills: [solid(token("line", 0.9))] });
        // The home column reads into the rule (name, minute, mark); the away one out of it (mark, minute, name).
        kit.column(sid, { gap: u(6), cross: team === 0 ? "end" : "start", width: { kind: "grow" }, part: sel("GoalTimeline", "side", sid.toLowerCase()) }, () => {
          for (const ev of events.filter((e) => (e.team ?? 0) === team)) {
            kit.row(keyId(ev.key ?? `${ev.name}${ev.minute}`), { gap: u(10), cross: "center", part: sel("GoalTimeline", "event") }, () => {
              const name = () => text("Name", ev.note ? `${ev.name ?? ""} (${ev.note})` : ev.name ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true });
              const minute = () => text("Minute", ev.minute ?? "", { step: nameStep, font: "mono", color: lit(mix(teamColor(team), { r: 1, g: 1, b: 1, a: 1 }, 0.35)), nowrap: true, align: team === 0 ? "end" : "start", width: px(minW) });
              if (team === 0) { name(); minute(); mark(ev, markSide); } else { mark(ev, markSide); minute(); name(); }
            });
            kit.semantics(a11y("text", ev.name ?? "", { value: `${ev.minute ?? ""} ${team === 0 ? "home" : "away"}` }));
          }
        });
      }
    }, { a11y: a11y("list", "Scorers") });
  });
}

// ── player plate ──────────────────────────────────────────────────────

export function playerPlate(id: string, desc: Api.PlayerPlateDesc = {}): void {
  const tk = frame().t;
  const c = desc.color ? paintRgba(desc.color, teamColor(0)) : teamColor(0);
  const stamina = clamp01(desc.stamina ?? 1);
  const segs = Math.max(1, desc.segments || 6);
  const low = stamina < 0.3;
  const numStep = ts(18, true);
  const nameStep = ts(15, true);
  const h = Math.ceil(stepSize(numStep).line) + tk.sm * 2;
  hudPlace(id, anchorOf(desc, "bottomLeft"), () => {
    box(id, { direction: "row", gap: 0, cross: "stretch", height: px(h), part: sel("PlayerPlate", "frame"), partClass: "hud" }, { ...chip(), clip: "shape" }, () => {
      if (desc.logo) {
        box("Crest", { width: px(h), height: pct(1), align: "center", cross: "center", part: sel("PlayerPlate", "crest") }, { fills: [solid(lit(mix(c, BLACK, 0.35), 0.85))] }, () => crest("Logo", desc.logo, Math.round(h * 0.72)));
      }
      kit.row("Body", { gap: tk.sm, cross: "center", padding: { left: tk.sm, right: tk.md, top: 0, bottom: 0 } }, () => {
        box("Number", { padding: { left: u(6), right: u(6), top: 0, bottom: 0 }, align: "center", cross: "center", part: sel("PlayerPlate", "number") }, {
          shape: { kind: "rect", radii: [3, 3, 3, 3] }, strokes: [stroke(1, token("lineStrong", 0.8), "inside")],
        }, () => text("Text", desc.number ?? "", { step: numStep, font: "display", color: token("ink"), nowrap: true }));
        // The stamina bar keeps the design's length at any text size; the name grows over it.
        kit.column("Info", { gap: u(4), width: px(u(170)) }, () => {
          text("Name", desc.name ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true, ellipsis: true, width: pct(1) });
          const segH = Math.max(4, u(5));
          const ink = low ? lit(role("warn")) : lit(mix(c, { r: 1, g: 1, b: 1, a: 1 }, 0.3));
          kit.row("Stamina", { gap: Math.max(1, u(2)), width: pct(1), height: px(segH), part: sel("PlayerPlate", "stamina", low ? "low" : null) }, () => {
            for (let i = 0; i < segs; i++) {
              const f = clamp01(stamina * segs - i);
              box(kit.idi("Seg", i), { width: { kind: "grow" }, height: pct(1), part: sel("PlayerPlate", "segment", f <= 0 ? "spent" : null) }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(lit(BLACK, 0.55))] }, () => {
                if (f > 0) leaf("Fill", { width: pct(f), height: pct(1) }, { shape: { kind: "rect", radii: [1, 1, 1, 1] }, fills: [solid(ink)] });
              });
            }
          });
        });
      });
    }, {
      a11y: a11y("status", `${desc.number ?? ""} ${desc.name ?? ""}`.trim(), { value: `${Math.round(stamina * 100)}% stamina` }),
      ...groups({ stamina: low ? "low" : "fresh" }, { stamina: ["fresh", "low"] }),
    });
  });
}

// ── substitution ──────────────────────────────────────────────────────

export function substitution(id: string, desc: Api.SubstitutionDesc = {}): void {
  const tk = frame().t;
  const off = desc.off ?? {};
  const on = desc.on ?? {};
  const nameStep = ts(16, true);
  const labelStep = ts(11, true);
  const port = Math.ceil(stepSize(nameStep).line + stepSize(labelStep).line) + tk.sm * 2;
  // It rises in when it appears, or when the players change.
  const e = edge("who", (`${off.name}|${on.name}`.length * 31 + (off.number ?? "").length * 7 + (on.number ?? "").length) | 0);
  const x = reduced() ? 1 : easeOut(Math.min(1, e.since / msToTicks(320)));
  hudPlace(id, anchorOf(desc, "bottom"), () => {
    kit.column(id, { gap: 0, cross: "center" }, () => {
      box("Caption", { padding: { left: tk.md, right: tk.md, top: u(2), bottom: u(2) }, part: sel("Substitution", "caption") }, {
        shape: { kind: "rect", radii: [4, 4, 0, 0] }, fills: [solid(token("hud", 0.94))], strokes: [stroke(1, token("lineStrong", 0.55))],
      }, () => text("Text", desc.caption ?? "Substitution", { step: labelStep, font: "display", color: token("primary"), nowrap: true }));
      box("Plate", { direction: "row", gap: tk.md, cross: "center", padding: { left: 0, right: 0, top: 0, bottom: 0 }, part: sel("Substitution", "frame"), partClass: "hud" }, { ...chip(), clip: "shape" }, () => {
        const player = (pid: "Off" | "On", p: Api.SubPlayer, going: boolean) => {
          const ink = going ? role("danger") : role("success");
          const portrait = () => box("Portrait", { width: px(port), height: px(port), part: sel("Substitution", "portrait") }, { fills: [solid(lit(BLACK, 0.5))], clip: "shape" }, () => {
            if (p.image) kit.image("Art", { texture: p.image, width: px(port), height: px(port), label: "" });
          });
          kit.row(pid, { gap: tk.sm, cross: "center", part: sel("Substitution", "player", going ? "off" : "on") }, () => {
            if (!going) leaf("Mark", { width: px(u(14)), height: px(u(12)), part: sel("Substitution", "mark", "on") }, { shape: { kind: "polygon", sides: 3 }, fills: [solid(lit(ink))] });
            if (going) portrait();
            if (going) leaf("Mark", { width: px(u(14)), height: px(u(12)), part: sel("Substitution", "mark", "off") }, { shape: { kind: "polygon", sides: 3, rotation: Math.PI }, fills: [solid(lit(ink))] });
            kit.column("Who", { gap: 0 }, () => {
              kit.row("Line", { gap: u(8), cross: "end" }, () => {
                text("Number", p.number ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true });
                text("Name", p.name ?? "", { step: nameStep, font: "display", color: token("ink"), nowrap: true });
              });
              text("Label", going ? (desc.offLabel ?? "Out") : (desc.onLabel ?? "In"), { step: labelStep, font: "display", color: lit(mix(ink, { r: 1, g: 1, b: 1, a: 1 }, 0.2)), nowrap: true });
            });
            if (!going) portrait();
          });
        };
        player("Off", off, true);
        leaf("Rule", { width: px(1), height: px(Math.round(port * 0.6)), part: sel("Substitution", "rule") }, { fills: [solid(token("line", 0.9))] });
        player("On", on, false);
      });
    }, );
    kit.props({ transform: { y: (1 - x) * u(24) }, opacity: x });
    kit.semantics(a11y("status", desc.caption ?? "Substitution", { value: `${off.number ?? ""} ${off.name ?? ""} off, ${on.number ?? ""} ${on.name ?? ""} on` }));
  });
}
