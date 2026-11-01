// The key-binding row (docs/ui.md §11, the settings tree's controls page):
// an action's bindings on a device, shown as og.input's glyphs, each slot
// captured with og.input's startCapture/capture/applyCapture, a conflict
// answered inline with og.input's conflict report (swap, replace, cancel).

import type * as Api from "@origingame/engine";
import { input } from "../../input.ts";
import {
  byText, clamp, control, drawn, fill, flags, frame, glyphCap, grow, kit, mix, paint, px, role, still, text, TOUCH, xy,
} from "./base.ts";
import { iconButton, pushButton } from "./buttons.ts";

type World = Parameters<typeof input.capture>[0];

/** Whether a control is a pad's (not a key, the mouse or a touch). */
export function isPadControl(c: string): boolean {
  return c.startsWith("pad.");
}

/** A binding's controls, a composite's parts in prompt order (up, left, down, right; negative, positive). */
export function bindingControls(b: Api.InputBinding): string[] {
  if (b.control) return [b.control];
  const parts = [b.up, b.left, b.down, b.right, b.negative, b.positive].filter((x): x is string => typeof x === "string" && x.length > 0);
  return parts;
}

/** The slots shown on a device: each binding there (its index among all the action's), then empty slots that add one. */
export function deviceSlots(bindings: readonly Api.InputBinding[], pad: boolean, count: number): Array<{ slot: number; binding: Api.InputBinding | null }> {
  const mine = bindings.map((b, i) => ({ b, i })).filter(({ b }) => {
    const cs = bindingControls(b);
    return cs.length > 0 && cs.every((c) => isPadControl(c) === pad);
  });
  const out: Array<{ slot: number; binding: Api.InputBinding | null }> = mine.slice(0, count).map(({ b, i }) => ({ slot: i, binding: b }));
  while (out.length < count) out.push({ slot: bindings.length, binding: null });
  return out;
}

/** A device's family for glyphs: a pad family, else the keyboard and mouse. */
function family(d: Api.InputDevice): Api.InputDevice {
  return d === "touch" ? "keyboardMouse" : d;
}

