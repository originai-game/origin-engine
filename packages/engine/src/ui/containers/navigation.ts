// Navigation (docs/ui.md §2): tabs with bumpers, pagination and the action
// bar (the carousel, breadcrumbs and radial wheel are component documents
// in src/ui/components). Each draws its parts as the style dresses them
// (`Tabs.tab[selected]`) and asks for UI actions
// (tabPrev, tabNext, confirm, back), never keys (docs/ui.md §2.1).

import type * as Api from "@origingame/engine";
import {
  byText, clamp, fit, flags, frame, glyphCap, grow, kit, lastGesture, over, px, overProps, parentOf, rememberPath, role, rtl, spring, still, surface, text, token, xy,
} from "./base.ts";

import { edge } from "./panels.ts";

// ── tabs ────────────────────────────────────────────────────────────────

/**
 * Tabs: a row of tabs over the panel the game builds for the chosen one.
 * The selected tab is marked by an indicator (an underline, or a pill
 * behind it) that slides from the old tab to the new one on the style's
 * ui spring. With `bumpers` the tabPrev and tabNext actions (Q and E, [
 * and ], LB and RB) step through the enabled tabs from anywhere on the
 * layer, their glyphs on the device used last drawn at the row's ends; the arrows move along the
 * row when it has the focus. A tab can carry a count badge, be disabled,
 * and the row scrolls sideways when it does not fit (the chosen tab kept
 * in view).
 */
