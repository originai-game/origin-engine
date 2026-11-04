// The settings tree (docs/ui.md §11 genre patterns, Appendix B row 2, roadmap G1-7). Read against B-settings and the
// settings of Forza Horizon, Destiny 2 and The Last of Us Part II (the pages as tabs over a column of rows, each its name
// and its control at the end, a detail panel with the focused entry's description, Apply, Revert and Defaults at the
// foot, a display change kept only when confirmed before a countdown runs out). The settings are data: pages of typed
// entries a game adds to or drops from; their values, the pending set, applying and reverting are the game's. Bindings
// are og.ui.keyBindingRow (by physical key code, shown by the layout's label); the HUD page mounts og.ui.hudLayoutEditor.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, eyebrow, flags, frame, glow, grow, kit, lastInteraction, line, pct, pointer, px, rememberRoom, role, safeWidth, shadow, st, surface, symbol,
  text, timerRing, token, u, xy, type Mut,
} from "./core.ts";
import { valueSlider } from "../controls/ranges.ts";
import { segmented, toggleSwitch } from "../controls/choices.ts";
import { dropdown } from "../controls/pickers.ts";
import { keyBindingRow } from "../controls/bindings.ts";

/**
 * A game's settings as data: see `fn settingsTree` in crates/og-api/api/ui.ogt.
 */
