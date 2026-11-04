// The genre patterns' preview screens of the roster and the wider game (docs/ui.md §11, U6d): a party lobby, a
// character select, a versus screen, a draft, a world map and a song select, each built from its pattern over the
// kit's fixtures with the game's side done as a game would (a ready flipped, a pick locked, a waypoint set).
// `pat:<area>:<key>:<value>` sets a preview's state so the playtests (tests/patterns.ts) can draw each one.
//
// Names (players, characters, songs, places' proper nouns) are invented and stay as they are in every locale; every
// other word is the game's, keyed in assets/strings.

import { og } from "@origingame/engine";
import { art, besideChrome, L, phone, type ArtName } from "./patterns.ts";
import type { DraftTeam, LobbyMember, RosterEntry, SelectCursor, Size, SongEntry, WorldMarker, WorldRegion } from "@origingame/engine";

type Mut<T> = { -readonly [K in keyof T]: T[K] };
const grow: Size = { kind: "grow" };
const px = (n: number): Size => ({ kind: "fixed", px: n });

/**
 * A genre screen's frame: the safe area, the kit's title in a header that leaves the Gallery button its corner on a
 * phone (the pattern then takes no title of its own), the pattern under it, and the action hints at the desktop's foot.
 * `bleed` takes the whole safe area with no header (a versus screen, a map).
 */
export function frame(id: string, title: string, build: () => void, hints: Array<{ action: string; hint: string }>, bleed = false): void {
  const t = og.ui.frameTheme();
  const vp = og.ui.frameViewport();
  const sa = vp.safeArea;
  const small = phone();
  const big = !small && vp.height >= 1000;
  const pad = bleed ? 0 : small ? t.space.sm : t.space.xl;
  // Room for the action hints at the foot: 72 px a line, or last tick's hints where they wrapped taller.
  const hintsBox = og.ui.bounds("Hints");
  const hintsH = hintsBox && hints.length > 0 ? vp.height - sa.bottom - hintsBox.y + t.space.sm : 0;
  const bottom = bleed ? 0 : small ? t.space.sm : Math.max(Math.round(72 * Math.max(1, t.textScale ?? 1)), Math.round(hintsH));
  const top = bleed ? 0 : small ? t.space.sm : t.space.lg;
  const height = vp.height - sa.top - sa.bottom;
  // The pattern's room is exact (the header's height taken off): a screen-filling pattern sizes its parts from it. The
  // header is its one line of title at this tick's step (the ramp carries the text size), under the Gallery chrome or
  // at least as tall as it beside it on a phone, so a change of text size or of the chrome's place never leaves the
  // body a tick too tall.
  const laidHead = og.ui.bounds(`${id}/Header`);
  const step = small ? "xl2" : big ? "xl6" : "xl4";
  const chrome = besideChrome(vp.width - sa.left - sa.right - 2 * pad);
  const titleH = t.typeRamp[step]?.lineHeight ?? 0;
  const headH = bleed ? 0 : Math.max(chrome.top + titleH, chrome.min, laidHead?.height ?? 0) + t.space.md;
  og.ui.anchor(id, { anchor: "topLeft", margin: 0, width: px(vp.width - sa.left - sa.right), height: px(height), direction: "column", gap: t.space.md, padding: { left: pad, top, right: pad, bottom } }, () => {
    if (!bleed) {
      og.ui.column("Header", { width: grow, height: { kind: "fit", min: chrome.min }, align: "center", padding: { left: 0, top: chrome.top, right: chrome.right, bottom: 0 } }, () => {
        og.ui.label("Title", title, { step, heading: true, font: "display", nowrap: true, ellipsis: true, width: grow });
      });
    }
    // Before the header has been laid out once (its height grows with the text size), the body takes what is left.
    og.ui.column("Body", { width: grow, height: bleed || laidHead ? px(Math.max(1, height - top - bottom - headH)) : grow }, build);
    if (hints.length > 0 && !small && !bleed) og.ui.actionHints("Hints", { hints });
  });
}

// ── lobby: the party, matchmaking and the ready check ──────────────────