export function tabBar(id: string, desc: Api.TabBarDesc, build: Api.UiItemBuild): Api.TabBarResult {
  const { t, phone, touch } = frame();
  const tabs = desc.tabs ?? [];
  const n = tabs.length;
  let selected = clamp(desc.selected ?? 0, 0, Math.max(0, n - 1));
  const enabled = (k: number) => k >= 0 && k < n && !(tabs[k]?.disabled ?? false);
  const step = (dir: number): number => {
    for (let k = 1; k <= n; k++) {
      const j = desc.wrap === false ? selected + dir * k : (((selected + dir * k) % n) + n) % n;
      if (j < 0 || j >= n) return selected;
      if (enabled(j)) return j;
    }
    return selected;
  };
  const before = selected;
  if (desc.bumpers ?? true) {
    if (kit.action("tabPrev")) selected = step(-1);
    if (kit.action("tabNext")) selected = step(1);
  }
  const style = desc.variant ?? "underline";
  const pill = style === "pill";
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: desc.gap ?? t.md, cross: "stretch" });
  try {
    kit.row("Bar", { width: grow(), gap: t.xs, cross: "center" }, () => {
      // The bumpers' hints: a key cap or a pad's shoulder; none under a finger, where the row scrolls.
      const hints = (desc.bumpers ?? true) && !touch;
      if (hints) bumper("Prev", "tabPrev", () => (selected = step(-1)));
      // The strip: a clipped window the tabs slide in. Its offset is the tab bar's own (memory, in
      // this row's scope), so it knows where the tabs are cut: it fades at that edge, keeps the
      // selected (or focused) tab clear of the fades, follows a drag with rubber past its ends, and
      // ends a gap short of the bumper caps. Tabs that fill the row share it, each at least as wide
      // as its own label; the widths are last tick's, so the row settles in one tick.
      const fade = 28;
      const gap = pill ? t.xs : 0;
      const pad = pill ? t.xxs : 0;
      // A tab never in view has not been measured (the window cuts its box): it counts as the
      // measured ones' mean until it scrolls in and is measured.
      const known = tabs.map((_, k) => kit.memory(`~nat${k}`, 0));
      const seen = known.filter((w) => w > 0);
      const guess = seen.length > 0 ? seen.reduce((a, b) => a + b, 0) / seen.length : byText(96);
      const nat = known.map((w) => w || guess);
      const natSum = nat.reduce((a, b) => a + b, 0) + 2 * pad + Math.max(0, n - 1) * gap;
      const clipW = kit.memory("~clipW", 0);
      const tallFallback = Math.max(phone ? 44 : 40, t.controlMd) + 2 * pad;
      const tall = kit.memory("~h", 0) || tallFallback;
      const contentW = Math.max(natSum, desc.fill ? clipW : 0);
      // A px past the end: a tab whose width is still being learnt (a cut label measures short) is never cut by a sliver.
      const maxOff = contentW > clipW + 0.5 ? contentW - clipW + 1 : 0;
      // Where a tab lies along the row, in the reading direction.
      const startOf = (k: number) => pad + nat.slice(0, k).reduce((a, b) => a + b, 0) + k * gap;
      let off = clamp(kit.memory("~off", 0), 0, maxOff);
      const ges = lastGesture(`tabs:${id}/Tabs`);
      const along = (dx: number) => (rtl() ? dx : -dx);
      if (ges && !ges.dragging && kit.memory("~held", 0) !== 0) off = clamp(off + along(kit.memory("~dx", 0) + ges.velocityX * 0.15), 0, maxOff);
      kit.remember("~held", ges?.dragging ? 1 : 0);
      kit.remember("~dx", ges?.dragging ? ges.offsetX : 0);
      const reveal = (k: number) => {
        if (clipW <= 0 || k < 0 || k >= n) return;
        const a = startOf(k);
        const b = a + (nat[k] ?? 0);
        const room = Math.min(fade, Math.max(0, (clipW - (b - a)) / 2));
        if (a - off < room) off = clamp(a - room, 0, maxOff);
        else if (b - off > clipW - room) off = clamp(b - clipW + room + 1, 0, maxOff);
      };
      // The selected tab comes into view when it changes (a bumper, a tap), and on the first build.
      // Again while its width is being learnt (a tab scrolled in for the first time).
      if (selected !== Math.round(kit.memory("~shown", -1)) || known[selected] !== kit.memory("~shownW", -1)) reveal(selected);
      const live = ges?.dragging ? clamp(off + along(ges.offsetX), -fade, maxOff + fade) : off;
      const shownOff = still() || ges?.dragging ? live : kit.animate("~offAnim", live, spring("ui"));
      // The pill track is the strip itself, so the sliding pill sits on it, behind the tabs.
      kit.open("Clip", { direction: "row", width: grow(), height: px(tall), radius: pill ? t.control + t.xxs : 0, background: pill ? token("control") : null, part: `Tabs.track${flags(style)}`, partClass: "tab" });
      // A scroll container is the window: it cuts the tabs' hits and the probe's boxes as well as
      // their drawing. It never scrolls by itself (the tabs are placed over it, so it has no extent):
      // the offset is the tab bar's.
      kit.openScroll("Strip", { width: grow(), height: grow(), axes: "horizontal", padding: xy(0, 0), gap: 0 });
      let focusedTab = -1;
      const measured: number[] = [];
      let row = "";
      try {
        kit.open("Tabs", { direction: "row", width: desc.fill ? px(contentW) : null, gap, cross: "stretch", padding: xy(pad, pad) });
        tabs.forEach((tab, k) => {
          const on = k === selected;
          const r = surface(kit.idi("Tab", k), {
            part: "Tabs.tab",
            cls: "tab",
            flags: [on ? "selected" : "", style],
            selected: on,
            fill: null,
            washBy: 0.08,
            disabled: tab.disabled ?? false,
            width: desc.fill ? grow(nat[k] ?? 0) : null,
            height: fit(Math.max(phone ? 44 : 40, t.controlMd)),
            padding: xy(t.md, 0),
            gap: t.xs,
            align: "center",
            radius: pill ? t.control : t.small,
            semantics: { role: "tab", selected: on, name: tab.badge ? `${tab.label ?? ""}, ${tab.badge}` : tab.label ?? "" },
          }, () => {
            if (tab.icon) kit.icon("Icon", tab.icon, { size: 18, tint: token(on ? "ink" : "inkMuted"), label: "" });
            text("Label", tab.label ?? "", { step: "sm", color: tab.disabled ? "inkFaint" : on ? "ink" : "inkMuted", nowrap: true, font: on ? "display" : "body" });
            if (tab.badge) kit.badge("Badge", tab.badge, tab.badgeTone ?? "primary");
          });
          if (r.clicked && k !== selected) selected = k;
          if (r.focused) focusedTab = k;
          // Its own width: its parts' laid out, never what a share squeezed it to. A part the clip
          // cuts measures short, so a narrower measure is kept only for a tab wholly in view.
          const tp = kit.lastPath();
          const parts = ["Icon", "Label", "Badge"].map((c) => kit.bounds(`${tp}/${c}`)?.width ?? -1).filter((w) => w >= 0);
          const w = parts.length > 0 ? Math.ceil(parts.reduce((a, b) => a + b, 0) + (parts.length - 1) * t.xs + 2 * t.md) : 0;
          const inView = startOf(k) >= off - 0.5 && startOf(k) + (nat[k] ?? 0) <= off + clipW + 0.5;
          measured.push(w > (known[k] ?? 0) || (inView && w > 0) ? w : known[k] ?? 0);
        });
        kit.close();
        row = kit.lastPath();
        kit.semantics({ role: "tablist", name: desc.label ?? "" });
        kit.props({
          ...overProps({ insets: { start: 0, top: 0 } }),
          transform: { x: rtl() ? shownOff : -shownOff },
          drag: { axis: "x", returnHome: true, rubber: 0.55, threshold: 6, spring: { kind: "instant" } },
        });
        rememberPath(`tabs:${id}/Tabs`, row);
        // The indicator: under (or behind) the chosen tab, sliding between tabs. Where the tabs
        // share a row that fits, the tab's laid-out box; otherwise each tab is its own width.
        const tabB = kit.bounds(`${row}/${kit.idi("Tab", before)}`);
        const clipB = kit.bounds(parentOf(row));
        const shared = desc.fill && maxOff === 0 && tabB !== null && clipB !== null;
        const ix = shared ? (rtl() ? clipB!.x + clipB!.width - (tabB!.x + tabB!.width) : tabB!.x - clipB!.x) : startOf(before) - shownOff;
        const iw = shared ? tabB!.width : nat[before] ?? 0;
        if (iw > 0) indicator("Indicator", ix, iw, pad, tall - 2 * pad, pill);
        kit.close();
        // The fades, over the tabs at the edge where more lie.
        edge("FadeStart", true, kit.animate("~fs", off > 0.5 ? 1 : 0, spring("ui")), true, fade, "panel");
        edge("FadeEnd", false, kit.animate("~fe", off < maxOff - 0.5 ? 1 : 0, spring("ui")), true, fade, "panel");
      } finally {
        kit.close();
      }
      const cb = kit.bounds(kit.lastPath());
      // A focused tab (keys, a pad) is kept in view as the focus moves.
      if (focusedTab >= 0 && focusedTab !== Math.round(kit.memory("~focused", -1))) reveal(focusedTab);
      kit.remember("~focused", focusedTab);
      measured.forEach((w, k) => kit.remember(`~nat${k}`, w));
      if (cb) kit.remember("~clipW", cb.width);
      const tb = kit.bounds(row);
      if (tb) kit.remember("~h", Math.max(tb.height, tallFallback));
      kit.remember("~off", off);
      kit.remember("~shown", selected);
      kit.remember("~shownW", known[selected] ?? 0);
      if (hints) bumper("Next", "tabNext", () => (selected = step(1)));
    });
    if (style === "underline") kit.divider("Rule", false);
    kit.column("Panel", { width: grow(), height: desc.height ?? null, cross: "stretch", gap: t.sm }, () => build(selected));
    kit.semantics({ role: "tabpanel", name: tabs[selected]?.label ?? "" });
    if (selected !== before) kit.feedback("tab", -1);
  } finally {
    kit.close();
  }
  return { selected, changed: selected !== before };
}

