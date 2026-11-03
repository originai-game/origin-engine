// The RTS command layer (docs/ui.md §11 genre patterns, command cards and
// control groups; Appendix D: box select, select same type, control
// groups, shift queue). Read against StarCraft II's grid hotkeys and
// Age of Empires IV's command card: og.ui.rtsConsole draws the console
// (main's HUD pieces, not copied here) with the current command page in
// its card; this layer adds what players expect of it: sub-cards opened by
// a command and closed by Back, hotkeys by physical key through UI
// actions, Shift to queue, a targeting prompt, refusals, and control
// groups set, added to, recalled and focused by their keys.

import type * as Api from "@origingame/engine";
import { frame, kit, line, shadow, st, text, token, u, xy, type Mut } from "./core.ts";
import { rtsConsole } from "../hud/rts.ts";

const BACK = "PageBack";

/** Whether a command can fire now: not locked, disabled, unaffordable or cooling down. */
const ready = (c: Api.AbilitySlotDesc): boolean => !c.locked && !c.disabled && !c.unaffordable && !((c.cooldown ?? 0) > 0) && (c.maxCharges ? (c.charges ?? 0) > 0 : true);

export function rtsCommands(id: string, desc: Api.RtsCommandsDesc): Api.RtsCommandsResult {
  const out: Mut<Api.RtsCommandsResult> = {
    command: null, queued: false, refused: null, page: null, cancelTarget: false, recall: null, focus: null, assign: null, add: null,
    clicked: null, selectType: null, deselect: null, mapClicked: false, cancelled: null,
  };
  const { t } = frame();
  const pages = desc.pages ?? [];
  const root = pages[0]?.key ?? "";
  const page = pages.find((p) => p.key === desc.page) ?? pages[0] ?? null;
  const sub = !!page && page.key !== root;
  const targeting = desc.targeting ?? "";
  const mods = kit.framePointer();
  const con = desc.console ?? {};
  const cd = con.commands ?? {};
  const cols = Math.max(1, cd.columns || 5);

  // The page's commands, the one awaiting its target lit; a sub-page ends with Back in its last cell (the grid's foot end).
  const cmds: Api.AbilitySlotDesc[] = (page?.commands ?? []).map((c) => (targeting && c.key === targeting ? { ...c, active: true } : c));
  if (sub) {
    const cells = Math.max(cols * 3, Math.ceil((cmds.length + 1) / cols) * cols);
    while (cmds.length < cells - 1) cmds.push({ key: "" });
    cmds[cells - 1] = { key: BACK, name: desc.backLabel ?? "Back", icon: "back", action: "back" };
  }
  const r = rtsConsole(id, { ...con, commands: { ...cd, commands: cmds } });

  // A command: a page opens; Back returns to the root; one that cannot fire is refused; the rest fire, queued with Shift.
  const fire = (key: string): void => {
    if (key === BACK) {
      out.page = root;
      kit.feedback("back", -1);
      return;
    }
    if (pages.some((p) => p.key === key)) {
      out.page = key;
      kit.feedback("select", -1);
      return;
    }
    const c = cmds.find((x) => x.key === key);
    if (!c) return;
    if (!ready(c)) {
      out.refused = key;
      kit.feedback("error", -1);
      return;
    }
    out.command = key;
    out.queued = mods.shift;
    kit.feedback("confirm", -1);
  };
  if (r.command) fire(r.command);
  else {
    for (const c of cmds) {
      if (c.key && c.key !== BACK && c.action && kit.action(c.action)) {
        fire(c.key);
        break;
      }
    }
  }
  // Back: cancels a targeting command first, then leaves a sub-page.
  if (!r.command && kit.action("back")) {
    if (targeting) {
      out.cancelTarget = true;
      kit.feedback("back", -1);
    } else if (sub) {
      out.page = root;
      kit.feedback("back", -1);
    }
  }

  // The control groups: a key or a tab recalls; Ctrl sets, Shift adds; a second tap within doubleTapMs focuses.
  const groups = con.selection?.groups ?? [];
  // Each group's last tap, read every tick so the UI keeps it (an unread memory is dropped).
  const taps = groups.map((_, i) => kit.memory(`~rc.tap.${i}`, -1e9));
  const recall = (g: string, i: number): void => {
    const now = kit.seconds();
    const last = taps[i] ?? -1e9;
    if ((now - last) * 1000 <= (desc.doubleTapMs ?? 350)) {
      out.focus = g;
      kit.remember(`~rc.tap.${i}`, -1e9);
    } else {
      out.recall = g;
      kit.remember(`~rc.tap.${i}`, now);
    }
    kit.feedback("tab", -1);
  };
  (desc.groupActions ?? []).forEach((a, i) => {
    const g = groups[i]?.key;
    if (!g || !a || !kit.action(a)) return;
    if (mods.ctrl) {
      out.assign = g;
      kit.feedback("confirm", -1);
    } else if (mods.shift) {
      out.add = g;
      kit.feedback("confirm", -1);
    } else recall(g, i);
  });
  if (r.group) recall(r.group, groups.findIndex((g) => g.key === r.group));

  // A unit's tile: Ctrl takes its type, Shift takes it out, a press focuses it.
  if (r.clicked) {
    if (mods.ctrl) out.selectType = r.clicked;
    else if (mods.shift) out.deselect = r.clicked;
    else out.clicked = r.clicked;
  }
  out.mapClicked = r.mapClicked;
  out.cancelled = r.cancelled;

  // The targeting prompt over the console: the command's name, Back to cancel, Shift to queue.
  if (targeting) {
    const c = cmds.find((x) => x.key === targeting);
    const consoleH = kit.bounds(`${id}Hud`)?.height ?? u(180);
    kit.anchor(`${id}Prompt`, { anchor: "bottom", offsetY: -(consoleH + t.sm), direction: "row", gap: u(14), cross: "center", padding: xy(u(18), u(10)) }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("panel", 0.94) }], strokes: [line(u(1.5), token("highlight", 0.7))], effects: [shadow(u(6), u(18), 0.5)] });
      text("Text", (desc.targetLabel ?? "{command}: choose a target").replace("{command}", c?.name ?? targeting), { step: st(16), font: "display", nowrap: true });
      const hint = (k: string, glyph: string, word: string): void => {
        kit.row(k, { gap: u(6), cross: "center" }, () => {
          kit.row("Key", { padding: xy(u(6), u(1)), radius: u(4), background: token("ink", 0.14), cross: "center" }, () => text("Glyph", glyph, { step: st(12), font: "mono", nowrap: true }));
          text("Word", word, { step: st(13), color: "inkMuted", nowrap: true });
        });
      };
      hint("Cancel", kit.frameGlyph("back")?.text ?? "Esc", desc.cancelLabel ?? "Cancel");
      if (!frame().touch) hint("Queue", "Shift", desc.queueLabel ?? "Queue");
    });
    kit.semantics({ role: "status", name: (desc.targetLabel ?? "{command}: choose a target").replace("{command}", c?.name ?? targeting) });
  }
  return out;
}
