// og.ui.tooltipCard: rich tooltips (docs/ui.md §5.6, §11, Appendix D
// "Compare tooltips with stat deltas", "Nested, pinnable tooltips with
// keyword links"). Read against Diablo IV's item cards (a title in the
// rarity's colour, stat rows, the equipped item beside it with up and
// down marks), Crusader Kings III's tooltips (a ring fills, the card
// locks, its keywords open cards of their own, as deep as they go) and
// Baldur's Gate 3's keyword links. Each card is an anchored group in the
// tooltip layer, placed beside its trigger's last rectangle, flipped and
// clamped inside the safe area, mirrored right to left.

import type * as Api from "@origingame/engine";
import { a11y, arc, bp, box, clamp, clamp01, close, drop, frame, groups, icon, kit, leaf, lit, msToTicks, open, px, rarityColor, rarityPips, role, sel, shapeOf, solid, sound, stroke, text, token, vgrad, xy, type Rgba, ts, apply } from "./core.ts";

interface Placed {
  x: number;
  y: number;
}

/** Where a card of `w` × `h` sits beside `r`, preferring `side`, inside the safe area (screen coordinates). */
export function place(r: Api.UiRect, w: number, h: number, side: Api.Anchor, gap: number): Placed {
  const f = frame();
  const vp = f.viewport;
  const s = vp.safeArea;
  const m = f.t.sm;
  const lo = { x: s.left + m, y: s.top + m };
  const hi = { x: vp.width - s.right - m, y: vp.height - s.bottom - m };
  // Right to left, the end side is the left.
  const rtl = kit.frameDirection() === "rtl";
  const want = rtl && (side === "right" || side === "left") ? (side === "right" ? "left" : "right") : side;
  const fitsRight = r.x + r.width + gap + w <= hi.x;
  const fitsLeft = r.x - gap - w >= lo.x;
  const fitsBelow = r.y + r.height + gap + h <= hi.y;
  const fitsAbove = r.y - gap - h >= lo.y;
  let x: number;
  let y: number;
  if (want === "top" || want === "bottom") {
    const below = want === "bottom" ? fitsBelow || !fitsAbove : !fitsAbove && fitsBelow;
    y = below ? r.y + r.height + gap : r.y - gap - h;
    x = r.x + r.width / 2 - w / 2;
  } else {
    const right = want === "right" ? fitsRight || !fitsLeft : !fitsLeft && fitsRight;
    x = right ? r.x + r.width + gap : r.x - gap - w;
    y = r.y;
  }
  return { x: clamp(x, lo.x, Math.max(lo.x, hi.x - w)), y: clamp(y, lo.y, Math.max(lo.y, hi.y - h)) };
}

/** The anchor offsets that put a group's top left at screen (x, y): og-ui mirrors a right-to-left frame after layout. */
export function anchorAt(x: number, y: number, w: number): Api.AnchorDesc {
  const f = frame();
  const s = f.viewport.safeArea;
  const lx = kit.frameDirection() === "rtl" ? f.viewport.width - x - w : x;
  return { anchor: "topLeft", margin: 0, offsetX: lx - s.left, offsetY: y - s.top };
}

interface Result {
  visible: boolean;
  pinned: boolean;
  link: string | null;
  depth: number;
}

export function tooltipCard(id: string, desc: Api.TooltipCardDesc = {}, build?: Api.UiBuild): Api.TooltipCardResult {
  const parent = kit.openPath();
  const t = desc.target ?? "";
  const target = !t ? "" : t.includes("/") || !parent ? t : `${parent}/${t}`;
  const out: Result = { visible: false, pinned: false, link: null, depth: 0 };
  card(id, desc, target, desc.open ?? false, 0, out, build);
  return out;
}

/**
 * One card: its rest, pin and hold are numbers in the UI's memory under
 * the open element (so a snapshot carries an open tooltip). Answers
 * whether the pointer is on it or on a card it opened (what keeps a
 * pinned card and its parent up).
 */
