// Navigation (docs/ui.md §11): tabs with bumpers, the carousel with snap,
// breadcrumbs, pagination, the action bar and the radial wheel. Each
// draws its parts as the style dresses them (`Tabs.tab[selected]`,
// `Carousel.dot[current]`, `Radial.slice[hover]`) and asks for UI actions
// (tabPrev, tabNext, confirm, back), never keys (docs/ui.md §8.2).

import type * as Api from "@origingame/engine";
import {
  chevron, clamp, fit, flags, frame, glyphCap, grow, kit, lastGesture, mark, over, pointerFrom, px, rememberPath, role, rtl, spring, still, surface, text, token, xy,
} from "./base.ts";
import { menu } from "./layers.ts";

// ── tabs ────────────────────────────────────────────────────────────────

/**
 * Tabs: a row of tabs over the panel the game builds for the chosen one.
 * The selected tab is marked by an indicator (an underline, or a pill
 * behind it) that slides from the old tab to the new one on the style's
 * ui spring. With `bumpers` the tabPrev and tabNext actions (Q and E, [
 * and ], LB and RB) step through the enabled tabs from anywhere on the
 * layer, their glyphs drawn at the row's ends; the arrows move along the
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
      kit.openScroll("Strip", { width: grow(), axes: "horizontal", padding: xy(0, 0), gap: 0 });
      try {
        kit.open("Tabs", { direction: "row", width: desc.fill ? grow() : null, gap: pill ? t.xs : 0, cross: "stretch", padding: pill ? xy(t.xxs, t.xxs) : xy(0, 0), radius: pill ? t.control + t.xxs : 0, background: pill ? token("control") : null, part: `Tabs.track${flags(style)}`, partClass: "tab" });
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
            width: desc.fill ? grow() : null,
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
        });
        kit.close();
        const row = kit.lastPath();
        kit.semantics({ role: "tablist", name: desc.label ?? "" });
        // The indicator: under (or behind) the chosen tab, sliding between tabs.
        indicator("Indicator", row, before, pill);
      } finally {
        kit.close();
      }
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

/** The selected tab's indicator: a bar under it (or a pill behind it) whose position and width follow the tab on the ui spring. */
function indicator(id: string, row: string, k: number, pill: boolean): void {
  const tab = kit.bounds(`${row}/${kit.idi("Tab", k)}`);
  const base = kit.bounds(row);
  if (!tab || !base) return;
  const x = kit.animate("~x", tab.x - base.x, spring("ui"));
  const w = kit.animate("~w", tab.width, spring("ui"));
  const h = pill ? tab.height : 3;
  kit.open(id, { width: px(Math.max(0, w - (pill ? 0 : 16))), height: px(h), radius: pill ? frame().t.control : 2, background: token(pill ? "panelRaised" : "primary"), part: `Tabs.indicator${flags(pill ? "pill" : "underline")}`, partClass: "tab" });
  kit.close();
  const top = pill ? tab.y - base.y : tab.y - base.y + tab.height - h;
  const startX = rtl() ? base.width - (x + w) : x;
  over({ insets: { start: startX + (pill ? 0 : 8), top }, inert: true, z: pill ? -1 : 2 });
}

/** A bumper hint at a tab row's end: its action's key cap (Q, E); a press steps as the action does. */
function bumper(id: string, action: "tabPrev" | "tabNext", onPress: () => void): void {
  const g = action === "tabPrev" ? { text: "Q", icon: "key.q" } : { text: "E", icon: "key.e" };
  const r = surface(id, { part: "Tabs.bumper", cls: "chip", fill: null, focusable: false, width: fit(44), padding: xy(4, 4), align: "center", cross: "center", height: fit(44), semantics: { role: "button", name: action === "tabPrev" ? "Previous tab" : "Next tab", description: g.text } }, () => {
    glyphCap("Glyph", g, 24);
  });
  if (r.clicked) onPress();
}

// ── carousel ────────────────────────────────────────────────────────────

/**
 * A carousel: pages (banners, featured items, character cards) side by
 * side in a track that a drag or a flick moves and a release snaps to a
 * page, with rubber past the first and last. Arrows and dots step it; the
 * arrows (and a pad's left and right) step it when it has the focus; with
 * `autoplayMs` it moves on by itself until the player touches it (never
 * under reduced motion). Pages next to the current one peek at its sides
 * (`peek`) and are dimmed and scaled down a little.
 */
