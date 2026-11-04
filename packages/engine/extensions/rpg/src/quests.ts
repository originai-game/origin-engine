// The quest log and the quest tracker (docs/ui.md §11 genre patterns,
// Appendix B: quest log, journal and codex), fed by og.story.quests. Read
// against The Witcher 3's journal (the list beside the quest, its
// objectives and rewards), Genshin Impact's (category tabs, a quest's
// giver and region, Track and Navigate) and Destiny 2's (the tracked one
// pinned, counts and bars). Which quest is shown and which tab is the
// game's state; on a phone in portrait, whether the quest is open over
// the list is the component's own (og.ui.memory).

import type * as Api from "@origingame/engine";
import { objectiveTracker } from "@origingame/engine/ui-parts/hud";
import { artSlot, btn, chevron, clamp, eyebrow, fill, flags, frame, grow, kit, line, mix, over, paint, pips, px, rarityColor, role, rtl, shadow, spring, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type SymbolKind } from "@origingame/engine/ui-parts/screen";
import type { QuestLogDesc, QuestLogResult, QuestTrackerDesc } from "../types.ts";

/** A category's mark: the main story a crown, a side quest a flag, a daily a clock, anything else a dot. */
function categoryMark(c: string): SymbolKind {
  return c === "main" || c === "story" ? "crown" : c === "side" ? "flag" : c === "daily" || c === "weekly" ? "clock" : c === "event" ? "star" : "dot";
}

/** A box ticked, crossed or open (an objective's), drawn as shapes. */
function checkBox(id: string, state: "open" | "done" | "failed", size: number, popped: boolean): void {
  const ink = role("ink");
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", part: `QuestLog.check${flags(state !== "open" && state)}`, partClass: "badge" });
  kit.setDraw({
    shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] },
    fills: state === "done" ? [{ kind: "solid", color: paint(ink) }] : state === "failed" ? [{ kind: "solid", color: token("danger", 0.2) }] : [],
    strokes: state === "open" ? [line(1.5, token("ink", 0.55))] : state === "failed" ? [line(1.5, token("danger"))] : [],
  });
  if (state === "done") symbol("Tick", "check", Math.round(size * 0.72), role("panel"));
  if (state === "failed") symbol("Cross", "cross", Math.round(size * 0.62), role("danger"));
  kit.close();
  if (popped && !still()) kit.props({ enter: { values: [{ prop: "scale", value: 1.5 }], transition: spring("emphasis") } });
}

/** A small dot: new (breathing, in the highlight) or updated (still, in the ink). */
function dot(id: string, kind: "new" | "updated"): void {
  const s = u(kind === "new" ? 9 : 7);
  const breathe = kind === "new" && !still() ? 0.75 + 0.25 * Math.sin(kit.tick() * 0.08) : 1;
  kit.open(id, { width: px(s), height: px(s), part: `QuestLog.dot${flags(kind)}`, partClass: "badge" });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: kind === "new" ? token("highlight", breathe) : token("ink", 0.7) }], ...(kind === "new" ? { effects: [{ kind: "outerGlow", blur: u(6), spread: 0, color: token("highlight", 0.5 * breathe) }] } : {}) } as Api.DrawDesc);
  kit.close();
}