function card(id: string, desc: Api.TooltipCardDesc, target: string, open_: boolean, depth: number, out: Result, build?: Api.UiBuild): boolean {
  const f = frame();
  const b = bp();
  const tk = f.t;
  const delay = msToTicks(desc.delayMs || (f.theme.durationsMs.slow ?? 400));
  const pinTicks = msToTicks(desc.pinMs || 900);
  const pinnable = depth > 0 || desc.pinnable !== false;
  const rest = open_ ? kit.memory(`${id}~rest`, 0) + 1 : 0;
  kit.remember(`${id}~rest`, rest);
  let pinned = kit.memory(`${id}~pin`, 0) === 1;
  const held = kit.memory(`${id}~held`, 0) === 1; // the pointer was on it or its children last tick
  const shownFor = kit.memory(`${id}~shown`, 0);
  // Leaving a pinned card closes it after a short grace (the pointer
  // crossing a gap between cards).
  const away = pinned && !open_ && !held ? kit.memory(`${id}~away`, 0) + 1 : 0;
  kit.remember(`${id}~away`, away);
  if (pinned && away > 18) pinned = false;
  if (!pinned && pinnable && open_ && rest >= delay + pinTicks) pinned = true;
  if (!pinned && pinnable && open_ && rest >= delay && kit.action("pin")) pinned = true;
  const visible = (open_ && rest >= delay) || pinned || (held && shownFor > 0);
  if (pinned && kit.memory(`${id}~pin`, 0) === 0) sound("open", true);
  kit.remember(`${id}~pin`, pinned ? 1 : 0);
  kit.remember(`${id}~shown`, visible ? shownFor + 1 : 0);
  if (depth === 0) {
    out.visible = visible;
    out.pinned = pinned;
  }
  if (!visible) {
    kit.remember(`${id}~held`, 0);
    return false;
  }
  out.depth = Math.max(out.depth, depth);

  const rare = rarityColor(desc.rarity);
  const width = desc.width || (b.phone ? 300 : 320);
  const compare = !!desc.compareTitle && (desc.compareStats?.length ?? 0) > 0;
  const total = compare ? width * 2 + tk.sm : width;
  const trig = target ? kit.bounds(target) : null;
  const own = kit.bounds(id);
  const h = own?.height ?? 160;
  const at = trig ? place(trig, total, h, desc.placement ?? "right", tk.sm) : { x: f.viewport.width / 2 - total / 2, y: f.viewport.height / 3 };
  const first = shownFor === 0;
  const pinProgress = pinnable && !pinned ? clamp01((rest - delay) / pinTicks) : pinned ? 1 : 0;

  let hoveredLink: string | null = null;
  let bodyPath = "";
  let overCard = false;
  kit.anchor(id, { ...anchorAt(at.x, at.y, total), direction: "row", gap: tk.sm, cross: "start", over: true }, () => {
    if (compare) {
      overCard = sheet("Compare", desc, true, width, rare, 0, () => {}) || overCard;
    }
    overCard = sheet("Card", desc, false, width, rare, pinProgress, (path) => {
      bodyPath = `${path}/Body`;
      if (desc.body) {
        const rt = kit.richText("Body", desc.body, { step: ts(14), width: { kind: "grow" }, color: token("ink") }) as Api.RichTextResult;
        hoveredLink = rt.hovered ?? null;
        if (rt.link) out.link = rt.link;
      }
      build?.();
    }, pinned) || overCard;
  });
  apply({
    opacity: first ? 0 : 1,
    motion: { kind: "token", token: "ui" },
    enter: { kind: "fade", transition: { kind: "token", token: "enter" } },
    exit: { kind: "fade", transition: { kind: "token", token: "exit" } },
    a11y: a11y("tooltip", desc.title ?? "", { description: (desc.body ?? "").replace(/\[[^\]]*\]/g, "") }),
    ...groups({ pin: pinned ? "pinned" : pinProgress > 0 ? "pinning" : "resting" }, { pin: ["resting", "pinning", "pinned"] }),
  });

  // Nested cards: a hovered keyword opens its card beside this one.
  let childHeld = false;
  for (const l of desc.links ?? []) {
    const key = l.key ?? "";
    if (!key) continue;
    const childId = `${id}~${key}`;
    const childOpen = hoveredLink === key;
    const wasUp = kit.memory(`${childId}~shown`, 0) > 0 || kit.memory(`${childId}~pin`, 0) === 1;
    if (!childOpen && !wasUp) continue;
    const sub: Api.TooltipCardDesc = { title: l.title ?? key, body: l.body ?? "", links: desc.links ?? [], width, pinnable: true, placement: "right", delayMs: 120 };
    const r: Result = { visible: false, pinned: false, link: null, depth: 0 };
    childHeld = card(childId, sub, bodyPath ? `${bodyPath}/${key}` : id, childOpen, depth + 1, r) || childHeld;
    if (r.link) out.link = r.link;
    out.depth = Math.max(out.depth, r.depth, depth + 1);
  }
  const nowHeld = overCard || childHeld;
  kit.remember(`${id}~held`, nowHeld ? 1 : 0);
  return nowHeld || pinned;
}

