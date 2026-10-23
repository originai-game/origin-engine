// HUDs: platformer, shooter, puzzle, idle/incremental and racing, each a
// set of anchored groups over the game in the safe area, plus the touch
// controls they show on phones and tablets. HUDs turn keyboard and gamepad
// navigation off (the keys move the player); Back still answers `paused`,
// except under an open dialog, which keeps it (og.ui.back). Every word a HUD
// draws or names is a field of its desc with the English default (og-i18n
// §2), so a game in another language has none of the kit's English left.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { all, capsule, clock, fit, fmt, frame, grow, iconOr, lapTime, px, token } from "./util.ts";

type Hud = { paused: boolean; action: string | null; moveX: number; moveY: number; held: string[] };

function hud(): Hud {
  kit.setNavigation(false);
  return { paused: kit.back(), action: null, moveX: 0, moveY: 0, held: [] };
}

/** The pause button: its icon, named by the desc's `pauseLabel` (what a screen reader says). */
function pauseButton(out: Hud, label: string): void {
  const r = kit.actionButton("Pause", label, { size: 44, icon: "pause" });
  if (r.clicked) out.paused = true;
}

/** A big number with a small caption under it. */
function stat(id: string, value: string, caption: string, step: Api.TypeStepName = "xl3", align: Api.TextAlign = "start"): void {
  kit.column(id, { gap: 0, cross: align === "end" ? "end" : align === "center" ? "center" : "start" }, () => {
    kit.label("Value", value, { step, font: "mono", nowrap: true, align });
    kit.label("Caption", caption, { step: "meta", color: token("inkMuted"), nowrap: true, align });
  });
}

/** Touch controls: a d-pad or a stick bottom left, action buttons bottom right. */
function touchControls(out: Hud, desc: { dpad?: boolean; buttons: ReadonlyArray<{ id: string; label: string; primary?: boolean; icon?: string }> }): void {
  const { t } = frame();
  kit.anchor("TouchMove", { anchor: "bottomLeft" }, () => {
    if (desc.dpad) {
      const d = kit.dpad("Dpad", {});
      out.moveX = (d.right ? 1 : 0) - (d.left ? 1 : 0);
      out.moveY = (d.down ? 1 : 0) - (d.up ? 1 : 0);
    } else {
      const j = kit.joystick("Stick", {});
      out.moveX = j.x;
      out.moveY = j.y;
    }
  });
  kit.anchor("TouchActions", { anchor: "bottomRight", gap: t.md, cross: "end" }, () => {
    for (const b of desc.buttons) {
      const r = kit.actionButton(b.id, b.label, { primary: b.primary ?? false, icon: b.icon ?? "", size: b.primary ? 76 : 60 });
      if (r.pressed) out.held.push(b.id);
      if (r.clicked) out.action = b.id;
    }
  });
}

