// The roster patterns (docs/ui.md §11 genre patterns, Appendix B: lobby,
// party, matchmaking and ready check). Read against Valorant's and
// Fortnite's lobbies (the party as a lineup of tall character cards, the
// local player first, one call to action at the foot), Rocket League's
// (the mode on a card beside it, the time in queue counting) and League of
// Legends' ready check (Match found over everything, a ring that runs out,
// Accept large beside Decline). Who is in the party, who is ready and the
// queue are the game's; a press answers what the player asked for.

import type * as Api from "@origingame/engine";
import {
  firstRoom,
  artSlot, btn, clamp, eyebrow, fill, flags, frame, glow, grow, inkOn, keyHash, kit, lastInteraction, line, mix, over, paint, pointer, px, recallPath, rememberPath,
  rememberRoom, role, rtl, scrimFill, shadow, spring, st, still, surface, symbol, teamColor, text, timerRing, token, u, vgrad, vstate, xy, safeHeight, safeWidth, type Mut, type Rgba,
} from "./core.ts";
import { stepSize } from "../feedback/core.ts";

/** What a character card shows: its art, its name and the lines under it, how it stands. */
interface Figure {
  texture: Api.Texture | null | undefined;
  name: string;
  sub: string;
  width: number;
  height: number;
  /** The name's step: the larger on a card the room makes wide, whatever the text size. */
  big: boolean;
  /** The art's point kept in view; the face by default. */
  focal?: Api.Vector2;
  /** Lit in the accent: the local player, the one picked. */
  lit: boolean;
  /** Under the pointer or the focus. */
  hot: boolean;
  /** Dimmed (taken, banned, locked). */
  dim: boolean;
  part: string;
}

/**
 * A tall character card: the art filling it, a scrim rising at its foot
 * under the name in the display face and the line under it; lit, a rim in
 * the accent with a soft glow round it (the selection grammar, never a
 * white outline); dimmed, the art down to a third of its value. Built
 * inside the surface it dresses; `foot` adds what stands under the name.
 */
function figureCard(f: Figure, foot?: () => void): void {
  const { t } = frame();
  const r = Math.max(6, t.panel);
  const radii: [number, number, number, number] = [r, r, r, r];
  const ink = role("ink");
  const accent = role("focus");
  kit.setDraw({
    shape: { kind: "rect", radii },
    fills: [vgrad([[0, token("panelRaised")], [1, token("panel")]])],
    strokes: f.lit ? [line(u(2.5), paint(accent))] : f.hot ? [line(u(1.5), paint(mix(accent, ink, 0.4), 0.8))] : [line(1, token("shadow", 0.55), "outside")],
    effects: [shadow(u(f.hot || f.lit ? 10 : 6), u(f.hot || f.lit ? 24 : 14), 0.5), ...(f.lit ? [glow(u(7), paint(accent, 0.45), 0)] : [])],
  });
  artSlot("Art", `${f.part.split(".")[0]}.art`, { texture: f.texture ?? null, kind: "character", width: px(f.width), height: px(f.height), radius: radii, focal: f.focal ?? { x: 0.5, y: 0.22 } }, () => {
    if (f.dim) {
      kit.open("Dim", { width: grow(), height: grow() });
      kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.2), 0.55) }, { kind: "solid", color: token("shadow", 0.38) }] });
      kit.close();
    }
  });
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
  kit.open("Shade", { width: grow(), height: grow() });
  kit.setDraw({ shape: { kind: "rect", radii }, fills: [scrimFill("bottom", 0.9)] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
  kit.open("Plate", { direction: "column", width: grow(), gap: u(2), padding: { left: u(14), right: u(14), top: 0, bottom: u(12) }, part: `${f.part.split(".")[0]}.plate`, partClass: "badge" });
  text("Name", f.name, { step: st(f.big ? 22 : 18), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow(), color: f.dim ? "inkMuted" : "ink" });
  if (f.sub) text("Sub", f.sub, { step: st(14), color: "inkMuted", width: grow() });
  foot?.();
  kit.close();
  over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
}

/** A crown in the gold on a dark disc (the party's leader), at a card's top start. */
function crown(id: string, size: number, label: string): void {
  kit.open(id, { width: px(size), height: px(size), align: "center", cross: "center", part: "Lobby.crown", partClass: "badge" });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("shadow", 0.72) }], strokes: [line(1, token("gold", 0.6))], effects: [shadow(2, 6, 0.5)] });
  symbol("Mark", "crown", Math.round(size * 0.58), role("gold"), { label });
  kit.close();
}

/** The breathing dot of a live search. */
function liveDot(id: string, ink: Rgba): void {
  const a = still() ? 1 : 0.55 + 0.45 * Math.abs(Math.sin(kit.tick() * 0.07));
  kit.open(id, { width: px(u(8)), height: px(u(8)) });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: paint(ink, a) }], effects: [glow(u(6), paint(ink, 0.5 * a), 0)] });
  kit.close();
}

/** Seconds as m:ss. */
function clock(s: number): string {
  const n = Math.max(0, Math.floor(s));
  return `${Math.floor(n / 60)}:${String(n % 60).padStart(2, "0")}`;
}

/**
 * A party lobby (Valorant, Fortnite, Rocket League): see `fn lobby` in
 * crates/og-api/api/ui.ogt.
 */
