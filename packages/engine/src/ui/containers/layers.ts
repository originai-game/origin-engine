// Floating containers (docs/ui.md §11, Containers and navigation): dialogs
// and confirms, popovers, menus and context menus, and sheets. Each sits
// on the modal layer (a focus scope: focus moves in, stays in, and comes
// back when it closes; Back and a press outside dismiss it unless locked)
// and is placed by base.ts's floating(): beside its anchor, flipped and
// kept on screen, or on an edge.

import type * as Api from "@origingame/engine";
import {
  all, beside, chevron, clamp, fit, flags, floating, floatingContent, frame, grow, kit, mark, over, paint, px, role, rtl, spring, still,
  surface, text, token, xy,
  type Placed, type Rgba, type Side,
} from "./base.ts";

// ── dialog and confirm ──────────────────────────────────────────────────

const TONE_INK: Record<string, Api.ThemeColor> = { info: "info", success: "success", warn: "warn", danger: "danger", neutral: "inkMuted", primary: "primary", accent: "highlight" };
const TONE_MARK: Record<string, "plus" | "tick" | "cross" | "dot"> = { info: "dot", success: "tick", warn: "dot", danger: "cross", neutral: "dot", primary: "dot", accent: "dot" };

/**
 * A dialog: a title, a tone mark (info, success, warning, danger: an icon
 * as well as a colour), a message in rich text, a body the game builds and
 * a row of actions. On a desktop the actions sit at the end, the primary
 * last; on a phone they stack full width, the primary first. Back and the
 * backdrop dismiss it unless locked.
 */
export function dialog(id: string, open: boolean, desc: Api.DialogDesc, build: Api.UiBuild): Api.DialogResult {
  const { t, phone, viewport } = frame();
  const actions = desc.actions ?? [];
  let action: string | null = null;
  const width = desc.width ?? px(Math.min(desc.wide ? 640 : 460, viewport.width - 2 * t.gutter));
  const m = kit.modal(id, open, { title: desc.title ?? "", locked: desc.locked ?? false, width, transition: desc.transition ?? null }, () => {
    const tone = desc.tone && desc.tone !== "neutral" ? desc.tone : null;
    kit.row("Message", { width: grow(), gap: t.md, cross: "start" }, () => {
      if (tone) toneMark("Tone", tone, 36);
      kit.column("Text", { width: grow(), gap: t.sm }, () => {
        if (desc.message) kit.richText("Body", desc.message, { step: "reading", color: token("inkMuted"), width: grow() });
        build();
      });
    });
    if (actions.length > 0) {
      const ordered = phone ? [...actions].sort((a, b) => rank(b) - rank(a)) : [...actions].sort((a, b) => rank(a) - rank(b));
      const stack = phone || actions.length > 3;
      kit.open("Actions", { direction: stack ? "column" : "row", width: grow(), gap: t.sm, align: "end", cross: stack ? "stretch" : "center", padding: { left: 0, top: t.sm, right: 0, bottom: 0 } });
      for (const a of ordered) {
        const r = kit.button(a.id ?? "", a.label ?? a.id ?? "", { variant: a.variant ?? "secondary", disabled: a.disabled ?? false, width: stack ? grow() : null, autofocus: a.autofocus ?? false, size: phone ? "large" : "medium" }) as Api.Response;
        if (r.clicked) action = a.id ?? null;
      }
      kit.close();
    }
  }) as Api.ModalResult;
  if (action !== null) kit.feedback(action === (desc.cancelId ?? "cancel") ? "cancel" : "confirm", -1);
  return { action, dismissed: m.dismissed, visible: m.visible };
}

/** Buttons by weight: quiet, secondary, danger, primary (the primary last in a row, first in a stack). */
function rank(a: Api.DialogAction): number {
  return { quiet: 0, secondary: 1, danger: 2, primary: 3 }[a.variant ?? "secondary"] ?? 1;
}

