import { og } from "@origingame/engine";

import { type Engine } from "../../../../src/runtime.ts";

import { versus, type CharacterSelectResult, type ChatChannel, type ChatMessage, type ChatResult, type LobbyMember, type LobbyResult, type RosterEntry } from "@origingame/engine/extensions/versus/index.ts";

import { run } from "../../../../tests/support/patterns.ts";

// ── lobby ───────────────────────────────────────────────────────────────

/** A party of three in four seats, the local player second in the game's order; the game applying what it is asked. */
export function party(ready = false) {
  const results: LobbyResult[] = [];
  const state = { check: false, accepted: false, ready };
  const engine = run(() => {
    const members: LobbyMember[] = [
      { key: "kael", name: "Kael", level: 42, ready: true, accepted: state.check },
      { key: "me", name: "Aria", level: 37, you: true, ready: state.ready, accepted: state.accepted },
      { key: "mira", name: "Mira", level: 18 },
    ];
    let r = {} as LobbyResult;
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      r = versus.lobby("Lobby", { title: "Play", members, slots: 4, mode: { name: "Ranked" }, ...(state.check ? { readyCheck: { secondsLeft: 8, seconds: 10 } } : {}) });
    });
    results.push(r);
    if (r.ready) state.ready = !state.ready;
    if (r.accept) state.accepted = true;
  });
  return { engine, results, state };
}

// ── character select and versus ─────────────────────────────────────────

export const ROSTER: RosterEntry[] = [
  { key: "aria", name: "Aria", role: "Support" },
  { key: "kael", name: "Kael", role: "Duelist" },
  { key: "mira", name: "Mira", role: "Ranger", taken: true },
  { key: "brann", name: "Brann", role: "Tank" },
  { key: "vell", name: "Vell", role: "Mage", locked: true, lockedHint: "Unlock at level 10" },
];

export function selectScreen() {
  const results: CharacterSelectResult[] = [];
  const me = { key: "p1", name: "P1", entry: "kael", you: true, confirmed: false, team: 0 };
  const engine = run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 1232 }, height: { kind: "fixed", px: 672 } }, () => {
      const r = versus.characterSelect("Sel", { title: "Choose", roster: ROSTER, randomLabel: "", cursors: [me, { key: "p2", name: "P2", entry: "brann", team: 1 }] });
      results.push(r);
      if (r.hovered) me.entry = r.hovered;
      if (r.picked) me.confirmed = true;
      if (r.cancelled) me.confirmed = false;
    });
  });
  engine.advance(20);
  return { engine, results, me };
}

// ── chat ────────────────────────────────────────────────────────────────

export const CHANNELS: ChatChannel[] = [
  { key: "all", name: "All", all: true },
  { key: "party", name: "Party", color: "info" },
  { key: "guild", name: "Guild", color: "success" },
  { key: "system", name: "System", readOnly: true },
];

export function chatRun(st: { draft: string; sendTo: string; messages: ChatMessage[] }, results: ChatResult[], h = 420): Engine {
  return run(() => {
    og.ui.column("Box", { width: { kind: "fixed", px: 520 }, height: { kind: "fixed", px: h } }, () => {
      const r = versus.chat("Chat", {
        channels: CHANNELS,
        channel: "all",
        messages: st.messages,
        me: "Lumen",
        members: [{ key: "lumen", name: "Lumen" }, { key: "luka", name: "Luka" }, { key: "riven", name: "Riven" }],
        draft: st.draft,
        sendTo: st.sendTo,
      });
      results.push(r);
      if (r.draft !== null) st.draft = r.draft;
      if (r.sendTo !== null) st.sendTo = r.sendTo;
    });
  });
}