export function platformerHud(id: string, desc: Api.PlatformerHudDesc = {}): Api.HudResult {
  const { t, touch } = frame();
  const out = hud();
  // What the game leaves out is not drawn: no hearts with maxLives 0 (and
  // no health bar), no coins or score capsule without coins or score.
  const health = desc.health ?? null;
  const maxLives = desc.maxLives ?? 3;
  const coins = desc.coins ?? null;
  const score = desc.score ?? null;
  if (health !== null || maxLives > 0 || desc.level) {
    kit.anchor(`${id}.left`, { anchor: "topLeft", direction: "column", gap: t.sm }, () => {
      if (health !== null) capsule("Lives", () => kit.meter("Health", health, 1, { width: px(120) }));
      else if (maxLives > 0) {
        capsule("Lives", () => {
          for (let i = 0; i < maxLives; i++) iconOr(kit.idi("Life", i), "heart", 20, token(i < (desc.lives ?? 0) ? "danger" : "controlStrong"));
        });
      }
      if (desc.level) kit.label("Level", desc.level, { step: "meta", color: token("inkMuted"), nowrap: true });
    });
  }
  if (desc.timeLeft !== null && desc.timeLeft !== undefined) {
    const low = desc.timeLeft <= 10;
    kit.anchor(`${id}.top`, { anchor: "top" }, () => {
      capsule("Timer", () => kit.label("Time", clock(desc.timeLeft!), { step: "lg", font: "mono", color: token(low ? "danger" : "ink"), nowrap: true }));
    });
  }
  kit.anchor(`${id}.right`, { anchor: "topRight", gap: t.sm, cross: "center" }, () => {
    if (coins !== null) {
      capsule("Coins", () => {
        iconOr("Coin", "coin", 20, token("gold"));
        kit.label("Count", fmt(coins), { font: "mono", nowrap: true });
      });
    }
    if (score !== null) capsule("Score", () => kit.label("Points", fmt(score), { font: "mono", nowrap: true }));
    pauseButton(out, desc.pauseLabel ?? "Pause");
  });
  if ((desc.touch ?? false) && touch) touchControls(out, { dpad: true, buttons: [{ id: "jump", label: desc.jumpLabel ?? "Jump", primary: true }] });
  return out;
}

/** A crosshair at the centre: four short strokes and a gap. */
function crosshair(): void {
  const ink = token("ink", 0.9);
  const bar = (id: string, w: number, h: number): void => kit.image(id, { tint: ink, width: px(w), height: px(h) });
  kit.anchor("Crosshair", { anchor: "center", direction: "column", cross: "center", gap: 6 }, () => {
    bar("N", 2, 8);
    kit.row("Mid", { gap: 14, cross: "center" }, () => {
      bar("W", 8, 2);
      bar("E", 8, 2);
    });
    bar("S", 2, 8);
  });
}

export function shooterHud(id: string, desc: Api.ShooterHudDesc = {}): Api.HudResult {
  const { t, touch } = frame();
  const out = hud();
  const showTouch = (desc.touch ?? false) && touch;
  const maxHealth = desc.maxHealth ?? 100;
  const health = desc.health ?? 0;
  // Vitals sit bottom left, or top left when the stick takes that corner.
  kit.anchor(`${id}.vitals`, { anchor: showTouch ? "topLeft" : "bottomLeft", direction: "column", gap: t.xs, width: px(240) }, () => {
    kit.meter("Health", health, maxHealth, { kind: "health", label: `${Math.ceil(health)}`, width: grow() });
    if ((desc.maxArmor ?? 0) > 0) kit.meter("Armor", desc.armor ?? 0, desc.maxArmor!, { color: token("info"), label: `${Math.ceil(desc.armor ?? 0)}`, width: grow() });
  });
  if (desc.map && !showTouch) kit.anchor(`${id}.map`, { anchor: "topLeft" }, () => kit.minimap("Map", desc.map!));
  kit.anchor(`${id}.top`, { anchor: "topRight", gap: t.sm, cross: "center" }, () => {
    capsule("Kills", () => {
      kit.label("Count", fmt(desc.kills ?? 0), { font: "mono", nowrap: true });
      kit.label("Unit", desc.killsLabel ?? "kills", { step: "meta", color: token("inkMuted"), nowrap: true });
    });
    pauseButton(out, desc.pauseLabel ?? "Pause");
  });
 kit.anchor(`${id}.ammo`, { anchor: showTouch ? "topRight" : "bottomRight", offsetY: showTouch ? 56 : 0, direction: "column", cross: "end", gap: t.xs }, () => {
    const mag = desc.magazine ?? 0;
    const ammo = desc.ammo ?? 0;
    kit.row("Rounds", { gap: t.xs, cross: "end" }, () => {
      kit.label("Ammo", `${ammo}`, { step: "xl5", font: "mono", color: token(mag > 0 && ammo <= Math.floor(mag / 4) ? "warn" : "ink"), nowrap: true });
      kit.label("Reserve", `/ ${desc.reserve ?? 0}`, { step: "lg", font: "mono", color: token("inkMuted"), nowrap: true });
    });
    if (desc.weapon) kit.label("Weapon", desc.weapon, { step: "meta", color: token("inkMuted"), nowrap: true });
  });
  const abilities = desc.abilities ?? [];
  if (abilities.length > 0) {
    // Above the touch controls when they show, else along the bottom edge.
    kit.anchor(`${id}.abilities`, showTouch ? { anchor: "right", direction: "column", gap: t.md, offsetY: 120 } : { anchor: "bottom", gap: t.md }, () => {
      abilities.forEach((a, i) => {
        const r = kit.cooldown(a.id ?? kit.idi("Ability", i), a.remaining ?? 0, { label: a.key ?? "", icon: a.icon ?? "", size: 56 });
        if (r.clicked) out.action = a.id ?? `ability${i}`;
      });
    });
  }
  if (desc.crosshair ?? true) crosshair();
  if (showTouch) touchControls(out, { buttons: [{ id: "fire", label: desc.fireLabel ?? "Fire", primary: true }] });
  return out;
}

