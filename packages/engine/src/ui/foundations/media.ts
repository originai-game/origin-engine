// Foundations (docs/ui.md §2): the icon, the image and art slot, the
// decorative shape and the framed viewport. Each draws from the style: an
// icon the set lacks is a drawn mark, art not yet there is a placeholder of
// the style's soft fills and the kind's silhouette, never a grey box.

import type * as Api from "@origingame/engine";
import {
  clamp, control, drawn, frame, grow, kit, over, role, still, text, token, xy,
} from "../controls/base.ts";

// ── art slot ────────────────────────────────────────────────────────────

/** The style's slot radius (its shape language for slots). */
export function slotRadius(): number {
  const shape = (frame().theme.tokens as { shape?: Record<string, { radius?: number }> } | null)?.shape;
  return Math.min(16, shape?.slot?.radius ?? frame().t.control);
}

/** A sheen passing across a placeholder while it loads: by the UI clock, still under reduced motion. */
function sheen(id: string): void {
  if (still()) return;
  const k = (kit.seconds() % 1.6) / 1.6;
  kit.open(id, { width: grow(), height: grow() });
  kit.setDraw({
    fills: [{
      kind: "gradient",
      type: "linear",
      from: { x: k * 1.6 - 0.6, y: 0 },
      to: { x: k * 1.6 - 0.1, y: 1 },
      stops: [{ offset: 0, color: token("sheen", 0) }, { offset: 0.5, color: token("sheen", 0.35) }, { offset: 1, color: token("sheen", 0) }],
    }],
  });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
}

// ── viewport ────────────────────────────────────────────────────────────

export function viewFrame(id: string, desc: Api.ViewFrameDesc = {}): Api.ViewFrameResult {
  const { t } = frame();
  const orbit = desc.orbit ?? true;
  const minZ = desc.minZoom ?? 0.6;
  const maxZ = desc.maxZoom ?? 2.5;
  const minP = desc.minPitch ?? -30;
  const maxP = desc.maxPitch ?? 60;
  let yaw = desc.yaw ?? 0;
  let pitch = desc.pitch ?? 0;
  let zoom = desc.zoom ?? 1;
  let info: Api.SceneViewportInfo | null = null;
  const aspect = desc.aspect && desc.aspect > 0 ? desc.aspect : 1;
  // Turned once: the hint has done its job.
  const turned = kit.memory(`${id}~moved`, 0) > 0;
  const built = control(id, {
    part: "ViewFrame.frame",
    cls: "slot",
    direction: "column",
    width: desc.width ?? grow(),
    height: desc.height ?? null,
    fill: role("well"),
    wash: null,
    lift: 0,
    press: 1,
    adjustable: orbit,
    radius: slotRadius(),
    semantics: { role: "image", name: desc.label || "View", value: `${Math.round(((yaw % 360) + 360) % 360)}°` },
  }, () => {
    kit.open("Box", { direction: "column", width: grow(), aspect: desc.height ? 0 : aspect, height: desc.height ? grow() : null, align: "center", cross: "center" });
    if (desc.texture) {
      // The target covers the box (cropped at its edges, never squashed): a target drawn at
      // view.pixelWidth × view.pixelHeight fills it exactly, one of another shape (the frame
      // before a resize lands, a still) keeps its proportions.
      const shape: Api.DrawShape = { kind: "rect", radii: [slotRadius(), slotRadius(), slotRadius(), slotRadius()] };
      kit.open("Picture", { direction: "column", width: grow(), height: grow() });
      kit.setDraw({ shape, clip: "shape", fills: [{ kind: "image", texture: desc.texture, fit: "cover", focal: { x: 0.5, y: 0.5 } }] } as Api.DrawDesc);
      info = kit.sceneViewport("View", {
        width: grow(),
        height: grow(),
        label: desc.label ?? "",
        ...(desc.cameraPosition ? { cameraPosition: desc.cameraPosition } : {}),
        ...(desc.cameraTarget ? { cameraTarget: desc.cameraTarget } : {}),
        fov: desc.fov ?? 50,
      }) as Api.SceneViewportInfo;
      kit.close();
    } else {
      kit.open("Placeholder", { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", gap: t.sm });
      kit.spinner("Spinner", 28);
      kit.close();
    }
    if (desc.hint && !turned) {
      // Centred over the bottom: a row across it, the hint in its middle.
      kit.open("HintRow", { direction: "row", width: grow(), align: "center" });
      kit.open("Hint", { direction: "row", padding: xy(t.sm, t.xxs), radius: t.control, background: token("scrim", 0.6), part: "ViewFrame.hint", partClass: "chip" });
      text("Text", desc.hint, { step: "caption", color: "ink", nowrap: true });
      kit.close();
      kit.close();
      over({ insets: { start: 0, end: 0, bottom: t.sm }, z: 2, inert: true });
    }
    kit.close();
  });
  if (orbit) {
    const vel = kit.memory(`${id}~vel`, 0);
    let v = vel;
    const i = info as Api.SceneViewportInfo | null;
    if (i && i.pressed) {
      v = i.drag.x * 0.45;
      yaw += v;
      pitch = clamp(pitch + i.drag.y * 0.3, minP, maxP);
    } else if (!still() && Math.abs(v) > 0.02) {
      // A fling keeps turning, slowing to rest.
      yaw += v;
      v *= 0.88;
    } else v = 0;
    if (i && i.wheel.y !== 0) zoom = clamp(zoom * (1 - i.wheel.y * 0.0015), minZ, maxZ);
    if (built.r.adjust) yaw += 15 * built.r.adjust;
    // A press on the picture (the viewport takes it, for the drag) gives the frame the focus, as a press on its edge does.
    if (i && i.pressed && !built.r.focused) kit.setFocus(built.path);
    kit.remember(`${id}~vel`, v);
    if (yaw !== (desc.yaw ?? 0) || pitch !== (desc.pitch ?? 0)) kit.remember(`${id}~moved`, 1);
  }
  const empty: Api.SceneViewportInfo = { width: 0, height: 0, pixelWidth: 0, pixelHeight: 0, hovered: false, pressed: false, clicked: false, drag: { x: 0, y: 0 }, wheel: { x: 0, y: 0 }, at: null, ray: null };
  return { yaw, pitch, zoom, view: (info as Api.SceneViewportInfo | null) ?? empty };
}
