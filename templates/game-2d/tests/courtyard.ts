// The courtyard's playtests (the top-down-2d view, src/courtyard.ts):
// tests/smoke.og.ts and tests/keeper.og.ts play these in place of the
// rooftops' when src/creation.ts's view is top-down-2d.
//
// The smoke: Escape pauses and resumes; the courier walks up the yard's
// middle over two coins, into the canal (a life, back on the south bank),
// dashes over it (ghosts behind it) and reaches the gate, which clears the
// courtyard. Its frames are compared with references/metal/ in the three
// scaffolds that have them: alone, two on one desktop, two on a portrait
// phone.
//
// The keeper: the lamp keeper breathes beside the gate, both feet's IK
// reached on the paving, its lantern lit, and the courier walks up to it
// and no further.
import type { PlaytestDesc } from "@origingame/engine/playtest";
import { creation } from "../src/creation.ts";
import { checkFrames } from "./frames.ts";

const platform: string = creation.platform;
const players: string = creation.players;
const seats: number = creation.seats;

interface Got<T> {
  value: T;
}

interface Game {
  coins: number;
  lives: number;
  won: boolean;
  paused: boolean;
  heroX: number;
  heroY: number;
}

interface EntityDump {
  components: Record<string, Record<string, number | boolean | string | number[]>>;
}

interface UiDump {
  elements: Array<{ id: string; text: string }>;
}

/** The canal's water (src/courtyard.ts's Yard, from the map's `canal`), the gate's step and the yard's middle column. */
const CANAL = { y0: 15, y1: 16 } as const;
const GATE_Y = 19;
const MIDDLE = 17;

/** The scaffolds whose frames are recorded: alone, two on a desktop, two on a portrait phone. */
const frames =
  players === "single" && platform === "desktop" ? "courtyard-single-" :
  players === "local-multiplayer" && platform === "desktop" ? "courtyard-local-" :
  players === "local-multiplayer" && platform === "phone-portrait" ? "courtyard-phone-local-" :
  null;

const size = platform === "phone-portrait" ? { size: "390x844" } : platform === "phone-landscape" ? { size: "844x390" } : {};