export function lobby(id: string, desc: Api.LobbyDesc): Api.LobbyResult {
  const out: Mut<Api.LobbyResult> = { ready: false, start: false, cancel: false, invite: null, member: null, mode: false, accept: false, decline: false };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const all = desc.members ?? [];
  // The local player first, the rest in the game's order.
  const members = [...all.filter((m) => m.you), ...all.filter((m) => !m.you)];
  const me = members.find((m) => m.you) ?? null;
  const seats = Math.max(members.length, desc.slots ?? 0);
  const check = desc.readyCheck ?? null;
  // Its own box as it was last tick (the game gives it less than the safe area: a padding, a bar), else a first guess:
  // the whole safe area wrapped a phone's two cards one a row, past the screen.
  const W = kit.memory(`~l.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~l.h.${id}`, 0) || firstRoom().h;
  const shown = kit.memory("~l.t0", -1);
  if (shown < 0) kit.remember("~l.t0", tick);
  const since = tick - (shown < 0 ? tick : shown);

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 14 : 24), part: "Lobby.frame", partClass: "panel" });

  // The head: the title in the display face; the search chip beside it while in queue. Without either (the game titles
  // the screen itself) there is none, so the lineup takes its room.
  if (desc.title || desc.searching) kit.row("Head", { width: grow(), gap: u(16), cross: "center", wrap: true }, () => {
    if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
    if (desc.searching) {
      // On a phone the chip takes the head's width and its estimate wraps under the count.
      kit.row("Search", { gap: u(10), cross: "center", wrap: phone, width: phone ? grow() : null, padding: xy(u(14), u(8)), part: "Lobby.search", partClass: "chip" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [u(20), u(20), u(20), u(20)] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }, vgrad([[0, token("sheen", 0.08)], [1, token("sheen", 0)]])], effects: [shadow(u(2), u(8), 0.4)] });
        liveDot("Live", role("success"));
        text("Label", (desc.searchingLabel ?? "Searching"), { uppercase: true, step: st(13), font: "display", color: "inkMuted", nowrap: true });
        text("Time", clock(desc.searchSeconds ?? 0), { step: st(18), font: "mono", nowrap: true });
        if ((desc.estimateSeconds ?? 0) > 0) text("Estimate", fill(desc.estimateLabel ?? "Est. {time}", { time: clock(desc.estimateSeconds ?? 0) }), { step: st(13), color: "inkMuted", nowrap: true });
      });
      kit.semantics({ role: "timer", name: `${desc.searchingLabel ?? "Searching"} ${clock(desc.searchSeconds ?? 0)}` });
    }
  });

  // The lineup: tall cards side by side (two columns on a phone in portrait), the local player's a step larger.
  const gap = u(phone ? 10 : 18);
  const footH = Math.round(u(phone ? 140 : 150) * Math.min(1.5, ts));
  const cols = portrait ? 2 : Math.max(1, seats);
  const rows = Math.ceil(seats / cols);
  // The room the lineup was given last tick (it grows between the head and the foot, which grow with the text), else an
  // estimate. The lineup scrolls, so its laid height is the room the column leaves it, never what its cards ask.
  const laidH = kit.bounds(recallPath(`${id}~lineup`))?.height ?? 0;
  const lineH = laidH > 0 ? laidH : H - footH - u(phone ? 60 : 96);
  // A card's plate from the type ramp: the name, the line under it and the ready foot over the bottom padding, with the
  // crown's corner and a strip of the face above it. A card never stands shorter than its plate with the name and the
  // ready word (at 200 % text the plate ran up over the head and the card above); a card too short for the line under
  // the name drops it (names win over marks), and the lineup scrolls when such cards outgrow its room.
  const lineOf = (s: Api.TypeStepName): number => stepSize(s).line;
  // The ready word wraps rather than being cut: the plate as laid without the line under the name (its tallest last tick,
  // or as long ago as one was laid) holds the cards up too.
  const leanLaid = kit.memory(`~l.lean.${id}`, 0);
  const lean = Math.max(u(12) + lineOf(st(22)) + u(2) + u(6) + Math.max(u(14), lineOf(st(13))), leanLaid) + u(44);
  const full = lean + u(2) + lineOf(st(14));
  const roomH = Math.max(lean, (lineH - u(8) - (rows - 1) * gap) / rows / (me && !portrait ? 1.12 : 1));
  // The widest a card may stand, from the room's width and the seats alone: the name's step follows it, not the card's
  // width, which narrows with the rows' height as the text grows (at 200 % the names fell a step, growing 1.6 times).
  const maxW = portrait ? (W - gap) / 2 : Math.min(u(250), (W - gap * (cols - 1)) / (cols + (me ? 0.12 : 0)));
  // On a phone in portrait the cards take the column's width and stand as tall as the rows allow (a squarer card, its
  // face over the plate), rather than narrowing to keep their aspect and leaving a gutter at each side.
  const cw = Math.floor(portrait ? maxW : Math.min(maxW, roomH / 1.75));
  const ch = Math.max(lean, Math.floor(Math.min(roomH, cw * (portrait ? 1.25 : 1.75))));
  const big = (m: Api.LobbyMember | null): number => (m?.you && !portrait ? 1.12 : 1);
  let leanNow = 0;
  kit.openScroll("Room", { width: grow(), height: grow(), gap: 0, padding: xy(0, 0) } as never);
  kit.open("Lineup", { direction: "row", width: grow(), height: { kind: "fit", min: Math.max(0, lineH) }, gap, wrap: portrait, align: "center", cross: portrait ? "start" : "end", padding: { left: 0, right: 0, top: u(8), bottom: 0 }, part: "Lobby.lineup", partClass: "panel" });
  members.forEach((m, i) => {
    const key = m.key || `m${i}`;
    const k = big(m);
    const w = Math.round(cw * k);
    const h = Math.round(ch * k);
    const ix = lastInteraction(key);
    const wasReady = kit.memory(`~l.r.${key}`, m.ready ? 1 : 0) === 1;
    kit.remember(`~l.r.${key}`, m.ready ? 1 : 0);
    const popped = m.ready && !wasReady;
    const sub = [(m.level ?? 0) > 0 ? fill(desc.levelLabel ?? "Lv {level}", { level: m.level ?? 0 }) : "", m.title ?? ""].filter(Boolean).join(" · ");
    const readyWord = m.status || (m.ready ? desc.readyLabel ?? "Ready" : desc.notReadyLabel ?? "Not ready");
    const r = surface(key, {
      part: "Lobby.member",
      cls: "card",
      flags: [m.you ? "you" : "", m.leader ? "leader" : "", m.ready ? "ready" : ""],
      fill: null,
      width: px(w),
      height: px(h),
      radius: Math.max(6, t.panel),
      lift: -u(6),
      semantics: { role: "button", name: [m.name, sub, m.leader ? desc.leaderLabel ?? "Party leader" : "", readyWord].filter(Boolean).join(", ") },
      props: {
        ...(since < 2 && !still() ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "y", value: u(40) }], transition: spring("panel"), delayMs: 80 * i } } : {}),
        states: [{ name: "ready", states: [vstate("waiting"), vstate("ready")], transitions: [] }],
        state: [{ group: "ready", state: m.ready ? "ready" : "waiting" }],
      },
    }, () => {
      // A card too short for its whole plate at a large text size keeps the name and the ready word (names win over
      // marks); the level and title stay in its spoken name.
      // The line under the name wraps rather than being cut; where its lines would take the plate up past the crown's
      // corner, it is dropped too. The plate's height with it is the one laid when it was last shown at this width.
      const plateH = kit.memory(`~l.pw.${key}`, 0) === w ? kit.memory(`~l.ph.${key}`, 0) : 0;
      const tight = h < full || plateH > Math.min(h - u(44), h * 0.6);
      figureCard({ texture: m.portrait, name: m.name ?? "", sub: tight ? "" : sub, width: w, height: h, big: maxW >= u(180), lit: !!m.you, hot: ix.hovered || ix.focused, dim: false, part: "Lobby.member" }, () => {
        // The ready foot: a tick in success and the word, or the muted word (a status in its place).
        kit.row("Foot", { width: grow(), gap: u(6), cross: "center", padding: { left: 0, right: 0, top: u(6), bottom: 0 }, part: `Lobby.foot${flags(m.ready && "ready")}`, partClass: "badge" }, () => {
          if (m.ready && !m.status) symbol("Tick", "check", u(14), role("success"));
          text("Word", readyWord, { uppercase: true, step: st(13), font: "display", width: grow(), color: m.ready && !m.status ? mix(role("success"), role("ink"), 0.25) : "inkMuted" });
        });
        if (popped && !still()) kit.props({ enter: { values: [{ prop: "scale", value: 1.25 }], transition: spring("emphasis") } });
      });
      // The plate as laid last tick is its height with the line only when the line was shown then; else without it.
      const shownLast = kit.memory(`~l.ps.${key}`, 0) === 1;
      const laid = kit.bounds(kit.lastPath())?.height ?? 0;
      if (!shownLast) leanNow = Math.max(leanNow, laid);
      const laidPlate = shownLast ? laid : 0;
      const withSub = laidPlate > 0 ? laidPlate : plateH;
      kit.remember(`~l.ph.${key}`, withSub);
      kit.remember(`~l.pw.${key}`, withSub > 0 ? w : 0);
      kit.remember(`~l.ps.${key}`, tight ? 0 : 1);
      if (m.ready && !m.status) {
        // A ready card's foot carries a band of success light along its bottom edge.
        kit.open("Band", { width: grow(), height: px(u(4)) });
        kit.setDraw({ shape: { kind: "rect", radii: [0, 0, Math.max(6, t.panel), Math.max(6, t.panel)] }, fills: [{ kind: "solid", color: token("success") }], effects: [glow(u(8), token("success", 0.5), 0)] });
        kit.close();
        over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 3 });
      }
      if (m.leader) {
        crown("Crown", u(30), desc.leaderLabel ?? "Party leader");
        over({ insets: { start: u(10), top: u(10) }, inert: true, z: 3 });
      }
    });
    if (r.clicked) {
      out.member = key;
      kit.feedback("select", -1);
    }
  });
  for (let s = 0; s < seats - members.length; s++) {
    // An empty seat: a well with a plus and the invite word, the size of a member's card.
    const r = surface(kit.idi("Seat", s), {
      part: "Lobby.seat",
      cls: "card",
      fill: { ...role("shadow"), a: 0.35 },
      washBy: 0.12,
      direction: "column",
      width: px(cw),
      height: px(ch),
      align: "center",
      cross: "center",
      gap: u(10),
      radius: Math.max(6, t.panel),
      semantics: { role: "button", name: desc.inviteLabel ?? "Invite" },
    }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [Math.max(6, t.panel), Math.max(6, t.panel), Math.max(6, t.panel), Math.max(6, t.panel)] }, fills: [{ kind: "solid", color: token("shadow", 0.35) }], strokes: [line(u(1.5), token("ink", 0.22), "inside", { dash: u(6), gap: u(5) })], effects: [{ kind: "innerShadow", offset: { x: 0, y: u(2) }, blur: u(10), spread: 0, color: token("shadow", 0.5) }] });
      kit.open("Plus", { width: px(u(48)), height: px(u(48)), align: "center", cross: "center" });
      kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("ink", 0.08) }], strokes: [line(u(1.5), token("ink", 0.3))] });
      symbol("Mark", "plus", u(20), role("ink"), { weight: Math.max(2, u(2.4)) });
      kit.close();
      // Narrower than its word at a large text size, the seat cuts the word rather than laying it over the next card.
      text("Label", (desc.inviteLabel ?? "Invite"), { uppercase: true, step: st(14), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: "center" });
    });
    if (r.clicked) {
      out.invite = s;
      kit.feedback("select", -1);
    }
  }
  kit.close();
  kit.semantics({ role: "list", name: desc.title ?? "Party" });
  kit.close();
  rememberPath(`${id}~lineup`, kit.lastPath());
  kit.remember(`~l.lean.${id}`, leanNow || leanLaid);

  // The foot: the mode card at the start, the one call to action at the end (under the thumb on a phone).
  kit.open("Bar", { direction: portrait ? "column" : "row", width: grow(), gap: u(phone ? 10 : 18), cross: portrait ? "stretch" : "center", part: "Lobby.bar", partClass: "panel" });
  const mode = desc.mode ?? {};
  const modeR = surface("Mode", {
    part: "Lobby.mode",
    cls: "card",
    fill: { ...role("shadow"), a: 0.55 },
    washBy: 0.1,
    gap: u(14),
    padding: { left: u(8), right: u(18), top: u(8), bottom: u(8) },
    radius: Math.max(6, t.panel),
    width: portrait ? grow() : null,
    semantics: { role: "button", name: [desc.modeLabel ?? "Mode", mode.name ?? "", mode.detail ?? ""].filter(Boolean).join(", ") },
  }, () => {
    const tw = u(phone ? 96 : 128);
    artSlot("Art", "Lobby.modeArt", { texture: mode.image ?? null, kind: "thumbnail", width: px(tw), height: px(Math.round(tw * 0.5625)), radius: Math.max(4, t.small) });
    kit.column("Words", { gap: u(2), width: portrait ? grow() : null }, () => {
      text("Eyebrow", (desc.modeLabel ?? "Mode"), { uppercase: true, step: "caption", font: "display", color: "inkMuted", nowrap: true });
      text("Name", mode.name ?? "", { step: st(phone ? 18 : 22), font: "display", heading: true, nowrap: true, ellipsis: true });
      // Across a phone's width it wraps; beside the action the card takes its words' width.
      if (mode.detail) text("Detail", mode.detail, { step: st(14), color: "inkMuted", ...(portrait ? { width: grow() } : { nowrap: true }) });
    });
  });
  if (modeR.clicked && !desc.searching) {
    out.mode = true;
    kit.feedback("select", -1);
  }
  if (!portrait) kit.spacer("Gap", grow());
  const actionW = portrait ? grow() : px(u(280));
  if (desc.searching) {
    if (btn("Action", desc.cancelLabel ?? "Cancel", { variant: "secondary", size: "lg", width: actionW, part: "Lobby.action[cancel]", sym: "cross" }).clicked) {
      out.cancel = true;
      kit.feedback("back", -1);
    }
  } else if (desc.canStart) {
    if (btn("Action", desc.startLabel ?? "Find match", { variant: "primary", size: "lg", width: actionW, part: "Lobby.action[start]", raised: true }).clicked) {
      out.start = true;
      kit.feedback("confirm", -1);
    }
  } else if (me) {
    // A member's toggle: Ready up in the accent; once ready, the quieter Ready with its tick (pressed again, not ready).
    if (btn("Action", me.ready ? desc.readyLabel ?? "Ready" : desc.readyUpLabel ?? "Ready up", { variant: me.ready ? "secondary" : "primary", size: "lg", width: actionW, part: `Lobby.action[ready]${me.ready ? "[on]" : ""}`, ...(me.ready ? { sym: "check" as const } : {}), selected: !!me.ready, raised: !me.ready }).clicked) {
      out.ready = true;
      kit.feedback(me.ready ? "toggle" : "confirm", -1);
    }
  }
  kit.close();

  // The ready check: over everything, on a scrim; the keys answer it whatever has the focus.
  if (check) {
    const total = (check.seconds ?? 0) > 0 ? check.seconds! : 10;
    const left = clamp(check.secondsLeft ?? 0, 0, total);
    const was = kit.memory("~l.check", 0);
    if (was === 0) {
      kit.remember("~l.check", tick);
      kit.feedback("open", -1);
    }
    const answered = me?.accepted ?? false;
    const everyone = members.length > 0 && members.every((m) => m.accepted);
    if (everyone && kit.memory("~l.all", 0) === 0) {
      kit.remember("~l.all", 1);
      kit.feedback("success", -1);
    }
    kit.open("Check", { direction: "column", width: grow(), height: grow(), align: "center", cross: "center", part: "Lobby.scrim", partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow", 0.74) }] });
    kit.open("Panel", { direction: "column", width: px(Math.min(W - u(24), u(560))), gap: u(18), cross: "center", padding: { left: u(32), right: u(32), top: u(30), bottom: u(28) }, part: "Lobby.check", partClass: "dialog" });
    kit.setDraw({ shape: { kind: "rect", radii: [u(16), u(16), u(16), u(16)] }, fills: [vgrad([[0, token("panelRaised")], [1, token("panel")]])], strokes: [line(1, token("ink", 0.12))], effects: [shadow(u(16), u(48), 0.65)] });
    if (tick - (was || tick) < 2 && !still()) kit.props({ enter: { values: [{ prop: "opacity", value: 0 }, { prop: "scale", value: 0.86 }], transition: spring("emphasis") } });
    text("Title", desc.matchFoundLabel ?? "Match found", { step: phone ? "xl4" : "displayMd", font: "display", heading: true, align: "center", width: grow(), maxLines: 2 });
    if (mode.name) text("Mode", [mode.name, mode.detail ?? ""].filter(Boolean).join(" · "), { step: st(16), color: "inkMuted", align: "center", width: grow(), maxLines: 2 });
    timerRing("Ring", left / total, u(96), String(Math.ceil(left)));
    kit.row("Pips", { gap: u(8), cross: "center", align: "center", wrap: true }, () => {
      members.forEach((m, i) => {
        const state = m.declined ? "declined" : m.accepted ? "accepted" : "pending";
        const c = m.declined ? role("danger") : m.accepted ? role("success") : role("ink");
        kit.open(m.key || `m${i}`, { width: px(u(34)), height: px(u(10)), part: `Lobby.pip${flags(state !== "pending" && state)}`, partClass: "badge" });
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: paint(c, state === "pending" ? 0.16 : 1) }], ...(state === "accepted" ? { effects: [glow(u(6), paint(c, 0.5), 0)] } : {}) });
        kit.close();
        kit.props({ states: [{ name: "answer", states: [vstate("pending"), vstate("accepted"), vstate("declined")], transitions: [] }], state: [{ group: "answer", state }] });
        kit.semantics({ role: "status", name: `${m.name}: ${state}` });
      });
    });
    kit.row("Actions", { width: grow(), gap: u(12), cross: "center", align: "center" }, () => {
      const a = btn("Accept", answered ? desc.acceptedLabel ?? "Accepted" : desc.acceptLabel ?? "Accept", { variant: "primary", size: "lg", width: grow(), disabled: answered, part: `Lobby.accept${answered ? "[on]" : ""}`, ...(answered ? { sym: "check" as const } : {}), raised: true });
      if (a.clicked && !answered) {
        out.accept = true;
        kit.feedback("confirm", -1);
      }
      if (!answered && btn("Decline", desc.declineLabel ?? "Decline", { variant: "secondary", size: "lg", part: "Lobby.decline" }).clicked) {
        out.decline = true;
        kit.feedback("back", -1);
      }
    });
    kit.close();
    kit.semantics({ role: "dialog", name: desc.matchFoundLabel ?? "Match found" });
    kit.close();
    over({ insets: { start: -vp.safeArea.left, top: -vp.safeArea.top, end: -vp.safeArea.right, bottom: -vp.safeArea.bottom }, z: 20 });
    if (!answered && kit.action("confirm") && !out.accept) {
      out.accept = true;
      kit.feedback("confirm", -1);
    }
    if (!answered && kit.back()) {
      out.decline = true;
      kit.feedback("back", -1);
    }
  } else {
    kit.remember("~l.check", 0);
    kit.remember("~l.all", 0);
  }
  kit.close();
  rememberRoom("l", id);
  return out;
}