/** The selected tab's indicator: a bar under it (or a pill behind it) at `x` (from the strip's start, in the reading direction), `w` wide, whose position and width follow the tab on the ui spring. */
function indicator(id: string, x0: number, w0: number, top0: number, tabH: number, pill: boolean): void {
  const x = kit.animate("~x", x0, spring("ui"));
  const w = kit.animate("~w", w0, spring("ui"));
  const h = pill ? tabH : 3;
  kit.open(id, { width: px(Math.max(0, w - (pill ? 0 : 16))), height: px(h), radius: pill ? frame().t.control : 2, background: token(pill ? "panelRaised" : "primary"), part: `Tabs.indicator${flags(pill ? "pill" : "underline")}`, partClass: "tab" });
  kit.close();
  over({ insets: { start: x + (pill ? 0 : 8), top: pill ? top0 : top0 + tabH - h }, inert: true, z: pill ? -1 : 2 });
}

/**
 * A bumper hint at a tab row's end: its action's glyph on the device used
 * last (Q and E on a keyboard, LB and RB on an Xbox pad, L1 and R1 on a
 * PlayStation's, L and R on a Switch's), following the player's rebinding;
 * a press steps as the action does.
 */
function bumper(id: string, action: "tabPrev" | "tabNext", onPress: () => void): void {
  const g = deviceGlyph(action) ?? (action === "tabPrev" ? { text: "Q", icon: "key.q" } : { text: "E", icon: "key.e" });
  const r = surface(id, { part: "Tabs.bumper", cls: "chip", fill: null, focusable: false, width: fit(44), padding: xy(4, 4), align: "center", cross: "center", height: fit(44), semantics: { role: frame().touch ? "button" : "image", name: action === "tabPrev" ? "Previous tab" : "Next tab", description: g.text } }, () => {
    glyphCap("Glyph", g, 24);
  });
  if (r.clicked) onPress();
}