export function carousel(id: string, desc: Api.CarouselDesc, build: Api.UiItemBuild): Api.CarouselResult {
  const { t, phone } = frame();
  const count = Math.max(0, desc.count ?? 0);
  const peek = desc.peek ?? (phone ? 24 : 0);
  const gap = desc.gap ?? t.md;
  let index = clamp(desc.index ?? 0, 0, Math.max(0, count - 1));
  const before = index;
  const key = `carousel:${id}`;
  const flip = rtl();
  kit.open(id, { direction: "column", width: desc.width ?? grow(), gap: t.sm, cross: "stretch" });
  try {
    const size = kit.memory("~view", 0);
    const pageW = Math.max(1, size - 2 * peek);
    const stride = pageW + gap;
    // A release picks the page its momentum carries it to.
    const ges = lastGesture(`${key}/Track`);
    const held = kit.memory("~held", 0);
    if (ges && !ges.dragging && held !== 0) {
      const dx = (kit.memory("~dx", 0) + ges.velocityX * 0.2) * (flip ? -1 : 1);
      const moved = Math.round(-dx / stride);
      index = clamp(index + clamp(moved, -1, 1) * (Math.abs(dx) > stride * 0.12 || Math.abs(ges.velocityX) > 400 ? 1 : 0) * (moved === 0 ? Math.sign(-dx) : 1), 0, count - 1);
    }
    kit.remember("~held", ges?.dragging ? 1 : 0);
    kit.remember("~dx", ges?.dragging ? ges.offsetX : 0);
    // Autoplay: ticks since the player last touched it.
    const idle = kit.memory("~idle", 0);
    const auto = desc.autoplayMs ?? 0;
    let nextIdle = ges?.held ? 0 : idle + 1000 / 60;
    if (auto > 0 && !still() && count > 1 && nextIdle >= auto) {
      index = (index + 1) % count;
      nextIdle = 0;
    }
    kit.remember("~idle", nextIdle);
    const row = (): void => {
      kit.row("Viewport", { width: grow(), cross: "center", gap: t.xs }, () => {
        const arrows = desc.arrows ?? !phone;
        if (arrows) arrow("Prev", "back", index > 0 || (desc.loop ?? false), () => (index = desc.loop ? (index - 1 + count) % count : Math.max(0, index - 1)));
        kit.open("Clip", { direction: "row", width: grow(), height: desc.height ?? null, padding: xy(peek, 0), interactive: true, focusable: true, part: "Carousel.viewport", partClass: "panel" });
        kit.setDraw({ clip: "shape" });
        // As wide as one page: the pages past it overflow into the clip instead of widening it.
        kit.open("Track", { direction: "row", width: px(pageW), gap, cross: "stretch" });
        for (let k = 0; k < count; k++) {
          const dist = Math.abs(k - index);
          kit.open(kit.idi("Page", k), { direction: "column", width: px(pageW), cross: "stretch", part: `Carousel.page${flags(k === index && "current")}`, partClass: "card" });
          if (dist <= 2) build(k);
          kit.close();
          kit.semantics({ role: "group", name: `${k + 1} / ${count}`, selected: k === index });
          const s = k === index ? 1 : 0.94;
          kit.props({ transform: { scaleX: s, scaleY: s }, opacity: k === index ? 1 : 0.55, motion: spring("ui"), inert: k !== index });
        }
        kit.close();
        // The track rests at the current page and follows a drag on top of that.
        const rest = -index * stride * (flip ? -1 : 1);
        const x = still() ? rest : kit.animate("~x", rest, spring("panel"));
        kit.props({ transform: { x }, drag: { axis: "x", returnHome: true, rubber: 0.55, threshold: 6, spring: { kind: "instant" } } });
        rememberPath(`${key}/Track`, kit.lastPath());
        const r = kit.close() as Api.Response;
        kit.semantics({ role: "region", name: desc.label ?? "", value: `${index + 1} / ${count}` });
        const clip = kit.lastPath();
        const vb = kit.bounds(clip);
        if (vb) viewW = vb.width;
        if (r.focused) {
          // The focused carousel takes the arrows, as a pad's left and right.
          if (kit.action("tabPrev")) index = Math.max(0, index - 1);
          if (kit.action("tabNext")) index = Math.min(count - 1, index + 1);
        }
        if (arrows) arrow("Next", "forward", index < count - 1 || (desc.loop ?? false), () => (index = desc.loop ? (index + 1) % count : Math.min(count - 1, index + 1)));
      });
    };
    let viewW = size;
    row();
    // Remembered in the carousel's own scope, where the next tick reads it.
    kit.remember("~view", viewW);
    if ((desc.dots ?? true) && count > 1) dots("Dots", count, index, (k) => (index = k));
  } finally {
    kit.close();
  }
  if (index !== before) kit.feedback("select", -1);
  return { index, changed: index !== before };
}

