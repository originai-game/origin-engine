// og.ui.subtitles: subtitles and captions to the Xbox Accessibility
// Guidelines' 104 (docs/ui.md §11, §15.2). Read against The Last of Us
// Part II's and Forza's subtitle settings (size presets, background
// opacity, speaker names in colour, direction arrows) and XAG 104 itself:
// at the bottom centre of the safe area, at most two lines of speech,
// about 42 characters a line, never under the minimum size, on a box of
// the player's opacity or outlined when it is clear, a quick fade in and
// no movement.

import type * as Api from "@origingame/engine";
import { a11y, box, bp, clamp01, frame, hudText, kit, leaf, minText, msToTicks, now, px, scale, sel, solid, token, type Paint, apply } from "./core.ts";

const SIZE: Record<Api.SubtitleSize, number> = { small: 0.022, medium: 0.03, large: 0.038, huge: 0.048 };

/** The ramp step nearest above `px` (subtitles size by the screen's height, as TV-distance text does). */
function stepFor(want: number): Api.TypeStepName {
  const ramp = frame().theme.typeRamp;
  let best: Api.TypeStepName = "xl8";
  let bestSize = Infinity;
  for (const [name, st] of Object.entries(ramp)) {
    if (name.startsWith("display")) continue;
    if (st.size >= want - 0.01 && st.size < bestSize) {
      best = name as Api.TypeStepName;
      bestSize = st.size;
    }
  }
  return best;
}

/** A Paint as rich-text colour markup: a palette role by name, a literal by hex. */
function markupColor(p: Paint | undefined): string {
  if (!p) return "highlight";
  if (p.token) return p.token;
  const c = p.color ?? { r: 1, g: 1, b: 1 };
  const h = (v: number) => Math.round(clamp01(v) * 255).toString(16).padStart(2, "0");
  return `#${h(c.r)}${h(c.g)}${h(c.b)}`;
}

const escape = (s: string): string => s.replace(/\[/g, "[[");

export function subtitles(id: string, desc: Api.SubtitlesDesc = {}): void {
  const f = frame();
  const b = bp();
  const tk = f.t;
  const set: Api.SubtitleSettings = desc.settings ?? {};
  if (set.enabled === false) return;
  const vp = f.viewport;
  const size = Math.max(minText(), vp.height * SIZE[set.size ?? "medium"] * (vp.coarsePointer ? 1.15 : 1));
  const step = stepFor(size);
  const em = f.theme.typeRamp[step]?.size ?? size;
  const safeW = vp.width - vp.safeArea.left - vp.safeArea.right;
  // About 42 characters a line at this size, never wider than 70 % of the safe area (92 % on a phone).
  const width = Math.min(safeW * (b.phone ? 0.92 : 0.7), em * 0.55 * 42 + tk.lg * 2);
  const bg = clamp01(set.background ?? 0.75);
  const all = desc.lines ?? [];
  const captions = set.captions ? all.filter((l) => l.caption) : [];
  const speech = all.filter((l) => !l.caption).slice(-2);
  if (captions.length === 0 && speech.length === 0) return;
  const t = now();
  kit.anchor(id, { anchor: "bottom", direction: "column", cross: "center", gap: tk.xs, offsetY: -(desc.bottom ?? 0) - tk.md * scale() }, () => {
    for (const l of [...captions, ...speech]) line(l, set, step, em, width, bg, t);
  });
  apply({ a11y: a11y("log", "Subtitles"), inert: true });
}

function line(l: Api.SubtitleLine, set: Api.SubtitleSettings, step: Api.TypeStepName, em: number, width: number, bg: number, t: number): void {
  const tk = frame().t;
  const key = (l.key ?? l.text ?? "").replace(/[/#\s]/g, "-") || "line";
  const born = kit.memory(`born:${key}`, t);
  const fade = clamp01((t - born) / msToTicks(120));
  const speaker = set.speakerNames !== false && l.speaker ? l.speaker : "";
  const color = set.speakerColors !== false ? markupColor(l.color ?? undefined) : "#ffffff";
  // A caption is bracketed (once: a text that brings its own brackets keeps them), its bracket escaped as the text's are.
  const bare = l.caption ? (l.text ?? "").replace(/^\[(.*)\]$/s, "$1") : (l.text ?? "");
  const body = l.caption ? `[[${escape(bare)}]` : escape(bare);
  const markup = speaker ? `[color=${color}][b]${escape(speaker)}:[/b][/color] ${body}` : body;
  box(key, {
    direction: "row",
    cross: "center",
    gap: tk.sm,
    padding: { left: em * 0.5, top: em * 0.18, right: em * 0.5, bottom: em * 0.22 },
    part: sel("Subtitles", l.caption ? "caption" : "line"),
    partClass: "hud",
  }, {
    shape: { kind: "rect", radii: [4, 4, 4, 4] },
    fills: bg > 0 ? [solid({ color: { r: 0, g: 0, b: 0 }, alpha: bg })] : [],
  }, () => {
    if (set.directions !== false && l.direction !== null && l.direction !== undefined) {
      // A triangle pointing toward the source: up is ahead, right is to the right.
      leaf("Arrow", { width: px(em * 0.62), height: px(em * 0.62), part: sel("Subtitles", "arrow") }, {
        shape: { kind: "polygon", sides: 3, rotation: l.direction, rounding: 1.5 },
        fills: [solid(l.color ?? { color: { r: 1, g: 0.86, b: 0.4 } })],
      });
    }
    kit.richText("Text", markup, { step, width: { kind: "fit", max: width }, align: "center", color: { color: l.caption ? { r: 0.82, g: 0.84, b: 0.88 } : { r: 1, g: 1, b: 1 } } });
    if (bg < 0.5) kit.setDraw({ text: hudText(1.2) });
  }, {
    opacity: fade,
    enter: { kind: "fade", transition: { kind: "tween", durationMs: 120, ease: "out" } },
    exit: { kind: "fade", transition: { kind: "tween", durationMs: 160, ease: "exit" } },
    a11y: a11y("text", speaker ? `${speaker}: ${l.text ?? ""}` : (l.text ?? "")),
  });
}