/** A player's tag in its team's colour (P1, Mira): ink on the colour, outlined so it reads on any art. */
function playerTag(id: string, name: string, team: Rgba, big = false): void {
  const ink = inkOn(team);
  kit.open(id, { direction: "row", padding: xy(u(big ? 10 : 6), u(big ? 3 : 1)), part: "Select.tag", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: paint(team) }], strokes: [line(1, token("shadow", 0.6), "outside")], effects: [shadow(2, 4, 0.45)] });
  text("Text", name, { step: st(big ? 15 : 12), font: "display", color: ink, nowrap: true });
  kit.close();
}

/** How a roster tile stands: who is on it, whether a press is refused, whether it is banned. */
interface TileState {
  /** The pattern's part prefix (Select, Draft). */
  part: string;
  width: number;
  height: number;
  state: string;
  blocked: boolean;
  banned: boolean;
  tags: Array<{ name: string; team: number; confirmed: boolean }>;
  shake: number;
  name: string;
}

/**
 * A roster tile: the portrait filling it, the name on a scrim at its foot,
 * a cursor's rim and glow in its player's team colour with the player's tag
 * at the top end, a confirmed pick's tick at the top start; locked, the art
 * dark behind a lock; taken, dimmed; banned, dimmed under a no-entry mark in its corner.
 */