function arrow(id: string, way: "back" | "forward", on: boolean, act: () => void): void {
  const r = surface(id, { part: "Carousel.arrow", cls: "button", flags: ["round"], fill: role("panelRaised"), disabled: !on, width: px(44), height: px(44), align: "center", cross: "center", radius: 22, semantics: { role: "button", name: way === "back" ? "Previous" : "Next" } }, () => chevron("Chevron", way, 18, role(on ? "ink" : "inkFaint")));
  if (r.clicked) act();
}

/** Page dots: the current one a longer pill (so it reads without colour); each a 44 px target. */
function dots(id: string, count: number, index: number, go: (k: number) => void): void {
  kit.row(id, { width: grow(), align: "center", cross: "center", gap: 0 }, () => {
    for (let k = 0; k < count; k++) {
      const on = k === index;
      const r = surface(kit.idi("Dot", k), { part: "Carousel.dot", cls: "chip", flags: [on ? "current" : ""], selected: on, fill: null, focusable: false, width: px(on ? 36 : 22), height: px(28), align: "center", cross: "center", semantics: { role: "button", name: `${k + 1}`, selected: on } }, () => {
        const w = kit.animate("~w", on ? 20 : 8, spring("ui"));
        kit.open("Pip", { width: px(w), height: px(8), radius: 4, background: token(on ? "primary" : "lineStrong") });
        kit.close();
      });
      if (r.clicked) go(k);
    }
  });
}

// ── breadcrumbs ─────────────────────────────────────────────────────────

/**
 * Breadcrumbs: the path to where the player is (Inventory › Weapons ›
 * Swords). Every crumb but the last is a link back; the last is the
 * current page. When they do not fit the middle collapses into an ellipsis
 * that opens a menu of the hidden crumbs. Separators turn under a right-to-
 * left locale.
 */
export function breadcrumbs(id: string, desc: Api.BreadcrumbsDesc): Api.BreadcrumbsResult {
  const { t, phone, touch } = frame();
  const items = desc.items ?? [];
  const max = desc.maxShown ?? (phone ? 3 : 5);
  // Every crumb a whole target: 44 px under a finger.
  const tall = touch ? 44 : 36;
  let chosen: number | null = null;
  const collapse = items.length > max;
  const shown = collapse ? [0, -1, ...items.map((_, k) => k).slice(items.length - (max - 2))] : items.map((_, k) => k);
  const openMore = kit.memory(`${id}~more`, 0) > 0;
  let nextMore = openMore;
  kit.row(id, { width: desc.width ?? grow(), gap: t.xxs, cross: "center" }, () => {
    shown.forEach((k, j) => {
      if (j > 0) chevron(kit.idi("Sep", j), "forward", 14, role("inkFaint"), 0, 1.5);
      if (k === -1) {
        const r = surface("More", { part: "Breadcrumbs.more", cls: "chip", fill: null, width: fit(tall), height: fit(tall), padding: xy(t.xs, 0), align: "center", cross: "center", semantics: { role: "button", name: desc.moreLabel || "Show path", expanded: openMore } }, () => mark("Dots", "more", 18, role("inkMuted")));
        if (r.clicked) nextMore = !openMore;
        return;
      }
      const last = k === items.length - 1;
      const it = items[k]!;
      if (last) {
        text(kit.idi("Crumb", k), it.label ?? "", { step: "sm", nowrap: true, ellipsis: true, font: "display" });
        kit.semantics({ role: "link", name: it.label ?? "", selected: true, value: "current page" });
      } else {
        const r = surface(kit.idi("Crumb", k), { part: "Breadcrumbs.crumb", cls: "chip", fill: null, height: fit(tall), padding: xy(t.xs, 0), gap: t.xs, semantics: { role: "link", name: it.label ?? "" } }, () => {
          if (it.icon) kit.icon("Icon", it.icon, { size: 16, tint: token("inkMuted"), label: "" });
          text("Label", it.label ?? "", { step: "sm", color: "inkMuted", nowrap: true, ellipsis: true });
        });
        if (r.clicked) chosen = k;
      }
    });
  });
  kit.semantics({ role: "navigation", name: desc.label || "Breadcrumb" });
  const path = kit.lastPath();
  if (collapse) {
    const hidden = items.map((it, k) => ({ it, k })).slice(1, items.length - (max - 2));
    const r = menu(`${id}.more`, nextMore, { anchor: `${path}/More`, entries: hidden.map(({ it, k }) => ({ id: String(k), label: it.label ?? "", icon: it.icon ?? "" })) });
    if (r.chosen !== null) chosen = Number(r.chosen);
    if (r.dismissed) nextMore = false;
  }
  kit.remember(`${id}~more`, nextMore ? 1 : 0);
  if (chosen !== null) kit.feedback("back", -1);
  return { chosen };
}

