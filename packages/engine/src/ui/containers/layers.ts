// Floating containers (docs/ui.md §2, Containers and navigation): dialogs
// and confirms, popovers, menus and context menus. Each sits
// on the modal layer (a focus scope: focus moves in, stays in, and comes
// back when it closes; Back and a press outside dismiss it unless locked)
// and is placed by base.ts's floating(): beside its anchor, flipped and
// kept on screen, or on an edge.

import type * as Api from "@origingame/engine";
import {
  all, beside, byText, chevron, clamp, fit, flags, floating, floatingContent, frame, grow, kit, mark, over, paint, px, role, rtl, surface, text, token, xy, type Placed, type Rgba, type Side,
} from "./base.ts";

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
 * locale) and a pad's forward direction from its row; where none fits
 * beside the menu (a phone) a submenu drills in, its rows replacing the
 * menu's under a back row, and Back goes up a level. It hangs from an
 * element (`anchor`) or at a point (`at`: a context menu). Keys and pads
 * walk its rows; the back arrow and Back close a submenu (the focus
 * returns to its row), then Back the menu. Answers the entry chosen this
 * tick.
 */
export function menu(id: string, open: boolean, desc: Api.MenuDesc): Api.MenuChoice {
  const { viewport, phone } = frame();
  const entries = desc.entries ?? [];
  const at = desc.at ?? null;
  const anchor = at ? { x: at.x, y: at.y, width: 0, height: 0 } : kit.bounds(desc.anchor ?? "") ?? { x: viewport.width / 2, y: viewport.height / 2, width: 0, height: 0 };
  let chosen: string | null = null;
  let dismissed = false;
  // The submenu open under this menu, by its parent entry's index + 1 (0: none).
  const sub = kit.memory(`${id}~sub`, 0);
  let nextSub = sub;
  const side: Side = at ? "below" : desc.side ?? "below";
  const parent = sub > 0 ? entries[sub - 1] : undefined;
  // Where no submenu fits beside the menu (a phone; a menu with no room at either side, as
  // measured last tick), a submenu drills in: its rows replace the menu's under a back row, so
  // the parent is never covered.
  const drill = kit.memory(`${id}~drill`, phone ? 1 : 0) > 0;
  const f = floating(id, open, {}, (size) => beside(anchor, size, side, at ? 2 : 4, at ? "start" : desc.align ?? "start"), () => {
    if (drill && parent) {
      const r = menuRows(entries, parent.id ?? "", desc, () => {}, -1, () => (nextSub = 0), parent.label ?? "");
      if (r.chosen !== null) chosen = r.chosen;
      return;
    }
    const r = menuRows(entries, "", desc, (i, why) => {
      if (why === "open") nextSub = i + 1;
      // Hover opens a submenu beside its row; drilling in waits for a press or the forward arrow.
      else if (why === "hover" && !drill && nextSub !== i + 1) nextSub = i + 1;
    }, drill ? -1 : sub - 1, () => {});
    if (r.chosen !== null) chosen = r.chosen;
    if (r.hoveredPlain && !drill) nextSub = 0;
  }, (place) => rise(place, side));
  // Back inside a drilled-in submenu goes up a level; an outside press still closes the menu.
  if (f.dismissed && drill && parent && kit.action("back")) nextSub = 0;
  else if (f.dismissed) dismissed = true;
  const mb = kit.bounds(f.path);
  if (mb && !(drill && parent)) {
    const sa = viewport.safeArea;
    const room = Math.max(viewport.width - sa.right - (mb.x + mb.width), mb.x - sa.left) - 2;
    kit.remember(`${id}~drill`, phone || room < mb.width ? 1 : 0);
  }
  // The submenu, beside its row.
  const subOpen = open && parent !== undefined && nextSub === sub && !drill;
  if (parent && !drill && (subOpen || kit.memory(`${id}~subVisible`, 0) > 0)) {
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

function menuRows(entries: readonly Api.MenuEntry[], parent: string, desc: Api.MenuDesc, onSub: (index: number, why: "open" | "hover") => void, openSub: number, onBack: () => void, backLabel: string | null = null): { chosen: string | null; hoveredPlain: boolean } {
  const { t, viewport } = frame();
  // The arrow that goes deeper: Right, Left under a right-to-left locale (Response.adjust is physical).
  const forward = rtl() ? -1 : 1;
  let chosen: string | null = null;
  let hoveredPlain = false;
  const anyCheck = entries.some((e) => (e.parent ?? "") === parent && (e.kind === "check" || e.kind === "radio"));
  const anyIcon = entries.some((e) => (e.parent ?? "") === parent && e.icon);
  const minW = desc.minWidth || 200;
  floatingContent({ direction: "column", width: fit(Math.min(minW, viewport.width - 2 * t.gutter), Math.min(desc.maxWidth || byText(360), viewport.width - 2 * t.gutter)), padding: xy(t.xs, t.xs), gap: 0, cross: "stretch", background: token("panelRaised"), part: "Menu.frame", partClass: "tooltip" }, () => {
    // A drilled-in submenu: a back row naming where it came from, then its rows.
    if (backLabel !== null) {
      const b = surface("Up", {
        part: "Menu.back",
        cls: "chip",
        fill: null,
        washBy: 0.14,
        adjustable: true,
        width: grow(),
        height: fit(Math.max(ROW, t.controlMd)),
        padding: xy(t.sm, 0),
        gap: t.sm,
        semantics: { role: "menuitem", name: backLabel, description: "Back" },
      }, () => {
        chevron("Back", "back", 16, role("inkMuted"));
        text("Label", backLabel, { color: "inkMuted", heading: true, width: grow() });
      });
      if (b.clicked || b.adjust === -forward) onBack();
      kit.open("Rule", { width: grow(), padding: xy(0, t.xs) });
      kit.divider("Line", false);
      kit.close();
    }
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
          text("Text", e.label ?? "", { step: "caption", color: "inkMuted", width: grow() });
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
        // The label sizes the menu (it widens with its longest label up to the screen, then the
        // label wraps); never cut. A spring pushes the shortcut and the chevron to the end.
        text("Label", e.label ?? "", { color: ink });
        kit.spacer("Push", grow());
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