function rosterTile(k: string, e: Api.RosterEntry, d: TileState): Api.Response {
  const { t } = frame();
  const ix = lastInteraction(k);
  const rim = d.tags[0] ? teamColor(d.tags[0].team) : null;
  return surface(k, {
    part: `${d.part}.tile`,
    cls: "slot",
    flags: [d.state === "open" ? "" : d.state],
    fill: null,
    direction: "column",
    width: px(d.width),
    height: px(d.height),
    radius: Math.max(4, t.small),
    lift: d.blocked ? 0 : -u(3),
    semantics: { role: "button", name: d.name, disabled: d.blocked },
    props: {
      transform: { x: d.shake },
      states: [{ name: "pick", states: [vstate("open"), vstate("cursor"), vstate("confirmed"), vstate("locked"), vstate("taken"), vstate("banned")], transitions: [] }],
      state: [{ group: "pick", state: d.state }],
    },
  }, () => {
    const rr = Math.max(4, t.small);
    const radii: [number, number, number, number] = [rr, rr, rr, rr];
    kit.setDraw({
      shape: { kind: "rect", radii },
      fills: [vgrad([[0, token("panelRaised")], [1, token("panel")]])],
      strokes: rim ? [line(u(3), paint(rim))] : (ix.hovered || ix.focused) && !d.blocked ? [line(u(1.5), paint(mix(role("focus"), role("ink"), 0.4)))] : [line(1, token("shadow", 0.55), "outside")],
      effects: [shadow(u(3), u(8), 0.45), ...(rim ? [glow(u(7), paint(rim, 0.5), 0)] : [])],
    });
    artSlot("Art", `${d.part}.art`, { texture: e.portrait ?? null, kind: "portrait", width: px(d.width), height: px(d.height), radius: radii, focal: { x: 0.5, y: 0.3 } }, () => {
      if (d.blocked) {
        kit.open("Dim", { width: grow(), height: grow() });
        // Banned or taken: the portrait still reads at about 35 % under the shade. Locked: a grey well, darker.
        const gone = !e.locked && (d.banned || d.state === "taken");
        kit.setDraw({ shape: { kind: "rect", radii }, fills: gone ? [{ kind: "solid", color: token("shadow", 0.65) }] : [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.2), 0.6) }, { kind: "solid", color: token("shadow", e.locked ? 0.45 : 0.3) }] });
        kit.close();
      }
    });
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
    kit.open("Shade", { width: grow(), height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii }, fills: [scrimFill("bottom", 0.85)] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
    text("Name", e.name ?? k, { step: st(d.width >= u(110) ? 14 : 12), font: "display", nowrap: true, ellipsis: true, width: px(d.width - u(12)), align: "center", color: d.blocked ? "inkMuted" : "ink" });
    over({ insets: { centerX: true, bottom: u(6) }, inert: true, z: 2 });
    if (e.locked) {
      symbol("Lock", "lock", u(22), role("ink"), { icon: "lock" });
      over({ insets: { centerX: true, centerY: true }, inert: true, z: 2 });
    }
    // A ban is a small mark in the corner over the dimmed portrait; the red slash is the ban row's own.
    if (d.banned) banMark("Ban", Math.round(Math.max(u(22), Math.min(d.width, d.height) * 0.22)));
    if (d.tags.length > 0) {
      // The cursors' tags at the top end; a confirmed pick's tick at the top start.
      kit.row("Tags", { gap: u(2) }, () => d.tags.forEach((c, i) => playerTag(kit.idi("Tag", i), c.name, teamColor(c.team))));
      over({ insets: { end: u(4), top: u(4) }, inert: true, z: 3 });
      if (d.tags.some((c) => c.confirmed)) {
        kit.open("Tick", { width: px(u(22)), height: px(u(22)), align: "center", cross: "center" });
        kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("success") }], strokes: [line(1.5, token("shadow", 0.6), "outside")] });
        symbol("Mark", "check", u(13), inkOn(role("success")));
        kit.close();
        over({ insets: { start: u(4), top: u(4) }, inert: true, z: 3 });
      }
    }
  });
}

/** A banned tile's mark at its top start: the no-entry sign (a ring and its bar) in danger on a dark disc. */
function banMark(id: string, size: number): void {
  const c = size / 2;
  const r = size * 0.3;
  const k = r * Math.SQRT1_2;
  kit.open(id, { width: px(size), height: px(size) });
  kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("shadow", 0.75) }] });
  kit.open("Ring", { width: px(size), height: px(size) });
  kit.setDraw({ shape: { kind: "path", points: Array.from({ length: 25 }, (_, i) => ({ x: c + r * Math.cos((i / 24) * Math.PI * 2), y: c + r * Math.sin((i / 24) * Math.PI * 2) })), width: Math.max(1.5, size * 0.1), cap: "round" }, fills: [{ kind: "solid", color: token("danger") }] } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  kit.open("Bar", { width: px(size), height: px(size) });
  kit.setDraw({ shape: { kind: "path", points: [{ x: c - k, y: c - k }, { x: c + k, y: c + k }], width: Math.max(1.5, size * 0.1), cap: "round" }, fills: [{ kind: "solid", color: token("danger") }] } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true });
  kit.close();
  over({ insets: { start: u(4), top: u(4) }, inert: true, z: 3 });
}

/** A ban's slash across a tile, corner to corner, in danger with a dark edge. */
function slash(id: string, w: number, h: number): void {
  kit.open(id, { width: px(w), height: px(h) });
  kit.setDraw({ shape: { kind: "path", points: [{ x: w * 0.18, y: h * 0.82 }, { x: w * 0.82, y: h * 0.18 }], width: Math.max(3, u(4)), cap: "round" }, fills: [{ kind: "solid", color: token("danger") }], effects: [{ kind: "dropShadow", offset: { x: 0, y: 0 }, blur: 0, spread: u(1.5), color: token("shadow", 0.8) }] } as Api.DrawDesc);
  kit.close();
  over({ insets: { start: 0, top: 0 }, inert: true, z: 2 });
}

/** A splash's name step: the display face, a step smaller on a narrow splash. */
function splashName(width: number): Api.TypeStepName {
  return width >= u(320) ? "displayMd" : "xl4";
}

/**
 * A side's splash: the character's art filling it, a scrim at its foot under the role, the name and the description.
 * The words take what the splash leaves under the player's tag, from the type ramp: the name first (a second line only
 * where it wraps and there is room), then the role, then as many lines of the description as fit, up to three. A splash
 * too short for the tag over the name drops the tag (the tile carries it, the action says Locked in). At 200 % text the
 * words of a 92 px phone splash ran 300 px up over the screen's header and the timer.
 */
function splashPanel(id: string, e: Api.RosterEntry | null, c: Api.SelectCursor, side: "start" | "end", width: number, height: number, desc: Api.CharacterSelectDesc): void {
  const { t } = frame();
  const team = teamColor(c.team ?? 0);
  const key = keyHash(`${c.key}|${e?.key ?? ""}`);
  const was = kit.memory(`~s.sp.${c.key}`, 0);
  kit.remember(`~s.sp.${c.key}`, key);
  const changed = was !== key && !still();
  const conf = kit.memory(`~s.cf.${c.key}`, 0);
  kit.remember(`~s.cf.${c.key}`, c.confirmed ? 1 : 0);
  const justConfirmed = !!c.confirmed && conf === 0 && !still();
  const r = Math.max(6, t.panel);
  const radii: [number, number, number, number] = [r, r, r, r];
  const lineOf = (s: Api.TypeStepName): number => stepSize(s).line;
  const nameStep = splashName(width);
  const nameL = lineOf(nameStep);
  const tagH = Math.max(lineOf(st(15)) + 2 * u(3), c.confirmed ? lineOf(st(12)) + 2 * u(3) : 0);
  let room = height - u(18);
  const tagged = room - (u(14) + tagH + u(8)) >= nameL;
  if (tagged) room -= u(14) + tagH + u(8);
  // Whether the name wraps, by an estimate of the face's advance (0.6 em a character).
  const wraps = [...(e?.name ?? "")].length * stepSize(nameStep).size * 0.6 > width - u(40);
  const nameLines = wraps && room >= 2 * nameL ? 2 : 1;
  room -= nameLines * nameL;
  const roleL = lineOf("caption") + u(4);
  const showRole = !!e?.role && room >= roleL;
  if (showRole) room -= roleL;
  const descLines = e?.description ? clamp(Math.floor((room - u(4)) / lineOf(st(15))), 0, 3) : 0;
  kit.open(id, { direction: "column", width: px(width), height: px(height), part: `Select.splash${flags(c.you ? "you" : "rival", c.confirmed && "confirmed")}`, partClass: "card" });
  kit.setDraw({ shape: { kind: "rect", radii }, fills: [vgrad([[0, paint(mix(role("panel"), team, 0.28))], [1, token("panel")]])], effects: [shadow(u(10), u(28), 0.5)] });
  artSlot("Art", "Select.art", { texture: e?.splash ?? e?.portrait ?? null, kind: "splash", width: px(width), height: px(height), radius: radii, focal: { x: 0.5, y: 0.28 } });
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, ...(changed ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "x", value: (side === "end") !== rtl() ? u(40) : -u(40) }], transition: spring("panel") } } : {}) });
  // The team's light from the side edge, and the scrim at the foot that holds the words.
  kit.open("Shade", { width: grow(), height: grow() });
  kit.setDraw({ shape: { kind: "rect", radii }, fills: [scrimFill("bottom", 0.92), vgrad([[0, paint(team, 0.22)], [0.4, paint(team, 0)]], side === "start" ? 270 : 90)] });
  kit.close();
  over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
  kit.open("Words", { direction: "column", width: grow(), gap: u(4), padding: { left: u(20), right: u(20), top: 0, bottom: u(18) }, cross: side === "end" ? "end" : "start" });
  if (showRole) eyebrow("Role", e!.role!, mix(team, role("ink"), 0.5));
  text("Name", e?.name ?? "", { step: nameStep, font: "display", heading: true, ...(nameLines > 1 ? { maxLines: 2 } : { nowrap: true, ellipsis: true }), width: grow(), align: side === "end" ? "end" : "start" });
  if (justConfirmed) kit.props({ enter: { values: [{ prop: "scale", value: 1.18 }], transition: spring("emphasis") } });
  if (descLines > 0) text("Description", e!.description!, { step: st(15), color: "inkMuted", maxLines: descLines, width: grow(), align: side === "end" ? "end" : "start" });
  kit.close();
  over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
  if (tagged) kit.row("Head", { gap: u(8), cross: "center" }, () => {
    playerTag("Tag", c.name ?? c.key ?? "", team, true);
    if (c.confirmed) {
      kit.row("Locked", { gap: u(4), cross: "center", padding: xy(u(8), u(3)), part: "Select.lockedIn", partClass: "badge" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("shadow", 0.7) }] });
        symbol("Tick", "check", u(13), role("success"));
        text("Text", (desc.lockedInLabel ?? "Locked in"), { uppercase: true, step: st(12), font: "display", nowrap: true, color: mix(role("success"), role("ink"), 0.3) });
      });
    }
  });
  if (tagged) over(side === "end" ? { insets: { end: u(14), top: u(14) }, inert: true, z: 3 } : { insets: { start: u(14), top: u(14) }, inert: true, z: 3 });
  kit.close();
  kit.semantics({ role: "region", name: `${c.name ?? c.key}: ${e?.name ?? ""}` });
}

