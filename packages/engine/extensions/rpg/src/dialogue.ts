// The RPG kit's conversation (docs/ui.md §11 genre patterns, Appendix B:
// dialogue with choices, timers and checks), fed by og.story: og.story.view
// hands over each line resolved and whole, and this draws it. The reveal's
// pace and the roll's moment are the component's own state (og.ui.memory,
// so a snapshot carries them); what the player did is answered for the game
// to pass on to og.story (advance, choose), which owns the story.
//
// Read against Mass Effect's and Baldur's Gate 3's conversations (the
// speaker large at a side, the line a caption, the choices stacked on the
// other side, a check's odds on its choice, the d20's roll) and Disco
// Elysium's (a check tinted by its odds, a disabled choice that says why).
// Moved from the engine's patterns/story.ts (docs/ui.md §17).

import type * as Api from "@origingame/engine";
import { tps, accent, artSlot, clamp, fill, flags, frame, glow, grow, hex, kit, keyHash, line, mix, over, paint, px, role, rtl, shadow, spring, st, still, surface, symbol, text, token, u, vgrad, vstate, xy, type Mut, type Rgba } from "@origingame/engine/ui-parts/screen";
import type { ConversationDesc, ConversationResult } from "../types.ts";

/** A stat's words: the game's, else its variable as written. */
function statName(names: unknown, stat: string): string {
  const n = names && typeof names === "object" ? (names as Record<string, unknown>)[stat] : undefined;
  return typeof n === "string" && n ? n : stat;
}

/** An active check's odds as an ink: likely in success, even in warning, long in danger. */
function oddsInk(chance: number): Rgba {
  return chance >= 0.66 ? role("success") : chance >= 0.33 ? role("warn") : role("danger");
}

/** Which physical side a speaker stands on, as an inset side: left is the start edge unless the layout is mirrored. */
function sideEdge(side: string): "start" | "end" | "center" {
  // Staged as written for a left-to-right reader: a right-to-left layout mirrors it with everything else (insets do).
  if (side === "center") return "center";
  return side === "right" ? "end" : "start";
}

/** A tool at the top end (Skip, Log, Auto): the device's glyph and its word, on a quiet pill; pointer only, keys use its action. */
function tool(id: string, label: string, action: string, on: boolean | null): boolean {
  const g = frame().touch ? null : (kit.frameGlyph(action) as Api.UiGlyph | null);
  const r = surface(id, {
    part: `Conversation.tool${flags(on === true && "on")}`,
    cls: "badge",
    fill: on ? mix(role("panelRaised"), role("ink"), 0.12) : null,
    wash: role("ink"),
    washBy: 0.14,
    gap: u(8),
    // A whole touch target on a touch screen.
    height: { kind: "fit", min: frame().touch ? 44 : 0 },
    padding: xy(u(10), u(6)),
    radius: u(8),
    semantics: { role: on === null ? "button" : "switch", name: label, ...(on === null ? {} : { checked: on }) },
  }, () => {
    if (g) {
      kit.open("Glyph", { direction: "row", align: "center", cross: "center", padding: xy(u(6), 0), height: px(u(22)), part: "Conversation.keycap", partClass: "badge" });
      kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("ink", 0.1) }], strokes: [line(1, token("ink", 0.42))] });
      text("Text", g.text, { step: st(12), font: "mono", nowrap: true });
      kit.close();
    } else if (on !== null) {
      // A switch's dot: lit when on.
      kit.open("Dot", { width: px(u(8)), height: px(u(8)) });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: on ? token("ink") : token("ink", 0.25) }] });
      kit.close();
    }
    text("Text", label, { step: st(15), nowrap: true, color: on === false ? "inkMuted" : "ink" });
  });
  return r.clicked;
}