export function puzzleHud(id: string, desc: Api.PuzzleHudDesc = {}): Api.HudResult {
  const { t } = frame();
  const out = hud();
  const target = Math.max(1, desc.target ?? 1000);
  const score = desc.score ?? 0;
  const marks = desc.stars && desc.stars.length > 0 ? desc.stars : [Math.round(target / 3), Math.round((2 * target) / 3), target];
  kit.anchor(`${id}.left`, { anchor: "topLeft", direction: "column", gap: t.xs }, () => {
    if (desc.level) kit.label("Level", desc.level, { step: "meta", color: token("inkMuted"), nowrap: true });
    if (desc.moves !== null && desc.moves !== undefined) stat("Moves", `${desc.moves}`, desc.movesLabel ?? "moves left", "xl3");
  });
  kit.anchor(`${id}.score`, { anchor: "top", direction: "column", cross: "center", gap: t.xs, width: px(260) }, () => {
    kit.label("Score", fmt(score), { step: "displaySm", font: "display", align: "center", width: grow() });
    kit.meter("Progress", score, target, { kind: "primary", width: grow(), segments: 3 });
    kit.row("Stars", { gap: t.sm, cross: "center" }, () => {
      marks.slice(0, 3).forEach((m, i) => iconOr(kit.idi("Star", i), "star", 20, token(score >= m ? "primary" : "controlStrong")));
      kit.label("Target", (desc.targetLabel ?? "Target {target}").replaceAll("{target}", fmt(target)), { step: "meta", color: token("inkMuted"), nowrap: true });
    });
  });
  kit.anchor(`${id}.right`, { anchor: "topRight", gap: t.sm, cross: "center" }, () => {
    if (desc.timeLeft !== null && desc.timeLeft !== undefined) {
      capsule("Timer", () => kit.label("Time", clock(desc.timeLeft!), { font: "mono", color: token(desc.timeLeft! <= 10 ? "danger" : "ink"), nowrap: true }));
    }
    pauseButton(out, desc.pauseLabel ?? "Pause");
  });
  return out;
}

