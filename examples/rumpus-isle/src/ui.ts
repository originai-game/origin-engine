// Rumpus Isle's screens on og.ui, dressed by the rumpus style (the cartoon
// pack recoloured; the look applies it): the title (Play, Settings, the
// records), the lobby (four seats, the colours, Ready and Start, every pad
// with a focus ring of its own: og.ui.setPlayers), the round's HUD (the
// round and its clock at the top, a box per player at the bottom), its
// card and count-in, pause, the settings (og.ui.settingsScreen, every word
// from the tables, English and French), the scoreboard between rounds and
// the podium. The screens change through an iris (a fade with reduced
// motion). What a screen decides it asks Rules to do (Game.ask).

import { og, defineSystem } from "@origingame/engine";
import type { Value, World } from "@origingame/engine";
import { Ask, COLOURS, Game, MAX_PLAYERS, Phase, Prefs, ROUND_KEYS, ROUNDS, Round, Sfx, Sound, Source } from "./components.ts";
import { sourceKey } from "./input.ts";
import { freeColour, LAUNCH_TICKS, sit } from "./lobby.ts";
import { LANGUAGES, settingsOf, takeSettings } from "./prefs.ts";
import { INTRO_TICKS } from "./rules.ts";
import { joined, put, seat, seats, type Seat } from "./seats.ts";
import { clock, must, say } from "./util.ts";

type GameV = Value<typeof Game>;
type PrefsV = Value<typeof Prefs>;

const S = { xs: 4, sm: 8, md: 12, lg: 16, xl: 24 } as const;
/** The rumpus style's radius block (assets/styles/rumpus/style.ogt, the cartoon pack's). */
const R = { control: 6, panel: 10 } as const;

/** A colour's sRGB as a Paint (the players' own colours: not chrome, so not a token). */
function ink(colour: number) {
  const h = COLOURS[colour]!.rgb;
  return { color: { r: parseInt(h.slice(1, 3), 16) / 255, g: parseInt(h.slice(3, 5), 16) / 255, b: parseInt(h.slice(5, 7), 16) / 255 } };
}

function nameOf(k: number, s: Seat): string {
  return `P${k + 1} · ${COLOURS[s.colour]!.name}`;
}

function sourceName(world: World, source: number): string {
  return say(world, sourceKey(source), { n: source + 1 });
}

// ------------------------------------------------------------------ title

function title(world: World, g: GameV): void {
  const records: string[] = [];
  if (g.bestBerries > 0) records.push(say(world, "title.recordBerries", { n: g.bestBerries }));
  if (g.bestCrumble > 0) records.push(say(world, "title.recordTiles", { s: g.bestCrumble.toFixed(1) }));
  const r = og.ui.mainMenu("Title", {
    title: "Rumpus Isle",
    subtitle: say(world, "title.subtitle"),
    items: [
      { id: "play", label: say(world, "title.play"), primary: true },
      { id: "settings", label: say(world, "title.settings") },
    ],
    footer: records.length > 0 ? records.join("  ·  ") : say(world, "title.footer"),
    // Built settled while it is the screen the game opens on (the frame behind the play gate, og-ui §6).
    instant: !g.leftTitle,
  });
  if (g.settingsOpen) return;
  if (r.selected === "play") {
    g.leftTitle = true;
    g.ask = Ask.Lobby;
  } else if (r.selected === "settings") g.settingsOpen = true;
}

// ------------------------------------------------------------------ lobby

function seatCard(world: World, g: GameV, k: number, s: Seat): void {
  og.ui.column(og.ui.idi("Seat", k), { width: { kind: "grow" }, gap: S.xs, cross: "center", padding: { left: S.sm, top: S.md, right: S.sm, bottom: S.md }, background: { token: "well" }, radius: R.panel }, () => {
    if (!s.joined) {
      og.ui.label("Name", `P${k + 1}`, { step: "lg", font: "display", color: { token: "inkInverse" }, nowrap: true });
      og.ui.label("Join", say(world, "lobby.join"), { color: { token: "inkInverse" }, align: "center", width: { kind: "grow" } });
      return;
    }
    og.ui.row("Who", { gap: S.xs, cross: "center" }, () => {
      og.ui.icon("Dot", "person", { size: 22, tint: ink(s.colour) });
      og.ui.label("Name", nameOf(k, s), { step: "lg", font: "display", color: ink(s.colour), nowrap: true });
    });
    og.ui.label("Source", sourceName(world, s.source), { step: "meta", color: { token: "inkInverse" }, nowrap: true });
    og.ui.badge("State", say(world, s.ready ? "lobby.ready" : "lobby.choosing"), s.ready ? "success" : "neutral");
  });
}