export function settingsTree(world: Api.World, id: string, desc: Api.SettingsTreeDesc, custom: Api.SettingsCustomBuild): Api.SettingsTreeResult {
  const out: Mut<Api.SettingsTreeResult> = { page: null, change: null, action: null, bindingChanged: false, apply: false, revert: false, defaults: false, keep: false, confirmRevert: false, hudDone: false };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const W = kit.memory(`~st.w.${id}`, 0) || firstRoom().w;
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const pages = desc.pages ?? [];
  const page = pages.find((p) => p.key === desc.page) ?? pages[0] ?? null;
  const entries = page?.entries ?? [];
  // The entry hovered or focused last tick: the detail's. Read outside the pattern's element (a memory key is the open element's).
  const lastFocus = kit.memory(`~st.f.${id}`, -1);
  let focus = -1;
  const confirming = (desc.confirmSeconds ?? 0) > 0;
  const pending = !!desc.pending;
  const editing = !!page?.hud && kit.hudEdit().on;

  if (!confirming && !editing) {
    const step = (kit.action("tabNext") ? 1 : 0) - (kit.action("tabPrev") ? 1 : 0);
    if (step !== 0 && pages.length > 1) {
      const i = Math.max(0, pages.findIndex((p) => p.key === page?.key));
      out.page = pages[(i + step + pages.length) % pages.length]!.key ?? null;
      kit.feedback("tab", -1);
    }
  }

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 10 : 16), part: "Settings.frame", partClass: "panel" });

  // The tabs: a lit underline under the page shown (chips that scroll on a phone).
  kit.openScroll("Tabs", { width: grow(), gap: 0, padding: xy(0, 0), axes: "horizontal" } as never);
  kit.open("Row", { direction: "row", gap: u(portrait ? 6 : 8), cross: "center" });
  for (const p of pages) {
    const k = p.key ?? "";
    const on = !!page && page.key === k;
    const r = surface(k, {
      part: `Settings.tab${flags(on && "selected")}`,
      cls: portrait ? "chip" : "listItem",
      fill: portrait ? (on ? { ...role("primary"), a: 0.24 } : { ...role("shadow"), a: 0.55 }) : null,
      washBy: 0.12,
      direction: "column",
      gap: u(6),
      align: "center",
      cross: "center",
      ...(portrait ? { height: { kind: "fit", min: 44 } as Api.Size } : {}),
      padding: xy(u(portrait ? 12 : 16), u(portrait ? 8 : 6)),
      radius: portrait ? 999 : Math.max(4, t.small),
      semantics: { role: "tab", name: p.label ?? k, selected: on },
    }, () => {
      text("Label", p.label ?? k, { uppercase: true, step: st(portrait ? 13 : 15), font: "display", nowrap: true, color: on ? "primary" : "inkMuted" });
      if (!portrait) {
        kit.open("Line", { width: grow(), height: px(Math.max(2, u(3))) });
        kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: on ? token("primary") : token("ink", 0) }], ...(on ? { effects: [glow(u(6), token("primary", 0.5), 0)] } : {}) });
        kit.close();
      }
    });
    if (r.clicked && !on) {
      out.page = k;
      kit.feedback("tab", -1);
    }
  }
  kit.close();
  kit.close();
  kit.semantics({ role: "tablist", name: desc.title ?? "Settings" });

  // An entry's row: its name (a changed one marked), its description under it on a phone, its control at the end.
  const ctlW = px(Math.round(Math.min(u(phone ? 170 : 360), W * (phone ? 0.46 : 0.3)) * Math.min(1.2, ts)));
  const row = (e: Api.SettingEntry, i: number): void => {
    const k = e.key ?? "";
    const binding = e.kind === "binding";
    const ix = lastInteraction(`${k}~row`);
    kit.open(k, { direction: binding || (portrait && e.kind === "slider") ? "column" : "row", width: grow(), gap: u(binding ? 4 : 12), cross: binding ? "stretch" : "center", padding: xy(u(phone ? 12 : 18), u(phone ? 8 : 10)), part: `Settings.row${flags(!!e.dirty && "changed", !!e.disabled && "disabled")}`, partClass: "listItem" });
    const rr = Math.max(4, t.small);
    kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: i === lastFocus ? token("primary", 0.12) : token("panel", 0.6) }], strokes: [line(1, i === lastFocus ? token("primary", 0.6) : token("ink", 0.08))] });
    if (i === lastFocus) {
      kit.open("Edge", { width: px(u(3)), height: grow() });
      kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("primary") }], effects: [glow(u(6), token("primary", 0.5), 0)] });
      kit.close();
      kit.props({ position: "absolute", insets: { start: 0, top: u(6), bottom: u(6) }, inert: true });
    }
    if (!binding) {
      kit.column("Words", { width: grow(), gap: u(2) }, () => {
        kit.row("Name", { gap: u(8), cross: "center", width: grow() }, () => {
          text("Text", e.label ?? k, { uppercase: true, step: st(phone ? 14 : 15), font: "display", width: grow(), maxLines: 2, color: e.disabled ? "inkFaint" : "ink" });
          if (e.dirty) {
            kit.open("Changed", { width: px(u(8)), height: px(u(8)) });
            kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("highlight") }] });
            kit.close();
          }
        });
        if (portrait && e.description) text("Description", e.description, { step: st(12), color: "inkMuted", width: grow(), maxLines: 3 });
      });
    }
    const name = [e.label ?? k, e.dirty ? desc.changedLabel ?? "changed" : ""].filter(Boolean).join(", ");
    const ch = (c: Partial<Api.SettingChange>): void => void (out.change = { key: k, on: !!e.on, value: e.value ?? 0, index: e.index ?? 0, ...c });
    kit.row("Control", { width: binding || (portrait && e.kind === "slider") ? grow() : e.kind === "toggle" || e.kind === "action" ? null : ctlW, gap: u(8), cross: "center", align: "end" }, () => {
      if (e.kind === "toggle") {
        const r = toggleSwitch("Switch", !!e.on, { label: e.label ?? k, side: "end", disabled: !!e.disabled, ...(phone ? {} : { onLabel: "", offLabel: "" }) } as never);
        if (r.changed) {
          ch({ on: r.on });
          kit.feedback("toggle", -1);
        }
      } else if (e.kind === "slider") {
        const r = valueSlider("Slider", e.value ?? 0, { min: e.min ?? 0, max: e.max ?? 1, step: e.step ?? 0, disabled: !!e.disabled, width: grow(), format: e.unit === "%" ? { percent: true } : { unit: e.unit ?? "" } });
        if (r.changed) ch({ value: r.value });
      } else if (e.kind === "choice") {
        const opts = e.options ?? [];
        const r = opts.length <= 3 && !portrait
          ? segmented("Choice", { items: opts.map((label) => ({ label })), selected: e.index ?? 0, width: grow(), label: e.label ?? k, disabled: !!e.disabled })
          : dropdown(`Choice-${k}`, { options: opts.map((label) => ({ label })), selected: e.index ?? 0, width: grow(), label: "", disabled: !!e.disabled, searchable: false });
        if (r.changed && r.selected >= 0) ch({ index: r.selected });
      } else if (e.kind === "binding") {
        const r = keyBindingRow(world, "Binding", { action: e.action ?? "", label: e.label ?? "", slots: phone ? 1 : 2 });
        if (r.changed) out.bindingChanged = true;
      } else if (e.kind === "action") {
        if (btn("Button", e.label ?? k, { variant: "secondary", disabled: !!e.disabled }).clicked) out.action = k;
      } else custom(k);
    });
    kit.close();
    kit.semantics({ role: "group", name });
    const r = kit.bounds(kit.lastPath());
    const p = pointer();
    const inside = !!r && p.inside && p.x >= r.x && p.x < r.x + r.width && p.y >= r.y && p.y < r.y + r.height;
    if (inside || ix.focused) focus = i;
  };

  // The HUD page: its line, Edit HUD, and the B2 editor itself while editing.
  const hudPage = (): void => {
    kit.column("Hud", { width: grow(), gap: u(14), padding: xy(u(18), u(16)) }, () => {
      text("Note", desc.hudNote ?? "Move, resize and fade the HUD's elements over the game.", { step: st(16), color: "inkMuted", width: grow() });
      if (!editing) {
        if (btn("Edit", desc.editHudLabel ?? "Edit HUD", { variant: "primary", size: "lg" }).clicked) {
          kit.setHudEdit(true);
          kit.feedback("confirm", -1);
        }
      } else {
        const r = kit.hudLayoutEditor("Editor", {});
        if (r.done) out.hudDone = true;
      }
    });
  };

  kit.row("Body", { width: grow(), height: grow(), gap: u(16) }, () => {
    kit.column("Page", { width: grow(), height: grow(), gap: 0, part: "Settings.page", partClass: "panel" }, () => {
      if (page?.hud && entries.length === 0) hudPage();
      else {
        kit.openScroll("Rows", { width: grow(), height: grow(), gap: u(6), padding: xy(u(2), u(2)), fade: 24, fadeInto: "shadow" } as never);
        let section = "";
        entries.forEach((e, i) => {
          if (e.section && e.section !== section) {
            kit.row(`${e.key ?? i}~Section`, { width: grow(), padding: { left: u(6), right: u(6), top: u(i === 0 ? 2 : 12), bottom: u(2) }, part: "Settings.section", partClass: "badge" }, () => eyebrow("Text", e.section!));
          }
          section = e.section ?? "";
          row(e, i);
        });
        kit.close();
      }
    });
    // The detail: the hovered or focused entry's name and description (a phone shows it under the name).
    if (!portrait) {
      // Before anything is hovered or focused, the page's first entry: the panel always says what a row does.
      const e = entries[lastFocus] ?? entries[0] ?? null;
      // It scrolls: a long description and a preview at large text may be taller than the room beside the rows.
      kit.openScroll("Detail", { width: px(Math.round(Math.min(u(360), W * 0.26) * Math.min(1.25, ts))), height: grow(), gap: u(12), padding: xy(u(20), u(20)), part: "Settings.detail", partClass: "panel" } as never);
      {
        kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panel", 0.82) }], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(6), u(22), 0.45)] });
        if (e) {
          eyebrow("Title", e.label ?? "");
          if (e.description) text("Description", e.description, { step: st(16), color: "inkMuted", width: grow() });
          if (e.preview) artSlot("Preview", "Settings.preview", { texture: e.preview, kind: "banner", width: grow(), aspect: 16 / 9 });
          if (e.display) kit.row("Display", { gap: u(8), cross: "center" }, () => {
            symbol("Mark", "eye", u(14), role("highlight"));
            text("Text", desc.confirmTitle ?? "Keep these display settings?", { step: st(13), color: "highlight", width: grow() });
          });
        } else eyebrow("Title", page?.label ?? "");
      }
      kit.close();
      kit.semantics({ role: "region", name: e?.label ?? page?.label ?? "" });
    }
  });

  // The foot: Defaults, Revert and Apply.
  // On a phone with large text the three no longer fit one row: Defaults takes a row of its own above Revert and Apply.
  const footStack = portrait && ts > 1.25;
  if (!editing) kit.row("Foot", { width: grow(), gap: u(10), cross: "center", align: "end", wrap: footStack, part: "Settings.foot", partClass: "panel" }, () => {
    const w = portrait ? grow() : null;
    if (btn("Defaults", desc.defaultsLabel ?? "Defaults", { variant: "quiet", width: footStack ? pct(1) : w, sym: "rotate" }).clicked) out.defaults = true;
    if (btn("Revert", desc.revertLabel ?? "Revert", { variant: "secondary", width: w, disabled: !pending }).clicked) {
      out.revert = true;
      kit.feedback("back", -1);
    }
    if (btn("Apply", desc.applyLabel ?? "Apply", { variant: "primary", width: w, disabled: !pending, raised: pending }).clicked) {
      out.apply = true;
      kit.feedback("confirm", -1);
    }
  });

  // The confirm-or-revert dialog over everything, its seconds on a ring.
  if (confirming) {
    const left = desc.confirmSeconds ?? 0;
    const total = Math.max(left, desc.confirmTotal ?? 15);
    if (kit.back()) out.confirmRevert = true;
    kit.column("Scrim", { width: grow(), height: grow(), align: "center", cross: "center" }, () => {
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.6) }] });
      kit.column("Confirm", { width: px(Math.round(Math.min(u(480) * Math.min(1.3, ts), W - u(24)))), gap: u(16), cross: "center", padding: xy(u(24), u(22)), part: "Settings.confirm", partClass: "panel" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("panelRaised", 0.98) }], strokes: [line(1, token("primary", 0.5))], effects: [shadow(u(10), u(36), 0.6)] });
        text("Title", desc.confirmTitle ?? "Keep these display settings?", { step: st(20), font: "display", heading: true, align: "center", width: grow() });
        timerRing("Ring", left / total, u(72), String(Math.ceil(left)));
        text("Line", (desc.revertingLabel ?? "Reverting in {n} s").replace("{n}", String(Math.ceil(left))), { step: st(15), color: "inkMuted", align: "center", width: grow() });
        kit.row("Buttons", { width: grow(), gap: u(10) }, () => {
          if (btn("Revert", desc.revertLabel ?? "Revert", { variant: "secondary", width: grow() }).clicked) out.confirmRevert = true;
          if (btn("Keep", desc.keepLabel ?? "Keep", { variant: "primary", width: grow(), raised: true }).clicked) {
            out.keep = true;
            kit.feedback("confirm", -1);
          }
        });
      });
      kit.semantics({ role: "alertdialog", name: desc.confirmTitle ?? "Keep these display settings?" } as never);
    });
    kit.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, z: 30, overlaps: ["*"] });
    if (out.confirmRevert) kit.feedback("back", -1);
  }

  kit.close();
  rememberRoom("st", id);
  kit.remember(`~st.f.${id}`, focus);
  return out;
}