// ── pagination ──────────────────────────────────────────────────────────

/**
 * Pagination: previous and next, the first and last pages, the pages
 * around the current one and ellipses between (1 … 4 5 6 … 20); on a
 * phone a compact `4 / 20` between the arrows. The tabPrev and tabNext
 * actions page while `bumpers` is set.
 */
export function pagination(id: string, desc: Api.PaginationDesc): Api.PageResult {
  const { t, phone } = frame();
  const pages = Math.max(1, desc.pages ?? 1);
  let page = clamp(desc.page ?? 0, 0, pages - 1);
  const before = page;
  if (desc.bumpers) {
    if (kit.action("tabPrev")) page = Math.max(0, page - 1);
    if (kit.action("tabNext")) page = Math.min(pages - 1, page + 1);
  }
  const compact = desc.compact ?? phone;
  const around = desc.siblings ?? 1;
  const list: number[] = [];
  if (!compact) {
    for (let k = 0; k < pages; k++) {
      if (k === 0 || k === pages - 1 || Math.abs(k - page) <= around) list.push(k);
      else if (list[list.length - 1] !== -1) list.push(-1);
    }
  }
  kit.row(id, { gap: t.xxs, cross: "center", width: desc.width ?? null, align: "center" }, () => {
    const prev = surface("Prev", { part: "Pagination.step", cls: "button", fill: null, disabled: page === 0, width: px(44), height: px(44), align: "center", cross: "center", semantics: { role: "button", name: desc.prevLabel || "Previous page" } }, () => chevron("Chevron", "back", 16, role(page === 0 ? "inkFaint" : "ink")));
    if (prev.clicked) page = Math.max(0, page - 1);
    if (compact) {
      text("Count", `${page + 1} / ${pages}`, { step: "sm", font: "mono", nowrap: true });
      kit.semantics({ role: "status", name: `${page + 1} / ${pages}` });
    } else {
      list.forEach((k, j) => {
        if (k === -1) {
          text(kit.idi("Gap", j), "…", { step: "sm", color: "inkFaint" });
          return;
        }
        const on = k === page;
        const r = surface(kit.idi("Page", k), { part: "Pagination.page", cls: "button", flags: [on ? "current" : ""], selected: on, fill: null, selectedFill: role("primary"), width: fit(40), height: fit(40), padding: xy(t.xs, 0), align: "center", cross: "center", semantics: { role: "button", name: `${k + 1}`, selected: on } }, () => {
          text("Number", `${k + 1}`, { step: "sm", font: "mono", color: on ? "onPrimary" : "ink", nowrap: true });
        });
        if (r.clicked) page = k;
      });
    }
    const next = surface("Next", { part: "Pagination.step", cls: "button", fill: null, disabled: page === pages - 1, width: px(44), height: px(44), align: "center", cross: "center", semantics: { role: "button", name: desc.nextLabel || "Next page" } }, () => chevron("Chevron", "forward", 16, role(page === pages - 1 ? "inkFaint" : "ink")));
    if (next.clicked) page = Math.min(pages - 1, page + 1);
  });
  kit.semantics({ role: "navigation", name: desc.label || "Pagination", value: `${page + 1} / ${pages}` });
  if (page !== before) kit.feedback("select", -1);
  return { page, changed: page !== before };
}

// ── action bar ──────────────────────────────────────────────────────────