/** One sheet of a tooltip: the main card or the equipped item beside it. Answers whether the pointer is on it. */
function sheet(id: string, d: Api.TooltipCardDesc, equipped: boolean, width: number, rare: Rgba | null, pin: number, body: (path: string) => void, pinned = false): boolean {
  const f = frame();
  const tk = f.t;
  const sh = shapeOf("tooltip");
  const title = equipped ? (d.compareTitle ?? "") : (d.title ?? "");
  const stats = equipped ? (d.compareStats ?? []) : (d.stats ?? []);
  const titleInk = equipped ? token("ink") : rare ? lit(rare) : token("ink");
  const path = open(id, {
    direction: "column",
    width: px(width),
    padding: xy(tk.md + 2, tk.md),
    gap: tk.sm,
    interactive: true,
    part: sel("TooltipCard", equipped ? "compare" : "card", d.rarity && d.rarity !== "none" ? d.rarity : null, pinned ? "pinned" : null),
    partClass: "tooltip",
  }, {
    shape: { kind: "rect", radii: [sh.radius, sh.radius, sh.radius, sh.radius], corner: sh.corner },
    fills: [vgrad([[0, token("panelRaised", 0.98)], [1, token("panel", 0.98)]])],
    strokes: [stroke(rare && !equipped ? 1.5 : 1, rare && !equipped ? lit(rare, 0.7) : token("lineStrong", 0.8), "inside")],
    effects: [drop(12, 32, token("shadow", 0.55)), drop(2, 6, token("shadow", 0.35))],
  });
  if (equipped) text("Eyebrow", title.toUpperCase(), { step: ts(13), color: token("inkMuted"), nowrap: true });
  // Header: the art slot or icon, the title in its rarity's colour with pips, the subtitle.
  kit.row("Header", { gap: tk.sm, cross: "center", width: { kind: "grow" } }, () => {
    if (!equipped && (d.image || d.icon)) {
      box("Well", { width: px(44), height: px(44), align: "center", cross: "center", part: sel("TooltipCard", "well"), partClass: "slot" }, {
        shape: { kind: "rect", radii: [6, 6, 6, 6] },
        fills: [solid(rare ? lit(rare, 0.18) : token("well"))],
        strokes: [stroke(1, rare ? lit(rare, 0.6) : token("line"))],
        clip: "shape",
      }, () => {
        if (d.image) kit.image("Art", { texture: d.image, width: px(44), height: px(44), label: "" });
        else icon("Icon", d.icon ?? "", 28, rare ? lit(rare) : token("ink"));
      });
    }
    kit.column("Names", { gap: 2, width: { kind: "grow" } }, () => {
      if (!equipped) text("Title", title, { step: ts(18), font: "display", color: titleInk, width: { kind: "grow" }, maxLines: 2, ellipsis: true });
      if (d.subtitle) {
        kit.row("Sub", { gap: tk.xs, cross: "center" }, () => {
          const pips = rarityPips(d.rarity);
          for (let i = 0; i < pips; i++) leaf(kit.idi("Pip", i), { width: px(5), height: px(5) }, { shape: { kind: "polygon", sides: 4 }, fills: [solid(rare ? lit(rare) : token("inkMuted"))] });
          text("Subtitle", d.subtitle ?? "", { step: ts(13), color: token("inkMuted"), nowrap: true, ellipsis: true });
        });
      }
    });
    if (!equipped && pin > 0) {
      // The pin ring fills at the corner; pinned it is a full ring with a dot.
      box("Pin", { width: px(16), height: px(16), align: "center", cross: "center", part: sel("TooltipCard", "pin", pinned ? "pinned" : null), partClass: "badge" }, {
        shape: arc(pin, 2.5),
        fills: [solid(token("highlight"))],
      }, () => {
        if (pinned) leaf("Dot", { width: px(6), height: px(6) }, { shape: { kind: "ellipse" }, fills: [solid(token("highlight"))] });
      });
    }
  });
  leaf("Rule", { width: { kind: "grow" }, height: px(1) }, { fills: [solid(rare && !equipped ? lit(rare, 0.45) : token("line"))] });
  if (stats.length > 0) {
    kit.column("Stats", { gap: 3, width: { kind: "grow" } }, () => {
      stats.forEach((s, i) => statRow(kit.idi("Stat", i), s, !equipped));
    });
  }
  if (!equipped) body(path);
  if (!equipped && d.footer) text("Footer", d.footer, { step: ts(13), color: token("inkMuted"), width: { kind: "grow" } });
  if (!equipped && (d.hints?.length ?? 0) > 0) {
    kit.row("Hints", { gap: tk.sm, wrap: true, width: { kind: "grow" } }, () => {
      d.hints!.forEach((h, i) => {
        box(kit.idi("Hint", i), { padding: xy(tk.xs + 2, 2), part: sel("TooltipCard", "hint"), partClass: "chip" }, {
          shape: { kind: "rect", radii: [4, 4, 4, 4] },
          fills: [solid(token("control"))],
        }, () => text("T", h, { step: ts(13), color: token("inkMuted"), nowrap: true }));
      });
    });
  }
  if (!equipped && pinned) text("Pinned", d.pinLabel ?? "Pinned", { step: ts(13), color: token("highlight"), nowrap: true });
  return close().hovered;
}

function statRow(id: string, s: Api.TooltipStat, deltas: boolean): void {
  const tk = frame().t;
  kit.row(id, { gap: tk.sm, cross: "center", width: { kind: "grow" } }, () => {
    text("Label", s.label ?? "", { step: ts(14), color: token("inkMuted"), width: { kind: "grow" }, nowrap: true, ellipsis: true });
    text("Value", s.value ?? "", { step: ts(14), font: "mono", nowrap: true });
    const d = s.delta ?? 0;
    if (deltas && d !== 0) {
      const good = s.better ? d < 0 : d > 0;
      const ink = role(good ? "success" : "danger");
      box("Delta", { direction: "row", gap: 2, cross: "center", part: sel("TooltipCard", "delta", d > 0 ? "up" : "down") }, undefined, () => {
        // An up or down mark, so better and worse are not told by colour alone.
        leaf("Mark", { width: px(8), height: px(7) }, {
          shape: { kind: "polygon", sides: 3, rotation: d > 0 ? 0 : Math.PI },
          fills: [solid(lit(ink))],
        });
        text("Text", s.deltaText || String(Math.abs(d)), { step: ts(13), font: "mono", nowrap: true, color: lit(ink) });
      });
    }
  });
}

