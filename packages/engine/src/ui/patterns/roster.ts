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
  artSlot, btn, clamp, fill, recallPath, rememberPath, flags, frame, glow, grow, kit, lastInteraction, line, mix, over, paint, px, role, scrimFill, shadow, spring,
  st, still, surface, symbol, text, timerRing, token, u, vgrad, vstate, xy, safeHeight, safeWidth, type Mut, type Rgba,
} from "./core.ts";

/** What a character card shows: its art, its name and the lines under it, how it stands. */
interface Figure {
  texture: Api.Texture | null | undefined;
  name: string;
  sub: string;
  width: number;
  height: number;
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
  text("Name", f.name, { step: st(f.width >= u(180) ? 22 : 18), font: "display", heading: true, nowrap: true, ellipsis: true, width: grow(), color: f.dim ? "inkMuted" : "ink" });
  if (f.sub) text("Sub", f.sub, { step: st(14), color: "inkMuted", nowrap: true, ellipsis: true, width: grow() });
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
  // Its own box as it was last tick (the game gives it less than the safe area: a padding, a bar), else the safe area.
  const W = kit.memory(`~l.w.${id}`, 0) || safeWidth();
  const H = kit.memory(`~l.h.${id}`, 0) || safeHeight();
  const shown = kit.memory("~l.t0", -1);
  if (shown < 0) kit.remember("~l.t0", tick);
  const since = tick - (shown < 0 ? tick : shown);

  kit.open(id, { direction: "column", width: grow(), height: grow(), gap: u(phone ? 14 : 24), part: "Lobby.frame", partClass: "panel" });

  // The head: the title in the display face; the search chip beside it while in queue.
  kit.row("Head", { width: grow(), gap: u(16), cross: "center", wrap: true }, () => {
    if (desc.title) text("Title", desc.title, { step: phone ? "xl3" : "xl5", heading: true, font: "display", nowrap: true, ellipsis: true, width: grow() });
    if (desc.searching) {
      // On a phone the chip takes the head's width and its estimate wraps under the count.
      kit.row("Search", { gap: u(10), cross: "center", wrap: phone, width: phone ? grow() : null, padding: xy(u(14), u(8)), part: "Lobby.search", partClass: "chip" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [u(20), u(20), u(20), u(20)] }, fills: [{ kind: "solid", color: token("shadow", 0.6) }, vgrad([[0, token("sheen", 0.08)], [1, token("sheen", 0)]])], effects: [shadow(u(2), u(8), 0.4)] });
        liveDot("Live", role("success"));
        text("Label", (desc.searchingLabel ?? "Searching").toLocaleUpperCase(), { step: st(13), font: "display", color: "inkMuted", nowrap: true });
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
  // The room the lineup was given last tick (it grows between the head and the foot, which grow with the text), else an estimate.
  const laidH = kit.bounds(recallPath(`${id}~lineup`))?.height ?? 0;
  const lineH = laidH > 0 ? laidH : H - footH - u(phone ? 60 : 96);
  const roomH = Math.max(u(120), (lineH - u(8) - (rows - 1) * gap) / rows / (me && !portrait ? 1.12 : 1));
  const maxW = portrait ? (W - gap) / 2 : Math.min(u(250), (W - gap * (cols - 1)) / (cols + (me ? 0.12 : 0)));
  const cw = Math.floor(Math.min(maxW, roomH / (portrait ? 1.25 : 1.75)));
  const ch = Math.floor(Math.min(roomH, cw * (portrait ? 1.25 : 1.75)));
  const big = (m: Api.LobbyMember | null): number => (m?.you && !portrait ? 1.12 : 1);
  kit.open("Lineup", { direction: "row", width: grow(), height: grow(), gap, wrap: portrait, align: "center", cross: portrait ? "start" : "end", part: "Lobby.lineup", partClass: "panel" });
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
      figureCard({ texture: m.portrait, name: m.name ?? "", sub, width: w, height: h, lit: !!m.you, hot: ix.hovered || ix.focused, dim: false, part: "Lobby.member" }, () => {
        // The ready foot: a tick in success and the word, or the muted word (a status in its place).
        kit.row("Foot", { width: grow(), gap: u(6), cross: "center", padding: { left: 0, right: 0, top: u(6), bottom: 0 }, part: `Lobby.foot${flags(m.ready && "ready")}`, partClass: "badge" }, () => {
          if (m.ready && !m.status) symbol("Tick", "check", u(14), role("success"));
          text("Word", readyWord.toLocaleUpperCase(), { step: st(13), font: "display", nowrap: true, ellipsis: true, width: grow(), color: m.ready && !m.status ? mix(role("success"), role("ink"), 0.25) : "inkMuted" });
        });
        if (popped && !still()) kit.props({ enter: { values: [{ prop: "scale", value: 1.25 }], transition: spring("emphasis") } });
      });
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
      text("Label", (desc.inviteLabel ?? "Invite").toLocaleUpperCase(), { step: st(14), font: "display", color: "inkMuted", nowrap: true });
    });
    if (r.clicked) {
      out.invite = s;
      kit.feedback("select", -1);
    }
  }
  kit.close();
  rememberPath(`${id}~lineup`, kit.lastPath());
  kit.semantics({ role: "list", name: desc.title ?? "Party" });

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
      text("Eyebrow", (desc.modeLabel ?? "Mode").toLocaleUpperCase(), { step: "caption", font: "display", color: "inkMuted", nowrap: true });
      text("Name", mode.name ?? "", { step: st(phone ? 18 : 22), font: "display", heading: true, nowrap: true, ellipsis: true });
      if (mode.detail) text("Detail", mode.detail, { step: st(14), color: "inkMuted", nowrap: true, ellipsis: true });
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
  const self = kit.bounds(kit.lastPath());
  if (self && self.width > 0 && self.height > 0) {
    kit.remember(`~l.w.${id}`, Math.round(self.width));
    kit.remember(`~l.h.${id}`, Math.round(self.height));
  }
  return out;
}
