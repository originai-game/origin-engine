// og.ui.celebration and og.ui.starRating: the celebration set (docs/ui.md
// §6, Appendix E "Celebrations (confetti, banners, podium, level up)").
// Read against Candy Crush and Royal Match's results (three stars landing
// in turn, the middle one larger, a burst and a rising pitch each), Diablo
// and WoW's level-up flourishes (a ring of light, rays turning behind the
// word), Fall Guys' confetti and the fireworks of party games. Particles
// are the UI's own, seeded, so a replay throws the same.

import type * as Api from "@origingame/engine";
import { a11y, absolute, box, clamp01, easeBack, easeOut, frame, groups, kit, leaf, lit, msToTicks, now, pct, px, radial, reduced, role, sel, solid, sound, stroke, text, token, glow, vgrad, fill as fillText, ts, apply, fitWidth, claimAnnouncement } from "./core.ts";

const DURATION_MS: Record<Api.CelebrationKind, number> = { confetti: 3000, burst: 1000, levelUp: 2600, fireworks: 3500 };

function palette(desc: { colors?: readonly Api.Paint[] }): Api.Paint[] {
  if (desc.colors && desc.colors.length > 0) return [...desc.colors];
  const tokens = frame().theme.tokens as { series?: string[] } | null;
  const series = (tokens?.series ?? []).slice(0, 4).map((h) => ({ color: hexRgb(h) }));
  return [token("primary"), token("gold"), token("highlight"), ...series];
}

function hexRgb(h: string): { r: number; g: number; b: number } {
  const n = (i: number) => Number.parseInt(h.slice(i, i + 2), 16) / 255;
  return { r: n(1), g: n(3), b: n(5) };
}