const lob0 = () => ({ ready: false, searching: false, since: 0, found: false, accepted: false, mode: false });
let lob = lob0();
const PARTY = [
  { key: "aria", name: "Lumen", level: 42, rank: "diamond", art: "portraits/aria", you: true, leader: true },
  { key: "kael", name: "Riven", level: 57, rank: "immortal", art: "portraits/kael", ready: true },
  { key: "mira", name: "Juno", level: 38, rank: "platinum", art: "portraits/mira", ready: true },
  { key: "wren", name: "Sable", level: 21, rank: "gold", art: "portraits/wren", ready: true },
] as const;

/** A party lobby (Valorant, Fortnite): the lineup, the mode, Ready up, then Find match, the queue and the ready check. */
export function lobbyScreen(): void {
  if (lob.since < 0) lob.since = og.ui.tick();
  const secs = lob.searching ? (og.ui.tick() - lob.since) / 60 + 47 : 0;
  frame("Lobby", L("lobby.title"), () => {
    const members: LobbyMember[] = PARTY.map((m) => ({
      key: m.key,
      name: m.name,
      level: m.level,
      title: L(`lobby.rank.${m.rank}`),
      portrait: art(m.art),
      ready: "you" in m ? lob.ready : true,
      leader: "leader" in m,
      you: "you" in m,
      accepted: lob.found && ("you" in m ? lob.accepted : m.key !== "wren"),
    }));
    const r = og.ui.lobby("Party", {
      members,
      slots: 5,
      mode: { name: L("lobby.mode.ranked"), detail: L("lobby.mode.detail"), image: art("scenes/golden-valley") },
      searching: lob.searching,
      searchSeconds: secs,
      estimateSeconds: 90,
      canStart: lob.ready,
      readyCheck: lob.found ? { secondsLeft: 7.4, seconds: 10 } : null,
      readyLabel: L("lobby.ready"),
      notReadyLabel: L("lobby.notReady"),
      readyUpLabel: L("lobby.readyUp"),
      startLabel: L("lobby.start"),
      cancelLabel: L("lobby.cancel"),
      searchingLabel: L("lobby.searching"),
      estimateLabel: L("lobby.estimate"),
      inviteLabel: L("lobby.invite"),
      levelLabel: L("lobby.level"),
      leaderLabel: L("lobby.leader"),
      matchFoundLabel: L("lobby.found"),
      acceptLabel: L("lobby.accept"),
      acceptedLabel: L("lobby.accepted"),
      declineLabel: L("lobby.decline"),
      modeLabel: L("lobby.modeLabel"),
    });
    if (r.ready) lob.ready = !lob.ready;
    if (r.start) {
      lob.searching = true;
      lob.since = og.ui.tick();
    }
    if (r.cancel) lob.searching = false;
    if (r.accept) lob.accepted = true;
    if (r.decline) {
      lob.found = false;
      lob.searching = false;
      lob.accepted = false;
    }
  }, [{ action: "confirm", hint: L(lob.searching ? "lobby.cancel" : lob.ready ? "lobby.start" : "lobby.readyUp") }, { action: "back", hint: L("hint.back") }]);
}

// ── character select and versus ─────────────────────────────────────────

/** The roster as the game holds it: each fighter's art, its role and whether the player owns it. */
const ROSTER = [
  { key: "aria", portrait: "portraits/aria", splash: "figures/aria", role: "mage" },
  { key: "kael", portrait: "portraits/kael", splash: "figures/kael", role: "duelist" },
  { key: "mira", portrait: "portraits/mira", splash: "figures/mira", role: "support" },
  { key: "wren", portrait: "portraits/wren", role: "ranger" },
  { key: "vesper", portrait: "portraits/vesper", role: "mage" },
  { key: "maren", portrait: "portraits/maren", role: "vanguard" },
  { key: "torvin", portrait: "portraits/torvin", role: "vanguard" },
  { key: "brann", portrait: "portraits/brann", role: "support" },
  { key: "vell", portrait: "portraits/aria-vell", role: "duelist" },
  { key: "cultist", portrait: "portraits/cultist", role: "mage", locked: true },
] as const;
const NAMES: Record<string, string> = { aria: "Aria", kael: "Kael", mira: "Mira", wren: "Wren", vesper: "Vesper", maren: "Maren", torvin: "Torvin", brann: "Brann", vell: "Vell", cultist: "Morrow" };
const roster = (): RosterEntry[] =>
  ROSTER.map((e) => ({
    key: e.key,
    name: NAMES[e.key]!,
    role: L(`chars.role.${e.role}`),
    portrait: art(e.portrait as ArtName),
    splash: "splash" in e ? art(e.splash as ArtName) : null,
    description: L(`chars.about.${e.key}`),
    locked: "locked" in e,
    lockedHint: "locked" in e ? L("chars.unlock", { level: 30 }) : "",
  }));