/**
 * A character select (Street Fighter 6, Valorant, Smash): see
 * `fn characterSelect` in crates/og-api/api/ui.ogt.
 */
export function characterSelect(id: string, desc: Api.CharacterSelectDesc): Api.CharacterSelectResult {
  const out: Mut<Api.CharacterSelectResult> = { hovered: null, picked: null, random: false, refused: null, cancelled: false };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const roster = desc.roster ?? [];
  const cursors = desc.cursors ?? [];
  const me = cursors.find((c) => c.you) ?? cursors[0] ?? null;
  const rival = cursors.find((c) => c !== me) ?? null;
  const byKey = new Map(roster.map((e) => [e.key ?? "", e]));
  const ts = Math.max(1, frame().theme.textScale ?? 1);
  const W = kit.memory(`~s.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~s.h.${id}`, 0) || firstRoom().h;
  const rt = kit.memory("~s.rt", 0);
  const rk = kit.memory("~s.rk", 0);

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 12 : 20), part: "Select.frame", partClass: "panel" });
  kit.row("Head", { width: grow(), gap: u(16), cross: "center" }, () => {
    if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
    if ((desc.secondsLeft ?? -1) >= 0) {
      const s = Math.ceil(desc.secondsLeft ?? 0);
      kit.row("Timer", { gap: u(6), cross: "center", padding: xy(u(12), u(6)), part: `Select.timer${flags(s <= 5 && "urgent")}`, partClass: "chip" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: s <= 5 ? token("danger", 0.85) : token("shadow", 0.6) }] });
        symbol("Clock", "clock", u(16), role("ink"), { icon: "clock" });
        text("Time", String(s), { step: st(20), font: "mono", nowrap: true });
      });
      kit.semantics({ role: "timer", name: String(s) });
    }
  });
  rememberPath(`${id}~head`, kit.lastPath());

  // Sizes: the splashes beside the grid (a third of the width each with a rival, two fifths alone), above it on a phone in portrait.
  // The stage is exactly what the head and the foot leave (as they were laid last tick, else an estimate), so the foot's
  // action stays inside the room whatever the text size (at 200 % a stage of at least 200 px put Lock in under the
  // phone's safe area).
  const headH = kit.bounds(recallPath(`${id}~head`))?.height ?? u(60);
  const footH = kit.bounds(recallPath(`${id}~foot`))?.height ?? Math.round(u(phone ? 52 : 60) * Math.min(1.5, ts));
  const stageH = Math.max(u(60), H - headH - footH - u(phone ? 24 : 40));
  const splashW = portrait ? W : Math.round(W * (rival ? 0.3 : 0.4));
  const gridW = portrait ? W : W - splashW * (rival ? 2 : 1) - u(24) * (rival ? 2 : 1);
  const gap = u(phone ? 6 : 10);
  const cols = desc.columns && desc.columns > 0 ? desc.columns : Math.max(portrait ? 4 : 3, Math.floor((gridW - u(12) + gap) / (u(96) + gap)));
  const tw = Math.floor((gridW - u(12) - gap * (cols - 1)) / cols);
  const th = Math.round(tw * 1.18);
  // On a phone the splash keeps at least its name's line and leaves the grid under it a whole row of tiles; where the
  // stage holds not both it stands down and the grid takes the stage (the pick's name is on its tile, Lock in under it):
  // at 200 % text under a tall header a display name's line left the grid a 29 px strip.
  const splashMin = stepSize(splashName(W)).line + u(26);
  const splashH = !portrait ? stageH : stageH - splashMin - u(12) >= th + u(12) ? Math.min(stageH - th - u(24), Math.max(Math.round(stageH * 0.46), splashMin)) : 0;

  kit.open("Stage", { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(portrait ? 12 : 24), part: "Select.stage", partClass: "panel" });
  if (me && splashH > 0) splashPanel("Start", byKey.get(me.entry ?? "") ?? null, me, "start", splashW, splashH, desc);
  // The roster: whole rows, scrolling when they outgrow the room.
  const tiles = [...roster.map((e) => e.key ?? ""), ...(desc.randomLabel ? ["Random"] : [])];
  const rows = Math.ceil(tiles.length / cols);
  const roomH = portrait ? stageH - (me && splashH > 0 ? splashH + u(12) : 0) : stageH;
  // Whole rows within the room less the roster's padding; a room under one row (a phone at 200 % text) shows the row it
  // can, cut, rather than pushing the foot off.
  const whole = Math.floor((roomH - u(12) + gap) / (th + gap)) * (th + gap) - gap;
  const wholeH = Math.min(rows * (th + gap) - gap, whole >= th ? whole : Math.max(0, roomH - u(12)));
  let hot = "";
  kit.openScroll("Roster", { width: px(gridW), height: px(wholeH + u(12)), gap, padding: xy(u(6), u(6)), part: "Select.roster" } as never);
  for (let row = 0; row < rows; row++) {
    kit.row(kit.idi("Row", row), { gap, width: grow() }, () => {
      for (let cI = 0; cI < cols; cI++) {
        const k = tiles[row * cols + cI];
        if (k === undefined) break;
        if (k === "Random" && desc.randomLabel) {
          const r = surface("Random", { part: "Select.tile[random]", cls: "slot", fill: { ...role("shadow"), a: 0.5 }, direction: "column", width: px(tw), height: px(th), align: "center", cross: "center", gap: u(6), radius: Math.max(4, t.small), semantics: { role: "button", name: desc.randomLabel } }, () => {
            symbol("Mark", "rotate", u(28), role("inkMuted"));
            text("Label", desc.randomLabel!, { uppercase: true, step: st(12), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: "center" });
          });
          if (r.clicked && !me?.confirmed) {
            out.random = true;
            kit.feedback("confirm", -1);
          }
          continue;
        }
        const e = byKey.get(k)!;
        const on = cursors.filter((c) => c.entry === k);
        const mine = !!me && me.entry === k;
        // Taken stays taken under the local cursor (it may rest there); only the player's own locked-in pick, which the
        // game may list as taken, is not refused.
        const own = mine && !!me?.confirmed;
        const blocked = !!e.locked || (!!e.taken && !own);
        const state = e.locked ? "locked" : e.taken && !own ? "taken" : on.some((c) => c.confirmed) ? "confirmed" : on.length > 0 ? "cursor" : "open";
        const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
        const ix = lastInteraction(k);
        const r = rosterTile(k, e, {
          part: "Select",
          width: tw,
          height: th,
          state,
          blocked,
          banned: false,
          tags: on.map((c) => ({ name: c.name ?? c.key ?? "", team: c.team ?? 0, confirmed: !!c.confirmed })),
          shake: shaking,
          name: [e.name ?? k, e.role ?? "", e.locked ? e.lockedHint || "locked" : e.taken && !own ? desc.takenLabel ?? "Taken" : "", ...on.map((c) => c.name ?? c.key ?? "")].filter(Boolean).join(", "),
        });
        if (ix.hovered) hot = k;
        else if (ix.focused && !hot) hot = k;
        if (r.clicked) {
          if (blocked || me?.confirmed) {
            if (blocked) {
              out.refused = k;
              kit.feedback("error", -1);
            }
          } else {
            out.picked = k;
            kit.feedback("confirm", -1);
          }
        }
        // The local cursor is the focus: on the first tick it lands on the cursor's tile.
        if (mine && kit.memory(`~s.f.${id}`, 0) === 0) kit.setFocus(kit.lastPath());
      }
    });
  }
  kit.close();
  // The cursor follows the tile the pointer or the focus moves onto: asked once, when that tile changes, so a cursor the
  // game moves by other means (a pad's stick, a random pick) is not pulled back to a focus that stayed where it was.
  const hotHash = hot ? keyHash(hot) : 0;
  if (hot && hotHash !== kit.memory(`~s.hot.${id}`, 0) && !me?.confirmed && me?.entry !== hot) out.hovered = hot;
  kit.remember(`~s.hot.${id}`, hotHash);
  kit.remember(`~s.f.${id}`, me?.entry ? 1 : 0);
  kit.semantics({ role: "grid", name: desc.title ?? "Roster" });
  if (rival && !portrait) splashPanel("End", byKey.get(rival.entry ?? "") ?? null, rival, "end", splashW, splashH, desc);
  kit.close();
  if (out.refused) {
    kit.remember("~s.rt", tick);
    kit.remember("~s.rk", keyHash(out.refused));
  }

  kit.row("Foot", { width: grow(), gap: u(12), cross: "center", align: "end" }, () => {
    const conf = !!me?.confirmed;
    const cur = me ? byKey.get(me.entry ?? "") : undefined;
    const can = !!cur && !cur.locked;
    const r = btn("Lock", conf ? desc.lockedInLabel ?? "Locked in" : desc.lockLabel ?? "Lock in", { variant: conf ? "secondary" : "primary", size: "lg", width: portrait ? grow() : px(u(260)), disabled: !can && !conf, part: `Select.confirm${conf ? "[on]" : ""}`, ...(conf ? { sym: "check" as const, selected: true } : { raised: true }) });
    if (r.clicked && !conf && cur && can) {
      out.picked = cur.key ?? null;
      kit.feedback("confirm", -1);
    }
  });
  rememberPath(`${id}~foot`, kit.lastPath());
  if (me?.confirmed && kit.back()) {
    out.cancelled = true;
    kit.feedback("back", -1);
  }
  kit.close();
  rememberRoom("s", id);
  return out;
}