function lobby(world: World, g: GameV, sfx: Value<typeof Sfx>): void {
  // Every pad gets a focus ring of its own on this screen (og-ui §4).
  og.ui.setPlayers(MAX_PLAYERS);
  const all = seats(g);
  const who = joined(g);
  const vp = og.ui.frameViewport();
  const phone = vp.breakpoint === "phonePortrait" || vp.breakpoint === "phoneLandscape";
  og.ui.anchor("Lobby", {
    anchor: "bottom",
    direction: "column",
    gap: S.md,
    width: phone ? { kind: "grow" } : { kind: "fixed", px: 860 },
    padding: { left: S.xl, top: S.lg, right: S.xl, bottom: S.lg },
    frame: "panel",
    background: { token: "panel" },
  }, () => {
    og.ui.label("Title", say(world, "lobby.title"), { step: "xl2", font: "display", heading: true });
    og.ui.richText("Hint", say(world, "lobby.hint"), { color: { token: "inkMuted" }, width: { kind: "grow" } });
    og.ui.row("Seats", { gap: S.sm, width: { kind: "grow" }, wrap: phone }, () => {
      for (let k = 0; k < MAX_PLAYERS; k++) seatCard(world, g, k, all[k]!);
    });
    // The colours: a pad's ring presses one for its own player (Response.pad).
    og.ui.row("Colours", { gap: S.xs, width: { kind: "grow" }, wrap: true, cross: "center" }, () => {
      og.ui.label("Label", say(world, "lobby.colour"), { color: { token: "inkMuted" }, nowrap: true });
      COLOURS.forEach((c, i) => {
        const taken = all.some((s) => s.joined && s.colour === i);
        const r = og.ui.button(og.ui.idi("Colour", i), say(world, c.key), { size: "small", disabled: taken });
        if (r.clicked && r.pad !== null) {
          const k = all.findIndex((s) => s.joined && s.source === r.pad);
          if (k >= 0) put(g, k, { colour: freeColour(g, k, i), ready: false });
        }
      });
    });
    og.ui.row("Actions", { gap: S.sm, width: { kind: "grow" }, cross: "center" }, () => {
      const ready = og.ui.button("Ready", say(world, "lobby.readyButton"), { disabled: who.length === 0 });
      if (ready.clicked && ready.pad !== null) {
        let k = all.findIndex((s) => s.joined && s.source === ready.pad);
        // A pad not seated yet sits down with its press of Ready.
        if (k < 0 && sit(g, ready.pad, sfx)) k = seats(g).findIndex((s) => s.joined && s.source === ready.pad);
        else if (k >= 0) {
          put(g, k, { ready: !all[k]!.ready });
          sfx.bits |= all[k]!.ready ? Sound.Leave : Sound.Ready;
        }
      }
      og.ui.spacer("Gap", { kind: "grow" });
      if (g.launch > 0) og.ui.label("Launch", say(world, "lobby.starting", { n: Math.ceil((LAUNCH_TICKS - g.launch) / 60) }), { step: "lg", font: "display", color: { token: "highlight" }, nowrap: true });
      if (og.ui.button("Back", say(world, "lobby.back"), { variant: "secondary" }).clicked) g.ask = Ask.Title;
      const start = og.ui.button("Start", say(world, "lobby.start"), { variant: "primary", disabled: who.length < 2 });
      if (start.clicked && who.length >= 2) g.ask = Ask.Match;
    });
  });
}

// ------------------------------------------------------------------ the round

function value(world: World, g: GameV, s: Seat): string {
  if (g.round === Round.Berry) return `${s.berries}`;
  if (g.phase === Phase.Over || !s.alive) return s.place > 0 ? say(world, "place.short", { n: s.place }) : "";
  return "";
}