const cs0 = () => ({ entry: "mira", confirmed: false, rival: "kael", rivalConfirmed: true });
let cs = cs0();

/** A character select (Street Fighter 6, Overwatch): the player's fighter large at the start, the rival's at the end, the roster between. */
export function charactersScreen(): void {
  frame("Select", L("chars.title"), () => {
    const cursors: SelectCursor[] = [
      { key: "p1", name: "P1", entry: cs.entry, confirmed: cs.confirmed, team: 0, you: true },
      { key: "p2", name: "P2", entry: cs.rival, confirmed: cs.rivalConfirmed, team: 1 },
    ];
    const r = og.ui.characterSelect("Chars", {
      roster: roster(),
      cursors,
      secondsLeft: 24,
      lockLabel: L("chars.lock"),
      lockedInLabel: L("chars.locked"),
      takenLabel: L("chars.taken"),
      randomLabel: L("chars.random"),
    });
    if (r.hovered !== null && !cs.confirmed) cs.entry = r.hovered;
    if (r.picked !== null) {
      cs.entry = r.picked;
      cs.confirmed = true;
    }
    if (r.random) {
      // The game's pick: an owned fighter the rival does not hold, by the tick (the engine's seeded random in a game).
      const open = ROSTER.filter((e) => !("locked" in e) && e.key !== cs.rival);
      cs.entry = open[og.ui.tick() % open.length]!.key;
      cs.confirmed = true;
    }
    if (r.cancelled) cs.confirmed = false;
  }, [{ action: "confirm", hint: L("chars.lock") }, { action: "back", hint: L("hint.back") }]);
}

let vsProgress = 0.64;

/** A versus screen (Tekken, a MOBA's loading screen): both fighters on their halves, the mark between, the load along the foot. */
export function versusScreen(): void {
  frame("Versus", "", () => {
    const r = og.ui.versus("Vs", {
      left: { name: "Lumen", character: "Aria", splash: art("figures/aria"), detail: L("vs.record", { w: 128, l: 64 }), team: 0 },
      right: { name: "Riven", character: "Kael", splash: art("figures/kael"), detail: L("vs.record", { w: 211, l: 97 }), team: 1 },
      versusLabel: L("vs.mark"),
      stage: L("vs.stage"),
      progress: vsProgress,
      skipLabel: L("vs.skip"),
    });
    if (r.skipped) vsProgress = 1;
  }, [], true);
}

// ── draft and ban ───────────────────────────────────────────────────────

const dr0 = () => ({ phase: "pick" as "pick" | "ban", hovered: "vesper", mine: "", yourTurn: true });
let dr = dr0();