/** A tone's mark: a tinted disc with a shape in it, so the tone is never colour alone (XAG 103). */
export function toneMark(id: string, tone: string, size: number): void {
  const ink = role(TONE_INK[tone] ?? "info");
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", radius: size / 2, background: paint(ink, 0.16), part: `Dialog.tone${flags("round", `tone=${tone}`)}`, partClass: "avatar" });
  if (tone === "warn") {
    // An exclamation: a bar and a dot.
    kit.column("Bang", { gap: size * 0.08, cross: "center" }, () => {
      kit.open("Bar", { width: px(size * 0.1), height: px(size * 0.32), radius: size * 0.05, background: paint(ink) });
      kit.close();
      kit.open("Dot", { width: px(size * 0.12), height: px(size * 0.12), radius: size * 0.06, background: paint(ink) });
      kit.close();
    });
  } else if (tone === "info") {
    kit.column("I", { gap: size * 0.08, cross: "center" }, () => {
      kit.open("Dot", { width: px(size * 0.12), height: px(size * 0.12), radius: size * 0.06, background: paint(ink) });
      kit.close();
      kit.open("Bar", { width: px(size * 0.1), height: px(size * 0.3), radius: size * 0.05, background: paint(ink) });
      kit.close();
    });
  } else {
    mark("Mark", TONE_MARK[tone] ?? "dot", size * 0.5, ink, Math.max(2, size / 14));
  }
  kit.close();
}

/**
 * A confirmation: a dialog with a message, Cancel and the action (in the
 * danger variant for something that cannot be undone). Back and the
 * backdrop cancel; the action button takes the focus only when it is safe.
 */
export function confirm(id: string, open: boolean, desc: Api.ConfirmationDesc): Api.ConfirmResult {
  const danger = desc.danger ?? false;
  const r = dialog(id, open, {
    title: desc.title ?? "",
    message: desc.message ?? "",
    tone: danger ? "danger" : desc.tone ?? "neutral",
    locked: desc.locked ?? false,
    actions: [
      { id: "cancel", label: desc.cancelLabel || "Cancel", variant: "secondary", autofocus: danger },
      { id: "confirm", label: desc.confirmLabel || "Confirm", variant: danger ? "danger" : "primary", autofocus: !danger },
    ],
  }, () => {});
  return { confirmed: r.action === "confirm", cancelled: r.action === "cancel" || r.dismissed };
}

// ── popover ─────────────────────────────────────────────────────────────

/**
 * A popover beside an element (its path, as og.ui.bounds takes it) on the
 * side asked, flipped where there is no room and kept on screen, with an
 * arrow pointing at the element. It rises out of that side on the style's
 * spring. Back or a press outside dismisses it.
 */
export function popover(id: string, open: boolean, desc: Api.PopoverDesc, build: Api.UiBuild): Api.PopoverResult {
  const { t, viewport } = frame();
  const anchor = kit.bounds(desc.anchor ?? "") ?? { x: viewport.width / 2, y: viewport.height / 2, width: 0, height: 0 };
  const arrow = desc.arrow ?? true;
  const gap = (arrow ? 10 : 6) + (desc.offset ?? 0);
  let side: Side = desc.side ?? "below";
  const f = floating(id, open, { width: desc.width ?? fit(0, Math.min(360, viewport.width - 2 * t.gutter)) }, (size) => {
    const p = beside(anchor, size, desc.side ?? "below", gap, desc.align ?? "center");
    side = p.side;
    return p;
  }, (at) => {
    floatingContent({ direction: "column", width: desc.width ?? fit(0, Math.min(360, viewport.width - 2 * t.gutter)), padding: all(desc.padding ?? t.md), gap: t.sm, background: token("panelRaised"), part: `Popover.frame${flags(side)}`, partClass: "tooltip", radius: t.panel }, () => {
      if (desc.title) text("Title", desc.title, { step: "base", heading: true, width: grow() });
      build();
      if (arrow && at.measured) pointer("Arrow", anchor, at, side, role("panelRaised"));
    });
  }, (at) => rise(at, side));
  return { dismissed: f.dismissed, visible: f.visible };
}