function hud(world: World, g: GameV): boolean {
  let paused = false;
  const roundName = say(world, ROUND_KEYS[g.round]!);
  // The top: the round and its clock, in the middle (the top-left corner is the web shell's Back).
  og.ui.anchor("Top", { anchor: "top", direction: "row", gap: S.sm, cross: "center" }, () => {
    og.ui.row("Round", { gap: S.sm, cross: "center", padding: { left: S.md, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, () => {
      og.ui.label("Name", say(world, "hud.round", { n: g.round + 1, of: ROUNDS, name: roundName }), { step: "lg", font: "display", nowrap: true });
      og.ui.icon("ClockIcon", "clock", { size: 20, tint: { token: "inkMuted" } });
      og.ui.label("Clock", clock(g.phase === Phase.Intro ? g.left : g.ticks / 60), { step: "lg", font: "mono", nowrap: true });
    });
    if (og.ui.button("Pause", say(world, "hud.pause"), { size: "small", variant: "secondary" }).clicked) paused = true;
  });
  // The bottom: a box per player, in their colour.
  og.ui.anchor("Players", { anchor: "bottom", direction: "row", gap: S.sm, cross: "center" }, () => {
    seats(g).forEach((s, k) => {
      if (!s.joined) return;
      og.ui.row(og.ui.idi("Player", k), { gap: S.xs, cross: "center", padding: { left: S.sm, top: S.xs, right: S.md, bottom: S.xs }, frame: "hud", background: { token: "panel" } }, () => {
        og.ui.icon("Dot", g.round === Round.Berry ? "apple" : s.alive ? "person" : "drop", { size: 22, tint: ink(s.colour) });
        og.ui.label("Name", COLOURS[s.colour]!.name, { font: "display", color: { token: "ink" }, nowrap: true });
        const v = value(world, g, s);
        if (v !== "") og.ui.label("Value", v, { step: "lg", font: "mono", color: { token: "highlight" }, nowrap: true });
      });
    });
  });
  return paused;
}

/** The round's card and count-in, over the island. */
function intro(world: World, g: GameV): void {
  const left = g.ticks;
  const count = left > 180 ? -1 : Math.ceil(left / 60);
  og.ui.anchor("Card", { anchor: "center", direction: "column", gap: S.sm, cross: "center", padding: { left: S.xl, top: S.lg, right: S.xl, bottom: S.lg }, frame: "panel", background: { token: "panel" } }, () => {
    og.ui.label("Round", say(world, "intro.round", { n: g.round + 1, of: ROUNDS }), { step: "lg", color: { token: "inkMuted" } });
    og.ui.label("Name", say(world, ROUND_KEYS[g.round]!), { step: "displayMd", font: "display", heading: true });
    og.ui.label("How", say(world, `${ROUND_KEYS[g.round]}.how`), { align: "center", width: { kind: "fixed", px: 420 } });
    og.ui.transition(og.ui.idi("Count", count), count > 0, { kind: "pop", durationMs: 220 }, () => {
      og.ui.label("Number", `${count}`, { step: "displayLg", font: "display", color: { token: "highlight" } });
    });
  });
  void INTRO_TICKS;
}

/** The round's last beat: who won it. */
function over(world: World, g: GameV): void {
  const all = seats(g);
  const winners = joined(g).filter((k) => all[k]!.place === 1);
  og.ui.transition("Banner", true, { kind: "pop" }, () => {
    og.ui.anchor("Winner", { anchor: "center", direction: "column", cross: "center", gap: S.xs, padding: { left: S.xl, top: S.md, right: S.xl, bottom: S.md }, frame: "panel", background: { token: "panel" } }, () => {
      const names = winners.map((k) => COLOURS[all[k]!.colour]!.name);
      og.ui.label("Text", winners.length === 1 ? say(world, "over.one", { name: names[0]! }) : say(world, "over.tie", { names: names.join(" & ") }), { step: "displaySm", font: "display", color: winners.length === 1 ? ink(all[winners[0]!]!.colour) : { token: "ink" }, heading: true });
    });
  });
}

function pause(world: World, g: GameV): void {
  if (g.paused) og.ui.setPlayers(MAX_PLAYERS);
  const r = og.ui.pauseMenu("Pause", {
    open: g.paused && !g.settingsOpen,
    title: say(world, "pause.title"),
    items: [
      { id: "resume", label: say(world, "pause.resume"), primary: true },
      { id: "settings", label: say(world, "pause.settings") },
      { id: "lobby", label: say(world, "pause.lobby") },
      { id: "quit", label: say(world, "pause.quit") },
    ],
  });
  switch (r.selected) {
    case "resume":
      g.paused = false;
      break;
    case "settings":
      g.settingsOpen = true;
      break;
    case "lobby":
      g.paused = false;
      g.ask = Ask.Lobby;
      break;
    case "quit":
      g.paused = false;
      g.ask = Ask.Title;
      break;
  }
}

// ------------------------------------------------------------------ scores and podium

function table(world: World, g: GameV, total: boolean): void {
  const all = seats(g);
  const order = joined(g).sort((a, b) => (total ? all[b]!.points - all[a]!.points : all[a]!.place - all[b]!.place) || a - b);
  og.ui.column("Table", { gap: S.xs, width: { kind: "grow" } }, () => {
    order.forEach((k, i) => {
      const s = all[k]!;
      og.ui.row(og.ui.idi("Row", i), { gap: S.sm, cross: "center", width: { kind: "grow" }, padding: { left: S.md, top: S.xs, right: S.md, bottom: S.xs }, background: { token: "control" }, radius: R.control }, () => {
        const place = total ? 1 + order.filter((o) => all[o]!.points > s.points).length : s.place;
        og.ui.label("Place", place > 0 ? say(world, "place.short", { n: place }) : "–", { step: "lg", font: "display", color: { token: "inkMuted" }, width: { kind: "fixed", px: 52 } });
        og.ui.icon("Dot", "person", { size: 22, tint: ink(s.colour) });
        og.ui.label("Name", nameOf(k, s), { step: "lg", font: "display", width: { kind: "grow" }, nowrap: true });
        if (!total && g.round === Round.Berry) og.ui.label("Berries", say(world, "scores.berries", { n: s.berries }), { color: { token: "inkMuted" }, nowrap: true });
        if (!total && g.round === Round.Crumble && s.lasted > 0) og.ui.label("Lasted", say(world, "scores.lasted", { s: s.lasted.toFixed(1) }), { color: { token: "inkMuted" }, nowrap: true });
        if (!total) og.ui.badge("Gained", `+${s.gained}`, s.gained > 0 ? "success" : "neutral");
        og.ui.counter("Points", s.points, { from: total ? 0 : s.points - s.gained, step: "xl", font: "display", durationMs: 700, suffix: ` ${say(world, "scores.pts")}` });
      });
    });
  });
}

function scores(world: World, g: GameV): void {
  og.ui.setPlayers(MAX_PLAYERS);
  const last = g.round + 1 >= ROUNDS;
  og.ui.anchor("Scores", { anchor: "center", direction: "column", gap: S.md, width: { kind: "fixed", px: 620 }, padding: { left: S.xl, top: S.lg, right: S.xl, bottom: S.lg }, frame: "panel", background: { token: "panel" } }, () => {
    og.ui.label("Title", say(world, "scores.title", { n: g.round + 1, name: say(world, ROUND_KEYS[g.round]!) }), { step: "xl2", font: "display", heading: true });
    table(world, g, false);
    if (g.newRecord) {
      og.ui.badge("Record", g.round === Round.Berry ? say(world, "scores.recordBerries", { n: g.bestBerries }) : say(world, "scores.recordTiles", { s: g.bestCrumble.toFixed(1) }), "primary");
      og.ui.burst({ count: 12 });
    }
    og.ui.label("DropIn", say(world, "scores.dropIn"), { step: "meta", color: { token: "inkMuted" }, width: { kind: "grow" } });
    const next = og.ui.button("Next", say(world, last ? "scores.podium" : "scores.next"), { variant: "primary", size: "large", autofocus: true, width: { kind: "grow" } });
    if (next.clicked) g.ask = Ask.Next;
  });
}

function podium(world: World, g: GameV): void {
  og.ui.setPlayers(MAX_PLAYERS);
  const all = seats(g);
  const who = joined(g);
  const top = Math.max(...who.map((k) => all[k]!.points));
  const winners = who.filter((k) => all[k]!.points === top);
  og.ui.anchor("Podium", { anchor: "bottom", direction: "column", gap: S.md, width: { kind: "fixed", px: 640 }, padding: { left: S.xl, top: S.lg, right: S.xl, bottom: S.lg }, frame: "panel", background: { token: "panel" } }, () => {
    const names = winners.map((k) => COLOURS[all[k]!.colour]!.name);
    og.ui.label("Title", winners.length === 1 ? say(world, "podium.one", { name: names[0]! }) : say(world, "podium.tie", { names: names.join(" & ") }), { step: "xl3", font: "display", heading: true, color: winners.length === 1 ? ink(all[winners[0]!]!.colour) : { token: "ink" } });
    table(world, g, true);
    og.ui.row("Actions", { gap: S.sm, width: { kind: "grow" } }, () => {
      if (og.ui.button("Again", say(world, "podium.again"), { variant: "primary", autofocus: true, width: { kind: "grow" } }).clicked) g.ask = Ask.Again;
      if (og.ui.button("Lobby", say(world, "podium.lobby"), { width: { kind: "grow" } }).clicked) g.ask = Ask.Lobby;
      if (og.ui.button("Title", say(world, "podium.title"), { variant: "secondary", width: { kind: "grow" } }).clicked) g.ask = Ask.Title;
    });
  });
}

// ------------------------------------------------------------------ settings

function settings(world: World, g: GameV, p: PrefsV): void {
  const r = og.ui.settingsScreen("Settings", {
    open: g.settingsOpen,
    settings: settingsOf(p),
    languages: LANGUAGES.map((l) => ({ code: l.code, label: say(world, l.key) })),
    title: say(world, "settings.title"),
    audioLabel: say(world, "settings.audio"),
    graphicsLabel: say(world, "settings.graphics"),
    controlsLabel: say(world, "settings.controls"),
    languageLabel: say(world, "settings.language"),
    masterLabel: say(world, "settings.master"),
    musicLabel: say(world, "settings.music"),
    effectsLabel: say(world, "settings.effects"),
    muteLabel: say(world, "settings.mute"),
    qualityLabel: say(world, "settings.quality"),
    lowLabel: say(world, "settings.low"),
    mediumLabel: say(world, "settings.medium"),
    highLabel: say(world, "settings.high"),
    fpsLabel: say(world, "settings.fps"),
    motionLabel: say(world, "settings.motion"),
    contrastLabel: say(world, "settings.contrast"),
    sensitivityLabel: say(world, "settings.sensitivity"),
    invertLabel: say(world, "settings.invert"),
    vibrationLabel: say(world, "settings.vibration"),
    doneLabel: say(world, "settings.done"),
  });
  if (!g.settingsOpen) return;
  if (r.changed) takeSettings(world, p, r.settings);
  if (r.closed) {
    g.settingsOpen = false;
    g.saveSettings = true;
  }
}

// ------------------------------------------------------------------ the system

function screenOf(phase: number): string {
  switch (phase) {
    case Phase.Title:
      return "title";
    case Phase.Lobby:
      return "lobby";
    case Phase.Scores:
      return "scores";
    case Phase.Podium:
      return "podium";
    default:
      return "round";
  }
}

export const Ui = defineSystem("Ui", { reads: [og.Gamepad], writes: [Game, Prefs, Sfx, og.Audio] }, (_q, time, world) => {
  const g = must(world.resource(Game));
  const p = must(world.resource(Prefs));
  const sfx = must(world.resource(Sfx));
  const pads = must(world.resource(og.Gamepad));

  const inRound = g.phase === Phase.Intro || g.phase === Phase.Play || g.phase === Phase.Over;
  og.time.setScale(world, inRound && (g.paused || g.settingsOpen) ? 0 : 1);

  const shown = og.ui.screenTransition("Screens", screenOf(g.phase), { kind: p.reducedMotion ? "fade" : "iris" }).shown;
  let pauseAsked = false;
  if (shown === "title") title(world, g);
  else if (shown === "lobby") lobby(world, g, sfx);
  else if (shown === "round") {
    pauseAsked = hud(world, g);
    if (g.phase === Phase.Intro) intro(world, g);
    else if (g.phase === Phase.Over) over(world, g);
    pause(world, g);
  } else if (shown === "scores") scores(world, g);
  else if (shown === "podium") podium(world, g);
  settings(world, g, p);

  // Back (Escape, pad 0's B) with nothing open: pause in a round, the title from the lobby. Any seated pad's
  // Start pauses and resumes. Asked after the screens are built (og-ui §4).
  const back = og.ui.back();
  if (inRound && !g.settingsOpen) {
    const start = seats(g).some((s) => s.joined && s.source <= Source.Pad3 && pads.connected(s.source) && pads.pressed("start", s.source));
    if (!g.paused && (back || start || pauseAsked)) g.paused = true;
    else if (g.paused && start) g.paused = false;
  } else if (g.phase === Phase.Lobby && back && !g.settingsOpen) {
    // Pad 0's B also leaves its seat (src/lobby.ts): only an unseated Back goes to the title.
    const pad0Left = pads.connected(0) && pads.pressed("b", 0);
    if (!pad0Left) g.ask = Ask.Title;
  }
  // In a round the keys and sticks drive the critters: focus navigation off, dialogs turn it back on (og-ui §4).
  if (inRound && !g.paused && !g.settingsOpen) og.ui.setNavigation(false);

  if (time.tick % 30 === 0) {
    const tree = og.ui.a11y();
    g.a11yNodes = tree.nodes.length;
    g.a11yUnnamed = tree.nodes.filter((n) => n.states.includes("focusable") && n.name === "").length;
  }
});
