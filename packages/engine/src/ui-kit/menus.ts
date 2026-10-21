// Menus and dialogs: the title screen, pause, confirm, alert, loading and
// results. Each is one call per tick: data in, what happened out.

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { all, clock, fmt, frame, iconOr, lapTime, menuItems, px, grow, token } from "./util.ts";

/** The buttons of a menu in a column; answers the chosen id. With `autofocus` its first enabled item takes the focus when the menu appears (og-ui §4: what opens takes the focus). */
function menuColumn(id: string, items: ReturnType<typeof menuItems>, width: Api.Size, autofocus = false): string | null {
  const { t } = frame();
  let chosen: string | null = null;
  const first = autofocus ? items.findIndex((i) => !i.disabled) : -1;
  kit.column(id, { width, gap: t.sm, cross: "stretch" }, () => {
    items.forEach((item, k) => {
      const r = kit.button(item.id, item.label, {
        variant: item.primary ? "primary" : "secondary",
        size: "large",
        disabled: item.disabled,
        width: grow(),
        autofocus: k === first,
      }) as Api.Response;
      if (r.clicked) chosen = item.id;
    });
  });
  return chosen;
}

export function mainMenu(id: string, desc: Api.MainMenuDesc = {}): Api.MenuResult {
  const { t, phone } = frame();
  const items = menuItems(desc.items, [["play", "Play"]]);
  let chosen: string | null = null;
  kit.anchor(id, {
    anchor: phone ? "bottom" : "center",
    direction: "column",
    gap: t.xl,
    cross: "center",
    width: phone ? grow() : px(380),
    padding: all(phone ? t.lg : t.xl),
    background: token("materialThick"),
    radius: t.panel,
  }, () => {
    kit.column("Heading", { gap: t.xs, cross: "center", width: grow() }, () => {
      kit.label("Title", desc.title ?? "", { step: phone ? "displayMd" : "displayLg", font: "display", align: "center", width: grow(), heading: true });
      if (desc.subtitle) kit.label("Subtitle", desc.subtitle, { color: token("fgMuted"), align: "center", width: grow() });
    });
    // A title menu is not a dialog, so it takes the focus itself: Enter and the pad's A answer at once.
    chosen = menuColumn("Items", items, grow(), true);
    if (desc.footer) kit.label("Footer", desc.footer, { step: "meta", color: token("fgSubtle"), align: "center", width: grow() });
  });
  if (chosen !== null) desc.onSelect?.(chosen);
  return { selected: chosen };
}

export function pauseMenu(id: string, desc: Api.PauseMenuDesc = {}): Api.MenuResult {
  const items = menuItems(desc.items, [["resume", "Resume"], ["settings", "Settings"], ["quit", "Quit to menu"]]);
  let chosen: string | null = null;
  const m = kit.modal(id, desc.open ?? false, { title: desc.title ?? "Paused", width: px(Math.min(340, frame().viewport.width - 32)) }, () => {
    chosen = menuColumn("Items", items, grow());
  });
  if (m.dismissed) chosen = "resume";
  if (chosen !== null) desc.onSelect?.(chosen);
  return { selected: chosen };
}

export function confirmDialog(id: string, desc: Api.ConfirmDesc = {}): Api.ConfirmResult {
  const { t } = frame();
  let confirmed = false;
  let cancelled = false;
  const m = kit.modal(id, desc.open ?? false, { title: desc.title ?? "" }, () => {
    if (desc.message) kit.richText("Message", desc.message, { step: "reading", color: token("fgMuted"), width: grow() });
    kit.row("Actions", { width: grow(), gap: t.sm, align: "end", padding: { left: 0, top: t.sm, right: 0, bottom: 0 } }, () => {
      if ((kit.button("Cancel", desc.cancelLabel ?? "Cancel", {}) as Api.Response).clicked) cancelled = true;
      if ((kit.button("Confirm", desc.confirmLabel ?? "Confirm", { variant: desc.danger ? "danger" : "primary" }) as Api.Response).clicked) confirmed = true;
    });
  });
  if (m.dismissed) cancelled = true;
  return { confirmed, cancelled };
}

export function alertDialog(id: string, desc: Api.AlertDesc = {}): boolean {
  const { t } = frame();
  let done = false;
  const m = kit.modal(id, desc.open ?? false, { title: desc.title ?? "" }, () => {
    if (desc.message) kit.richText("Message", desc.message, { step: "reading", color: token("fgMuted"), width: grow() });
    kit.row("Actions", { width: grow(), align: "end", padding: { left: 0, top: t.sm, right: 0, bottom: 0 } }, () => {
      if ((kit.button("Ok", desc.okLabel ?? "OK", { variant: "primary" }) as Api.Response).clicked) done = true;
    });
  });
  return done || m.dismissed;
}