/**
 * The action bar (docs/ui.md §8.2): the actions the top layer accepts,
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
  const where: Api.Anchor = desc.anchor ?? "bottomRight";
  const right = where.endsWith("Right");
  // Within the screen: hints that do not fit on one line (a phone, a large text size) wrap.
  const maxW = Math.max(120, viewport.width - viewport.safeArea.left - viewport.safeArea.right - 2 * t.gutter);
  kit.anchor(id, { anchor: where, direction: "column", width: fit(0, maxW), padding: xy(t.md, t.xs), background: desc.background === false ? null : token("hud"), radius: t.control, frame: "hud" }, () => kit.row("Row", { width: grow(), gap: touch ? t.sm : t.lg, cross: "center", align: right ? "end" : where.endsWith("Left") ? "start" : "center", wrap: true }, () => {
    hints.forEach((h, k) => {
      const glyph = { text: h.text ?? "", icon: h.icon ?? "" };
      const r = surface(kit.idi("Hint", k), {
        part: "ActionBar.hint",
        cls: "chip",
        flags: [h.hold ? "hold" : "", touch ? "touch" : ""],
        fill: touch ? role("control") : null,
        focusable: touch,
        height: fit(touch ? 44 : 36),
        padding: xy(touch ? t.md : t.xs, 0),
        gap: t.sm,
        semantics: { role: "button", name: h.hint ?? h.action ?? "", ...(glyph.text && !touch ? { description: glyph.text } : {}) },
      }, () => {
        if (!touch && glyph.text) {
          glyphCap("Glyph", glyph, 26);
          if (h.hold) {
            const p = clamp(h.progress ?? 0, 0, 1);
            if (p > 0) {
              kit.open("Ring", { width: px(30), height: px(30) });
              kit.setDraw({ shape: { kind: "arc", start: -Math.PI / 2, sweep: Math.PI * 2 * p, thickness: 3, cap: "round" }, fills: [{ kind: "solid", color: token("primary") }] });
              kit.close();
              over({ insets: { start: 0, top: 3 }, inert: true });
            }
          }
        }
        text("Hint", h.hint ?? h.action ?? "", { step: "sm", nowrap: true });
      });
      if (r.clicked) pressed = h.action ?? null;
    });
  }));
  kit.semantics({ role: "toolbar", name: desc.label || "Actions" });
  return { pressed };
}

// ── radial wheel ────────────────────────────────────────────────────────

/**
 * A radial wheel (weapon, emote, ping, build wheels): slices around a
 * centre, chosen by the direction the pointer or the right or left stick
 * points from the centre (past a dead zone), with the choice shown in the
 * middle. A release of the press that opened it (or of the stick) picks
 * the slice under it; a tap on a slice picks it; the arrows and the pad
 * walk the slices when it has the focus. The chosen slice swells on the
 * ui spring and a tick plays as the choice moves.
 */
