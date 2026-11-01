// og-ui's element model and motion (docs/ui.md §6, §7), driven from game
// code: a hand of five cards fanned by per-element transforms about their
// bottom edge, dealt by a staggered timeline, reordered with layout
// animation (each card springs from its old slot and turn to its new
// ones), lifted by an interaction state under the pointer; and a results
// card whose panel, stars, score rows and Continue button arrive in one
// sequence, the button inert until a timeline's state event lets it go.
//
// Commands: deal, reorder, results, hand.
//
//   og run examples/ui-motion --size 1280x720 --out film --commands "$(cat film.txt)"
//
// film.txt is the filmstrip docs/ui.md's U2 report reads: the deal, the
// fan, the reorder, a card lifted under the pointer, and the results
// sequence.

import { og, defineSystem } from "@origingame/engine";
import type { Choreo, ElementProps, Transition, UiTimeline } from "@origingame/engine";

const CARDS = [
  { name: "Ember", cost: 1, color: { r: 0.86, g: 0.33, b: 0.24 } },
  { name: "Tide", cost: 2, color: { r: 0.2, g: 0.52, b: 0.86 } },
  { name: "Grove", cost: 3, color: { r: 0.27, g: 0.66, b: 0.38 } },
  { name: "Spark", cost: 2, color: { r: 0.93, g: 0.71, b: 0.22 } },
  { name: "Shade", cost: 4, color: { r: 0.52, g: 0.36, b: 0.78 } },
] as const;

const spring = (durationMs: number, bounce: number): Transition => ({ kind: "spring", durationMs, bounce });

/** The game's state: which screen, and the hand's order (card indices by slot). */
const state = { screen: "hand" as "hand" | "results", order: [0, 1, 2, 3, 4], defined: false };

function timelines(): void {
  // A card flies up from under the screen into its place, turning as it lands.
  const deal: UiTimeline = {
    tracks: [
      { target: "", prop: "y", keys: [{ atMs: 0, value: 360, ease: { kind: "hold" } }, { atMs: 520, value: 0, ease: { kind: "spring", durationMs: 520, bounce: 0.22 } }] },
      { target: "", prop: "rotate", keys: [{ atMs: 0, value: -0.7, ease: { kind: "hold" } }, { atMs: 520, value: 0, ease: { kind: "spring", durationMs: 520, bounce: 0.22 } }] },
      { target: "", prop: "opacity", keys: [{ atMs: 0, value: 0, ease: { kind: "hold" } }, { atMs: 120, value: 1, ease: { kind: "linear" } }] },
    ],
    events: [{ atMs: 0, target: "", action: { kind: "sound", name: "press", pitch: 1 } }],
  };
  // The results panel rises and settles; a star pops with a burst; a row slides in.
  const panelIn: UiTimeline = {
    tracks: [
      { target: "", prop: "y", keys: [{ atMs: 0, value: 80, ease: { kind: "hold" } }, { atMs: 420, value: 0, ease: { kind: "spring", durationMs: 420, bounce: 0.15 } }] },
      { target: "", prop: "opacity", keys: [{ atMs: 0, value: 0, ease: { kind: "hold" } }, { atMs: 200, value: 1, ease: { kind: "ease", ease: "out" } }] },
    ],
  };
  const pop: UiTimeline = {
    tracks: [
      { target: "", prop: "scale", keys: [{ atMs: 0, value: 0, ease: { kind: "hold" } }, { atMs: 450, value: 1, ease: { kind: "spring", durationMs: 450, bounce: 0.5 } }] },
      // Ends on the star's own turn (a diamond), so nothing jumps when the play lets go.
      { target: "", prop: "rotate", keys: [{ atMs: 0, value: -0.8, ease: { kind: "hold" } }, { atMs: 450, value: 0.785, ease: { kind: "spring", durationMs: 450, bounce: 0.4 } }] },
    ],
    events: [
      { atMs: 120, target: "", action: { kind: "burst", count: 12, radius: 46, color: { r: 1, g: 0.85, b: 0.3 } } },
      { atMs: 120, target: "", action: { kind: "sound", name: "success", pitch: 1 } },
    ],
  };
  const rise: UiTimeline = {
    tracks: [
      { target: "", prop: "x", keys: [{ atMs: 0, value: -40, ease: { kind: "hold" } }, { atMs: 260, value: 0, ease: { kind: "ease", ease: "out" } }] },
      { target: "", prop: "opacity", keys: [{ atMs: 0, value: 0, ease: { kind: "hold" } }, { atMs: 260, value: 1, ease: { kind: "linear" } }] },
    ],
  };
  // The button wakes: its `ready` group goes to `on` (the state's transition fades it in).
  const ready: UiTimeline = {
    events: [{ atMs: 0, target: "", action: { kind: "state", group: "ready", state: "on" } }],
    durationMs: 1,
  };
  og.ui.defineTimeline("deal", deal);
  og.ui.defineTimeline("panelIn", panelIn);
  og.ui.defineTimeline("pop", pop);
  og.ui.defineTimeline("rise", rise);
  og.ui.defineTimeline("ready", ready);
}

/** A card's fan in slot k of n: turned out from the middle about its bottom edge, the outer ones lower. */
function fan(k: number, n: number): ElementProps {
  const d = k - (n - 1) / 2;
  return {
    transform: { rotate: d * 0.13, y: d * d * 7, pivotX: 0.5, pivotY: 1 },
    z: k,
    // Turn and drop spring to a new slot's; the slot itself moves by layout animation.
    motion: spring(480, 0.25),
    layoutMotion: spring(520, 0.2),
    states: [
      {
        name: "interaction",
        states: [
          { name: "normal", values: [], onEnter: "", onExit: "" },
          { name: "hover", values: [{ prop: "y", value: -46 }, { prop: "scale", value: 1.12 }, { prop: "rotate", value: 0 }, { prop: "z", value: 20 }], onEnter: "", onExit: "" },
        ],
        transitions: [{ from: "*", to: "*", transition: spring(260, 0.3) }],
      },
    ],
    overlaps: ["Hand"],
  };
}

