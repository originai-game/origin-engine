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
  artSlot, btn, clamp, eyebrow, fill, flags, frame, glow, grow, inkOn, keyHash, kit, lastInteraction, line, mix, over, paint, pointer, px, recallPath, rememberPath,
  role, rtl, scrimFill, shadow, spring, st, still, surface, symbol, teamColor, text, timerRing, token, u, vgrad, vstate, xy, safeHeight, safeWidth, type Mut, type Rgba,
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

/** A player's tag in its team's colour (P1, Mira): ink on the colour, outlined so it reads on any art. */
function playerTag(id: string, name: string, team: Rgba, big = false): void {
  const ink = inkOn(team);
  kit.open(id, { direction: "row", padding: xy(u(big ? 10 : 6), u(big ? 3 : 1)), part: "Select.tag", partClass: "badge" });
  kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: paint(team) }], strokes: [line(1, token("shadow", 0.6), "outside")], effects: [shadow(2, 4, 0.45)] });
  text("Text", name, { step: st(big ? 15 : 12), font: "display", color: ink, nowrap: true });
  kit.close();
}

/** A side's splash: the character's art filling it, a scrim at its foot under the role, the name and the description. */
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
  if (e?.role) eyebrow("Role", e.role, mix(team, role("ink"), 0.5));
  text("Name", e?.name ?? "", { step: width >= u(320) ? "displayMd" : "xl4", font: "display", heading: true, maxLines: 2, width: grow(), align: side === "end" ? "end" : "start" });
  if (justConfirmed) kit.props({ enter: { values: [{ prop: "scale", value: 1.18 }], transition: spring("emphasis") } });
  if (e?.description) text("Description", e.description, { step: st(15), color: "inkMuted", maxLines: 3, width: grow(), align: side === "end" ? "end" : "start" });
  kit.close();
  over({ insets: { start: 0, end: 0, bottom: 0 }, inert: true, z: 2 });
  kit.row("Head", { gap: u(8), cross: "center" }, () => {
    playerTag("Tag", c.name ?? c.key ?? "", team, true);
    if (c.confirmed) {
      kit.row("Locked", { gap: u(4), cross: "center", padding: xy(u(8), u(3)), part: "Select.lockedIn", partClass: "badge" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [u(4), u(4), u(4), u(4)] }, fills: [{ kind: "solid", color: token("shadow", 0.7) }] });
        symbol("Tick", "check", u(13), role("success"));
        text("Text", (desc.lockedInLabel ?? "Locked in").toLocaleUpperCase(), { step: st(12), font: "display", nowrap: true, color: mix(role("success"), role("ink"), 0.3) });
      });
    }
  });
  over(side === "end" ? { insets: { end: u(14), top: u(14) }, inert: true, z: 3 } : { insets: { start: u(14), top: u(14) }, inert: true, z: 3 });
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
  const W = kit.memory(`~s.w.${id}`, 0) || safeWidth();
  const H = kit.memory(`~s.h.${id}`, 0) || safeHeight();
  const rt = kit.memory("~s.rt", 0);
  const rk = kit.memory("~s.rk", 0);
  const ts = Math.max(1, frame().theme.textScale ?? 1);

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
  const headH = kit.bounds(recallPath(`${id}~head`))?.height ?? u(60);
  const footH = Math.round(u(phone ? 52 : 60) * Math.min(1.5, ts));
  const stageH = Math.max(u(200), H - headH - footH - u(phone ? 24 : 40));
  const splashW = portrait ? W : Math.round(W * (rival ? 0.3 : 0.4));
  const splashH = portrait ? Math.round(stageH * 0.46) : stageH;
  const gridW = portrait ? W : W - splashW * (rival ? 2 : 1) - u(24) * (rival ? 2 : 1);
  const gap = u(phone ? 6 : 10);
  const cols = desc.columns && desc.columns > 0 ? desc.columns : Math.max(portrait ? 4 : 3, Math.floor((gridW - u(12) + gap) / (u(96) + gap)));
  const tw = Math.floor((gridW - u(12) - gap * (cols - 1)) / cols);
  const th = Math.round(tw * 1.18);

  kit.open("Stage", { direction: portrait ? "column" : "row", width: grow(), height: grow(), gap: u(portrait ? 12 : 24), part: "Select.stage", partClass: "panel" });
  if (me) splashPanel("Start", byKey.get(me.entry ?? "") ?? null, me, "start", splashW, splashH, desc);
  // The roster: whole rows, scrolling when they outgrow the room.
  const tiles = [...roster.map((e) => e.key ?? ""), ...(desc.randomLabel ? ["Random"] : [])];
  const rows = Math.ceil(tiles.length / cols);
  const roomH = portrait ? stageH - splashH - u(12) : stageH;
  const wholeH = Math.min(rows * (th + gap) - gap, Math.max(th, Math.floor((roomH + gap) / (th + gap)) * (th + gap) - gap));
  kit.openScroll("Roster", { width: px(gridW), height: px(wholeH + u(12)), gap, padding: xy(u(6), u(6)), part: "Select.roster" } as never);
  for (let row = 0; row < rows; row++) {
    kit.row(kit.idi("Row", row), { gap, width: grow() }, () => {
      for (let cI = 0; cI < cols; cI++) {
        const k = tiles[row * cols + cI];
        if (k === undefined) break;
        if (k === "Random" && desc.randomLabel) {
          const r = surface("Random", { part: "Select.tile[random]", cls: "slot", fill: { ...role("shadow"), a: 0.5 }, direction: "column", width: px(tw), height: px(th), align: "center", cross: "center", gap: u(6), radius: Math.max(4, t.small), semantics: { role: "button", name: desc.randomLabel } }, () => {
            symbol("Mark", "rotate", u(28), role("inkMuted"));
            text("Label", desc.randomLabel!.toLocaleUpperCase(), { step: st(12), font: "display", color: "inkMuted", nowrap: true, ellipsis: true, width: grow(), align: "center" });
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
        const blocked = !!e.locked || (!!e.taken && !mine);
        const state = e.locked ? "locked" : e.taken && !mine ? "taken" : on.some((c) => c.confirmed) ? "confirmed" : on.length > 0 ? "cursor" : "open";
        const rim = on[0] ? teamColor(on[0].team ?? 0) : null;
        const shaking = rt > 0 && tick - rt < 18 && rk === keyHash(k) && !still();
        const ix = lastInteraction(k);
        const r = surface(k, {
          part: "Select.tile",
          cls: "slot",
          flags: [state === "open" ? "" : state],
          fill: null,
          direction: "column",
          width: px(tw),
          height: px(th),
          radius: Math.max(4, t.small),
          lift: -u(3),
          semantics: { role: "button", name: [e.name ?? k, e.role ?? "", e.locked ? e.lockedHint || "locked" : e.taken && !mine ? desc.takenLabel ?? "Taken" : "", ...on.map((c) => c.name ?? c.key ?? "")].filter(Boolean).join(", "), disabled: blocked },
          props: {
            transform: { x: shaking ? Math.sin((tick - rt) * 1.6) * (18 - (tick - rt)) * 0.5 : 0 },
            states: [{ name: "pick", states: [vstate("open"), vstate("cursor"), vstate("confirmed"), vstate("locked"), vstate("taken")], transitions: [] }],
            state: [{ group: "pick", state }],
          },
        }, () => {
          const rr = Math.max(4, t.small);
          const radii: [number, number, number, number] = [rr, rr, rr, rr];
          kit.setDraw({
            shape: { kind: "rect", radii },
            fills: [vgrad([[0, token("panelRaised")], [1, token("panel")]])],
            strokes: rim ? [line(u(3), paint(rim))] : ix.hovered || ix.focused ? [line(u(1.5), paint(mix(role("focus"), role("ink"), 0.4)))] : [line(1, token("shadow", 0.55), "outside")],
            effects: [shadow(u(3), u(8), 0.45), ...(rim ? [glow(u(7), paint(rim, 0.5), 0)] : [])],
          });
          artSlot("Art", "Select.art", { texture: e.portrait ?? null, kind: "portrait", width: px(tw), height: px(th), radius: radii, focal: { x: 0.5, y: 0.3 } }, () => {
            if (blocked) {
              kit.open("Dim", { width: grow(), height: grow() });
              kit.setDraw({ shape: { kind: "rect", radii }, fills: [{ kind: "solid", color: paint(mix(role("well"), role("inkMuted"), 0.2), 0.6) }, { kind: "solid", color: token("shadow", e.locked ? 0.45 : 0.3) }] });
              kit.close();
            }
          });
          over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
          kit.open("Shade", { width: grow(), height: grow() });
          kit.setDraw({ shape: { kind: "rect", radii }, fills: [scrimFill("bottom", 0.85)] });
          kit.close();
          over({ insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true });
          text("Name", e.name ?? k, { step: st(tw >= u(110) ? 14 : 12), font: "display", nowrap: true, ellipsis: true, width: px(tw - u(12)), align: "center", color: blocked ? "inkMuted" : "ink" });
          over({ insets: { centerX: true, bottom: u(6) }, inert: true, z: 2 });
          if (e.locked) {
            symbol("Lock", "lock", u(22), role("ink"), { icon: "lock" });
            over({ insets: { centerX: true, centerY: true }, inert: true, z: 2 });
          }
          if (on.length > 0) {
            // The cursors' tags at the top end; a confirmed pick's tick at the top start.
            kit.row("Tags", { gap: u(2) }, () => on.forEach((c, i) => playerTag(kit.idi("Tag", i), c.name ?? c.key ?? "", teamColor(c.team ?? 0))));
            over({ insets: { end: u(4), top: u(4) }, inert: true, z: 3 });
            if (on.some((c) => c.confirmed)) {
              kit.open("Tick", { width: px(u(22)), height: px(u(22)), align: "center", cross: "center" });
              kit.setDraw({ shape: { kind: "ellipse" }, fills: [{ kind: "solid", color: token("success") }], strokes: [line(1.5, token("shadow", 0.6), "outside")] });
              symbol("Mark", "check", u(13), inkOn(role("success")));
              kit.close();
              over({ insets: { start: u(4), top: u(4) }, inert: true, z: 3 });
            }
          }
        });
        if ((ix.hovered || ix.focused) && !me?.confirmed && me?.entry !== k) out.hovered = k;
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
  if (me?.confirmed && kit.back()) {
    out.cancelled = true;
    kit.feedback("back", -1);
  }
  kit.close();
  const self = kit.bounds(kit.lastPath());
  if (self && self.width > 0 && self.height > 0) {
    kit.remember(`~s.w.${id}`, Math.round(self.width));
    kit.remember(`~s.h.${id}`, Math.round(self.height));
  }
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
    text("Character", (s.character ?? "").toLocaleUpperCase(), { step: phone ? "xl5" : "displayLg", font: "display", heading: true, maxLines: 2, width: grow(), align: end ? "end" : "start" });
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
      kit.row("Stage", { padding: xy(u(12), u(4)), part: "Versus.stage", partClass: "badge" }, () => {
        kit.setDraw({ shape: { kind: "rect", radii: [999, 999, 999, 999] }, fills: [{ kind: "solid", color: token("shadow", 0.75) }] });
        text("Text", desc.stage!, { step: st(14), color: "inkMuted", nowrap: true });
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
  const self = kit.bounds(kit.lastPath());
  if (self && self.width > 0 && self.height > 0) {
    kit.remember(`~v.w.${id}`, Math.round(self.width));
    kit.remember(`~v.h.${id}`, Math.round(self.height));
  }
  return out;
}