export function keyBindingRow(world: Api.World, id: string, desc: Api.KeyBindingRowDesc): Api.KeyBindingResult {
  const w = world as unknown as World;
  const { t } = frame();
  const action = desc.action ?? "";
  const infos = input.actions(w) as Api.ActionInfo[];
  const info = infos.find((a) => a.name === action);
  const device = family((desc.device ?? (input.device(w) as Api.InputDevice)) as Api.InputDevice);
  const pad = device !== "keyboardMouse";
  const count = clamp(desc.slots ?? 2, 1, 4);
  const cap = input.capture(w) as Api.CaptureState;
  const ours = (cap.active || cap.done) && cap.action === action;
  const capturing = ours && cap.active;
  const conflict = ours && cap.done && cap.conflicts.length > 0;
  let changed = false;
  let slotHit = -1;
  const locked = info?.locked ?? false;
  const label = desc.label || info?.label || action;
  const slots = deviceSlots(info?.bindings ?? [], pad, count);
  const h = byText(Math.max(t.controlMd, frame().touch ? TOUCH : 0));
  const labelOf = (name: string) => infos.find((a) => a.name === name)?.label || name;
  kit.open(id, {
    direction: "column",
    width: grow(),
    padding: xy(t.sm, t.xs),
    gap: t.xs,
    radius: t.control,
    background: capturing || conflict ? paint(mix(role("panel"), role(conflict ? "warn" : "primary"), 0.08)) : null,
    part: `Rebind.row${flags(capturing && "capturing", conflict && "conflict", locked && "locked")}`,
    partClass: "panel",
  });
  try {
    kit.open("Main", { direction: "row", width: grow(), gap: t.sm, cross: "center" });
    text("Label", label, { step: "body", color: locked ? "inkMuted" : "ink", width: grow(), maxLines: 2 });
    if (locked) drawn("Lock", "lock", byText(16), role("inkMuted"));
    kit.open("Slots", { direction: "row", gap: t.xs, cross: "center" });
    slots.forEach((sl, i) => {
      const waiting = capturing && cap.slot === sl.slot;
      const pending = conflict && cap.slot === sl.slot;
      const controls = pending ? [cap.control] : sl.binding ? bindingControls(sl.binding) : [];
      const glyphs = controls.map((c) => input.controlGlyph(c, pad ? device : "keyboardMouse") as Api.InputGlyph | null).filter((g): g is Api.InputGlyph => !!g);
      const names = glyphs.map((g) => g.text).join(" ");
      const b = control(kit.idi("Slot", i), {
        part: "Rebind.slot",
        cls: "field",
        flags: [!sl.binding && !waiting && "empty", waiting && "capturing", pending && "conflict"],
        disabled: locked,
        direction: "row",
        width: { kind: "fit", min: byText(pad ? 72 : 120) },
        height: px(h),
        padding: xy(t.sm, 0),
        gap: t.xxs,
        align: "center",
        fill: waiting ? mix(role("well"), role("primary"), 0.18) : role("well"),
        washBy: 0.06,
        lift: 0,
        press: 0.98,
        semantics: { role: "button", name: `${label}, ${i + 1}: ${waiting ? "waiting" : names || desc.emptyLabel || "unbound"}` },
      }, () => {
        kit.setDraw({ strokes: [{ width: waiting || pending ? 2 : 1, align: "inside", paint: { kind: "solid", color: { token: pending ? "warn" : waiting ? "focus" : "line" } } }] });
        if (waiting) {
          // A slow breath while it waits (still under reduced motion), and the cancel hint.
          const k = still() ? 1 : 0.65 + 0.35 * (0.5 + 0.5 * Math.cos(kit.seconds() * Math.PI * 1.6));
          text("Waiting", desc.waitingLabel || (pad ? "Press a button…" : "Press a key…"), { step: "sm", color: "ink", nowrap: true });
          kit.props({ opacity: k });
        } else if (glyphs.length > 0) {
          glyphs.forEach((g, k) => glyphCap(kit.idi("Cap", k), g, Math.round(h * 0.62)));
        } else text("Empty", desc.emptyLabel || "—", { step: "body", color: "inkFaint", nowrap: true });
      });
      if (b.r.clicked && !locked) slotHit = sl.slot;
    });
    kit.close();
    if ((desc.reset ?? true) && info?.rebound && !locked) {
      if (iconButton("Reset", "reset", { label: desc.resetLabel || "Reset to default", size: "small" }).clicked) {
        input.resetBindings(w, action);
        if (ours) input.cancelCapture(w);
        changed = true;
        kit.feedback("back", -1);
      }
    } else kit.spacer("Reset", px(byText(frame().t.controlSm)));
    kit.close();
    if (capturing) {
      kit.open("Hint", { direction: "row", width: grow(), gap: t.xs, align: "end", cross: "center" });
      const g = kit.frameGlyph("back") as { text: string; icon: string } | null;
      if (g) glyphCap("Glyph", g, byText(20));
      text("Text", desc.cancelHint || (g ? "to cancel" : "Esc to cancel"), { step: "caption", color: "inkMuted", nowrap: true });
      kit.close();
    }
    if (conflict) {
      const c = cap.conflicts[0]!;
      const ctl = input.controlGlyph(cap.control, pad ? device : "keyboardMouse") as Api.InputGlyph | null;
      kit.open("Conflict", { direction: "row", width: grow(), gap: t.sm, cross: "center", wrap: true, part: "Rebind.conflict", partClass: "panel" });
      drawn("Icon", "warning", byText(16), role("warn"));
      text("Text", fill(desc.conflictLabel || "{control} is used by {action}", { control: ctl?.text ?? cap.control, action: labelOf(c.action) }), { step: "sm", color: "ink", width: grow() });
      kit.semantics({ role: "alert" });
      const swap = pushButton("Swap", desc.swapLabel || "Swap", { size: "small", variant: "primary", autofocus: true });
      const rep = pushButton("Replace", desc.replaceLabel || "Replace", { size: "small", variant: "secondary" });
      const can = pushButton("Cancel", desc.cancelLabel || "Cancel", { size: "small", variant: "quiet" });
      kit.close();
      if (swap.clicked || rep.clicked) {
        const r = input.applyCapture(w, swap.clicked ? "swap" : "replace") as { ok: boolean; value?: Api.RebindAnswer };
        changed = r.ok !== false;
        kit.feedback("confirm", -1);
      } else if (can.clicked) {
        input.cancelCapture(w);
        kit.feedback("cancel", -1);
      }
    }
  } finally {
    kit.close();
  }
  kit.semantics({ role: "group", name: label });
  // A capture with nothing in its way lands at once.
  if (ours && cap.done && cap.conflicts.length === 0) {
    input.applyCapture(w);
    changed = true;
    kit.feedback("confirm", -1);
  }
  if (slotHit >= 0) {
    if (cap.active || cap.done) input.cancelCapture(w);
    const r = input.startCapture(w, { action, slot: slotHit, accept: pad ? "gamepad" : "keyboardMouse", timeoutMs: desc.timeoutMs ?? 8000 }) as { ok?: boolean; error?: string };
    kit.feedback(r && r.ok === false ? "error" : "select", -1);
  }
  // While it waits, keys and pads are the capture's, not the focus's: navigation stops and comes back after.
  const was = kit.memory(`${id}~nav`, 0) > 0;
  const now = capturing || slotHit >= 0;
  if (now !== was) kit.setNavigation(!now);
  if (was && !now && !changed && !conflict) kit.feedback("cancel", -1);
  kit.remember(`${id}~nav`, now ? 1 : 0);
  return { capturing: now, changed, conflict, slot: capturing || conflict ? cap.slot : -1 };
}