/**
 * A versus screen (Street Fighter, Tekken, a MOBA's loading screen): see
 * `fn versus` in crates/og-api/api/ui.ogt.
 */
export function versus(id: string, desc: Api.VersusDesc): Api.VersusResult {
  const out: Mut<Api.VersusResult> = { skipped: false };
  const { phone } = frame();
  const tick = kit.tick();
  const W = kit.memory(`~v.w.${id}`, 0) || safeWidth();
  const H = kit.memory(`~v.h.${id}`, 0) || safeHeight();
  const t0 = kit.memory("~v.t0", -1);
  if (t0 < 0) {
    kit.remember("~v.t0", tick);
    kit.feedback("open", -1);
  }
  const age = tick - (t0 < 0 ? tick : t0);
  const fresh = age < 2 && !still();
  const drift = still() ? 0 : Math.min(1, age / 600) * u(14);
  const portraitish = H > W;

  kit.open(id, { direction: portraitish ? "column" : "row", width: grow(), height: grow(), part: "Versus.frame", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("shadow") }] });
  const side = (key: "Left" | "Right", s: Api.VersusSide, end: boolean): void => {
    const team = teamColor(s.team ?? (end ? 1 : 0));
    const sw = portraitish ? W : Math.round(W / 2);
    const sh = portraitish ? Math.round(H / 2) : H;
    kit.open(key, { direction: "column", width: px(sw), height: px(sh), part: `Versus.side${flags(end ? "right" : "left")}`, partClass: "badge" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, paint(mix(role("shadow"), team, 0.45))], [1, token("shadow")]], end ? 90 : 270)], clip: "shape" });
    kit.props(fresh ? { enter: { values: [{ prop: "x", value: portraitish ? 0 : end ? sw : -sw }, { prop: "y", value: portraitish ? (end ? sh : -sh) : 0 }], transition: spring("panel") } } : {});
    artSlot("Art", "Versus.splash", { texture: s.splash ?? null, kind: "splash", width: px(sw + u(28)), height: px(sh), radius: 0, focal: { x: 0.5, y: 0.25 } });
    over({ insets: { start: -u(14), top: 0 }, inert: true, transform: { x: end ? -drift : drift } });
    kit.open("Shade", { width: grow(), height: grow() });
    kit.setDraw({ shape: { kind: "rect" }, fills: [scrimFill("bottom", 0.9), vgrad([[0, paint(team, 0.32)], [0.5, paint(team, 0)]], end ? 90 : 270)] });
    kit.close();
    over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
    kit.open("Words", { direction: "column", width: grow(), gap: u(2), padding: { left: u(phone ? 18 : 40), right: u(phone ? 18 : 40), top: 0, bottom: u(phone ? 18 : 48) }, cross: end ? "end" : "start" });
    text("Character", (s.character ?? ""), { uppercase: true, step: phone ? "xl5" : "displayLg", font: "display", heading: true, maxLines: 2, width: grow(), align: end ? "end" : "start" });
    kit.row("Player", { gap: u(8), cross: "center", align: end ? "end" : "start", width: grow() }, () => {
      kit.open("Bar", { width: px(u(4)), height: px(u(22)), radius: 2, background: paint(team) });
      kit.close();
      text("Name", s.name ?? "", { step: st(phone ? 16 : 22), font: "display", nowrap: true, ellipsis: true });
    });
    if (s.detail) text("Detail", s.detail, { step: st(14), color: "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: end ? "end" : "start" });
    kit.close();
    over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
    kit.close();
  };
  side("Left", desc.left ?? {}, false);
  side("Right", desc.right ?? {}, true);
  // The seam: a slanted band of light between the sides, the mark on it and the stage under the mark.
  kit.open("Seam", { width: px(u(6)), height: px(Math.round((portraitish ? W : H) * 1.3)) });
  kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("ink", 0.9) }], effects: [glow(u(18), token("ink", 0.45), 0)] });
  kit.close();
  over({ insets: { centerX: true, centerY: true }, inert: true, z: 3, transform: { rotate: portraitish ? Math.PI / 2 - 0.2 : 0.2 } });
  kit.column("Mark", { gap: u(6), cross: "center", part: "Versus.mark", partClass: "badge" }, () => {
    text("Text", desc.versusLabel ?? "VS", { step: phone ? "displayMd" : "displayXl", font: "display", heading: true, nowrap: true });
    if (desc.stage) {
      // The stage's pill is at most the screen's width less a margin; a longer name wraps inside it.
      kit.row("Stage", { width: { kind: "fit", max: Math.max(u(120), W - u(32)) }, padding: xy(u(12), u(4)), part: "Versus.stage", partClass: "badge" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.75) }] });
        text("Text", desc.stage!, { step: st(14), color: "inkMuted", align: "center" });
      });
    }
  });
  over({ insets: { centerX: true, centerY: true }, inert: true, z: 4, ...(fresh ? { enter: { values: [{ prop: "opacity", value: 0 }, { prop: "scale", value: 2.2 }], transition: spring("emphasis"), delayMs: 200 } } : {}) });
  if ((desc.progress ?? -1) >= 0) {
    const f = clamp(desc.progress ?? 0, 0, 1);
    kit.open("Progress", { width: grow(), height: px(u(4)), part: "Versus.progress", partClass: "bar" });
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("ink", 0.12) }] });
    kit.open("Fill", { width: { kind: "percent", value: f * 100 } as never, height: grow() });
    kit.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: token("ink", 0.85) }] });
    kit.close();
    kit.close();
    over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 5 });
    kit.semantics({ role: "progressbar", name: `${Math.round(f * 100)} %`, value: f });
  }
  if (desc.skipLabel !== "") {
    const g = frame().touch ? null : (kit.frameGlyph("confirm") as Api.UiGlyph | null);
    kit.row("Skip", { gap: u(8), cross: "center", padding: xy(u(12), u(6)), part: "Versus.skip", partClass: "badge" }, () => {
      kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.5) }] });
      if (g) text("Glyph", g.text, { step: st(12), font: "mono", color: "inkMuted", nowrap: true });
      text("Text", desc.skipLabel ?? "Skip", { step: st(14), color: "inkMuted", nowrap: true });
    });
    over({ insets: { end: u(20), top: u(20) }, inert: true, z: 6 });
    const p = pointer();
    if (age > 10 && (kit.action("confirm") || kit.back() || p.released)) {
      out.skipped = true;
      kit.feedback("confirm", -1);
    }
  }
  kit.close();
  kit.semantics({ role: "region", name: `${desc.left?.character ?? ""} ${desc.versusLabel ?? "VS"} ${desc.right?.character ?? ""}` });
  rememberRoom("v", id);
  return out;
}