export const smoke: PlaytestDesc = {
  ...size,
  name: "The courier walks the courtyard over two coins, falls in the canal, dashes over it and reaches the gate; Escape pauses and resumes",
  commands:
    "0.1:key_down:Escape;0.15:key_up:Escape;0.25:get:Game:paused.json;0.3:key_down:Escape;0.35:key_up:Escape;0.45:get:Game:resumed.json;" +
    "0.48:screenshot:start.png;" +
    // Seat 0's own key (W): alone every key plays it, beside a second seat the arrows are seat 1's.
    "0.5:key_down:KeyW;3.1:key_up:KeyW;3.4:get:Game:fell.json;3.41:dump_entity:hero:fell-hero.json;" +
    "3.5:key_down:KeyW;3.5:key_down:Space;3.55:key_up:Space;3.6:screenshot:dash.png;3.75:get:Game:dashed.json;3.76:dump_entity:hero:dashed-hero.json;" +
    "4.6:key_up:KeyW;4.7:get:Game:gate.json;4.75:dump_ui:ui.json;4.8:screenshot:gate.png;4.9:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    const game = (f: string) => run.json<Got<Game>>(f).value;
    t.ok(game("paused.json").paused, "Escape opened the pause menu");
    t.ok(!game("resumed.json").paused, "Escape again resumed");
    const fell = game("fell.json");
    t.ok(fell.coins === 2, `the walk up the yard's middle took two coins (${fell.coins})`);
    t.ok(fell.lives === 2, `the canal cost a life (${fell.lives} left)`);
    t.ok(fell.heroY < CANAL.y0 && fell.heroY > CANAL.y0 - 1 && Math.abs(fell.heroX - MIDDLE) < 0.05, `and the courier stands again on the south bank (${fell.heroX.toFixed(2)}, ${fell.heroY.toFixed(2)})`);
    const fellWalker = run.json<EntityDump>("fell-hero.json").components["Walker"]!;
    t.ok(fellWalker["falls"] === 1, `it fell in once (${fellWalker["falls"]})`);
    const dashed = game("dashed.json");
    t.ok(dashed.heroY > CANAL.y1 && dashed.lives === 2, `a dash carried it over the canal dry (y ${dashed.heroY.toFixed(2)}, ${dashed.lives} lives)`);
    const hero = run.json<EntityDump>("dashed-hero.json").components;
    t.ok(hero["Walker"]!["dashes"] === 1, `one dash (${hero["Walker"]!["dashes"]})`);
    t.ok((hero["Walker"]!["cooldown"] as number) > 0, `whose cooldown is running (${hero["Walker"]!["cooldown"]} ticks)`);
    t.ok(hero["og.CharacterController"]!["gravityScale"] === 0, "the courier is og.CharacterController's top-down mode (gravityScale 0)");
    const gate = game("gate.json");
    t.ok(gate.won && gate.heroY > GATE_Y, `the courier reached the gate and cleared the courtyard (y ${gate.heroY.toFixed(2)}, won ${gate.won})`);
    const ui = run.json<UiDump>("ui.json");
    const count = ui.elements.find((e) => e.id === "Count");
    t.ok(count !== undefined && count.text === "2", `the HUD counts the coins ("${count?.text}")`);
    const level = ui.elements.find((e) => e.id === "Level");
    t.ok(level !== undefined && level.text === "Courtyard clear!", `the HUD says so ("${level?.text}")`);
    if (frames !== null) checkFrames(run, t, ["start.png", "dash.png", "gate.png"], frames);
    else t.frame(["start.png", "dash.png", "gate.png"], () => t.ok(run.png("gate.png").width > 0, `the frames were captured (${seats} seats, ${platform}: no references for this scaffold)`));
  },
};

export const keeper: PlaytestDesc = {
  name: "The lamp keeper breathes beside the gate with both feet planted and its lantern lit, and the courier walks up to it, not through it",
  commands:
    "1.0:dump_entity:keeper:keeper.json;1.01:dump_entity:keeper-leg-f:front.json;1.02:dump_entity:keeper-leg-b:back.json;1.03:dump_entity:lantern:lantern.json;" +
    "1.1:warp:12.5:17.5;1.2:key_down:KeyW;2.2:key_up:KeyW;2.3:get:Game:blocked.json;2.4:dump_entity:keeper:keeper-later.json;2.5:quit",
  check(run, t) {
    t.ok(run.quit, "the script reached quit");
    const keeper = run.json<EntityDump>("keeper.json").components;
    const s = keeper["og.Skeleton2D"]!;
    t.ok(s["clip"] === 0 && (s["time"] as number) > 0.5, `the keeper plays its idle clip (clip ${s["clip"]}, time ${s["time"]})`);
    const later = run.json<EntityDump>("keeper-later.json").components["og.Skeleton2D"]!;
    t.ok(later["time"] !== s["time"], `and goes on breathing (time ${s["time"]} then ${later["time"]})`);
    for (const [file, label] of [["front.json", "front"], ["back.json", "back"]] as const) {
      const ik = run.json<EntityDump>(file).components["og.IkTarget2D"]!;
      t.ok(ik["reached"] === true, `the ${label} foot's IK reached its place on the paving (error ${ik["error"]})`);
    }
    t.ok(run.json<EntityDump>("lantern.json").components["og.Light2D"] !== undefined, "its lantern is an og.Light2D");
    const blocked = run.json<Got<Game>>("blocked.json").value;
    t.ok(blocked.heroY > 18 && blocked.heroY < 18.8, `the courier walked up to the keeper and stopped below it (y ${blocked.heroY.toFixed(2)}; the wall would stop it at 19.6)`);
  },
};