export function radialMenu(id: string, open: boolean, desc: Api.RadialDesc): Api.RadialResult {
  const { viewport } = frame();
  const items = desc.items ?? [];
  const n = Math.max(1, items.length);
  const radius = desc.radius ?? Math.min(170, Math.min(viewport.width, viewport.height) * 0.36);
  const hole = radius * (desc.hole ?? 0.42);
  let hovered = Math.round(kit.memory(`${id}~hover`, -1));
  let chosen: number | null = null;
  const aim = desc.aim;
  const turn = (Math.PI * 2) / n;
  const pick = (x: number, y: number): number => {
    const dx = (rtl() ? -1 : 1) * x;
    const ang = Math.atan2(y, dx) + Math.PI / 2 + turn / 2;
    return ((Math.floor(ang / turn) % n) + n) % n;
  };
  if (aim && Math.hypot(aim.x, aim.y) > (desc.deadZone ?? 0.35)) hovered = pick(aim.x, aim.y);
  // The pointer picks by angle over the ring (the slices are wedges, not rectangles): when it
  // moves or presses, so a resting mouse leaves the stick its aim.
  const key = `${id}~wheel`;
  const ptr = open ? pointerFrom(key) : null;
  const moved = ptr !== null && (ptr.down || Math.round(ptr.x) !== kit.memory(`${id}~px`, -1) || Math.round(ptr.y) !== kit.memory(`${id}~py`, -1));
  const onRing = ptr !== null && ptr.d >= hole * 0.6 && ptr.d <= radius + 12;
  if (moved && onRing) hovered = pick(ptr.dx, ptr.dy);
  // A pointer press released off the ring (in the hole: the way to cancel) chooses nothing.
  const pointerRelease = kit.memory(`${id}~pd`, 0) > 0 && !(ptr?.down ?? false);
  kit.remember(`${id}~px`, ptr ? Math.round(ptr.x) : -1);
  kit.remember(`${id}~py`, ptr ? Math.round(ptr.y) : -1);
  kit.remember(`${id}~pd`, ptr?.down ? 1 : 0);
  const step = (dir: number): number => {
    for (let s = 1; s <= n; s++) {
      const j = (((hovered < 0 ? (dir > 0 ? -1 : 0) : hovered) + dir * s) % n + n) % n;
      if (!(items[j]?.disabled ?? false)) return j;
    }
    return hovered;
  };
  const was = hovered;
  const f = kit.modal(id, open, { bare: true, backdrop: token("scrim", desc.dim === false ? 0 : 0.55), transition: "scale", width: px(radius * 2) }, () => {
    // The wheel takes the pointer, keys (Left and Right step round it, Enter chooses) and pads.
    kit.open("Wheel", { width: px(radius * 2), height: px(radius * 2), interactive: true, focusable: true, adjustable: true, part: "Radial.frame", partClass: "dialog" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("panel", 0.82) }], effects: [{ kind: "backdrop", blur: 10 }] });
    items.forEach((it, k) => {
      const on = k === hovered;
      const a0 = -Math.PI / 2 - turn / 2 + k * turn;
      kit.open(kit.idi("Slice", k), { width: px(radius * 2), height: px(radius * 2), part: `Radial.slice${flags(on && "hover", it.disabled && "disabled")}`, partClass: "chip" });
      {
        const swell = kit.animate("~swell", on ? 1 : 0, spring("ui"));
        kit.setDraw({
          shape: { kind: "arc", start: rtl() ? Math.PI - a0 - turn + 0.02 : a0 + 0.02, sweep: turn - 0.04, thickness: radius - hole + swell * 6 },
          fills: [{ kind: "solid", color: on ? token("primary", 0.9) : token("control") }],
        });
        const mid = a0 + turn / 2;
        const rr = (radius + hole) / 2;
        const cx = radius + Math.cos(mid) * rr * (rtl() ? -1 : 1);
        const cy = radius + Math.sin(mid) * rr;
        kit.column("Label", { width: px(80), cross: "center", gap: 2 }, () => {
          if (it.icon) kit.icon("Icon", it.icon, { size: 28, tint: token(on ? "onPrimary" : "ink"), label: "" });
          if (it.label && desc.labels !== false) text("Text", it.label, { step: "caption", color: on ? "onPrimary" : "inkMuted", nowrap: true, ellipsis: true, align: "center", width: grow() });
        });
        over({ insets: { start: (rtl() ? radius * 2 - cx : cx) - 40, top: cy - 22 }, inert: true });
      }
      kit.close();
      over({ insets: { start: 0, top: 0 }, inert: true });
      kit.semantics({ role: "menuitem", name: it.label ?? "", selected: on, disabled: it.disabled ?? false });
    });
    // The centre: what is chosen.
    kit.column("Centre", { width: px(hole * 2 - 8), height: px(hole * 2 - 8), align: "center", cross: "center", gap: 2, part: "Radial.centre", partClass: "avatar" }, () => {
      const it = items[hovered];
      text("Name", it?.label ?? desc.title ?? "", { step: "base", heading: true, nowrap: true, ellipsis: true, align: "center", width: grow() });
      if (it?.detail) text("Detail", it.detail, { step: "caption", color: "inkMuted", align: "center", width: grow(), maxLines: 2, ellipsis: true });
    });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("panelRaised") }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: token("line") } }] });
    over({ insets: { start: radius - hole + 4, top: radius - hole + 4 }, inert: true, z: 2 });
    const w = kit.close() as Api.Response;
    rememberPath(key, kit.lastPath());
    kit.semantics({ role: "menu", name: desc.title ?? "", value: items[hovered]?.label ?? "" });
    // Right goes clockwise (counter-clockwise under a right-to-left locale, as the wheel is mirrored).
    if (w.adjust !== 0) hovered = step(w.adjust * (rtl() ? -1 : 1));
    if (w.clicked && hovered >= 0 && !(items[hovered]?.disabled ?? false) && (onRing || !pointerRelease)) chosen = hovered;
  }) as Api.ModalResult;
  if (open && desc.release && hovered >= 0 && !(items[hovered]?.disabled ?? false)) chosen = hovered;
  if (hovered !== was || (hovered !== Math.round(kit.memory(`${id}~heard`, -1)) && hovered >= 0)) {
    if (open && hovered >= 0) kit.feedback("scrollTick", -1);
    kit.remember(`${id}~heard`, hovered);
  } else kit.remember(`${id}~heard`, Math.round(kit.memory(`${id}~heard`, -1)));
  kit.remember(`${id}~hover`, open ? hovered : -1);
  if (chosen !== null) kit.feedback("confirm", -1);
  return { chosen, hovered: hovered >= 0 ? hovered : null, dismissed: f.dismissed };
}
