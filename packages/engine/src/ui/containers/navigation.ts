// Navigation (docs/ui.md §11): tabs with bumpers, the carousel with snap,
// breadcrumbs, pagination, the action bar and the radial wheel. Each
// draws its parts as the style dresses them (`Tabs.tab[selected]`,
// `Carousel.dot[current]`, `Radial.slice[hover]`) and asks for UI actions
// (tabPrev, tabNext, confirm, back), never keys (docs/ui.md §8.2).

import type * as Api from "@origingame/engine";
import {
  all, byText, chevron, clamp, fit, flags, frame, glyphCap, grow, kit, lastGesture, mark, over, pointerFrom, px, overProps, parentOf, rememberPath, role, rtl, spring, still, surface, target, text, token, wholeWords, xy,
} from "./base.ts";
import { menu } from "./layers.ts";
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
      const tallFallback = byText(Math.max(phone ? 44 : 40, t.controlMd)) + 2 * pad;
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

/** Page dots: the current one a longer pill (so it reads without colour); under a finger each a 44 px target round its small pip. */
function dots(id: string, count: number, index: number, go: (k: number) => void): void {
  kit.row(id, { width: grow(), align: "center", cross: "center", gap: 0 }, () => {
    for (let k = 0; k < count; k++) {
      const on = k === index;
      const r = surface(kit.idi("Dot", k), { part: "Carousel.dot", cls: "chip", flags: [on ? "current" : ""], selected: on, fill: null, width: px(target(on ? 36 : 22)), height: px(target(28)), align: "center", cross: "center", semantics: { role: "button", name: `${k + 1}`, selected: on } }, () => {
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
  // A path longer than its row wraps onto the next line (crumbs stay whole).
  kit.row(id, { width: desc.width ?? grow(), gap: t.xxs, cross: "center", wrap: true }, () => {
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
        // The current page: a target as tall as the links (a screen reader and a finger find it whole).
        kit.row(kit.idi("Crumb", k), { height: fit(tall), cross: "center", padding: xy(t.xs, 0) }, () => text("Label", it.label ?? "", { step: "sm", nowrap: true, font: "display" }));
        // Where the player is: text, not a link (it goes nowhere).
        kit.semantics({ role: "text", name: it.label ?? "", selected: true, value: "current page" });
      } else {
        const r = surface(kit.idi("Crumb", k), { part: "Breadcrumbs.crumb", cls: "chip", fill: null, width: fit(tall), height: fit(tall), align: "center", padding: xy(t.xs, 0), gap: t.xs, semantics: { role: "link", name: it.label ?? "" } }, () => {
          if (it.icon) kit.icon("Icon", it.icon, { size: 16, tint: token("inkMuted"), label: "" });
          text("Label", it.label ?? "", { step: "sm", color: "inkMuted", nowrap: true });
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
  // A pointer resting where the wheel opens aims at nothing until it moves: -1 is closed last tick
  // (just opened), -2 open with the pointer away (its coming in is a move).
  const prev = kit.memory(`${id}~px`, -1);
  const moved = ptr !== null && (ptr.down || (prev !== -1 && (prev === -2 || Math.round(ptr.x) !== prev || Math.round(ptr.y) !== kit.memory(`${id}~py`, -1))));
  const onRing = ptr !== null && ptr.d >= hole * 0.6 && ptr.d <= radius + 12;
  if (moved && onRing) hovered = pick(ptr.dx, ptr.dy);
  // A pointer press released off the ring (in the hole: the way to cancel) chooses nothing.
  const pointerRelease = kit.memory(`${id}~pd`, 0) > 0 && !(ptr?.down ?? false);
  kit.remember(`${id}~px`, ptr ? Math.round(ptr.x) : open ? -2 : -1);
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
    // The captions: each slice's icon over its name, placed where the box fits wholly inside its
    // wedge (between its two edges and the ring's inner and outer circles, a margin from each),
    // tried from the ring's middle outwards and inwards. The names' widths are last tick's (kept
    // while they are hidden). When any slice's would not fit (a small wheel, a large text size)
    // every slice shows its icon alone, as weapon wheels do, and the centre names the one aimed at.
    const thick = radius - hole;
    const iconPx = Math.round(clamp(thick * 0.34, 18, 28));
    const capLine = Math.round(16 * Math.max(1, frame().theme.textScale ?? 1));
    const gap = 2;
    const named = desc.labels !== false;
    const widths = items.map((it, k) => kit.memory(`~capW${k}`, 0) || (it.label?.length ?? 0) * capLine * 0.55);
    const margin = 6;
    // Where a w×h box centred on the bisector at `rho` lies inside wedge k (none: it does not).
    const inWedge = (k: number, w: number, h: number, rho: number): boolean => {
      const mid = -Math.PI / 2 + k * turn;
      const bx = Math.cos(mid) * rho;
      const by = Math.sin(mid) * rho;
      const edges = [mid - turn / 2, mid + turn / 2];
      for (const [dx, dy] of [[-1, -1], [1, -1], [1, 1], [-1, 1]] as const) {
        const x = bx + (dx * w) / 2;
        const y = by + (dy * h) / 2;
        const d = Math.hypot(x, y);
        if (d < hole + margin || d > radius - margin) return false;
        // Its distance inside each edge line (an edge is a ray from the centre at angle e).
        for (const [j, e] of edges.entries()) {
          const cross = Math.cos(e) * y - Math.sin(e) * x;
          if ((j === 0 ? cross : -cross) < margin) return false;
        }
      }
      return true;
    };
    const place = (k: number, w: number, h: number): number | null => {
      const middle = (radius + hole) / 2;
      for (const step of [0, 4, -4, 8, -8, 12, -12, 16, -16, 20, -20]) if (inWedge(k, w, h, middle + step)) return middle + step;
      return null;
    };
    const capH = iconPx + gap + capLine;
    // A cost takes a line of its own under the label.
    const costH = (it: Api.RadialItem) => (it.cost ? gap + capLine : 0);
    const plan = items.map((it, k) => (it.label && named ? place(k, Math.max(iconPx, widths[k]!), (it.icon ? capH : capLine) + costH(it)) : null));
    const captions = items.every((it, k) => !it.label || !named || plan[k] !== null);
    // The wheel takes the pointer, keys (Left and Right step round it, Enter chooses) and pads; its
    // focus ring follows its circle.
    const measuredW = [...widths];
    kit.open("Wheel", { width: px(radius * 2), height: px(radius * 2), radius, interactive: true, focusable: true, adjustable: true, part: "Radial.frame", partClass: "dialog" });
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
        const caption = captions && !!it.label && named;
        const rho = (caption ? plan[k] : null) ?? (radius + hole) / 2;
        const boxW = caption ? Math.max(iconPx, widths[k]!) : iconPx;
        const boxH = caption ? (it.icon ? capH : capLine) + costH(it) : iconPx;
        const cx = radius + Math.cos(mid) * rho * (rtl() ? -1 : 1);
        const cy = radius + Math.sin(mid) * rho;
        kit.column("Label", { width: fit(), cross: "center", gap }, () => {
          if (it.icon) kit.icon("Icon", it.icon, { size: iconPx, tint: token(on ? "onPrimary" : "ink"), label: "" });
          if (caption) text("Text", it.label ?? "", { step: "caption", color: on ? "onPrimary" : "inkMuted", align: "center", nowrap: true });
          if (caption && it.cost) {
            // The price in the ink, or in the danger hue while it cannot be paid (a build wheel's tower the gold does not reach).
            const ink = it.unaffordable ? "danger" : on ? "onPrimary" : "ink";
            kit.row("Cost", { gap: 2, cross: "center", part: `Radial.cost${flags(it.unaffordable && "unaffordable")}` }, () => {
              kit.icon("Icon", it.costIcon || "coin", { size: Math.round(capLine * 0.8), tint: token(it.unaffordable ? "danger" : "gold"), label: "" });
              text("Text", it.cost ?? "", { step: "caption", color: ink, nowrap: true });
            });
          }
        });
        over({ insets: { start: (rtl() ? radius * 2 - cx : cx) - boxW / 2, top: cy - boxH / 2 }, inert: true });
        const tb = caption ? kit.bounds(`${kit.lastPath()}/Text`) : null;
        if (tb) measuredW[k] = tb.width;
      }
      kit.close();
      over({ insets: { start: 0, top: 0 }, inert: true });
      kit.semantics({ role: "menuitem", name: it.label ?? "", selected: on, disabled: it.disabled ?? false });
    });
    // The centre: what is chosen.
    // The hub: a disc (drawn on the hub itself, right after it opens) naming what is aimed at; its
    // text keeps inside the disc's inscribed square.
    const hub = hole * 2 - 8;
    const inner = Math.round(hub * 0.7071);
    kit.open("Centre", { direction: "column", width: px(hub), height: px(hub), align: "center", cross: "center", gap: 2, padding: all(Math.round((hub - inner) / 2)), part: "Radial.centre", partClass: "avatar" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("panelRaised") }], strokes: [{ width: 1, align: "inside", paint: { kind: "solid", color: token("line") } }] });
    {
      const it = items[hovered];
      // Whole words only ("Quick select" never breaks into "Quic / k"): the name on at most two
      // lines and the detail on one, each at the largest step that keeps inside the square.
      const ramp = frame().theme.typeRamp as Record<string, Api.TypeStep>;
      const detailH = it?.detail ? (ramp.caption?.lineHeight ?? 16) + 2 : 0;
      wholeWords("Name", it?.label ?? desc.title ?? "", { width: inner, height: inner - detailH, maxLines: 2, steps: ["base", "sm", "caption", "meta", "xs"], heading: true });
      if (it?.detail) wholeWords("Detail", it.detail, { width: inner, height: detailH, maxLines: 1, steps: ["caption", "meta", "xs"], color: "inkMuted" });
      // Labels hidden on the ring: the aimed slice's cost shows here instead.
      if (it?.cost && !captions) text("Cost", it.cost, { step: "caption", color: it.unaffordable ? "danger" : "ink", nowrap: true });
    }
    kit.close();
    over({ insets: { start: radius - hole + 4, top: radius - hole + 4 }, inert: true, z: 2 });
    const w = kit.close() as Api.Response;
    rememberPath(key, kit.lastPath());
    measuredW.forEach((v, k) => kit.remember(`~capW${k}`, v));
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