/**
 * A draft with bans (League of Legends, Dota 2): see `fn draft` in
 * crates/og-api/api/ui.ogt.
 */
export function draft(id: string, desc: Api.DraftDesc): Api.DraftResult {
  const out: Mut<Api.DraftResult> = { hovered: null, locked: null, refused: null };
  const { viewport: vp, t, phone } = frame();
  const portrait = vp.breakpoint === "phonePortrait";
  const tick = kit.tick();
  const teams = desc.teams ?? [];
  const roster = desc.roster ?? [];
  const byKey = new Map(roster.map((e) => [e.key ?? "", e]));
  const banned = new Set(teams.flatMap((tm) => (tm.bans ?? []).filter(Boolean)));
  const picked = new Map<string, number>();
  teams.forEach((tm, i) => (tm.picks ?? []).forEach((p) => p.locked && p.entry && picked.set(p.entry, tm.team ?? i)));
  const mine = teams[0];
  const myTeam = mine?.team ?? 0;
  const W = kit.memory(`~d.w.${id}`, 0) || firstRoom().w;
  const H = kit.memory(`~d.h.${id}`, 0) || firstRoom().h;
  const rt = kit.memory("~d.rt", 0);
  const rk = kit.memory("~d.rk", 0);
  const ban = desc.phase === "ban";
  const total = (desc.seconds ?? 0) > 0 ? desc.seconds! : 30;
  const left = clamp(desc.secondsLeft ?? 0, 0, total);
  const urgent = !!desc.yourTurn && left <= 5;
  const secs = Math.ceil(left);
  if (urgent && kit.memory("~d.alert", -1) !== secs) {
    kit.remember("~d.alert", secs);
    kit.feedback("alert", -1);
  }
  const hov = desc.hovered ?? "";

  kit.open(id, { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(phone ? 10 : 20), part: "Draft.frame", partClass: "panel" });

  // A team's column: its picks, then its bans (on a phone in portrait a row of portraits under its name, the bans beside the name).
  const teamColumn = (key: "Start" | "End", tm: Api.DraftTeam | undefined, end: boolean): void => {
    if (!tm) return;
    const team = teamColor(tm.team ?? (end ? 1 : 0));
    const picks = tm.picks ?? [];
    const colW = portrait ? W : Math.round(Math.min(u(300), W * 0.24));
    // Its bans: small tiles struck through, the ones to come empty wells.
    const bans = tm.bans ?? [];
    const banRow = (): void => {
      if (bans.length === 0) return;
      kit.row("Bans", { gap: u(6), cross: "center", align: end && !portrait ? "end" : "start", width: portrait ? null : grow(), part: "Draft.bans", partClass: "badge" }, () => {
        if (!portrait) text("Label", (desc.bansLabel ?? "Bans"), { uppercase: true, step: "caption", font: "display", color: "inkMuted", nowrap: true });
        const bw = u(phone ? 34 : 40);
        bans.forEach((b, i) => {
          const e = b ? byKey.get(b) : undefined;
          kit.open(kit.idi("Ban", i), { width: px(bw), height: px(bw), part: `Draft.ban${flags(!e && "empty")}`, partClass: "slot" });
          const rr = Math.max(3, t.small - 2);
          kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("shadow", 0.5) }], strokes: [line(1, token("ink", e ? 0.1 : 0.18), "inside", e ? {} : { dash: u(4), gap: u(3) })] });
          if (e) {
            artSlot("Art", "Draft.banArt", { texture: e.portrait ?? null, kind: "portrait", width: px(bw), height: px(bw), radius: rr, focal: { x: 0.5, y: 0.3 } }, () => {
              kit.open("Dim", { width: grow(), height: grow() });
              kit.setDraw({ shape: { kind: "rect", radii: [rr, rr, rr, rr] }, fills: [{ kind: "solid", color: token("shadow", 0.45) }] });
              kit.close();
            });
            slash("Slash", bw, bw);
          }
          kit.close();
          kit.semantics({ role: "status", name: e ? `${desc.bannedLabel ?? "Banned"}: ${e.name ?? b}` : desc.bansLabel ?? "Bans" });
        });
      });
    };
    // A column's slots share what its name and its bans leave, from the type ramp, and grow with the text: never shorter
    // than the character's line, as tall as both lines where the room allows, else the player's line drops (it stays in
    // the slot's spoken name). Slots a fixed 104 px tall at 200 % text ran each one's words over the next and the name.
    const lineOf = (s: Api.TypeStepName): number => stepSize(s).line;
    const colGap = u(phone ? 6 : 8);
    const n = Math.max(1, picks.length);
    const bansH = bans.length > 0 ? Math.max(u(40), lineOf("caption")) + colGap : 0;
    const fitH = Math.floor((H - Math.max(u(18), lineOf(st(16))) - colGap - bansH - (n - 1) * colGap) / n);
    const wordsLean = lineOf(st(17)) + 2 * u(8);
    const wordsFull = wordsLean + lineOf(st(13));
    const slotH = portrait ? 0 : Math.max(u(64), wordsLean, Math.min(Math.max(u(104), wordsFull), fitH));
    const withPlayer = slotH >= wordsFull;
    kit.open(key, { direction: "column", width: px(colW), height: portrait ? null : grow(), gap: colGap, cross: end && !portrait ? "end" : "stretch", part: `Draft.team${flags(end ? "end" : "start")}`, partClass: "panel" });
    // On a phone in portrait the bans stand at the end of the team's name row, so both teams leave the roster its room.
    kit.row("Name", { width: grow(), gap: u(8), cross: "center", align: end && !portrait ? "end" : "start" }, () => {
      kit.open("Bar", { width: px(u(4)), height: px(u(18)), radius: 2, background: paint(team) });
      kit.close();
      text("Text", tm.name ?? "", { step: st(16), font: "display", nowrap: true, ellipsis: true, color: mix(team, role("ink"), 0.55), ...(portrait ? { width: grow() } : {}) });
      if (portrait) banRow();
    });
    kit.open("Picks", { direction: portrait ? "row" : "column", width: grow(), gap: u(phone ? 6 : 8) });
    picks.forEach((p, i) => {
      const e = p.entry ? byKey.get(p.entry) : undefined;
      const state = p.locked && e ? "locked" : e ? "hovered" : "empty";
      const was = kit.memory(`~d.l.${key}.${i}`, 0);
      kit.remember(`~d.l.${key}.${i}`, p.locked ? 1 : 0);
      const landed = !!p.locked && was === 0 && !still();
      const sw = portrait ? Math.floor((W - u(6) * (picks.length - 1)) / Math.max(1, picks.length)) : colW;
      // A phone's slots are wider than tall: the face over the character's name, two teams over the roster.
      const sh = portrait ? Math.round(sw * 0.72) : slotH;
      const rr = Math.max(4, t.small);
      const radii: [number, number, number, number] = [rr, rr, rr, rr];
      kit.open(kit.idi("Slot", i), { direction: "column", width: px(sw), height: px(sh), part: `Draft.slot${flags(p.active && "active", p.locked && "locked", p.you && "you")}`, partClass: "card" });
      kit.setDraw({
        shape: { kind: "rect", radii },
        fills: [vgrad([[0, paint(mix(role("panel"), team, 0.18))], [1, token("panel")]])],
        strokes: p.active ? [line(u(2.5), paint(team))] : [line(1, token("shadow", 0.55), "outside")],
        effects: [shadow(u(3), u(10), 0.45), ...(p.active ? [glow(u(8), paint(team, 0.5), 0)] : [])],
      });
      if (e) {
        artSlot("Art", "Draft.slotArt", { texture: e.splash ?? e.portrait ?? null, kind: "portrait", width: px(sw), height: px(sh), radius: radii, focal: { x: 0.5, y: portrait ? 0.3 : 0.25 } }, () => {
          if (!p.locked) {
            // Only hovered: the art at half its light, so a lock reads as the character arriving.
            kit.open("Dim", { width: grow(), height: grow() });
            kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "solid", color: token("shadow", 0.55) }] });
            kit.close();
          }
        });
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true, ...(landed ? { enter: { values: [{ prop: "opacity", value: 0.2 }, { prop: "x", value: end ? u(30) : -u(30) }], transition: spring("panel") } } : {}) });
      }
      kit.open("Shade", { width: grow(), height: grow() });
      kit.setDraw({ shape: { kind: "rect", radii }, fills: [portrait ? scrimFill("bottom", 0.9) : scrimFill(end ? "end" : "start", 0.88)] });
      kit.close();
      over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
      if (p.active && !still()) {
        // The active slot's light sweeping along it.
        const f = (tick % 90) / 90;
        kit.open("Light", { width: grow(), height: grow() });
        kit.setDraw({ shape: { kind: "rect", radii }, clip: "shape" } as Api.DrawDesc);
        kit.open("Sweep", { width: px(Math.round(sw * 0.35)), height: grow() });
        kit.setDraw({ shape: { kind: "rect" }, fills: [vgrad([[0, paint(team, 0)], [0.5, paint(team, 0.28)], [1, paint(team, 0)]], 90)] });
        kit.close();
        over({ insets: { start: Math.round(sw * 1.35 * f - sw * 0.35), top: 0, bottom: 0 }, inert: true });
        kit.close();
        over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
      }
      kit.column("Words", { gap: 0, width: grow(), padding: xy(u(portrait ? 4 : 12), u(portrait ? 4 : 8)), cross: end && !portrait ? "end" : "start" }, () => {
        if (!portrait || e) text("Character", e?.name ?? "", { step: st(portrait ? 11 : 17), font: "display", nowrap: true, ellipsis: true, width: grow(), align: end && !portrait ? "end" : "start", color: p.locked ? "ink" : "inkMuted" });
        if (!portrait && withPlayer) text("Player", p.player ?? "", { step: st(13), color: p.you ? mix(team, role("ink"), 0.5) : "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: end ? "end" : "start" });
      });
      over(portrait ? { insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 2 } : { insets: { start: 0, end: 0, centerY: true }, inert: true, z: 2 });
      kit.close();
      kit.props({ states: [{ name: "slot", states: [vstate("empty"), vstate("hovered"), vstate("locked")], transitions: [] }], state: [{ group: "slot", state }], ...(landed ? { enter: { values: [{ prop: "scale", value: 1.06 }], transition: spring("emphasis") } } : {}) });
      kit.semantics({ role: "listitem", name: [p.player ?? "", e?.name ?? "", state].filter(Boolean).join(", ") });
    });
    kit.close();
    kit.semantics({ role: "list", name: tm.name ?? "" });
    if (!portrait) banRow();
    kit.close();
  };
  if (!portrait) teamColumn("Start", teams[0], false);
  else {
    kit.row("Teams", { width: grow(), gap: u(10) }, () => {
      kit.column("Wrap", { width: grow(), gap: u(8) }, () => {
        teamColumn("Start", teams[0], false);
        teamColumn("End", teams[1], true);
      });
    });
  }

  // The middle: the phase banner and its timer, the roster, the action.
  kit.open("Middle", { direction: "column", width: grow(), height: grow(), gap: u(phone ? 8 : 14), cross: "stretch" });
  const bannerInk = ban ? role("danger") : role("ink");
  kit.column("Banner", { width: grow(), gap: u(8), padding: xy(u(16), u(12)), part: `Draft.banner${flags(ban ? "ban" : "pick")}`, partClass: "panel" }, () => {
    kit.setDraw({ shape: { kind: "rect", radii: [t.panel, t.panel, t.panel, t.panel] }, fills: [{ kind: "solid", color: token("shadow", 0.55) }, vgrad([[0, paint(bannerInk, ban ? 0.18 : 0.06)], [1, paint(bannerInk, 0)]])], effects: [shadow(u(3), u(10), 0.4)] });
    kit.row("Top", { width: grow(), gap: u(12), cross: "center" }, () => {
      text("Phase", desc.phaseLabel ?? "", { step: st(phone ? 17 : 22), font: "display", heading: true, maxLines: 2, width: grow(), color: ban ? mix(role("danger"), role("ink"), 0.35) : "ink" });
      text("Time", String(secs), { step: st(phone ? 20 : 28), font: "mono", nowrap: true, color: urgent ? "danger" : "ink" });
      kit.semantics({ role: "timer", name: String(secs) });
    });
    const f = left / total;
    kit.open("Track", { width: grow(), height: px(u(4)), part: "Draft.timer", partClass: "bar" });
    kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: token("ink", 0.12) }] });
    kit.open("Fill", { width: { kind: "percent", value: f * 100 } as never, height: grow() });
    kit.setDraw({ shape: { kind: "rect", radii: [2, 2, 2, 2] }, fills: [{ kind: "solid", color: urgent ? token("danger") : paint(teamColor(myTeam)) }], ...(urgent ? { effects: [glow(u(8), token("danger", 0.6), 0)] } : {}) });
    kit.close();
    kit.close();
  });
  const gridW = portrait ? W : W - 2 * Math.round(Math.min(u(300), W * 0.24)) - 2 * u(20);
  const gap = u(phone ? 6 : 8);
  const cols = desc.columns && desc.columns > 0 ? desc.columns : Math.max(4, Math.floor((gridW - u(12) + gap) / (u(88) + gap)));
  const tw = Math.floor((gridW - u(12) - gap * (cols - 1)) / cols);
  const th = Math.round(tw * 1.15);
  const rows = Math.ceil(roster.length / cols);
  // Whole rows: the room the roster was given last tick, cut to the rows it holds whole.
  const room = kit.bounds(recallPath(`${id}~room`))?.height ?? th * 3;
  const fitRows = Math.max(1, Math.floor((room - u(12) + gap) / (th + gap)));
  // A room under one row (a phone at 200 % text) shows the row it can, cut: still the grid, never pushing the action off
  // nor standing under it (the view a whole row tall laid its names under the action).
  const viewH = Math.min(room, Math.min(rows, fitRows) * (th + gap) - gap + u(12));
  kit.open("Room", { direction: "column", width: grow(), height: grow() });
  let focusKey = "";
  kit.openScroll("Roster", { width: grow(), height: px(viewH), gap, padding: xy(u(6), u(6)), part: "Draft.roster" } as never);
  for (let row = 0; row < rows; row++) {
    kit.row(kit.idi("Row", row), { gap, width: grow() }, () => {
      for (let c = 0; c < cols; c++) {
        const e = roster[row * cols + c];
        if (!e) break;
        const k = e.key ?? "";
        const isBanned = banned.has(k);
        const isPicked = picked.has(k);
        const blocked = isBanned || isPicked || !!e.locked;
        const isHov = hov === k && !blocked;
        const state = isBanned ? "banned" : isPicked ? "taken" : isHov ? "cursor" : "open";
        const shake = rt > 0 && tick - rt < 18 && rk === keyHash(k) && !still() ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0;
        const ix = lastInteraction(k);
        const r = rosterTile(k, e, {
          part: "Draft",
          width: tw,
          height: th,
          state,
          blocked,
          banned: isBanned,
          tags: isHov ? [{ name: mine?.picks?.find((p) => p.you)?.player ?? "", team: myTeam, confirmed: false }].filter((x) => x.name) : [],
          shake,
          name: [e.name ?? k, isBanned ? desc.bannedLabel ?? "Banned" : isPicked ? desc.pickedLabel ?? "Picked" : ""].filter(Boolean).join(", "),
        });
        if (ix.focused && !blocked) focusKey = k;
        if (r.clicked && desc.yourTurn && !blocked && hov !== k) out.hovered = k;
        if (r.clicked) {
          if (blocked) {
            out.refused = k;
            kit.feedback("error", -1);
          } else if (desc.yourTurn) kit.feedback("hover", -1);
        }
      }
    });
  }
  kit.close();
  // The focus moving onto an open character hovers it: asked once, when the focus changes tile, so a hover the game
  // sets by other means is not pulled back to a focus that stayed where it was.
  const focusHash = focusKey ? keyHash(focusKey) : 0;
  if (focusKey && focusHash !== kit.memory(`~d.hot.${id}`, 0) && desc.yourTurn && hov !== focusKey && out.hovered === null) out.hovered = focusKey;
  kit.remember(`~d.hot.${id}`, focusHash);
  kit.semantics({ role: "grid", name: desc.phaseLabel ?? "Roster" });
  // Laid over its room, so the room is what the column leaves it, never what the rows ask.
  over({ insets: { start: 0, top: 0, end: 0 } });
  kit.close();
  rememberPath(`${id}~room`, kit.lastPath());
  if (out.refused) {
    kit.remember("~d.rt", tick);
    kit.remember("~d.rk", keyHash(out.refused));
  }
  kit.row("Foot", { width: grow(), cross: "center", align: "center" }, () => {
    const live = !!desc.yourTurn && !!hov && !banned.has(hov) && !picked.has(hov) && desc.phase !== "done";
    const label = !desc.yourTurn || desc.phase === "done" ? desc.waitingLabel ?? "Waiting" : ban ? desc.banLabel ?? "Ban" : desc.lockLabel ?? "Lock in";
    const r = btn("Action", label, { variant: !desc.yourTurn ? "secondary" : ban ? "danger" : "primary", size: "lg", width: portrait ? grow() : px(u(300)), disabled: !live, part: `Draft.action${flags(!desc.yourTurn ? "waiting" : ban ? "ban" : "pick")}`, ...(desc.yourTurn ? { raised: true } : {}), ...(ban && desc.yourTurn ? { sym: "ban" as const } : {}) });
    if (r.clicked && live) {
      out.locked = hov;
      kit.feedback("confirm", -1);
    }
  });
  kit.close();
  if (!portrait) teamColumn("End", teams[1], true);
  kit.close();
  rememberRoom("d", id);
  return out;
}
