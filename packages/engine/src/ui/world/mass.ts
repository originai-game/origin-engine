// og.ui.overheadBars and og.ui.damageNumbers: the mass elements of §14 for
// what comes by the thousand. Read against StarCraft II's and League of
// Legends' overhead bars (a dark track, the fill in the unit's reaction,
// a ghost after a hit; hidden at full health in an RTS) and the combat
// text of Diablo IV, Genshin Impact and World of Warcraft (a number thrown
// from the hit, rising and drifting aside as it fades, crits larger and
// gold, popping past their size and settling, heals green with a +,
// damage over time smaller, misses as their word).

import type * as Api from "@origingame/engine";
import { clamp01, easeOut, kit, lit, massTemplate, msToTicks, now, paintRgba, reactionColor, role, token, type Rgba } from "./core.ts";

export function overheadBars(id: string, desc: Api.OverheadBarsDesc = {}): void {
  const w = desc.width || 56;
  const h = desc.height || 6;
  const reference = desc.reference ?? 12;
  const handle = massTemplate(id, `bar:${w}:${h}:${reference}`, () => ({
    kind: "bar",
    space: "world",
    size: { x: w, y: h },
    pivot: { x: 0.5, y: 1 },
    lifetime: 0,
    barRadius: h / 2,
    barBack: token("shadow", 0.85),
    barGhost: token("sheen", 0.75),
    reference,
    occlusion: "fade",
    occludedAlpha: 0.3,
  }));
  const t = now();
  const hold = msToTicks(350);
  const drain = msToTicks(500);
  const out: Api.MassInstance[] = [];
  for (const b of desc.bars ?? []) {
    const v = clamp01(b.value ?? 0);
    const key = `${id}:${b.key ?? ""}`;
    // The ghost of a drop holds, then drains (kept per unit in the UI's memory).
    const last = kit.memory(`${key}~v`, v);
    let g = kit.memory(`${key}~g`, v);
    let at = kit.memory(`${key}~t`, -1e9);
    if (v < last) {
      g = Math.max(g, last);
      at = t;
      kit.remember(`${key}~g`, g);
      kit.remember(`${key}~t`, at);
    }
    if (v !== last) kit.remember(`${key}~v`, v);
    const x = clamp01((t - at - hold) / drain);
    const ghost = Math.max(v, g + (v - g) * easeOut(x));
    if (x >= 1 && g !== v) kit.remember(`${key}~g`, v);
    if (desc.hideFull && v >= 1 && ghost >= 1) continue;
    const c: Rgba = paintRgba(b.color, reactionColor(b.reaction));
    out.push({ position: b.position ?? { x: 0, y: 0, z: 0 }, value: v, ghost, color: lit(c) });
  }
  if (out.length > 0) kit.pushMass(handle, out);
}

const WORDS = ["Miss", "Dodge", "Block", "Parry", "Immune", "Resist", "Evade", "Absorb"];

export function damageNumbers(id: string, desc: Api.DamageNumbersDesc = {}): void {
  const hits = desc.hits ?? [];
  if (hits.length === 0) return;
  const size = desc.size || 22;
  const reference = desc.reference ?? 10;
  const compact = desc.compact !== false;
  const life = msToTicks(950);
  // Pop in past its size, hold, rise and fade (the shared timeline; a crit is the same, larger).
  const keys: Api.MassKey[] = [
    { t: 0, scale: 0.5, alpha: 0, offsetY: 0 },
    { t: 0.07, scale: 1.3, alpha: 1, offsetY: -6, ease: "power1.out" },
    { t: 0.18, scale: 1, alpha: 1, offsetY: -10, ease: "back.out" },
    { t: 0.7, scale: 1, alpha: 1, offsetY: -34, ease: "power1.out" },
    { t: 1, scale: 0.9, alpha: 0, offsetY: -48, ease: "power1.in" },
  ];
  const base = (plus: boolean): Api.MassTemplateDesc => ({
    kind: "number",
    space: "world",
    size: { x: size, y: 0 },
    pivot: { x: 0.5, y: 1 },
    lifetime: life,
    keys,
    drift: 26,
    decimals: 0,
    plus,
    compact,
    reference,
    maxScale: 2,
    minScale: 0.5,
    occlusion: "none",
    outline: token("shadow", 0.9),
    outlineWidth: 2.5,
  });
  const numbers = massTemplate(`${id}.number`, `n:${size}:${reference}:${compact}`, () => base(false));
  const heals = massTemplate(`${id}.heal`, `h:${size}:${reference}:${compact}`, () => base(true));
  const extra = hits.filter((h) => (h.kind ?? "normal") === "miss" && h.word && !WORDS.includes(h.word)).map((h) => h.word ?? "");
  const words = [...WORDS, ...new Set(extra)];
  const said = massTemplate(`${id}.word`, `w:${size}:${reference}:${words.join("|")}`, () => ({
    ...base(false),
    kind: "text",
    strings: words,
    size: { x: Math.round(size * 0.8), y: 0 },
  }));
  const n: Api.MassInstance[] = [];
  const hl: Api.MassInstance[] = [];
  const ws: Api.MassInstance[] = [];
  hits.forEach((h, i) => {
    const kind: Api.DamageKind = h.kind ?? "normal";
    const seed = h.seed ?? i;
    const position = h.position ?? { x: 0, y: 0, z: 0 };
    const own = h.color ? paintRgba(h.color, role("ink")) : null;
    switch (kind) {
      case "heal":
        hl.push({ position, value: h.amount ?? 0, color: lit(own ?? role("success")), seed });
        break;
      case "miss":
        ws.push({ position, value: Math.max(0, words.indexOf(h.word || "Miss")), color: token("inkMuted"), seed, scale: 0.9 });
        break;
      case "critical":
        n.push({ position, value: h.amount ?? 0, color: lit(own ?? role("gold")), seed, scale: 1.55, rotation: (seed % 2 === 0 ? -1 : 1) * 0.06 });
        break;
      case "dot":
        n.push({ position, value: h.amount ?? 0, color: lit(own ?? role("ink"), 0.8), seed, scale: 0.75 });
        break;
      case "shield":
        n.push({ position, value: h.amount ?? 0, color: lit(own ?? role("info")), seed, scale: 0.9 });
        break;
      case "self":
        n.push({ position, value: h.amount ?? 0, color: lit(own ?? role("danger")), seed, scale: 1.1 });
        break;
      default:
        n.push({ position, value: h.amount ?? 0, color: lit(own ?? { r: 1, g: 1, b: 1, a: 1 }), seed });
    }
  });
  if (n.length) kit.spawnMass(numbers, n);
  if (hl.length) kit.spawnMass(heals, hl);
  if (ws.length) kit.spawnMass(said, ws);
}