export function idleHud(id: string, desc: Api.IdleHudDesc = {}): Api.HudResult {
  const { t, phone } = frame();
  const out = hud();
  const currencies = desc.currencies ?? [];
  const generators = desc.generators ?? [];
  kit.anchor(`${id}.wallet`, { anchor: "top", gap: t.sm, cross: "center" }, () => {
    currencies.forEach((c, i) => {
      capsule(c.id ?? kit.idi("Currency", i), () => {
        if (c.icon) iconOr("Icon", c.icon, 20, token("primary"));
        kit.label("Amount", fmt(c.amount ?? 0), { step: i === 0 ? "lg" : "body", font: "mono", nowrap: true });
        if ((c.perSecond ?? 0) !== 0) kit.label("Rate", (desc.rateLabel ?? "+{rate}/s").replaceAll("{rate}", fmt(c.perSecond ?? 0)), { step: "meta", color: token("success"), nowrap: true });
      }, { height: i === 0 ? t.controlLg : t.controlMd });
    });
    pauseButton(out, desc.pauseLabel ?? "Pause");
  });
  if (desc.tapLabel !== "") {
    kit.anchor(`${id}.tap`, { anchor: phone ? "top" : "center", offsetY: phone ? 96 : 0, offsetX: phone ? 0 : -150 }, () => {
      const r = kit.actionButton("Tap", desc.tapLabel ?? "Tap", { primary: true, size: phone ? 120 : 160 });
      if (r.clicked) out.action = "tap";
    });
  }
  const listHeight = phone ? Math.min(320, frame().viewport.height * 0.45) : Math.min(420, frame().viewport.height - 160);
  kit.anchor(`${id}.shop`, { anchor: phone ? "bottom" : "right", width: phone ? grow() : px(340) }, () => {
    kit.panel("Generators", { width: grow(), padding: all(t.sm), gap: t.xs }, () => {
      kit.list("List", { count: generators.length, rowHeight: 64, gap: t.xs, height: fit(0, listHeight), width: grow() }, (i) => {
        const g = generators[i]!;
        if (g.icon) iconOr("Icon", g.icon, 32, token("ink"));
        kit.column("Text", { width: grow(), gap: t.xxs }, () => {
          kit.row("Head", { gap: t.sm, cross: "center", width: grow() }, () => {
            kit.label("Name", g.name ?? g.id ?? "", { step: "lg", nowrap: true, width: grow() });
            kit.badge("Level", (desc.levelLabel ?? "Lv {level}").replaceAll("{level}", `${g.level ?? 0}`), "neutral");
          });
          kit.meter("Cycle", g.progress ?? 0, 1, { kind: "primary", height: 4, width: grow() });
        });
        const buy = kit.button("Buy", fmt(g.cost ?? 0), { size: "small", variant: g.affordable ? "primary" : "secondary", disabled: !g.affordable }) as Api.Response;
        if (buy.clicked) out.action = `buy:${g.id ?? i}`;
      });
    });
  });
  return out;
}

/** A delta in seconds, signed with the ASCII minus every pack's charset holds: `-0.412`, `+1.050`. */
function deltaText(seconds: number): string {
  const a = Math.abs(seconds);
  const body = a >= 60 ? lapTime(a) : a.toFixed(3);
  return `${seconds < 0 ? "-" : "+"}${body}`;
}

/** A delta's ink: ahead (negative) success, behind danger, even muted. */
function deltaInk(seconds: number): Api.Paint {
  return token(seconds < 0 ? "success" : seconds > 0 ? "danger" : "inkMuted");
}