export function loadingScreen(id: string, desc: Api.LoadingDesc = {}): void {
  const { t } = frame();
  const tips = desc.tips ?? [];
  const tip = tips.length > 0 ? tips[Math.floor(kit.seconds() / Math.max(1, desc.tipSeconds ?? 4)) % tips.length]! : "";
  kit.modal(id, true, { locked: true, bare: true, backdrop: token("bg"), transition: "fade", width: px(Math.min(380, frame().viewport.width - 32)) }, () => {
    kit.label("Title", desc.title ?? "Loading", { step: "xl", align: "center", width: grow(), heading: true });
    const p = desc.progress;
    if (p === null || p === undefined) {
      kit.spinner("Spinner", 0);
    } else {
      // A real value only: the bar never runs ahead of what loaded.
      kit.row("Bar", { width: grow(), gap: t.md, cross: "center" }, () => {
        kit.progress("Progress", p, { width: grow() });
        kit.label("Percent", `${Math.floor(Math.max(0, Math.min(1, p)) * 100)}%`, { step: "meta", font: "mono", color: token("fgMuted"), nowrap: true });
      });
    }
    if (tip) kit.label("Tip", tip, { step: "reading", color: token("fgMuted"), align: "center", width: grow() });
  });
}

/** Stars landing one by one: k appears 0.3 s after k-1, from the screen's opening; an earned one bursts. */
function stars(id: string, earned: number, max: number, since: number, size: number): void {
  const { t } = frame();
  kit.row(id, { gap: t.sm, cross: "center" }, () => {
    for (let k = 0; k < max; k++) {
      const shown = kit.seconds() - since >= 0.3 * (k + 1);
      const on = k < earned;
      kit.transition(kit.idi("Star", k), shown, { kind: "scale", instant: false }, () => {
        iconOr("Mark", "star", size, token(on ? "accent" : "fillStrong"));
        if (on) kit.burst({ delayMs: 120 });
      });
      if (!shown) kit.spacer(kit.idi("Gap", k), px(size));
    }
  });
}

export function resultsScreen(id: string, desc: Api.ResultsDesc = {}): Api.MenuResult {
  const { t, phone } = frame();
  const open = desc.open ?? false;
  // When it opened, in UI seconds (-1 while closed): the count-up and the stars run from there.
  let since = kit.memory(`${id}.since`, -1);
  if (!open) since = -1;
  else if (since < 0) since = kit.seconds();
  kit.remember(`${id}.since`, since);
  const items = menuItems(desc.actions, [["retry", "Retry"], ["next", "Next"], ["menu", "Menu"]]);
  let chosen: string | null = null;
  const score = desc.score ?? 0;
  const time = desc.format === "time";
  // The score counts up over 0.9 s from the opening; "New best" pops in when it lands.
  const landed = open && kit.seconds() - since >= 0.9;
  // A time is lower-is-better (a best of 0 is none); points higher-is-better.
  const prior = desc.best ?? 0;
  const best = time ? (desc.newBest ? (prior > 0 ? Math.min(prior, score) : score) : prior) : Math.max(prior, desc.newBest ? score : 0);
  kit.modal(id, open, { title: desc.title ?? "Level complete", locked: true, width: px(Math.min(phone ? 360 : 420, frame().viewport.width - 32)) }, () => {
    const max = desc.maxStars ?? 3;
    if (max > 0) kit.row("StarRow", { width: grow(), align: "center" }, () => stars("Stars", desc.stars ?? 0, max, since, 44));
    kit.column("ScoreBlock", { width: grow(), cross: "center", gap: t.xxs }, () => {
      if (desc.scoreLabel) kit.label("ScoreLabel", desc.scoreLabel, { step: "meta", color: token("fgMuted"), align: "center", width: grow(), nowrap: true });
      if (time) {
        // A time counts up as a clock (ease-out cubic over the same 0.9 s), the headline in m:ss.mmm.
        const k = open ? Math.min(1, Math.max(0, (kit.seconds() - since) / 0.9)) : 1;
        const shown = score * (1 - (1 - k) ** 3);
        kit.label("Score", k >= 1 ? lapTime(score) : clock(shown, true), { step: "displayLg", font: "display", align: "center", width: grow(), nowrap: true });
      } else {
        kit.counter("Score", score, { from: 0, durationMs: 900, step: "displayLg", font: "display", align: "center", width: grow() });
      }
      if (!time || best > 0 || desc.newBest) {
        kit.row("BestRow", { gap: t.sm, cross: "center" }, () => {
          if (!time || best > 0) kit.label("Best", (desc.bestLabel ?? "Best {best}").replaceAll("{best}", time ? lapTime(best) : fmt(best)), { step: "meta", color: token("fgMuted"), nowrap: true });
          if (desc.newBest) kit.transition("Reveal", landed, { kind: "pop" }, () => kit.badge("NewBest", desc.newBestLabel ?? "New best", "accent"));
        });
      }
    });
    const stats = desc.stats ?? [];
    if (stats.length > 0) {
      kit.column("Stats", { width: grow(), gap: 0 }, () => {
        stats.forEach((s, i) => kit.keyValue(kit.idi("Stat", i), s.label ?? "", s.value ?? "", true));
      });
    }
    kit.row("Actions", { width: grow(), gap: t.sm, wrap: phone }, () => {
      for (const item of items) {
        const r = kit.button(item.id, item.label, { variant: item.primary ? "primary" : "secondary", width: grow(), disabled: item.disabled }) as Api.Response;
        if (r.clicked) chosen = item.id;
      }
    });
    if (desc.shareText) {
      if ((kit.button("Share", desc.shareLabel ?? "Share", { variant: "quiet", width: grow() }) as Api.Response).clicked) {
        chosen = "share";
        desc.onShare?.(desc.shareText);
      }
    }
  });
  return { selected: chosen };
}