function hand(): void {
  og.ui.anchor("Hand", { anchor: "bottom", direction: "row", gap: 14, offsetY: -36 }, () => {
    state.order.forEach((card, k) => {
      const c = CARDS[card]!;
      og.ui.open(og.ui.idi("Card", card), {
        width: { kind: "fixed", px: 132 },
        height: { kind: "fixed", px: 188 },
        direction: "column",
        align: "spaceBetween",
        cross: "center",
        padding: { left: 10, top: 12, right: 10, bottom: 14 },
        background: { color: c.color, alpha: 1 },
        radius: 14,
        interactive: true,
      });
      og.ui.label("Cost", String(c.cost), { step: "xl2", color: { token: "inkInverse" } });
      og.ui.label("Name", c.name, { step: "lg", color: { token: "inkInverse" } });
      og.ui.close();
      og.ui.props(fan(k, state.order.length));
    });
  });
}

function results(): void {
  og.ui.anchor("Results", { anchor: "center", direction: "column", gap: 18, cross: "center", padding: { left: 40, top: 28, right: 40, bottom: 28 }, background: { token: "panel" }, radius: 20, width: { kind: "fixed", px: 460 } }, () => {
    og.ui.label("Title", "Victory", { step: "displayMd" });
    og.ui.row("Stars", { gap: 18 }, () => {
      for (let i = 0; i < 3; i++) {
        og.ui.open(og.ui.idi("Star", i), { width: { kind: "fixed", px: 64 }, height: { kind: "fixed", px: 64 }, background: { color: { r: 1, g: 0.78, b: 0.2 }, alpha: 1 }, radius: 18 });
        og.ui.close();
        og.ui.props({ transform: { rotate: 0.785 } });
      }
    });
    og.ui.column("Rows", { gap: 8, width: { kind: "grow" } }, () => {
      for (const [k, v] of [["Damage", "1,240"], ["Cards played", "17"], ["Best combo", "x6"]] as const) {
        og.ui.row(k, { width: { kind: "grow" }, align: "spaceBetween" }, () => {
          og.ui.label("Key", k, { color: { token: "inkMuted" } });
          og.ui.label("Value", v, { step: "lg" });
        });
      }
    });
    og.ui.button("Continue", "Continue", { width: { kind: "fixed", px: 220 } });
    og.ui.props({
      states: [
        {
          name: "ready",
          states: [
            { name: "off", values: [{ prop: "opacity", value: 0.25 }, { prop: "inert", value: 1 }], onEnter: "", onExit: "" },
            { name: "on", values: [{ prop: "opacity", value: 1 }, { prop: "inert", value: 0 }], onEnter: "", onExit: "" },
          ],
          transitions: [{ from: "off", to: "on", transition: spring(300, 0.2) }],
        },
      ],
    });
  });
}

/** The results sequence: the panel, then the stars one by one, the rows, then the button. */
const RESULTS: Choreo = {
  kind: "sequence",
  steps: [
    { kind: "play", timeline: "panelIn", target: "", delayMs: 0 },
    { kind: "stagger", timeline: "pop", targets: ["Stars/Star#0", "Stars/Star#1", "Stars/Star#2"], stepMs: 220, capMs: 600, from: "start" },
    { kind: "stagger", timeline: "rise", targets: ["Rows/Damage", "Rows/Cards played", "Rows/Best combo"], stepMs: 90, capMs: 300, from: "start" },
    { kind: "play", timeline: "ready", target: "Continue", delayMs: 0 },
  ],
};

let pending: "deal" | "results" | null = null;

export const Motion = defineSystem("Motion", { reads: [og.Input] }, () => {
  if (!state.defined) {
    timelines();
    state.defined = true;
  }
  if (state.screen === "hand") hand();
  else results();
  if (pending === "deal") og.ui.play("Hand", { kind: "stagger", timeline: "deal", targets: [], stepMs: 70, capMs: 280, from: "center" }, {});
  if (pending === "results") og.ui.play("Results", RESULTS, {});
  pending = null;
});

export default og.game({
  tick: 60,
  seed: 1,
  systems: [Motion],
  start(world) {
    const camera = world.resource(og.Camera);
    if (camera.ok) {
      camera.value.orthographic = true;
      camera.value.height = 360;
      camera.value.background = { r: 0.07, g: 0.08, b: 0.11 };
    }
    const font = og.load.font(world, "fonts/ui.ttf");
    if (font.ok) og.ui.configure(world, { font: font.value });
  },
  commands: {
    deal: () => {
      state.screen = "hand";
      pending = "deal";
    },
    // The first card goes to the end and the last two swap: every card moves.
    reorder: () => {
      const [first, ...rest] = state.order;
      state.order = [...rest, first!];
      const n = state.order.length;
      [state.order[n - 2], state.order[n - 1]] = [state.order[n - 1]!, state.order[n - 2]!];
    },
    results: () => {
      state.screen = "results";
      pending = "results";
    },
    hand: () => {
      state.screen = "hand";
    },
  },
  commandHelp: {
    deal: { usage: "", doc: "Deal the hand: a staggered timeline from the centre out." },
    reorder: { usage: "", doc: "Move the first card to the end and swap the last two (layout animation)." },
    results: { usage: "", doc: "Show the results card and play its sequence." },
    hand: { usage: "", doc: "Back to the hand." },
  },
});