/** The tabs: a lit chip with a rim under it for the one shown, quiet glass with muted words for the rest. */
function tabsRow(desc: QuestLogDesc, tab: string, counts: Record<string, number>, out: Mut<QuestLogResult>): void {
  const tabs = desc.tabs ?? [];
  if (tabs.length === 0) return;
  kit.open("Tabs", { direction: "row", gap: u(6), width: grow(), wrap: true, part: "QuestLog.tabs", partClass: "panel" });
  for (const tb of tabs) {
    const key = tb.key ?? "";
    const label = tb.label ?? key;
    const on = key === tab;
    const r = surface(key, {
      part: `QuestLog.tab${flags(on && "selected")}`,
      cls: "tab",
      fill: on ? mix(role("panelRaised"), role("ink"), 0.1) : { ...role("shadow"), a: 0.45 },
      selected: on,
      selectedFill: mix(role("panelRaised"), role("ink"), 0.1),
      gap: u(8),
      padding: xy(u(14), u(9)),
      // A whole touch target under a finger: the phone's tabs laid out 42 px tall.
      height: { kind: "fit", min: frame().touch ? 44 : 0 },
      radius: u(8),
      semantics: { role: "tab", name: `${label} (${counts[key] ?? 0})`, selected: on },
    }, () => {
      symbol("Mark", categoryMark(key), u(14), on ? role("ink") : role("inkMuted"));
      text("Text", label, { step: st(15), color: on ? "ink" : "inkMuted", nowrap: true, font: "display" });
      text("Count", String(counts[key] ?? 0), { step: st(13), color: "inkMuted", nowrap: true, font: "mono" });
      if (on) {
        kit.open("Rim", { width: px(u(24)), height: px(u(3)), radius: 999, background: token("ink"), part: "QuestLog.tabRim", partClass: "badge" });
        kit.close();
        over({ insets: { centerX: true, bottom: u(3) }, inert: true });
      }
    });
    if (r.clicked && !on) {
      out.tab = key;
      kit.feedback("tab", -1);
    }
  }
  kit.close();
  kit.semantics({ role: "tablist", name: desc.title ?? "Quests" });
  const at = tabs.findIndex((x) => x.key === tab);
  const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
  if (step !== 0 && at >= 0) {
    out.tab = tabs[(at + step + tabs.length) % tabs.length]!.key ?? null;
    kit.feedback("tab", -1);
  }
}

/** A quest's row in the list. */
function questRow(q: Api.QuestView, selected: boolean, desc: QuestLogDesc, out: Mut<QuestLogResult>, open: () => void): void {
  const ended = q.status !== "active";
  const ink = role("ink");
  const r = surface(q.id, {
    part: `QuestLog.row${flags(selected && "selected", q.tracked && "tracked", q.isNew && "new", q.updated && "updated", ended && q.status)}`,
    cls: "listItem",
    fill: { ...role("shadow"), a: 0.42 },
    selected,
    selectedFill: mix(role("panelRaised"), ink, 0.08),
    width: grow(),
    gap: u(12),
    padding: { left: u(14), right: u(14), top: u(12), bottom: u(12) },
    radius: u(8),
    semantics: {
      role: "option",
      name: [q.title, q.category, ended ? (q.status === "completed" ? desc.completedLabel ?? "Completed" : desc.failedLabel ?? "Failed") : "", q.isNew ? desc.newLabel ?? "New" : q.updated ? "updated" : "", q.tracked ? desc.trackedLabel ?? "Tracked" : ""].filter(Boolean).join(", "),
      selected,
    },
    props: { states: [{ name: "status", states: [vstate("active"), vstate("completed"), vstate("failed")], transitions: [] }], state: [{ group: "status", state: q.status === "completed" || q.status === "failed" ? q.status : "active" }] },
  }, () => {
    if (selected) {
      kit.open("Edge", { width: px(u(3)), height: grow() });
      kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink") }] });
      kit.close();
      over({ insets: { start: 0, top: u(10), bottom: u(10) }, inert: true });
    }
    kit.open("Mark", { width: px(u(32)), height: px(u(32)), align: "center", cross: "center", radius: 999, part: "QuestLog.mark", partClass: "badge" });
    kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("ink", ended ? 0.04 : 0.08) }] });
    if (q.status === "completed") symbol("Sym", "check", u(16), role("success"));
    else if (q.status === "failed") symbol("Sym", "cross", u(14), role("danger"));
    else symbol("Sym", categoryMark(q.category), u(16), ended ? role("inkFaint") : mix(ink, role("inkMuted"), 0.3));
    kit.close();
    kit.column("Words", { width: grow(), gap: u(2) }, () => {
      text("Title", q.title, { step: st(16), color: ended ? "inkMuted" : "ink", width: grow(), maxLines: 2, font: "display" });
      if (q.region) text("Region", q.region, { step: st(13), color: "inkMuted", width: grow(), maxLines: 2, ellipsis: true });
    });
    kit.column("Marks", { gap: u(6), cross: "end" }, () => {
      if (q.tracked && !ended) symbol("Tracked", "pin", u(16), ink, { label: desc.trackedLabel ?? "Tracked" });
      if (q.isNew) dot("New", "new");
      else if (q.updated) dot("Updated", "updated");
    });
  });
  if (r.clicked) {
    out.selected = q.id;
    kit.feedback("select", -1);
    open();
  }
}