export function racingHud(id: string, desc: Api.RacingHudDesc = {}): Api.HudResult {
  const { t, touch, phone } = frame();
  const out = hud();
  const showTouch = (desc.touch ?? false) && touch;
  const racers = desc.racers ?? 8;
  const boost = (desc.boostMeter ?? true) ? (desc.boost ?? 0) : null;
  const boostLabel = desc.boostLabel ?? "Boost";
  kit.anchor(`${id}.place`, { anchor: "topLeft", gap: t.lg }, () => {
    // A time trial (racers 0) has no place.
    if (racers > 0) {
      kit.row("Position", { gap: t.xxs, cross: "end" }, () => {
        kit.label("Place", `${desc.position ?? 1}`, { step: "displayXl", font: "display", nowrap: true });
        kit.label("Of", `/${racers}`, { step: "xl", font: "mono", color: token("inkMuted"), nowrap: true });
      });
    }
    stat("Lap", `${Math.min(desc.lap ?? 1, desc.laps ?? 3)}/${desc.laps ?? 3}`, desc.lapLabel ?? "lap", "xl3");
  });
  kit.anchor(`${id}.times`, { anchor: "topRight", direction: "column", cross: "end", gap: t.xxs }, () => {
    kit.row("Top", { gap: t.sm, cross: "center" }, () => {
      kit.label("Time", clock(desc.time ?? 0, true), { step: "xl", font: "mono", nowrap: true });
      pauseButton(out, desc.pauseLabel ?? "Pause");
    });
    const delta = desc.delta ?? null;
    if (delta !== null) kit.label("Delta", deltaText(delta), { step: "lg", font: "mono", color: deltaInk(delta), nowrap: true });
    if (desc.best !== null && desc.best !== undefined) {
      // A finished lap: m:ss.mmm as lapTime cuts it (30.717 s is 0:30.717, where the running clock's floor reads 0:30.716).
      kit.label("Best", (desc.bestLabel ?? "Best {best}").replaceAll("{best}", lapTime(desc.best)), { step: "meta", font: "mono", color: token("inkMuted"), nowrap: true });
    }
    const splits = desc.splits ?? [];
    if (splits.length > 0) {
      kit.column("Splits", { gap: 0, cross: "end" }, () => {
        splits.forEach((sp, i) => {
          kit.row(kit.idi("Split", i), { gap: t.sm, cross: "center" }, () => {
            kit.label("Label", sp.label ?? "", { step: "meta", color: token("inkMuted"), nowrap: true });
            const time = sp.time ?? null;
            kit.label("Time", time === null ? "-:--.---" : lapTime(time), { step: "meta", font: "mono", color: token(time === null ? "inkFaint" : "ink"), nowrap: true });
            const d = sp.delta ?? null;
            if (d !== null) kit.label("Delta", deltaText(d), { step: "meta", font: "mono", color: deltaInk(d), nowrap: true });
          });
        });
      });
    }
    if (desc.map) kit.minimap("Map", { ...desc.map, size: phone ? 112 : (desc.map.size ?? 140) });
  });
  kit.anchor(`${id}.dash`, { anchor: showTouch ? "bottom" : "bottomRight", offsetY: showTouch ? -120 : 0, direction: "column", cross: "end", gap: t.xs, width: px(220) }, () => {
    kit.row("Speed", { gap: t.sm, cross: "end", width: grow(), align: "end" }, () => {
      kit.label("Value", `${Math.round(desc.speed ?? 0)}`, { step: "displayXl2", font: "display", nowrap: true });
      kit.column("Side", { cross: "end" }, () => {
        kit.label("Gear", desc.gear ?? "N", { step: "xl", font: "mono", color: token("highlight"), nowrap: true });
        kit.label("Unit", desc.unit ?? "km/h", { step: "meta", color: token("inkMuted"), nowrap: true });
      });
    });
    kit.meter("Rev", desc.speed ?? 0, desc.maxSpeed ?? 300, { kind: "primary", width: grow(), height: 6 });
    if (boost !== null) kit.meter("Boost", boost, 1, { color: token("info"), label: boostLabel, width: grow(), segments: 4 });
  });
  if (showTouch) {
    kit.anchor("Steer", { anchor: "bottomLeft", gap: t.md }, () => {
      const l = kit.actionButton("left", desc.leftLabel ?? "Left", { size: 72 });
      const r = kit.actionButton("right", desc.rightLabel ?? "Right", { size: 72 });
      out.moveX = (r.pressed ? 1 : 0) - (l.pressed ? 1 : 0);
    });
    kit.anchor("Pedals", { anchor: "bottomRight", gap: t.md, cross: "end" }, () => {
      const pedals = [{ id: "brake", label: desc.brakeLabel ?? "Brake", primary: false }];
      if (boost !== null) pedals.push({ id: "boost", label: boostLabel, primary: true });
      for (const b of pedals) {
        const r = kit.actionButton(b.id, b.label, { primary: b.primary, size: b.primary ? 84 : 64 });
        if (r.pressed) out.held.push(b.id);
        if (r.clicked) out.action = b.id;
      }
    });
  }
  return out;
}