export function celebration(id: string, desc: Api.CelebrationDesc = {}): void {
  const kind: Api.CelebrationKind = desc.kind ?? "confetti";
  const t = now();
  const at = desc.at ?? 0;
  const age = t - at;
  const dur = msToTicks(desc.durationMs || DURATION_MS[kind]);
  const k = desc.intensity ?? 1;
  const colors = palette(desc);
  const playing = age >= 0 && age < dur;
  sound(kind === "levelUp" || kind === "fireworks" ? "reward" : "success", age === 0);
  const g = groups({ play: playing ? "playing" : "idle" }, { play: ["idle", "playing"] });
  const still = reduced();

  if (kind === "burst") {
    // From the open container's centre: a ring and sparks.
    const x = clamp01(age / dur);
    const parent = kit.openPath();
    box(id, { width: pct(1), height: pct(1), align: "center", cross: "center" }, undefined, () => {
      if (playing && !still) {
        leaf("Ring", { width: px(60), height: px(60), part: sel("Celebration", "ring", kind) }, {
          shape: { kind: "ellipse" },
          strokes: [stroke(4 * (1 - x) + 0.5, token("highlight"), "center")],
          effects: [glow(10, token("highlight", 0.6))],
        }, { transform: { scaleX: 0.4 + 2.4 * easeOut(x), scaleY: 0.4 + 2.4 * easeOut(x) }, opacity: 1 - x });
      }
      leaf("Sparks", { width: px(8), height: px(8) });
      kit.particles({ burst: Math.round(26 * k), fire: age === 0, colors, travelMin: 40 * k, travelMax: 140 * k, spin: 2, shape: "diamond", size: { x: 7, y: 7 }, align: true, endScale: 0.2 });
    }, { position: "absolute", insets: { start: 0, top: 0 }, inert: true, overlaps: parent ? [parent] : [], ...g });
    return;
  }

  if (kind === "confetti") {
    kit.anchor(id, { anchor: "top", width: pct(1), margin: 0, over: true }, () => {
      leaf("Edge", { width: pct(1), height: px(2) });
      const live = age >= 0 && age < msToTicks(1400);
      kit.particles({
        burst: Math.round(110 * k),
        fire: age === 0,
        rate: live ? 70 * k : 0,
        lifetimeMs: 2600,
        colors,
        shape: "square",
        size: { x: 10, y: 6 },
        direction: Math.PI / 2,
        spread: 0.9,
        travelMin: 260,
        travelMax: 620,
        gravity: 320,
        drag: true,
        spin: 6,
        from: "area",
        fade: true,
      });
    });
    apply({ inert: true, ...g });
    return;
  }

  if (kind === "fireworks") {
    kit.anchor(id, { anchor: "top", width: pct(1), height: pct(0.6), margin: 0, over: true }, () => {
      const vw = frame().viewport.width;
      const vh = frame().viewport.height * 0.6;
      const shells = [[0.22, 0.38, 0], [0.7, 0.3, 14], [0.46, 0.18, 30], [0.84, 0.46, 44], [0.12, 0.2, 58]] as const;
      box("Sky", { width: pct(1), height: pct(1) }, undefined, () => {
        shells.forEach(([sx, sy, delay], i) => {
          const local = age - delay;
          const flash = local >= 0 && local < 12 && !still ? 1 - local / 12 : 0;
          leaf(kit.idi("Shell", i), { width: px(10), height: px(10), part: sel("Celebration", "ring", kind) }, {
            shape: { kind: "ellipse" },
            fills: [solid(lit({ r: 1, g: 1, b: 1, a: 1 }, flash))],
            effects: flash > 0 ? [glow(24 * flash, colors[i % colors.length]!)] : [],
          }, absolute({ start: vw * sx, top: vh * sy }, { transform: { scaleX: 1 + 3 * (1 - flash), scaleY: 1 + 3 * (1 - flash) } }));
          kit.particles({ burst: Math.round(42 * k), fire: local === 0, colors: [colors[i % colors.length]!, token("highlight")], shape: "circle", size: { x: 5, y: 5 }, travelMin: 90 * k, travelMax: 170 * k, gravity: 70, lifetimeMs: 1500, endScale: 0.4, drag: true });
        });
      });
    });
    apply({ inert: true, ...g });
    return;
  }

  // levelUp: centred on the screen.
  if (!playing) return;
  const x = clamp01(age / dur);
  const enter = clamp01(age / msToTicks(450));
  const leave = clamp01((age - (dur - msToTicks(450))) / msToTicks(450));
  const gold = role("gold");
  const vp = frame().viewport;
  const stageW = Math.min(420, vp.width - vp.safeArea.left - vp.safeArea.right - frame().t.gutter * 2);
  const title = desc.title ?? "Level up";
  // The word fits the stage at its overshoot (1.1×): a long word or 200 % text steps down, never off the screen.
  const titleStep = fitWidth(ts(40, true), title.length * 0.6 * 1.1, stageW);
  // The level-up announces like a banner: over a pinned tooltip too, which steps under it and dims.
  claimAnnouncement();
  kit.anchor(id, { anchor: "center", direction: "column", cross: "center", over: true, tooltip: true }, () => {
    box("Stage", { width: px(stageW), height: px(300), align: "center", cross: "center", direction: "column" }, undefined, () => {
      if (!still) {
        // Rays turning behind the word, fading at their tips.
        leaf("Rays", { width: px(420), height: px(420), part: sel("Celebration", "rays") }, {
          shape: { kind: "polygon", sides: 14, inner: 0.18, rotation: age * 0.006 },
          fills: [radial([[0, lit(gold, 0.55)], [0.6, lit(gold, 0.16)], [1, lit(gold, 0)]])],
        }, absolute({ centerX: true, centerY: true }, { opacity: easeOut(enter) * (1 - leave) * 0.9, transform: { scaleX: 0.6 + 0.4 * easeOut(enter), scaleY: 0.6 + 0.4 * easeOut(enter) } }));
        // A ring of light expanding once.
        const rx = clamp01(age / msToTicks(800));
        leaf("Ring", { width: px(200), height: px(200), part: sel("Celebration", "ring", kind) }, {
          shape: { kind: "ellipse" },
          strokes: [stroke(6 * (1 - rx) + 1, lit(gold), "center")],
          effects: [glow(18, lit(gold, 0.7))],
        }, absolute({ centerX: true, centerY: true }, { opacity: 1 - rx, transform: { scaleX: 0.3 + 1.9 * easeOut(rx), scaleY: 0.3 + 1.9 * easeOut(rx) } }));
      } else {
        leaf("Glow", { width: px(260), height: px(160) }, { shape: { kind: "ellipse" }, fills: [radial([[0, lit(gold, 0.35)], [1, lit(gold, 0)]])] }, absolute({ centerX: true, centerY: true }, { opacity: 1 - leave }));
      }
      const s = still ? 1 : 0.4 + 0.6 * easeBack(enter, 2.4);
      // The word and the level share one ink: the gold gradient under a heavy outline, which holds 4.5:1 on a light scene.
      const goldInk: Api.Gradient = { type: "linear", from: { x: 0, y: 0 }, to: { x: 0, y: 1 }, stops: [{ offset: 0, color: lit({ r: 1, g: 0.97, b: 0.8, a: 1 }) }, { offset: 0.55, color: lit(gold) }, { offset: 1, color: lit({ r: gold.r * 0.8, g: gold.g * 0.7, b: gold.b * 0.5, a: 1 }) }] };
      text("Title", title, {
        uppercase: true,
        step: titleStep,
        font: "display",
        nowrap: true,
        color: lit(gold),
        outline: 1.6,
        gradient: goldInk,
      }, { transform: { scaleX: s, scaleY: s }, opacity: (still ? enter : 1) * (1 - leave), z: 2 });
      if (desc.value) {
        const vx = clamp01((age - msToTicks(300)) / msToTicks(220));
        text("Value", desc.value, { step: ts(48, true), font: "display", nowrap: true, color: lit(gold), outline: 1.8, gradient: goldInk }, { transform: { y: still ? 0 : 20 * (1 - easeOut(vx)) }, opacity: vx * (1 - leave), z: 2 });
      }
      leaf("Sparks", { width: px(300), height: px(4) }, undefined, absolute({ centerX: true, bottom: 40 }));
      kit.particles({ rate: age < msToTicks(1600) ? 40 : 0, burst: 30, fire: age === msToTicks(120), colors: [lit(gold), token("highlight")], direction: -Math.PI / 2, spread: 0.7, travelMin: 80, travelMax: 220, shape: "diamond", size: { x: 6, y: 6 }, lifetimeMs: 1200, from: "area", endScale: 0.3 });
    });
  });
  apply({ inert: true, a11y: a11y("status", [desc.title ?? "Level up", desc.value].filter((v) => v).join(" ")), ...g });
}