/** The quest itself: its title, giver and region, summary, objectives, rewards and actions. */
function questDetail(q: Api.QuestView, desc: QuestLogDesc, out: Mut<QuestLogResult>, fresh: boolean, phone: boolean): void {
  const ink = role("ink");
  const ended = q.status !== "active";
  const art = new Map((desc.art ?? []).map((a) => [a.path, a.texture]));
  kit.openScroll("Body", { width: grow(), height: grow(), gap: u(phone ? 14 : 18), padding: { left: 0, right: u(8), top: 0, bottom: u(8) }, fade: 40, fadeInto: "panel" } as never);
  kit.props(fresh && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: rtl() ? -u(20) : u(20) }], transition: spring("panel") } } : {});
  kit.column("Head", { width: grow(), gap: u(6) }, () => {
    eyebrow("Category", q.category || "", "inkMuted");
    kit.row("TitleRow", { width: grow(), gap: u(12), cross: "center", wrap: true }, () => {
      text("Title", q.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", maxLines: 3 });
      if (ended) {
        // A stamp: completed in success, failed in danger, a pill with its word (it says what the muted list row only hints).
        const c = q.status === "completed" ? role("success") : role("danger");
        kit.row("Stamp", { gap: u(6), cross: "center", padding: xy(u(10), u(4)), part: `QuestLog.stamp${flags(q.status)}`, partClass: "badge" }, () => {
          kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: paint(c, 0.18) }] });
          symbol("Sym", q.status === "completed" ? "check" : "cross", u(13), c);
          text("Text", (q.status === "completed" ? desc.completedLabel ?? "Completed" : desc.failedLabel ?? "Failed").toLocaleUpperCase(), { step: st(13), font: "display", color: mix(c, role("ink"), 0.35), nowrap: true });
        });
      }
    });
    kit.row("Meta", { width: grow(), gap: u(16), cross: "center", wrap: true, part: "QuestLog.meta", partClass: "badge" }, () => {
      if (q.giver) kit.row("Giver", { gap: u(6), cross: "center" }, () => {
        symbol("Sym", "chat", u(14), role("inkMuted"));
        text("Text", fill(desc.giverLabel ?? "From {giver}", { giver: q.giver }), { step: st(14), color: "inkMuted", nowrap: true });
      });
      if (q.region) kit.row("Region", { gap: u(6), cross: "center" }, () => {
        symbol("Sym", "pin", u(14), role("inkMuted"));
        text("Text", q.region, { step: st(14), color: "inkMuted", nowrap: true });
      });
    });
  });
  if (q.summary) text("Summary", q.summary, { step: st(phone ? 16 : 17), color: mix(ink, role("inkMuted"), 0.35), width: grow() });

  const shown = q.objectives;
  if (shown.length > 0) {
    eyebrow("ObjectivesHead", desc.objectivesLabel ?? "Objectives");
    kit.open("Objectives", { direction: "column", width: grow(), gap: u(10) });
    for (const o of shown) {
      const state = o.failed ? "failed" : o.done ? "done" : "open";
      const was = kit.memory(`~q.o.${q.id}.${o.id}`, -1);
      const popped = was === 0 && o.done;
      kit.remember(`~q.o.${q.id}.${o.id}`, o.done ? 1 : 0);
      if (popped) kit.feedback("success", -1);
      kit.open(o.id, { direction: "row", width: grow(), gap: u(12), cross: "start", part: `QuestLog.objective${flags(o.done && "done", o.failed && "failed", o.optional && "optional", o.hidden && "hidden")}`, partClass: "listItem" });
      kit.props({ states: [{ name: "step", states: [vstate("open"), vstate("done"), vstate("failed")], transitions: [] }], state: [{ group: "step", state }] });
      checkBox("Check", state, u(20), popped);
      kit.column("Words", { width: grow(), gap: u(6) }, () => {
        kit.row("Line", { width: grow(), gap: u(8), cross: "center", wrap: true }, () => {
          text("Text", o.hidden ? desc.hiddenLabel ?? "???" : o.text, { step: st(phone ? 15 : 16), color: o.hidden ? "inkFaint" : o.failed ? mix(role("danger"), ink, 0.35) : o.done ? "inkMuted" : "ink", width: grow(), maxLines: 3 });
          if (o.optional) {
            kit.row("Optional", { padding: xy(u(7), u(2)), part: "QuestLog.optional", partClass: "badge" }, () => {
              kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("ink", 0.08) }] });
              text("Text", desc.optionalLabel ?? "Optional", { step: st(12), color: "inkMuted", nowrap: true });
            });
          }
          if (o.target > 1 && !o.hidden) text("Count", `${Math.min(o.count, o.target)}/${o.target}`, { step: st(14), font: "mono", color: o.done ? "inkMuted" : "ink", nowrap: true });
        });
        if (o.target > 1 && !o.hidden && !o.done && !o.failed) {
          // Its bar: a well the width of the words, the share done in the ink.
          const f = clamp(o.count / o.target, 0, 1);
          kit.open("Bar", { width: grow(), height: px(u(4)), part: "QuestLog.bar", partClass: "badge" });
          kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink", 0.12) }] });
          kit.open("Fill", { width: { kind: "percent", value: f * 100 } as never, height: grow() });
          kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink", 0.85) }] });
          kit.close();
          kit.close();
        }
      });
      kit.close();
      // A line of the list the game ticks, never a box the player can (a checkbox no key or pad could reach): its
      // state read as its value.
      kit.semantics({ role: "listitem", name: `${o.hidden ? desc.hiddenLabel ?? "???" : o.text}${o.target > 1 && !o.hidden ? `, ${Math.min(o.count, o.target)} of ${o.target}` : ""}${o.optional ? `, ${desc.optionalLabel ?? "Optional"}` : ""}`, ...(o.done ? { value: desc.completedLabel ?? "Completed" } : o.failed ? { value: desc.failedLabel ?? "Failed" } : {}) });
    }
    kit.close();
    kit.semantics({ role: "list", name: desc.objectivesLabel ?? "Objectives" });
  }

  if (q.rewards.length > 0) {
    eyebrow("RewardsHead", desc.rewardsLabel ?? "Rewards");
    kit.open("Rewards", { direction: "row", gap: u(10), wrap: true, width: grow() });
    q.rewards.forEach((rw, i) => {
      const rarity = (rw.rarity || "none") as Api.Rarity;
      const tier = rarityColor(rarity);
      const tw = u(phone ? 76 : 88);
      // The tile is the card a style dresses; its caption stands in its own row under it, outside the card's frame
      // (inside it, a style's frame and notches cut the name).
      // The caption's row is wider than the tile, so a name of two words keeps them whole at 200 % text.
      kit.open(kit.idi("R", i), { direction: "column", width: px(Math.round(tw * 1.3)), gap: u(6), cross: "center" });
      kit.open("Tile", { width: px(tw), height: px(tw), part: `QuestLog.reward${flags(`rarity=${rarity}`)}`, partClass: "card" });
      artSlot("Art", "QuestLog.rewardArt", { texture: art.get(rw.image) ?? null, kind: "item", rarity, width: px(tw), height: px(tw), radius: u(10) }, () => {
        // The tier as light rising from the foot (undressed, so a style's dress keeps its gradient), no outline.
        if (tier) {
          kit.open("Tier", { width: grow(), height: px(Math.round(tw * 0.45)) });
          kit.setDraw({ shape: { kind: "rect", radii: [0, 0, u(10), u(10)] }, fills: [vgrad([[0, paint(tier, 0)], [0.6, paint(tier, 0.18)], [1, paint(tier, 0.5)]])] });
          kit.close();
          over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true });
        }
        if (rw.amount > 1) {
          kit.row("Amount", { padding: xy(u(5), u(1)), part: "QuestLog.amount", partClass: "badge" }, () => {
            kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("shadow", 0.78) }] });
            text("Text", `×${rw.amount}`, { step: st(13), font: "mono", nowrap: true });
          });
          over({ insets: { end: u(4), bottom: u(4) }, inert: true });
        }
      });
      kit.close();
      kit.column("Caption", { width: grow(), gap: u(4), cross: "center", part: "QuestLog.caption", partClass: "badge" }, () => {
        text("Name", rw.name, { step: st(13), color: "inkMuted", align: "center", width: grow(), maxLines: 3 });
        pips("Pips", rarity, u(5));
      });
      kit.close();
      kit.semantics({ role: "listitem", name: [rw.name, rw.amount > 1 ? `×${rw.amount}` : "", rarity !== "none" ? rarity : ""].filter(Boolean).join(", ") });
    });
    kit.close();
  }
  kit.close();

  if (!ended) {
    kit.row("Actions", { width: grow(), gap: u(10), cross: "center", wrap: true, padding: { left: 0, right: 0, top: u(10), bottom: 0 } }, () => {
      const r = btn("Track", q.tracked ? desc.trackedLabel ?? "Tracked" : desc.trackLabel ?? "Track", { variant: q.tracked ? "secondary" : "primary", sym: q.tracked ? "check" : "pin", selected: q.tracked, part: `QuestLog.track${flags(q.tracked && "on")}`, size: phone ? "md" : "lg" });
      if (r.clicked) {
        out.track = q.id;
        kit.feedback("confirm", -1);
      }
      if (q.marker && desc.mapLabel !== "") {
        if (btn("Map", desc.mapLabel ?? "Show on map", { variant: "glass", sym: "pin", part: "QuestLog.map", size: phone ? "md" : "lg" }).clicked) out.showOnMap = q.id;
      }
    });
  }
}