/** A d20 drawn as shapes: an outlined hexagon with its inner triangle, the roll in the middle. */
function die(id: string, roll: number, ink: Rgba, size: number): void {
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", part: "Conversation.die", partClass: "badge" });
  kit.setDraw({
    shape: { kind: "polygon", sides: 6, rotation: Math.PI / 6 },
    fills: [{ kind: "solid", color: token("shadow", 0.85) }],
    strokes: [line(2.5, paint(ink))],
    effects: [glow(u(14), paint(ink, 0.55), 0)],
  } as Api.DrawDesc);
  kit.open("Facet", { width: px(size), height: px(size) });
  const p = (x: number, y: number) => ({ x: x * size, y: y * size });
  kit.setDraw({ shape: { kind: "path", points: [p(0.5, 0.2), p(0.81, 0.7), p(0.19, 0.7)], closed: true, width: 1.2, join: "round" }, fills: [{ kind: "solid", color: paint(ink, 0.35) }] } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  text("Roll", String(roll), { step: st(Math.round(size * 0.36)), font: "display", nowrap: true });
  kit.close();
}

/** Three bars that move while a voice plays (a speaker's cue), still under reduced motion. */
function voiceCue(id: string, ink: Rgba): void {
  const t = kit.tick();
  kit.open(id, { direction: "row", gap: u(2), cross: "end", height: px(u(14)), part: "Conversation.voice", partClass: "badge" });
  for (let k = 0; k < 3; k++) {
    const h = still() ? [0.55, 1, 0.7][k]! : 0.35 + 0.65 * Math.abs(Math.sin(t * 0.21 + k * 1.9));
    kit.open(kit.idi("Bar", k), { width: px(u(3)), height: px(Math.max(2, Math.round(u(14) * h))) });
    kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: paint(ink) }] });
    kit.close();
  }
  kit.close();
}

/** The chevron that waits for a press, bobbing. */
function more(id: string, ink: Rgba): void {
  const bob = still() ? 0 : Math.round(Math.sin(kit.tick() * 0.12) * u(3));
  const s = u(14);
  kit.open(id, { width: px(s), height: px(s), part: "Conversation.more", partClass: "badge" });
  const p = (x: number, y: number) => ({ x: x * s, y: y * s });
  kit.setDraw({ shape: { kind: "path", points: [p(0.15, 0.32), p(0.5, 0.7), p(0.85, 0.32)], width: 2.5, cap: "round", join: "round" }, fills: [{ kind: "solid", color: paint(ink) }] } as Api.DrawDesc);
  kit.close();
  kit.props({ transform: { y: bob } });
}

/**
 * A conversation on screen (Mass Effect, Baldur's Gate 3, Persona): see
 * `fn conversation` in crates/og-api/api/ui.ogt.
 */
