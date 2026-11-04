// Foundations (docs/ui.md §11): the icon, the image and art slot, the
// decorative shape and the framed viewport. Each draws from the style: an
// icon the set lacks is a drawn mark, art not yet there is a placeholder of
// the style's soft fills and the kind's silhouette, never a grey box.

import type * as Api from "@origingame/engine";
import {
  byText, clamp, control, drawn, flags, frame, grow, hasMark, inkOn, kit, mix, over, paint, px, role, still, text, token, xy,
} from "../controls/base.ts";
import { toneColor } from "./text.ts";
import { FIGURE_KINDS, placeholderArt } from "../patterns/core.ts";

// ── icon ────────────────────────────────────────────────────────────────

/** A count as a badge says it: past 99, 99+. */
export function badgeText(b: string): string {
  return /^\d+$/.test(b) && Number(b) > 99 ? "99+" : b;
}

/** A badge (a count, a word) or a dot over the top end corner of the element being built. */
export function cornerBadge(id: string, badge: string, dot: boolean, size: number): void {
  if (badge) {
    // With the text: at 200 % the count is twice as tall and so is its pill, which is never shorter than the count's
    // line (sized from the icon alone, a style whose small step is tall stood the count out of its pill).
    const reach = byText(Math.max(16, Math.round(size * 0.62)));
    const h = Math.max(reach, Math.ceil(frame().theme.typeRamp["xs"]?.lineHeight ?? 0));
    kit.open(id, {
      direction: "row",
      height: px(h),
      width: { kind: "fit", min: h },
      padding: xy(Math.round(h * 0.3), 0),
      align: "center",
      cross: "center",
      radius: h / 2,
      background: token("danger"),
      part: "Icon.badge",
      partClass: "badge",
    });
    // The count in the ink that reads on the pill: onPrimary is the primary's ink, dark on a light style's danger red.
    text("Text", badgeText(badge), { step: "xs", color: inkOn(role("danger")), nowrap: true, font: "mono" });
    kit.close();
    // It overhangs its host's corner by the icon's measure: a pill grown for its count grows into its host, not out
    // over the control beside it.
    over({ insets: { top: -Math.round(reach * 0.35), end: -Math.round(reach * 0.35) }, z: 4, inert: true });
    kit.semantics({ role: "status", name: badgeText(badge) });
  } else if (dot) {
    const d = Math.max(8, Math.round(size * 0.3));
    kit.open(id, { width: px(d), height: px(d), radius: d / 2, background: token("danger"), part: "Icon.dot", partClass: "badge" });
    kit.close();
    over({ insets: { top: -1, end: -1 }, z: 4, inert: true });
  }
}

export function iconMark(id: string, name: string, desc: Api.IconMarkDesc = {}): void {
  const { t } = frame();
  const size = desc.size && desc.size > 0 ? desc.size : 24;
  const ink = role(toneColor(desc.tone));
  const plate = desc.plate ?? "none";
  const box = plate === "none" ? size : Math.round(size * 1.75);
  kit.open(id, {
    direction: "row",
    width: px(box),
    height: px(box),
    align: "center",
    cross: "center",
    radius: plate === "circle" ? box / 2 : t.control,
    background: plate === "none" ? null : paint(mix(role("panelRaised"), ink, 0.16)),
    part: `Icon.plate${flags(plate)}`,
    partClass: "chip",
  });
  try {
    if (kit.hasIcon(name)) kit.icon("Glyph", name, { size, tint: paint(ink), label: "" });
    else drawn("Glyph", hasMark(name) ? name : "dot", size, ink);
    cornerBadge("Badge", desc.badge ?? "", desc.dot ?? false, size);
  } finally {
    kit.close();
  }
  const name1 = desc.label === undefined || desc.label === null ? name : desc.label;
  if (name1) kit.semantics({ role: "image", name: name1 });
}

// ── art slot ────────────────────────────────────────────────────────────

function maskShape(mask: Api.ArtMask | undefined, radius: number): Api.DrawShape {
  switch (mask ?? "round") {
    case "circle": return { kind: "ellipse" };
    case "squircle": return { kind: "rect", corner: "squircle", radii: [radius, radius, radius, radius] };
    case "none": return { kind: "rect", radii: [0, 0, 0, 0] };
    default: return { kind: "rect", radii: [radius, radius, radius, radius] };
  }
}

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