/** The opening motion of a floating surface: out of the side it hangs on, a little scaled. */
function rise(at: Placed, side: Side): Api.ElementProps {
  const k = at.openness;
  const d = (1 - k) * 8;
  const flip = kit.frameDirection() === "rtl";
  const dx = side === "start" ? (flip ? -d : d) : side === "end" ? (flip ? d : -d) : 0;
  const dy = side === "below" ? -d : side === "above" ? d : 0;
  const s = 0.96 + 0.04 * k;
  return { transform: { x: dx, y: dy, scaleX: s, scaleY: s }, opacity: Math.min(1, k * 1.6) };
}

/** A popover's arrow: a small square turned 45° on its edge, pointing at the anchor's centre. */
function pointer(id: string, anchor: Api.UiRect, at: Placed, side: Side, fill: Rgba): void {
  const s = 12;
  const cx = clamp(anchor.x + anchor.width / 2 - at.x, 14, Math.max(14, at.width - 14));
  const cy = clamp(anchor.y + anchor.height / 2 - at.y, 14, Math.max(14, at.height - 14));
  const flip = kit.frameDirection() === "rtl";
  // Insets are from the start edge: mirrored under a right-to-left locale.
  const startX = (x: number) => (flip ? at.width - x : x);
  const physicalLeft = (side === "end") !== flip;
  const insets: Api.Edges = side === "below" ? { start: startX(cx) - s / 2, top: -s / 2 }
    : side === "above" ? { start: startX(cx) - s / 2, bottom: -s / 2 }
    : physicalLeft ? { start: flip ? at.width - s / 2 : -s / 2, top: cy - s / 2 } : { start: flip ? -s / 2 : at.width - s / 2, top: cy - s / 2 };
  kit.open(id, { width: px(s), height: px(s), background: paint(fill), part: "Popover.arrow", partClass: "tooltip" });
  kit.close();
  over({ insets, transform: { rotate: Math.PI / 4 }, inert: true, z: -1 });
}

// ── menus ───────────────────────────────────────────────────────────────

const ROW = 40;

/**
 * A menu: rows of commands with an icon or check slot, a label, a
 * shortcut or a submenu chevron; separators and group headers; disabled
 * and danger items; checkable and radio items; submenus (entries whose
 * `parent` names another's id) that open beside their row on hover or
 * activation, or on the forward arrow (Right; Left in a right-to-left
 * locale) and a pad's forward direction from its row. It hangs from an
 * element (`anchor`) or at a point (`at`: a context menu). Keys and pads
 * walk its rows; the back arrow and Back close a submenu (the focus
 * returns to its row), then Back the menu. Answers the entry chosen this
 * tick.
 */
export function menu(id: string, open: boolean, desc: Api.MenuDesc): Api.MenuChoice {
  const { viewport } = frame();
  const entries = desc.entries ?? [];
  const at = desc.at ?? null;
  const anchor = at ? { x: at.x, y: at.y, width: 0, height: 0 } : kit.bounds(desc.anchor ?? "") ?? { x: viewport.width / 2, y: viewport.height / 2, width: 0, height: 0 };
  let chosen: string | null = null;
  let dismissed = false;
  // The submenu open under this menu, by its parent entry's index + 1 (0: none).
  const sub = kit.memory(`${id}~sub`, 0);
  let nextSub = sub;
  const side: Side = at ? "below" : desc.side ?? "below";
  const f = floating(id, open, {}, (size) => beside(anchor, size, side, at ? 2 : 4, at ? "start" : desc.align ?? "start"), () => {
    const r = menuRows(entries, "", desc, (i, why) => {
      if (why === "open") nextSub = i + 1;
      else if (why === "hover" && nextSub !== i + 1) nextSub = i + 1;
    }, sub - 1, () => {});
    if (r.chosen !== null) chosen = r.chosen;
    if (r.hoveredPlain) nextSub = 0;
  }, (place) => rise(place, side));
  if (f.dismissed) dismissed = true;
  // The submenu, beside its row.
  const parent = sub > 0 ? entries[sub - 1] : undefined;
  const subOpen = open && parent !== undefined && nextSub === sub;
  if (parent && (subOpen || kit.memory(`${id}~subVisible`, 0) > 0)) {
    const row = kit.bounds(`${f.path}/${kit.idi("Item", sub - 1)}`) ?? anchor;
    const s = floating(`${id}.sub`, subOpen, {}, (size) => beside(row, size, "end", 2, "start"), () => {
      const r = menuRows(entries, parent.id ?? "", desc, () => {}, -1, () => (nextSub = 0));
      if (r.chosen !== null) chosen = r.chosen;
    }, (place) => rise(place, "end"));
    kit.remember(`${id}~subVisible`, s.visible ? 1 : 0);
    if (s.dismissed) nextSub = 0;
  } else kit.remember(`${id}~subVisible`, 0);
  if (!open || chosen !== null) nextSub = 0;
  kit.remember(`${id}~sub`, nextSub);
  if (chosen !== null) {
    const e = entries.find((x) => x.id === chosen);
    kit.feedback(e?.kind === "check" ? (e.checked ? "toggleOff" : "toggleOn") : "select", -1);
  }
  return { chosen, dismissed: dismissed || chosen !== null };
}