/** A draft with bans (League of Legends, Dota 2): both teams' picks and bans at the sides, the roster and the turn between. */
export function draftScreen(): void {
  const ban = dr.phase === "ban";
  const teams: DraftTeam[] = [
    {
      name: L("draft.blue"),
      team: 0,
      picks: [
        { player: "Lumen", entry: ban ? "" : dr.mine || dr.hovered, locked: !!dr.mine, active: !ban && !dr.mine, you: true },
        { player: "Ash", entry: ban ? "" : "maren", locked: !ban },
        { player: "Juno", entry: "" },
      ],
      bans: ban ? ["brann", ""] : ["brann", "vell"],
    },
    {
      name: L("draft.red"),
      team: 1,
      picks: [
        { player: "Riven", entry: ban ? "" : "kael", locked: !ban },
        { player: "Oto", entry: ban ? "" : "torvin", locked: !ban },
        { player: "Pell", entry: "", active: !!dr.mine },
      ],
      bans: ban ? ["mira"] : ["mira", "cultist"],
    },
  ];
  frame("Draft", L("draft.title"), () => {
    const r = og.ui.draft("Draft", {
      teams,
      roster: roster().map((e) => ({ ...e, locked: false })),
      phase: dr.phase,
      phaseLabel: L(ban ? "draft.phase.ban" : dr.mine ? "draft.phase.theirs" : "draft.phase.pick"),
      secondsLeft: 18,
      seconds: 30,
      yourTurn: dr.yourTurn && !dr.mine,
      hovered: dr.hovered,
      lockLabel: L("chars.lock"),
      banLabel: L("draft.ban"),
      waitingLabel: L("draft.waiting"),
      bansLabel: L("draft.bans"),
      bannedLabel: L("draft.banned"),
      pickedLabel: L("draft.picked"),
    });
    if (r.hovered !== null) dr.hovered = r.hovered;
    if (r.locked !== null) {
      if (ban) dr.phase = "pick";
      else dr.mine = r.locked;
    }
  }, [{ action: "confirm", hint: L(ban ? "draft.ban" : "chars.lock") }, { action: "back", hint: L("hint.back") }]);
}

// ── world map ───────────────────────────────────────────────────────────

/** The world as the game holds it (maps/world.jpg): its regions, and the places on it. */
const REGIONS = [
  { key: "frost", x: 0.5, y: 0.1, radius: 0.16, discovered: true },
  { key: "dusk", x: 0.2, y: 0.42, radius: 0.15, discovered: false },
  { key: "amber", x: 0.52, y: 0.42, radius: 0.14, discovered: true },
  { key: "redmesa", x: 0.8, y: 0.5, radius: 0.17, discovered: true },
  { key: "delta", x: 0.68, y: 0.78, radius: 0.14, discovered: true },
] as const;
const PLACES = [
  { key: "sela", kind: "fastTravel", x: 0.83, y: 0.72, art: "scenes/golden-valley" },
  { key: "amberhold", kind: "fastTravel", x: 0.55, y: 0.31, art: "scenes/storybook-vale" },
  { key: "mill", kind: "quest", x: 0.47, y: 0.53, tracked: true },
  { key: "market", kind: "shop", x: 0.62, y: 0.62 },
  { key: "duskhold", kind: "poi", x: 0.17, y: 0.16, art: "scenes/gothic-halls" },
  { key: "light", kind: "objective", x: 0.87, y: 0.82 },
  { key: "wyrm", kind: "boss", x: 0.14, y: 0.48, hidden: true },
] as const;
const KINDS = ["quest", "objective", "fastTravel", "shop", "poi", "boss"] as const;

const wm0 = () => ({ selected: "", hidden: [] as string[], tracked: "mill", player: { x: 0.5, y: 0.56 } });
let wm = wm0();

/**
 * A world map (The Witcher 3, Genshin Impact): the painted map under its fog, the places, the legend and a place's
 * detail; the whole screen on a desktop, under the kit's header on a phone (whose top end corner is the Gallery's).
 */