export function starRating(id: string, desc: Api.StarRatingDesc = {}): void {
  const max = Math.max(1, desc.max || 3);
  const earned = Math.min(max, desc.earned ?? 0);
  const base = desc.size || 48;
  const t = now();
  const stagger = msToTicks(desc.staggerMs || 320);
  const land = msToTicks(380);
  const reveal = desc.revealAt;
  const gold = role("gold");
  const still = reduced();
  const name = fillText(desc.name ?? "{earned} of {max} stars", { earned, max });
  box(id, { direction: "row", gap: Math.round(base * 0.16), cross: "end" }, undefined, () => {
    for (let i = 0; i < max; i++) {
      const middle = max === 3 && i === 1;
      const size = Math.round(middle ? base * 1.25 : base);
      const lift = middle ? -base * 0.18 : 0;
      const at = reveal === undefined || reveal === null ? -1e9 : reveal + i * stagger;
      const age = t - at;
      const isEarned = i < earned;
      const shown = isEarned && age >= 0;
      const x = clamp01(age / land);
      const landing = shown && x < 1;
      if (isEarned && age === land) kit.feedback("reward", i);
      const starShape: Api.DrawShape = { kind: "polygon", sides: 5, inner: 0.48, rounding: size * 0.06 };
      box(kit.idi("Star", i), { width: px(size), height: px(size), align: "center", cross: "center" }, undefined, () => {
        leaf("Socket", { width: px(size), height: px(size), part: sel("StarRating", "socket") }, {
          shape: starShape,
          fills: [solid(token("well"))],
          strokes: [stroke(2, token("line"), "inside")],
          effects: [{ kind: "innerShadow", offset: { x: 0, y: 2 }, blur: 4, spread: 0, color: token("shadow", 0.5) }],
        }, absolute({ start: 0, top: 0 }));
        if (shown) {
          const s = still ? 1 : landing ? 2.2 - 1.2 * easeBack(x, 2) : 1;
          const r = still ? 0 : landing ? -0.7 * (1 - easeOut(x)) : 0;
          leaf("Earned", { width: px(size), height: px(size), part: sel("StarRating", "star", "earned") }, {
            shape: starShape,
            fills: [vgrad([[0, lit({ r: 1, g: 0.95, b: 0.65, a: 1 })], [0.5, lit(gold)], [1, lit({ r: gold.r * 0.85, g: gold.g * 0.65, b: gold.b * 0.35, a: 1 })]])],
            strokes: [stroke(2, lit({ r: gold.r * 0.6, g: gold.g * 0.45, b: gold.b * 0.2, a: 1 }), "inside")],
            effects: [glow(landing ? 18 : 8, lit(gold, landing ? 0.8 : 0.45))],
          }, absolute({ start: 0, top: 0 }, { transform: { scaleX: s, scaleY: s, rotate: r }, opacity: still ? x : clamp01(x * 4), z: 1 }));
          kit.particles({ burst: 14, fire: age === land, colors: [lit(gold), token("highlight")], travelMin: size * 0.5, travelMax: size * 1.3, shape: "diamond", size: { x: 5, y: 5 }, endScale: 0.2 });
        }
      }, {
        transform: { y: lift },
        ...groups({ star: !isEarned ? "empty" : landing || age < 0 ? "landing" : "earned" }, { star: ["empty", "landing", "earned"] }),
      });
    }
  }, { a11y: a11y("image", name) });
}