/** An action's glyph on the device used last (og.ui.frameGlyph); null when it has none there. */
function deviceGlyph(action: string): { text: string; icon: string } | null {
  const g = kit.frameGlyph(action);
  return g && g.text ? { text: g.text, icon: g.icon } : null;
}

// ── action bar ──────────────────────────────────────────────────────────

/**
 * The action bar (docs/ui.md §2.1): the actions the top layer accepts,
 * each its glyph on the device used last (a key cap, a pad's face button,
 * a bumper's pill) and its hint, at a safe-area edge. The hints come from
 * og.ui.actionBar (the top screen document's `actions`), or `hints` given
 * here. On touch, where there are no glyphs, each hint is a button. A
 * press on a hint fires its action as the control would; a hold shows its
 * progress ring filling.
 */
export function actionHints(id: string, desc: Api.ActionHintsDesc): Api.ActionHintsResult {
  const { t, touch, viewport } = frame();
  const hints = desc.hints ?? [];
  let pressed: string | null = null;
  const sa = viewport.safeArea;
  // Under a finger a bar of buttons across the bottom, reaching into the safe area under it (its
  // buttons above the home indicator); elsewhere hints at an edge.
  const docked = touch && (desc.anchor === undefined || desc.anchor.startsWith("bottom"));
  const where: Api.Anchor = docked ? "bottom" : desc.anchor ?? "bottomRight";
  const right = where.endsWith("Right");
  // Within the screen: hints that do not fit on one line (a phone, a large text size) wrap.
  const maxW = Math.max(120, viewport.width - sa.left - sa.right - 2 * t.gutter);
  const cap = 26;
  // Room around a cap for a hold's ring (see glyphCap), at the text size.
  const ringRoom = byText(Math.max(2, Math.round(cap * 0.1)) * 2);
  kit.anchor(id, {
    anchor: where,
    direction: "column",
    ...(docked ? { margin: 0, offsetY: sa.bottom, width: px(viewport.width) } : { width: fit(0, maxW) }),
    padding: docked ? { left: sa.left + t.gutter, top: t.sm, right: sa.right + t.gutter, bottom: sa.bottom + t.sm } : xy(t.md, t.xs),
    background: desc.background === false ? null : token("hud"),
    radius: docked ? 0 : t.control,
    frame: "hud",
  }, () => kit.row("Row", { width: grow(), gap: touch ? t.sm : t.lg, cross: "center", align: docked ? "center" : right ? "end" : where.endsWith("Left") ? "start" : "center", wrap: true }, () => {
    hints.forEach((h, k) => {
      // The device's glyph for a UI action it knows, live; the hint's own text and icon otherwise.
      const glyph = (h.action ? deviceGlyph(h.action) : null) ?? { text: h.text ?? "", icon: h.icon ?? "" };
      const r = surface(kit.idi("Hint", k), {
        part: "ActionBar.hint",
        cls: "chip",
        flags: [h.hold ? "hold" : "", touch ? "touch" : ""],
        fill: touch ? role("control") : null,
        washBy: touch ? 0.14 : 0.1,
        pressScale: 0.96,
        // Keys and pads reach it too (a hint is a button wherever it shows).
        focusable: true,
        // As tall as its cap and the ring a hold draws round it: the cap never spills out.
        height: fit(touch ? 44 : 36),
        padding: xy(touch ? t.md : t.xs, touch ? t.xxs : ringRoom / 2),
        gap: t.sm,
        semantics: { role: "button", name: h.hint ?? h.action ?? "", ...(glyph.text && !touch ? { description: glyph.text } : {}) },
      }, () => {
        if (!touch && glyph.text) glyphCap("Glyph", glyph, cap, h.hold ? clamp(h.progress ?? 0, 0, 1) : 0);
        // The body's 16 px at 1080p, never smaller: a hint is read in play.
        text("Hint", h.hint ?? h.action ?? "", { step: "base", nowrap: !touch });
      });
      if (r.clicked) pressed = h.action ?? null;
    });
  }));
  kit.semantics({ role: "toolbar", name: desc.label || "Actions" });
  // The room it took from its edge last tick, safe area included: a screen keeps its content clear of it.
  const b = kit.bounds(id);
  const height = b ? Math.max(0, where.startsWith("bottom") ? viewport.height - b.y : where.startsWith("top") ? b.y + b.height : 0) : 0;
  return { pressed, height };
}