/**
 * A quest log (The Witcher 3, Genshin Impact, Destiny 2): see
 * `fn questLog` in crates/og-api/api/ui.ogt.
 */
export function questLog(id: string, desc: QuestLogDesc): QuestLogResult {
  const out: Mut<QuestLogResult> = { selected: null, track: null, showOnMap: null, tab: null, closed: false };
  const { viewport: vp, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const all = desc.quests ?? [];
  const tabs = desc.tabs ?? [];
  const tab = tabs.some((x) => x.key === desc.tab) ? desc.tab! : tabs[0]?.key ?? "";
  const counts: Record<string, number> = {};
  for (const q of all) if (q.status === "active") counts[q.category] = (counts[q.category] ?? 0) + 1;
  const inTab = tabs.length > 0 ? all.filter((q) => q.category === tab) : all;
  const active = inTab.filter((q) => q.status === "active");
  const ended = inTab.filter((q) => q.status !== "active");
  const shown = inTab.find((q) => q.id === desc.selected) ?? inTab.find((q) => q.tracked && q.status === "active") ?? active[0] ?? ended[0] ?? null;
  // A change of quest slides the detail in.
  const sk = shown ? shown.id : "";
  const hash = Array.from(sk).reduce((h, c) => (h * 31 + c.charCodeAt(0)) >>> 0, 7);
  const fresh = kit.memory("~q.shown", 0) !== hash;
  kit.remember("~q.shown", hash);
  // A phone in portrait shows the list or, once a quest is picked, the quest over it with Back.
  // og.ui.memory is scoped by the open element: read and written here, at the log's own level, never inside a row.
  const opened = portrait && kit.memory("~q.open", 0) === 1 && !!shown;
  let openNext = opened;

  kit.open(id, { direction: "row", width: grow(), height: grow(), gap: u(phone ? 12 : 24), part: "QuestLog.frame", partClass: "panel" });
  if (!opened) {
    const lw = portrait ? null : Math.min(u(440), Math.round((vp.width - vp.safeArea.left - vp.safeArea.right) * 0.38));
    kit.open("Side", { direction: "column", width: lw === null ? grow() : px(lw), height: grow(), gap: u(12) });
    tabsRow(desc, tab, counts, out);
    kit.openScroll("List", { width: grow(), height: grow(), gap: u(6), padding: { left: 0, right: u(6), top: 0, bottom: u(8) }, fade: 48, fadeInto: "shadow", part: "QuestLog.list" } as never);
    if (inTab.length === 0) text("Empty", desc.emptyLabel ?? "No quests yet.", { step: st(16), color: "inkMuted", width: grow() });
    const group = (key: string, label: string, list: Api.QuestView[]): void => {
      if (list.length === 0) return;
      kit.open(key, { direction: "column", width: grow(), gap: u(6), padding: { left: 0, right: 0, top: u(6), bottom: u(4) }, part: "QuestLog.group", partClass: "panel" });
      eyebrow("Head", `${label} · ${list.length}`);
      for (const q of list) questRow(q, !portrait && shown?.id === q.id, desc, out, () => {
        if (portrait) openNext = true;
      });
      kit.close();
    };
    group("Active", desc.activeLabel ?? "Active", active);
    group("Done", desc.completedLabel ?? "Completed", ended);
    kit.close();
    kit.semantics({ role: "listbox", name: desc.title ?? "Quests" });
    kit.close();
  }
  if (shown && (!portrait || opened)) {
    kit.open("Detail", { direction: "column", width: grow(), height: grow(), gap: u(14), padding: phone ? xy(u(16), u(14)) : { left: u(32), right: u(28), top: u(28), bottom: u(24) }, part: "QuestLog.detail", partClass: "panel" });
    kit.setDraw({ shape: { kind: "rect", radii: [u(16), u(16), u(16), u(16)] }, fills: [{ kind: "solid", color: token("panel", 0.88) }, vgrad([[0, token("sheen", 0.05)], [1, token("sheen", 0)]])], effects: [shadow(u(10), u(30), 0.45)] });
    if (portrait) {
      // The style's secondary button with a chevron to the start edge (a glass pane read as a flat grey box).
      if (btn("Back", desc.backLabel ?? "Back", { variant: "secondary", size: "sm", part: "QuestLog.back", before: () => chevron("Chevron", "back", u(14), role("ink")) }).clicked) {
        openNext = false;
        kit.feedback("back", -1);
      }
    }
    questDetail(shown, desc, out, fresh, phone);
    kit.close();
    kit.semantics({ role: "region", name: shown.title });
  }
  kit.close();
  if (kit.action("back")) {
    if (portrait && opened) openNext = false;
    else out.closed = true;
    kit.feedback("back", -1);
  }
  kit.remember("~q.open", openNext ? 1 : 0);
  return out;
}

/**
 * The tracked quests on the HUD: og.ui.objectiveTracker fed by og.story's
 * quests (see `fn questTracker` in crates/og-api/api/ui.ogt).
 */
export function questTracker(id: string, desc: QuestTrackerDesc): void {
  const tick = kit.tick();
  const quests: Api.ObjectiveQuest[] = [];
  for (const q of desc.quests ?? []) {
    // The tick a tracked quest completed: it flashes Complete and leaves (objectiveTracker's completedAt).
    const k = `${id}~qt.${q.id}`;
    const was = kit.memory(k, 0);
    if (q.status === "completed" && q.tracked && was === 0) kit.remember(k, tick);
    if (q.status === "active") kit.remember(k, 0);
    const doneAt = kit.memory(k, 0);
    if (!q.tracked || q.status === "failed" || (q.status === "completed" && doneAt === 0)) continue;
    const m = q.marker;
    const from = desc.from;
    const distance = m && from ? Math.hypot(m.position.x - from.x, m.position.y - from.y, m.position.z - from.z) : null;
    quests.push({
      key: q.id,
      title: q.title,
      icon: q.category === "main" ? "crown" : q.category === "side" ? "flag" : "",
      focused: quests.length === 0,
      ...(distance !== null ? { distance } : {}),
      ...(doneAt > 0 ? { completedAt: doneAt } : {}),
      completeLabel: desc.completeLabel ?? "Complete",
      steps: q.objectives.filter((o) => !o.hidden).map((o) => ({ key: o.id, text: o.text, done: o.done, failed: o.failed, count: o.count, target: o.target > 1 ? o.target : 0, optional: o.optional })),
    });
  }
  objectiveTracker(id, {
    quests,
    ...(desc.max ? { max: desc.max } : {}),
    ...(desc.width ? { width: desc.width } : {}),
    ...(desc.anchor !== undefined ? { anchor: desc.anchor } : {}),
  });
}