export function mapScreen(): void {
  // A pattern screen paints its own ground (no style backdrop under it): the pack's backdrop, edge to edge, so the
  // header's title is the pack's ink on the pack's ground (never a heading's red on the clear colour's black).
  const vp = og.ui.frameViewport();
  og.ui.anchor("Ground", { anchor: "topLeft", margin: 0, offsetX: -vp.safeArea.left, offsetY: -vp.safeArea.top, width: px(vp.width), height: px(vp.height) }, () => {
    og.ui.column("Fill", { width: grow, height: grow }, () => {});
    og.ui.setDraw({ shape: { kind: "rect" }, fills: [{ kind: "solid", color: { token: "backdrop" } }] } as never);
    og.ui.props({ position: "absolute", insets: { start: 0, top: 0, end: 0, bottom: 0 }, inert: true } as never);
  });
  frame("World", L("map.title"), () => {
    const regions: WorldRegion[] = REGIONS.map((g) => ({ key: g.key, name: L(`map.region.${g.key}`), x: g.x, y: g.y, radius: g.radius, discovered: g.discovered }));
    const markers: WorldMarker[] = PLACES.map((p) => ({
      key: p.key,
      kind: p.kind,
      name: L(`map.place.${p.key}`),
      description: L(`map.about.${p.key}`),
      x: p.x,
      y: p.y,
      discovered: !("hidden" in p),
      tracked: wm.tracked === p.key,
      image: "art" in p ? art(p.art as ArtName) : null,
    }));
    const r = og.ui.worldMap("Map", {
      texture: art("maps/world"),
      aspect: 16 / 9,
      regions,
      markers,
      playerX: wm.player.x,
      playerY: wm.player.y,
      heading: 0.6,
      selected: wm.selected,
      hidden: wm.hidden,
      title: phone() ? "" : L("map.title"),
      travelLabel: L("map.travel"),
      trackLabel: L("map.track"),
      untrackLabel: L("map.untrack"),
      undiscoveredLabel: L("map.unknown"),
      kindLabels: Object.fromEntries(KINDS.map((k) => [k, L(`map.kind.${k}`)])),
      closeLabel: L("map.close"),
    });
    if (r.selected !== null) wm.selected = r.selected;
    if (r.travel !== null) {
      const p = PLACES.find((x) => x.key === r.travel);
      if (p) wm.player = { x: p.x, y: p.y + 0.02 };
      wm.selected = "";
    }
    if (r.track !== null) wm.tracked = wm.tracked === r.track ? "" : r.track;
    if (r.toggled !== null) wm.hidden = wm.hidden.includes(r.toggled) ? wm.hidden.filter((k) => k !== r.toggled) : [...wm.hidden, r.toggled];
  }, [], !phone());
}

// ── song select ─────────────────────────────────────────────────────────

/** The songs as the game holds them: jacket, tempo, length, and the player's best on each chart. */
const SONGS = [
  { key: "neon-moon", title: "Neon Moonrise", artist: "KAIRO feat. Lune", bpm: 174, secs: 128, best: 982140, grade: "S", fc: true, acc: 0.987 },
  { key: "sakura-wind", title: "Petal Drift", artist: "Hanae Sora", bpm: 148, secs: 142, best: 941560, grade: "A", acc: 0.952 },
  { key: "stormbreak", title: "Stormbreak", artist: "Iron Tide", bpm: 200, secs: 116, best: 0, grade: "", acc: -1, isNew: true },
  { key: "desert-drum", title: "Sun Over Dunes", artist: "Oasis Line", bpm: 120, secs: 154, best: 873020, grade: "B", acc: 0.904 },
  { key: "crystal-echo", title: "Crystal Echo", artist: "Velvet Static", bpm: 162, secs: 137, best: 998000, grade: "S+", fc: true, acc: 0.996 },
  { key: "phoenix", title: "Ashes to Flight", artist: "Ember Choir", bpm: 186, secs: 161, best: 0, grade: "", acc: -1, locked: true },
] as const;
const CHARTS = [["easy", 5], ["normal", 11], ["hard", 19], ["expert", 26], ["master", 31]] as const;

/** A song's waveform: its peaks, made from its key as the game would read them from the audio. */
function peaks(key: string): number[] {
  let h = 0;
  for (const ch of key) h = (h * 31 + ch.charCodeAt(0)) >>> 0;
  return Array.from({ length: 96 }, (_, i) => {
    const s = Math.sin(i * 0.37 + (h % 7)) * 0.25 + Math.sin(i * 1.9 + (h % 13)) * 0.15 + Math.sin(i * 0.08) * 0.2;
    return Math.max(0.08, Math.min(1, 0.55 + s));
  });
}

const sg0 = () => ({ selected: "crystal-echo", chart: "expert", playing: true, since: 0 });
let sg = sg0();