export function conversation(id: string, desc: ConversationDesc): ConversationResult {
  const out: Mut<ConversationResult> = { advanced: false, chosen: null, skipped: false, log: false, autoToggled: false, revealed: false, hovered: null };
  const v = desc.view ?? null;
  const { viewport: vp, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const box = (desc.layout ?? "cinematic") === "box";
  // Its own box as it was last tick (the game may give it less than the safe area: a bar above it), else the safe area.
  const W = kit.memory(`~c.w.${id}`, 0) || vp.width - vp.safeArea.left - vp.safeArea.right;
  const H = kit.memory(`~c.h.${id}`, 0) || vp.height - vp.safeArea.top - vp.safeArea.bottom;
  const tick = kit.tick();
  const rate = tps();
  const ts = Math.max(1, frame().theme.textScale ?? 1);

  kit.open(id, { direction: "column", width: grow(), height: grow(), part: `Conversation.frame${flags(box ? "box" : "cinematic")}`, partClass: "panel" });
  if (!v) {
    kit.close();
    return out;
  }

  // The reveal: from the tick this line (its seq) was first shown, at charsPerSecond; a press shows it whole.
  const chars = Array.from(v.text);
  const seqWas = kit.memory("~c.seq", -1);
  const fresh = seqWas !== v.seq;
  if (fresh) {
    kit.remember("~c.seq", v.seq);
    kit.remember("~c.t0", tick);
    kit.remember("~c.whole", 0);
  }
  const t0 = fresh ? tick : kit.memory("~c.t0", tick);
  const cps = desc.charsPerSecond ?? 45;
  const whole = kit.memory("~c.whole", 0) === 1 || cps <= 0 || frame().theme.reducedMotion;
  const shown = whole ? chars.length : Math.min(chars.length, Math.floor(((tick - t0) / rate) * cps));
  const done = shown >= chars.length;
  out.revealed = done;
  const choices = v.ended ? [] : v.choices;
  const waiting = choices.length > 0;
  // The tick the whole line first showed (auto counts from there, and the choices rise in then).
  let doneAt = kit.memory("~c.done", -1);
  if (done && (fresh || doneAt < t0)) {
    doneAt = tick;
    kit.remember("~c.done", tick);
  }
  const voicing = v.voiceTicks > 0 && v.voiceLeft > 0;
  const typedOn = Math.floor(shown / 3);
  if (!done && typedOn !== kit.memory("~c.typed", 0)) kit.feedback("counterTick", -1);
  kit.remember("~c.typed", typedOn);

  const side = sideEdge(v.speaker.side || "left");
  // The narrator, or a speaker without a portrait, stands no figure.
  const hasFigure = !!v.speaker.portrait;
  const ink = role("ink");

  // The whole screen is the press that goes on: a layer under everything else.
  // A focus stop for keys and pads too (Enter or A on it goes on, as confirm does anywhere); with choices up
  // and the line whole, or at the end, a press does nothing, so it is disabled and the pad walks the choices.
  const hit = surface("Hit", { part: "Conversation.hit", cls: "badge", fill: null, washBy: 0, disabled: v.ended || (waiting && done), width: grow(), height: grow(), semantics: { role: "button", name: desc.continueLabel ?? "Continue" }, layer: { insets: { start: 0, top: 0, end: 0, bottom: 0 }, z: 0 } }, () => {});
  const press = hit.clicked || (!waiting && kit.action("confirm"));
  if (press && !v.ended) {
    if (!done) {
      kit.remember("~c.whole", 1);
    } else if (!waiting) {
      out.advanced = true;
      kit.feedback("confirm", -1);
    }
  }
  // Auto: a whole line goes on by itself once its voice has ended and the pause after that has passed, once a line.
  let quiet = kit.memory("~c.quiet", -1);
  if (voicing || fresh) quiet = voicing ? -1 : tick;
  else if (quiet < 0) quiet = tick;
  kit.remember("~c.quiet", quiet);
  if (desc.auto && done && !waiting && !voicing && !v.ended && !out.advanced && kit.memory("~c.autoSeq", -1) !== v.seq) {
    const pause = desc.autoDelayMs && desc.autoDelayMs > 0 ? desc.autoDelayMs : 900 + 30 * chars.length;
    if (tick - Math.max(doneAt, quiet) >= Math.round((pause / 1000) * rate)) {
      out.advanced = true;
      kit.remember("~c.autoSeq", v.seq);
    }
  }

  // The speaker's figure: large at its side, rising from the foot (cinematic), or standing behind the text box.
  if (hasFigure && side !== "center" || hasFigure && box) {
    const fw = portrait ? Math.round(W * 0.78) : Math.round(Math.min(W * (box ? 0.36 : 0.44), H * 0.9));
    const fh = portrait ? Math.round(H * 0.78) : Math.round(H * (box ? 0.88 : 0.96));
    // Undressed (no part): a style's slot dress strokes its box, and a figure in the scene has no box.
    kit.open("Figure", { width: px(fw), height: px(fh) });
    if (desc.portrait) {
      // Undressed: a cut-out figure stands in the scene, never inside a slot's frame.
      kit.open("Art", { width: px(fw), height: px(fh) });
      kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "image", texture: desc.portrait, fit: "contain", focal: { x: 0.5, y: 1 } }] } as Api.DrawDesc);
      kit.close();
    } else artSlot("Art", "Conversation.art", { texture: null, kind: "portrait", width: px(fw), height: px(fh), radius: 0, focal: { x: 0.5, y: 0.2 } } as never);
    kit.close();
    // On a phone the speaker stands on the screen's foot behind the words (a figure cut off mid-screen read as a box).
    const at = portrait ? { centerX: true, bottom: 0 } : side === "end" ? { end: box ? u(24) : 0, bottom: 0 } : { start: box ? u(24) : 0, bottom: 0 };
    over({ insets: at, inert: true, z: 1, ...(fresh && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: side === "end" ? u(24) : -u(24) }], transition: spring("panel") } } : {}) });
  }

  // The shade at the foot that keeps the caption legible over any scene (cinematic); the box carries its own ground.
  if (!box) {
    kit.open("Shade", { width: grow(), height: px(Math.round(H * (portrait ? 0.62 : 0.46))), part: "Conversation.shade", partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: token("shadow", 0) }, { offset: 0.45, color: token("shadow", 0.62) }, { offset: 1, color: token("shadow", 0.9) }] }] });
    kit.close();
    over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 1 });
  }

  // The line: the name over it in the display face, the words typed out, the chevron or the voice's bars after it.
  const captionW = box ? Math.min(W - u(phone ? 24 : 64), u(1100)) : portrait ? W - u(32) : Math.min(Math.round(W * 0.62), u(980));
  const lineStep: Api.TypeStepName = phone ? st(18) : box ? st(24) : st(26);
  // On a phone with choices the caption rises over them, off the shade onto the open scene: it carries its own soft
  // dark plate there, so the line reads at 4.5:1 on any backdrop (a pale sky left it at 2.3:1).
  const rises = portrait && waiting && done;
  const plated = !box && rises;
  const moreW = u(14) + u(10);
  // Read here, beside where it is kept (the UI's memory is the open element's).
  const wordsRoom = kit.memory(`~c.room.${id}`, 0) || 1e6;
  kit.open("Caption", {
    direction: "column",
    width: px(captionW),
    gap: u(box ? 10 : 8),
    // Stretched, never centred by its rows' width: a row fitted to a right-to-left line overran the screen.
    cross: "stretch",
    padding: box ? { left: u(28), right: u(28), top: u(26), bottom: u(22) } : plated ? xy(u(14), u(10)) : xy(0, 0),
    part: box ? "Conversation.box" : "Conversation.caption",
    partClass: box ? "panel" : "badge",
  });
  if (box) {
    // The VN box: a dark glass that holds over a bright scene, a light top edge, no outline.
    kit.setDraw({
      shape: { kind: "rect", radii: [u(14), u(14), u(14), u(14)] },
      fills: [{ kind: "gradient", type: "linear", from: { x: 0.5, y: 0 }, to: { x: 0.5, y: 1 }, stops: [{ offset: 0, color: token("panel", 0.9) }, { offset: 1, color: token("shadow", 0.92) }] }],
      strokes: [line(1, token("ink", 0.1))],
      effects: [shadow(u(10), u(30), 0.55)],
    });
  } else if (plated) {
    // Feathered by its shadow, so it reads as the shade lifted behind the words rather than a box.
    kit.setDraw({
      shape: { kind: "rect", radii: [u(12), u(12), u(12), u(12)] },
      // The shadow's colour at 0.86 whatever the token's own alpha: cream ink stays at 4.5:1 even over white.
      fills: [{ kind: "solid", color: paint({ ...role("shadow"), a: 0.86 }) }],
      effects: [shadow(0, u(18), 0.5)],
    });
  }
  const named = v.speaker.name !== "";
  if (named && !box) {
    // The name in the display face, tracked wide, in muted ink: it labels the line without competing with it.
    kit.row("Head", { gap: u(10), cross: "center", align: "center", width: grow() }, () => {
      // On its plate the name takes the body step in the accent: at 13 px a pixel face's name read as a speck.
      const nameInk = plated ? accent() : mix(ink, role("inkMuted"), 0.35);
      text("Name", v.speaker.name.toLocaleUpperCase(), { step: plated ? "base" : st(phone ? 13 : 16), font: "display", color: nameInk, nowrap: true });
      if (voicing) voiceCue("Voice", mix(ink, role("inkMuted"), 0.35));
    });
  }
  // A narrator's line (no name) reads in muted ink, set apart from what is said aloud.
  const lineInk = named ? ink : mix(ink, role("inkMuted"), 0.5);
  // The words scroll inside the room the conversation has (last tick's: its height less the tools, the caption's own
  // name and padding, and what it stands on): a long line at 200 % on a phone ran up out of the screen's room, under
  // the game's own buttons, and a spoken line is never cut. The chevron stands in the end padding, so the words do not
  // reflow when it appears.
  kit.openScroll("Words", { width: grow(), height: { kind: "fit", max: wordsRoom }, padding: { left: 0, top: 0, right: moreW, bottom: 0 } } as never);
  {
    // Laid out whole from the first letter (the rest clear), so the words never jump to the next line as they type;
    // whole, one span.
    kit.richText("Line", done ? esc(v.text) : `${esc(chars.slice(0, shown).join(""))}[color=#00000000]${esc(chars.slice(shown).join(""))}[/color]`, { step: lineStep, color: paint(lineInk), align: box ? "start" : "center", maxLines: phone ? 5 : 3, width: grow() } as Api.RichTextDesc);
    kit.props({ states: [{ name: "reveal", states: [vstate("typing"), vstate("shown")], transitions: [] }], state: [{ group: "reveal", state: done ? "shown" : "typing" }] });
    kit.semantics({ role: "status", name: `${v.speaker.name ? `${v.speaker.name}: ` : ""}${v.text}` });
    if (done && !waiting && !v.ended) {
      more("More", ink);
      over({ insets: { end: 0, bottom: 0 }, inert: true, z: 1 });
    }
  }
  kit.close();
  kit.close();
  const capBottom = box ? u(phone ? 16 : 28) : u(phone ? 22 : 44);
  // On a phone in portrait with choices, the caption rises over them; else it sits at the foot.
  const choiceRow = u(phone ? 52 : 58) * ts;
  const choicesH = waiting ? Math.min(Math.round(H * 0.5), choices.length * (choiceRow + u(8))) + (v.timer ? u(30) : 0) : 0;
  // The stack's height as it was last tick (rows with a tag or a reason are taller than one line), else the estimate.
  const base = kit.lastPath().replace(/\/Caption$/, "");
  const chB = kit.bounds(`${base}/Choices`);
  // The tools' foot at the top (last tick's; they grow with the text size): nothing stacks under them.
  const toolsB = kit.bounds(`${base}/Tools`);
  const toolsFoot = toolsB && toolsB.height > 0 ? Math.round(toolsB.height) + u(phone ? 8 : 20) + u(12) : u(76);
  const stackH = chB && chB.height > 0 ? Math.round(chB.height) : choicesH;
  const capInsets = rises ? { centerX: true, bottom: capBottom + stackH + u(16) } : { centerX: true, bottom: capBottom };
  over({ insets: capInsets, z: 2 });
  // The caption's height (last tick's): the plate sits on its top edge and the choices stack above it.
  const capH = kit.bounds(kit.lastPath())?.height ?? u(90);
  // The words' room next tick: the conversation's height above the caption's foot and below the tools, less what the
  // caption holds besides its words.
  const wordsH = kit.bounds(`${base}/Caption/Words`)?.height ?? 0;
  kit.remember(`~c.room.${id}`, Math.max(u(64), Math.round(H - capInsets.bottom - toolsFoot - (capH - wordsH))));
  if (box && named) {
    // The name on a plate over the box's top edge at the start, half over it.
    kit.row("Plate", { gap: u(10), cross: "center", padding: xy(u(18), u(7)), part: "Conversation.name", partClass: "badge" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [u(8), u(8), u(8), u(8)] }, fills: [{ kind: "solid", color: token("panelRaised") }], effects: [shadow(u(3), u(10), 0.5)] });
      text("Name", v.speaker.name, { step: st(phone ? 16 : 20), font: "display", nowrap: true });
      if (voicing) voiceCue("Voice", ink);
    });
    const plateH = kit.bounds(kit.lastPath())?.height ?? u(40);
    over({ insets: { start: Math.round((W - captionW) / 2) + u(20), bottom: capInsets.bottom + capH - Math.round(plateH / 2) }, z: 3, inert: true });
  }

  // The choices: stacked on the side the speaker does not stand (centred over the scene in a box, at the thumb on a phone).
  if (waiting && done) {
    const cw = portrait ? W - u(24) : box ? Math.min(u(620), W - u(48)) : Math.min(u(560), Math.round(W * 0.4));
    const sinceDone = tick - doneAt;
    kit.open("Choices", { direction: "column", width: px(cw), gap: u(8), part: "Conversation.choices", partClass: "panel" });
    // The rows scroll inside the room the screen leaves (six choices at 200 % text outgrow it); the timer stays under them.
    // Off a phone in portrait the stack stands on the caption (a box's plate above it) and stops under the tools.
    const above = capInsets.bottom + capH + u(box ? 36 : 24);
    // The timer's row and the gap over it, grown with the text size.
    const timerH = v.timer ? Math.round(u(38) * ts) : 0;
    // In portrait the caption stands on the stack, so the stack leaves it its height and the tools theirs.
    const room = portrait ? Math.max(u(120), H - capBottom - capH - u(16) - toolsFoot - timerH) : Math.max(u(120), H - above - toolsFoot - timerH);
    kit.openScroll("List", { width: grow(), height: { kind: "fit", max: room }, gap: u(8), padding: xy(0, 0) } as never);
    const isPad = /^(xbox|ps|switch|pad)\./.test((kit.frameGlyph("confirm") as Api.UiGlyph | null)?.icon ?? "");
    choices.forEach((c, i) => {
      const cid = c.id || `c${c.index}`;
      const isDefault = !!v.timer && v.timer.defaultChoice === c.index;
      const passive = c.check && c.check.kind === "passive";
      const active = c.check && c.check.kind === "active";
      const numbered = i < 9;
      const tagInk = active ? oddsInk(c.check!.chance) : role("success");
      const reason = !c.enabled ? (c.reason || desc.unavailableLabel || "Unavailable") : "";
      const tagText = c.check
        ? active
          ? fill(desc.checkLabel ?? "{stat} · {chance}", { stat: statName(desc.statNames, c.check.stat), chance: `${Math.round(c.check.chance * 100)} %` })
          : statName(desc.statNames, c.check.stat)
        : "";
      const picked = numbered && c.enabled && kit.action(`choice${i + 1}`);
      const r = surface(cid, {
        part: `Conversation.choice${flags(c.enabled ? "enabled" : "disabled", c.visited && "visited", isDefault && "default", !!c.check && "check")}`,
        cls: "button",
        fill: mix(role("shadow"), role("panel"), 0.35),
        wash: role("ink"),
        washBy: 0.1,
        disabled: !c.enabled,
        width: grow(),
        padding: { left: u(16), right: u(16), top: u(12), bottom: u(12) },
        gap: u(12),
        cross: "center",
        radius: u(8),
        lift: 0,
        semantics: {
          role: "button",
          name: [`${i + 1}`, tagText, c.text, reason].filter(Boolean).join(", "),
        },
        props: {
          ...(sinceDone < 2 && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: side === "end" ? -u(18) : u(18) }], transition: spring("panel"), delayMs: 70 * i } } : {}),
          states: [{ name: "choice", states: [vstate("open"), vstate("visited"), vstate("disabled")], transitions: [] }],
          state: [{ group: "choice", state: !c.enabled ? "disabled" : c.visited ? "visited" : "open" }],
        },
      }, () => {
        const ix = kit.memory(`${cid}~ix`, 0);
        const lit = c.enabled && (ix & 5) !== 0;
        // The ground: dark glass that holds over the scene; hovered or focused, a lit start edge and a lifted ground. No outline.
        kit.setDraw({
          shape: { kind: "rect", radii: [u(8), u(8), u(8), u(8)] },
          fills: lit
            ? [{ kind: "solid", color: token("panelRaised", 0.95) }, vgrad([[0, token("sheen", 0.08)], [1, token("sheen", 0)]])]
            : [{ kind: "solid", color: paint(mix(role("panel"), role("shadow"), 0.25), 0.74) }, vgrad([[0, token("sheen", 0.05)], [1, token("sheen", 0)]])],
          strokes: [line(1, token("ink", lit ? 0.16 : 0.07))],
          effects: [shadow(u(2), u(8), 0.35)],
        });
        if (lit) {
          kit.open("Edge", { width: px(u(3)), height: grow() });
          kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink") }] });
          kit.close();
          over({ insets: { start: 0, top: u(8), bottom: u(8) }, inert: true });
        }
        const muted = !c.enabled ? "inkFaint" : c.visited ? "inkMuted" : "ink";
        if (numbered) text("Number", `${i + 1}`, { step: st(14), font: "mono", color: c.enabled ? "inkMuted" : "inkFaint", nowrap: true });
        kit.column("Body", { width: grow(), gap: u(4) }, () => {
          kit.row("Top", { width: grow(), gap: u(8), cross: "center", wrap: true }, () => {
            if (!c.enabled) symbol("Lock", "lock", u(14), role("inkFaint"));
            if (tagText && c.enabled) {
              // A check's tag: its stat, and an active one's odds, in the odds' ink on a tint of it; a passed passive in success.
              kit.row("Tag", { gap: u(4), cross: "center", padding: xy(u(8), u(2)), part: `Conversation.tag${flags(passive ? "passive" : "active")}`, partClass: "badge" }, () => {
                kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: paint(tagInk, 0.16) }] });
                if (passive) symbol("Check", "check", u(12), tagInk);
                text("Text", tagText, { step: st(13), color: mix(tagInk, hex("#ffffff"), 0.3), nowrap: true, font: "display" });
              });
            }
            text("Text", c.text, { step: st(phone ? 16 : 18), color: muted, width: grow(), maxLines: 3 });
          });
          if (reason) text("Reason", reason, { step: st(13), color: "inkMuted", width: grow(), maxLines: 2 });
        });
        if (isDefault) symbol("Default", "clock", u(14), role(lit ? "ink" : "inkMuted"));
      });
      if ((r.hovered || r.focused) && c.enabled) out.hovered = c.index;
      if ((r.clicked || picked) && c.enabled && out.chosen === null) {
        out.chosen = c.index;
        kit.feedback("select", -1);
      }
      // A pad gets the focus on the first open choice when they rise in, so A takes it.
      if (isPad && sinceDone === 0 && c.enabled && !choices.slice(0, i).some((x) => x.enabled)) kit.setFocus(kit.lastPath());
    });
    kit.close();
    if (v.timer && v.timer.ticksTotal > 0) {
      // The timer: a bar along the stack that runs down, warming as it runs out, the seconds left at its end.
      const f = clamp(v.timer.ticksLeft / v.timer.ticksTotal, 0, 1);
      const low = f < 0.2;
      const barInk = low ? role("danger") : f < 0.45 ? role("warn") : ink;
      const secs = Math.ceil(v.timer.ticksLeft / rate);
      kit.row("Timer", { width: grow(), gap: u(12), cross: "center", padding: { left: u(4), right: 0, top: u(6), bottom: 0 }, part: `Conversation.timer${flags(low && "low")}`, partClass: "badge" }, () => {
        kit.open("Track", { width: grow(), height: px(u(4)) });
        kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink", 0.14) }] });
        kit.open("Fill", { width: px(Math.max(0, Math.round((cw - u(70)) * f))), height: grow() });
        kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: paint(barInk) }], ...(low ? { effects: [glow(u(8), paint(barInk, 0.6), 0)] } : {}) });
        kit.close();
        kit.close();
        const pulse = low && !still() ? 1 + 0.08 * Math.max(0, Math.sin(tick * 0.35)) : 1;
        text("Left", `${String(Math.floor(secs / 60)).padStart(2, "0")}:${String(secs % 60).padStart(2, "0")}`, { step: st(15), font: "mono", color: low ? barInk : "inkMuted", nowrap: true });
        kit.props({ transform: { scaleX: pulse, scaleY: pulse } });
      });
      kit.semantics({ role: "timer", name: `${secs}` });
    }
    kit.close();
    kit.semantics({ role: "list", name: v.speaker.name || "Choices" });
    const chInsets = portrait
      ? { centerX: true, bottom: capBottom }
      : box
        ? { centerX: true, bottom: above }
        : side === "end" || (side === "center" && !rtl())
          ? { start: u(56), bottom: above }
          : { end: u(56), bottom: above };
    over({ insets: chInsets, z: 3 });
  }

  // A roll: when an active check is rolled, the d20 and its word pop over the caption for a while.
  const lc = v.lastCheck;
  if (lc) {
    const key = keyHash(`${v.conversation}|${lc.stat}|${lc.target}|${lc.roll}|${lc.passed}|${v.seq}`);
    if (kit.memory("~c.rk", 0) !== key) {
      kit.remember("~c.rk", key);
      kit.remember("~c.rt", tick);
      kit.feedback(lc.passed ? "success" : "error", -1);
    }
    const since = tick - kit.memory("~c.rt", tick);
    if (since < rate * 3) {
      const rInk = lc.passed ? role("success") : role("danger");
      kit.row("Roll", { gap: u(14), cross: "center", padding: { left: u(10), right: u(22), top: u(8), bottom: u(8) }, part: `Conversation.roll${flags(lc.passed ? "passed" : "failed")}`, partClass: "badge" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.78) }], effects: [shadow(u(4), u(16), 0.5)] });
        die("Die", lc.roll, rInk, u(phone ? 40 : 48));
        kit.column("Words", { gap: 0 }, () => {
          text("Stat", statName(desc.statNames, lc.stat).toLocaleUpperCase(), { step: st(12), font: "display", color: "inkMuted", nowrap: true });
          text("Result", lc.passed ? desc.passedLabel ?? "Success" : desc.failedLabel ?? "Failure", { step: st(phone ? 18 : 22), font: "display", color: mix(rInk, hex("#ffffff"), 0.2), nowrap: true });
        });
      });
      kit.props({
        position: "absolute",
        insets: { centerX: true, top: Math.round(H * (portrait ? 0.08 : 0.16)) },
        z: 4,
        inert: true,
        states: [{ name: "roll", states: [vstate("passed"), vstate("failed")], transitions: [] }],
        state: [{ group: "roll", state: lc.passed ? "passed" : "failed" }],
        ...(since < 2 && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "scale", value: 1.6 }], transition: spring("emphasis") } } : {}),
        overlaps: ["*"],
      });
      kit.semantics({ role: "status", name: `${statName(desc.statNames, lc.stat)}: ${lc.roll}, ${lc.passed ? desc.passedLabel ?? "Success" : desc.failedLabel ?? "Failure"}` });
    }
  }

  // Skip, Log and Auto at the top end; their actions work whatever has the focus.
  kit.row("Tools", { gap: u(2), cross: "center", padding: xy(u(4), u(2)), part: "Conversation.tools", partClass: "badge" }, () => {
    // A soft ground so the words read over a figure or a bright sky.
    kit.setDraw({ shape: { kind: "rect", radii: [u(12), u(12), u(12), u(12)] }, fills: [{ kind: "solid", color: token("shadow", 0.42) }] });
    if (desc.skipLabel !== "" && tool("Skip", desc.skipLabel ?? "Skip", "back", null)) out.skipped = true;
    if (desc.logLabel !== "" && tool("Log", desc.logLabel ?? "Log", "details", null)) out.log = true;
    if (desc.autoLabel !== "" && tool("Auto", desc.autoLabel ?? "Auto", "", !!desc.auto)) out.autoToggled = true;
  });
  over({ insets: { end: u(phone ? 8 : 24), top: u(phone ? 8 : 20) }, z: 5 });
  if (desc.skipLabel !== "" && kit.action("back")) out.skipped = true;
  if (desc.logLabel !== "" && kit.action("details")) out.log = true;

  kit.close();
  kit.semantics({ role: "region", name: v.speaker.name || "Conversation" });
  const self = kit.bounds(kit.lastPath());
  if (self && self.width > 0 && self.height > 0) {
    kit.remember(`~c.w.${id}`, Math.round(self.width));
    kit.remember(`~c.h.${id}`, Math.round(self.height));
  }
  return out;
}

/** Text as rich-text markup: its brackets written as text. */
function esc(s: string): string {
  return s.replace(/\[/g, "[[");
}