/** A context menu at a point (the pointer's, where a right press or a long press happened): menu with `at`. */
export function contextMenu(id: string, open: boolean, at: Api.Vector2, desc: Api.MenuDesc): Api.MenuChoice {
  return menu(id, open, { ...desc, at });
}

function menuRows(entries: readonly Api.MenuEntry[], parent: string, desc: Api.MenuDesc, onSub: (index: number, why: "open" | "hover") => void, openSub: number, onBack: () => void): { chosen: string | null; hoveredPlain: boolean } {
  const { t, viewport } = frame();
  // The arrow that goes deeper: Right, Left under a right-to-left locale (Response.adjust is physical).
  const forward = rtl() ? -1 : 1;
  let chosen: string | null = null;
  let hoveredPlain = false;
  const anyCheck = entries.some((e) => (e.parent ?? "") === parent && (e.kind === "check" || e.kind === "radio"));
  const anyIcon = entries.some((e) => (e.parent ?? "") === parent && e.icon);
  const minW = desc.minWidth ?? 200;
  floatingContent({ direction: "column", width: fit(Math.min(minW, viewport.width - 2 * t.gutter), Math.min(desc.maxWidth ?? 360, viewport.width - 2 * t.gutter)), padding: xy(t.xs, t.xs), gap: 0, cross: "stretch", background: token("panelRaised"), part: "Menu.frame", partClass: "tooltip" }, () => {
    entries.forEach((e, i) => {
      if ((e.parent ?? "") !== parent) return;
      const kind = e.kind ?? "item";
      if (kind === "separator") {
        kit.open(kit.idi("Sep", i), { width: grow(), padding: xy(0, t.xs) });
        kit.divider("Line", false);
        kit.close();
        return;
      }
      if (kind === "header") {
        kit.row(kit.idi("Head", i), { width: grow(), padding: { left: t.sm, top: t.sm, right: t.sm, bottom: t.xxs } }, () => {
          text("Text", e.label ?? "", { step: "caption", color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
        });
        return;
      }
      const hasSub = entries.some((x) => (x.parent ?? "") === e.id && e.id);
      const checked = (kind === "check" || kind === "radio") && (e.checked ?? false);
      const danger = e.danger ?? false;
      const ink: Api.ThemeColor = e.disabled ? "inkFaint" : danger ? "danger" : "ink";
      const r = surface(kit.idi("Item", i), {
        part: "Menu.item",
        cls: "chip",
        flags: [danger ? "danger" : "", checked ? "checked" : "", hasSub && openSub === i ? "expanded" : ""],
        ...(hasSub ? { expanded: openSub === i } : {}),
        fill: null,
        wash: danger ? role("danger") : undefined,
        washBy: 0.14,
        disabled: e.disabled ?? false,
        // Left and Right are the rows' own: forward opens a submenu, back closes this one.
        adjustable: true,
        width: grow(),
        height: fit(Math.max(ROW, t.controlMd)),
        padding: xy(t.sm, 0),
        gap: t.sm,
        semantics: {
          role: kind === "check" ? "menuitemcheckbox" : kind === "radio" ? "menuitemradio" : "menuitem",
          name: e.label ?? "",
          ...(kind === "check" || kind === "radio" ? { checked } : {}),
          ...(hasSub ? { expanded: openSub === i } : {}),
          ...(e.shortcut ? { description: e.shortcut } : {}),
        },
      }, () => {
        if (anyCheck) {
          kit.open("Check", { width: px(18), height: px(18), align: "center", cross: "center" });
          if (checked) {
            if (kind === "radio") mark("Mark", "dot", 8, role(ink));
            else mark("Mark", "tick", 16, role(ink));
          }
          kit.close();
        }
        if (anyIcon) {
          if (e.icon) kit.icon("Icon", e.icon, { size: 18, tint: token(ink), label: "" });
          else kit.spacer("Icon", px(18));
        }
        text("Label", e.label ?? "", { color: ink, nowrap: true, ellipsis: true, width: grow() });
        if (e.shortcut) text("Shortcut", e.shortcut, { step: "caption", color: e.disabled ? "inkFaint" : "inkMuted", nowrap: true, font: "mono" });
        if (hasSub) chevron("More", "forward", 16, role(ink));
      });
      if (r.hovered) {
        if (hasSub) onSub(i, "hover");
        else hoveredPlain = true;
      }
      if (r.clicked) {
        if (hasSub) onSub(i, "open");
        else chosen = e.id ?? null;
      }
      if (r.adjust === forward && hasSub) onSub(i, "open");
      else if (r.adjust === -forward) onBack();
    });
  });
  kit.semantics({ role: "menu", name: desc.label ?? "" });
  return { chosen, hoveredPlain };
}

// ── sheet ───────────────────────────────────────────────────────────────

/**
 * A sheet: a surface that slides in from an edge (bottom by default; end
 * or start for a side sheet) over a scrim. A bottom sheet rests at one of
 * its detents (fractions of the screen's height); dragging it (anywhere
 * that is not a control or a scroll container) follows the finger with
 * rubber past its ends, and a release settles on the detent its momentum
 * reaches, or dismisses it past the lowest when it can be dismissed. A
 * side sheet slides in from its edge and dismisses by a swipe outwards.
 */
export function sheet(id: string, open: boolean, desc: Api.SheetDesc, build: Api.UiBuild): Api.SheetResult {
  const { t, viewport, phone } = frame();
  const edge = desc.edge ?? "bottom";
  const vertical = edge === "bottom";
  const flip = kit.frameDirection() === "rtl";
  const physicalLeft = edge === "start" ? !flip : edge === "end" ? flip : false;
  const sa = viewport.safeArea;
  const availH = viewport.height - sa.top - (vertical ? 0 : sa.bottom) - (vertical ? t.xl : 2 * t.sm);
  const detents = (desc.detents && desc.detents.length > 0 ? [...desc.detents] : [0.5, 0.92]).map((d) => clamp(d, 0.15, 1)).sort((a, b) => a - b);
  const top = detents[detents.length - 1]!;
  const H = Math.round(availH * top);
  // The modal layer keeps its content within the gutters: a phone's sheet floats a gutter in from each side.
  const inner = viewport.width - sa.left - sa.right - 2 * t.gutter;
  const W = vertical ? Math.min(phone ? inner : 640, inner) : Math.min(desc.size ?? 400, inner - t.xl);
  const dismissible = desc.dismissible ?? true;
  const want = clamp(desc.detent ?? 0, 0, detents.length - 1);
  // The offset (down, or outwards for a side sheet) of each detent from fully up.
  const offsetOf = (k: number) => Math.round(H * (1 - detents[k]! / top));
  const gone = vertical ? H + sa.bottom + 24 : W + 24;
  let detent = want;
  let dismissed = false;
  const key = `${id}/Panel/Content`;
  const ges = open ? kit.gesture(key) : null;
  const held = kit.memory(`${id}~held`, 0);
  if (ges && !ges.dragging && held !== 0) {
    // Released: where its momentum carries it picks the detent, or past the lowest one, away.
    const along = vertical ? ges.offsetY : physicalLeft ? -ges.offsetX : ges.offsetX;
    const v = vertical ? ges.velocityY : physicalLeft ? -ges.velocityX : ges.velocityX;
    const land = (vertical ? offsetOf(want) : 0) + kit.memory(`${id}~last`, along) + v * 0.18;
    if (dismissible && land > (vertical ? offsetOf(0) + H * detents[0]! * 0.45 : W * 0.4)) dismissed = true;
    else if (vertical) {
      let best = 0;
      for (let k = 1; k < detents.length; k++) if (Math.abs(offsetOf(k) - land) < Math.abs(offsetOf(best) - land)) best = k;
      detent = best;
    }
  }
  kit.remember(`${id}~held`, ges?.dragging ? 1 : 0);
  kit.remember(`${id}~last`, ges?.dragging ? (vertical ? ges.offsetY : physicalLeft ? -ges.offsetX : ges.offsetX) : 0);
  if (detent !== want) kit.feedback("select", -1);
  const f = floating(id, open, { scrim: desc.scrim ?? true, locked: !dismissible, width: fit(), spring: "panel", transition: "fade" }, () => {
    // Its own size, never the measured one: bounds are cut to the screen, and a sheet hangs past its edge.
    if (vertical) return { x: (viewport.width - W) / 2, y: viewport.height - H - sa.bottom };
    const x = physicalLeft ? sa.left : viewport.width - sa.right - W;
    return { x, y: sa.top + t.sm };
  }, () => {
    floatingContent({
      direction: "column",
      width: px(W),
      height: px(vertical ? H + sa.bottom : availH),
      padding: { left: t.lg, top: vertical ? t.sm : t.lg, right: t.lg, bottom: t.lg + (vertical ? sa.bottom : 0) },
      gap: t.md,
      cross: "stretch",
      background: token("panelRaised"),
      part: `Sheet.frame${flags(edge)}`,
      partClass: "dialog",
    }, () => {
      if (vertical && (desc.handle ?? true)) {
        kit.row("Handle", { width: grow(), height: px(20), align: "center", cross: "center" }, () => {
          kit.open("Grabber", { width: px(36), height: px(5), radius: 3, background: token("lineStrong"), part: "Sheet.handle", partClass: "bar" });
          kit.close();
        });
      }
      if (desc.title) {
        kit.row("Header", { width: grow(), gap: t.sm, cross: "center" }, () => {
          text("Title", desc.title ?? "", { step: "xl", heading: true, width: grow(), nowrap: true, ellipsis: true });
          if (dismissible && (desc.closeButton ?? !vertical)) {
            const r = surface("Close", { part: "Sheet.close", cls: "button", flags: ["quiet", "round"], fill: null, width: px(44), height: px(44), align: "center", cross: "center", semantics: { role: "button", name: desc.closeLabel || "Close" } }, () => mark("X", "cross", 16, role("inkMuted")));
            if (r.clicked) dismissed = true;
          }
        });
      }
      kit.column("Body", { width: grow(), height: grow(), gap: t.sm, cross: "stretch" }, build);
    });
  }, (at) => {
    // Its rest: the detent's offset (spring), away while it closes, and the drag on top of it.
    const rest = !open || !at.measured ? gone : vertical ? offsetOf(detent) : 0;
    const off = still() ? rest : kit.animate(`${id}~rest`, rest, spring("panel"));
    const outward = vertical ? off : physicalLeft ? -off : off;
    return {
      transform: vertical ? { y: outward } : { x: outward },
      drag: {
        axis: vertical ? "y" : "x",
        returnHome: true,
        rubber: 0.55,
        threshold: 6,
        bounds: vertical ? { x: 0, y: -offsetOf(detent), width: 0, height: gone } : physicalLeft ? { x: -gone, y: 0, width: gone, height: 0 } : { x: 0, y: 0, width: gone, height: 0 },
      },
    };
  });
  if (f.dismissed) dismissed = true;
  if (dismissed) kit.feedback("close", -1);
  return { detent, dismissed, visible: f.visible };
}

