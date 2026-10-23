// Story and teaching: the cutscene (letterbox and dialogue box) and the
// onboarding overlay (a spotlight on each step's target and a callout).

import type * as Api from "@origingame/engine";
import { kit } from "./bind.ts";
import { all, frame, grow, pct, px, token } from "./util.ts";

export function cutscene(id: string, desc: Api.CutsceneDesc = {}): Api.CutsceneResult {
  const { t, phone } = frame();
  const lines = desc.lines ?? [];
  let index = Math.min(desc.index ?? 0, Math.max(0, lines.length - 1));
  const out = { index, finished: false, skipped: false };
  const open = (desc.open ?? false) && lines.length > 0;
  if (!open) return out;
  kit.setNavigation(true);
  const letterbox = desc.letterbox ?? true;
  if (letterbox) {
    // Cinematic bars are black in every style: not a token, which a light
    // style (kestrel's paper) would turn into cream panels.
    for (const [name, anchor] of [["Top", "top"], ["Bottom", "bottom"]] as const) {
      kit.anchor(`${id}.${name}`, { anchor, margin: 0, width: grow(), height: pct(0.1), background: { color: { r: 0, g: 0, b: 0 } } }, () => {});
    }
  }
  const line = lines[index]!;
  kit.anchor(`${id}.box`, { anchor: "bottom", width: phone ? grow() : px(Math.min(760, frame().viewport.width - 64)), offsetY: letterbox ? -frame().viewport.height * 0.04 : 0 }, () => {
    const r = kit.dialogue("Dialogue", { speaker: line.speaker ?? "", text: line.text ?? "", portraitIcon: line.portrait ?? "" });
    if (r.advanced) {
      if (index + 1 >= lines.length) out.finished = true;
      else index += 1;
    }
  });
  if (desc.skippable ?? true) {
    kit.anchor(`${id}.skip`, { anchor: "topRight", offsetY: letterbox ? frame().viewport.height * 0.1 : 0 }, () => {
      if ((kit.button("Skip", desc.skipLabel ?? "Skip", { variant: "quiet", size: "small" }) as Api.Response).clicked) out.skipped = true;
    });
    if (kit.back()) out.skipped = true;
  }
  out.index = index;
  return out;
}

export function tutorialOverlay(id: string, desc: Api.TutorialDesc = {}): Api.TutorialResult {
  const { t, viewport } = frame();
  const steps = desc.steps ?? [];
  const index = Math.min(desc.index ?? 0, Math.max(0, steps.length - 1));
  const out = { index, finished: false, skipped: false };
  if (!(desc.open ?? false) || steps.length === 0) return out;
  const step = steps[index]!;
  const hole = step.target ? kit.bounds(step.target) : null;
  kit.spotlight(id, hole, {});
  // The callout beside the hole: under it when there is room, else over it.
  const width = Math.min(320, viewport.width - 2 * t.gutter);
  const last = kit.bounds(`${id}.callout`);
  const height = last?.height ?? 160;
  const safe = viewport.safeArea;
  let x = viewport.width / 2 - width / 2;
  let y = viewport.height / 2 - height / 2;
  if (hole) {
    x = Math.min(Math.max(hole.x, safe.left + t.gutter), viewport.width - safe.right - t.gutter - width);
    const below = hole.y + hole.height + t.lg;
    y = below + height <= viewport.height - safe.bottom - t.gutter ? below : Math.max(safe.top + t.gutter, hole.y - t.lg - height);
  }
  kit.anchor(`${id}.callout`, {
    anchor: "topLeft",
    over: true,
    margin: 0,
    offsetX: x - safe.left,
    offsetY: y - safe.top,
    width: px(width),
    direction: "column",
    gap: t.sm,
    padding: all(t.lg),
    background: token("panelRaised"),
    radius: t.panel,
  }, () => {
    const count = (desc.stepLabel ?? "{step} of {count}").replaceAll("{step}", `${index + 1}`).replaceAll("{count}", `${steps.length}`);
    kit.label("Count", count, { step: "meta", color: token("inkMuted") });
    if (step.title) kit.label("Title", step.title, { step: "lg", width: grow() });
    if (step.text) kit.label("Text", step.text, { step: "reading", color: token("inkMuted"), width: grow() });
    kit.row("Actions", { width: grow(), gap: t.sm, align: "spaceBetween", cross: "center" }, () => {
      if ((kit.button("Skip", desc.skipLabel ?? "Skip", { variant: "quiet", size: "small" }) as Api.Response).clicked) out.skipped = true;
      const lastStep = index + 1 >= steps.length;
      if ((kit.button("Next", lastStep ? (desc.doneLabel ?? "Done") : (desc.nextLabel ?? "Next"), { variant: "primary", size: "small" }) as Api.Response).clicked) {
        if (lastStep) out.finished = true;
        else out.index = index + 1;
      }
    });
  });
  if (kit.back()) out.skipped = true;
  return out;
}