export function artSlot(id: string, desc: Api.ArtSlotDesc = {}): void {
  const tex = desc.texture ?? null;
  const loading = desc.loading ?? false;
  const aspect = desc.aspect && desc.aspect > 0 ? desc.aspect : 1;
  const radius = desc.radius ?? slotRadius();
  const shape = maskShape(desc.mask, radius);
  kit.open(id, {
    direction: "row",
    width: desc.width ?? grow(),
    height: desc.height ?? null,
    aspect: desc.height ? 0 : aspect,
    align: "center",
    cross: "center",
    radius: desc.mask === "circle" ? 9999 : radius,
    part: `Art.frame${flags(!tex && "placeholder", loading && "loading")}`,
    // A body stands on whatever the slot sits on (a paperdoll's ground): no slot dressing round it.
    ...(desc.kind === "body" && !tex ? {} : { partClass: "slot" as const }),
  });
  try {
    const strokes: Api.Stroke[] = [{ width: 1, align: "inside", paint: { kind: "solid", color: token(desc.frame ?? "line", desc.frame ? 1 : 0.8) } }];
    // A portrait, a character or key art waits as a figure on a wash of two hues, its head at the focal point (§13.1).
    if (!tex && FIGURE_KINDS.has(desc.kind ?? "")) {
      const r = desc.mask === "circle" ? 9999 : desc.mask === "none" ? 0 : radius;
      placeholderArt({ kind: desc.kind, focal: desc.focal ?? undefined, aspect, width: desc.width, height: desc.height }, null, [r, r, r, r], strokes);
      if (loading) sheen("Sheen");
    } else {
      kit.setDraw({
        shape,
        clip: "shape",
        fills: tex ? [] : [
          { kind: "gradient", type: "linear", from: { x: 0, y: 0 }, to: { x: 1, y: 1 }, stops: [{ offset: 0, color: token("primarySoft") }, { offset: 1, color: token("well") }] },
          { kind: "gradient", type: "radial", from: { x: 0.5, y: 0.38 }, to: { x: 0.7, y: 0.7 }, stops: [{ offset: 0, color: token("sheen", 0.4) }, { offset: 1, color: token("sheen", 0) }] },
        ],
        strokes,
      } as Api.DrawDesc);
      if (tex) {
        kit.open("Image", { width: grow(), height: grow(), part: "Art.image", partClass: "slot" });
        kit.setDraw({ shape, clip: "shape", fills: [{ kind: "image", texture: tex, fit: desc.fit ?? "cover", focal: desc.focal ?? { x: 0.5, y: 0.5 } }] } as Api.DrawDesc);
        kit.close();
        // It fades in when it arrives (the style's enter motion), at once under reduced motion.
        kit.props({ enter: { values: [{ prop: "opacity", value: 0 }], transition: { kind: "token", token: "enter" } }, reduced: "fade" });
      } else {
        const ph = Math.min(48, Math.max(16, Math.round(((kit.bounds(kit.lastPath() || id)?.width ?? 96) * 0.28))));
        kit.open("Placeholder", { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", part: "Art.placeholder", partClass: "slot" });
        const icon = desc.kind && kit.hasIcon(desc.kind) ? desc.kind : "image";
        if (kit.hasIcon(icon)) kit.icon("Kind", icon, { size: ph, tint: token("inkFaint"), label: "" });
        else drawn("Kind", "star", ph, role("inkFaint"));
        kit.close();
        if (loading) sheen("Sheen");
      }
    }
  } finally {
    kit.close();
  }
  if (desc.label) kit.semantics({ role: "image", name: desc.label, busy: loading && !tex });
}

// ── shape ───────────────────────────────────────────────────────────────

export function shapeBox(id: string, desc: Api.ShapeBoxDesc = {}): void {
  const w = desc.width ?? px(24);
  const h = desc.height ?? w;
  const kind = desc.kind ?? "rect";
  const thick = desc.thickness ?? 2;
  const rot = ((desc.rotation ?? 0) * Math.PI) / 180;
  let shape: Api.DrawShape;
  switch (kind) {
    case "capsule": shape = { kind: "rect", radii: [9999, 9999, 9999, 9999] }; break;
    case "ellipse": shape = { kind: "ellipse" }; break;
    case "polygon": shape = { kind: "polygon", sides: desc.sides ?? 5, inner: 1, rotation: rot }; break;
    case "star": shape = { kind: "polygon", sides: desc.sides ?? 5, inner: 0.45, rounding: 0.08, rotation: rot }; break;
    case "ring": shape = { kind: "arc", thickness: thick, start: 0, sweep: Math.PI * 2 }; break;
    case "diamond": shape = { kind: "polygon", sides: 4, inner: 1, rotation: rot }; break;
    case "line": shape = { kind: "rect", radii: [thick / 2, thick / 2, thick / 2, thick / 2] }; break;
    default: shape = { kind: "rect", radii: [desc.radius ?? 0, desc.radius ?? 0, desc.radius ?? 0, desc.radius ?? 0] };
  }
  const from = desc.fill ?? "primary";
  const a = ((desc.angle ?? 0) * Math.PI) / 180;
  const fills: Api.Fill[] = desc.fillTo
    ? [{ kind: "gradient", type: "linear", from: { x: 0.5 - Math.sin(a) / 2, y: 0.5 - Math.cos(a) / 2 }, to: { x: 0.5 + Math.sin(a) / 2, y: 0.5 + Math.cos(a) / 2 }, stops: [{ offset: 0, color: token(from) }, { offset: 1, color: token(desc.fillTo) }] }]
    : [{ kind: "solid", color: token(from) }];
  const strokes: Api.Stroke[] = desc.stroke ? [{ width: desc.strokeWidth && desc.strokeWidth > 0 ? desc.strokeWidth : 1, align: "inside", paint: { kind: "solid", color: token(desc.stroke) } }] : [];
  const effects: Api.Effect[] = desc.glow ? [{ kind: "outerGlow", size: 8, color: token(desc.glow, 0.6) } as Api.Effect] : [];
  kit.open(id, { width: w, height: kind === "line" ? px(thick) : h, part: `Shape.${kind}`, partClass: "badge" });
  kit.setDraw({ shape, fills, strokes, effects, opacity: desc.opacity ?? 1 });
  kit.close();
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