/** A song select (Project Sekai, osu!): the wheel of jackets at the end side, the selected song's jacket, charts, best and preview. */
export function songsScreen(): void {
  if (sg.since === 0) sg.since = og.ui.tick();
  frame("Songs", L("song.title"), () => {
    const songs: SongEntry[] = SONGS.map((s) => ({
      key: s.key,
      title: s.title,
      artist: s.artist,
      jacket: art(`songs/${s.key}` as ArtName),
      bpm: s.bpm,
      seconds: s.secs,
      difficulties: CHARTS.map(([k, lv]) => ({ key: k, name: L(`song.chart.${k}`), level: lv, locked: k === "master" && s.grade !== "S+" })),
      best: s.best,
      grade: s.grade,
      fullCombo: "fc" in s,
      accuracy: s.acc,
      locked: "locked" in s,
      isNew: "isNew" in s,
    }));
    const song = SONGS.find((s) => s.key === sg.selected) ?? SONGS[0];
    const r = og.ui.songSelect("Wheel", {
      songs,
      selected: sg.selected,
      difficulty: sg.chart,
      preview: { playing: sg.playing, position: sg.playing ? Math.min(song.secs, (og.ui.tick() - sg.since) / 60 + 31) : 31, peaks: peaks(song.key) },
      bpmLabel: L("song.bpm"),
      lengthLabel: L("song.length"),
      bestLabel: L("song.best"),
      gradeLabel: L("song.grade"),
      fullComboLabel: L("song.fullCombo"),
      accuracyLabel: L("song.accuracy"),
      playLabel: L("song.play"),
      previewLabel: L("song.preview"),
      newLabel: L("song.new"),
    });
    if (r.selected !== null) {
      sg.selected = r.selected;
      sg.since = og.ui.tick();
    }
    if (r.difficulty !== null) sg.chart = r.difficulty;
    if (r.preview) sg.playing = !sg.playing;
  }, [{ action: "confirm", hint: L("song.play") }, { action: "back", hint: L("hint.back") }]);
}

/** The song select's backdrop: the selected song's jacket, blurred under the screen (a rhythm game's habit). */
export function genreScene(screen: string): [ArtName, { x: number; y: number }] | null {
  if (screen === "songs") return [`songs/${sg.selected}` as ArtName, { x: 0.5, y: 0.5 }];
  return null;
}

/** The previews' commands: `pat:<lobby|chars|vs|draft|map|song>:<key>:<value>`; false when the area is not one of these. */
export function genresCommand(area: string, key: string, value: string): boolean {
  if (area === "lobby") {
    if (key === "ready") lob.ready = value !== "off";
    else if (key === "search") {
      lob.ready = true;
      lob.searching = value !== "off";
      // A command runs outside the systems: the lobby takes the tick on its next build.
      lob.since = -1;
    } else if (key === "found") {
      lob.ready = true;
      lob.searching = true;
      lob.found = value !== "off";
    } else if (key === "reset") lob = lob0();
    else return false;
  } else if (area === "chars") {
    if (key === "hover") cs.entry = value;
    else if (key === "lock") cs.confirmed = value !== "off";
    else if (key === "reset") cs = cs0();
    else return false;
  } else if (area === "vs") {
    if (key === "progress") vsProgress = Number(value);
    else if (key === "reset") vsProgress = 0.64;
    else return false;
  } else if (area === "draft") {
    if (key === "phase") dr.phase = value === "ban" ? "ban" : "pick";
    else if (key === "hover") dr.hovered = value;
    else if (key === "lock") dr.mine = value;
    else if (key === "reset") dr = dr0();
    else return false;
  } else if (area === "map") {
    if (key === "select") wm.selected = value;
    else if (key === "hide") wm.hidden = value ? value.split(",") : [];
    else if (key === "reset") wm = wm0();
    else return false;
  } else if (area === "song") {
    if (key === "select") {
      sg.selected = value;
      sg.since = 0;
    } else if (key === "chart") sg.chart = value;
    else if (key === "playing") sg.playing = value !== "off";
    else if (key === "reset") sg = sg0();
    else return false;
  } else return false;
  return true;
}

/** Every genre preview back to its start (pat:reset). */
export function resetGenres(): void {
  lob = lob0();
  cs = cs0();
  vsProgress = 0.64;
  dr = dr0();
  wm = wm0();
  sg = sg0();
}
