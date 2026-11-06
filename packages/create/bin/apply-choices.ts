import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { CreationChoices } from "./choices.ts";
import { applyLanguages } from "./languages.ts";
import { creationModule, STARTERS, VIEW_PRESETS } from "./starters.ts";
import type { StarterName } from "./starters.ts";

const here = dirname(fileURLToPath(import.meta.url));

/** Packs are also copied into the released create/ directory by pack-template.ts. */
function styleRoot(): string {
  for (const root of [join(here, "../../../templates/styles"), join(here, "../styles")]) {
    if (existsSync(join(root, "pixel/style.ogt"))) return root;
  }
  throw new Error("creation UI styles are missing; reinstall the engine dist");
}

/** The official kits: packages/engine/extensions in the repository, engine/extensions beside create/ in a dist. */
function kitRoot(): string {
  const root = join(here, "../../engine/extensions");
  if (existsSync(join(root, "shooter/extension.json"))) return root;
  throw new Error("the engine's UI kits are missing (engine/extensions beside create/); reinstall the engine dist");
}

function files(root: string, prefix: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? files(join(root, e.name), `${prefix}/${e.name}`) : [`${prefix}/${e.name}`]));
}

/** The view's camera as the look's ArtCamera block (docs/art.md §2.4): replaces one the look had. */
function artCamera(look: string, c: CreationChoices): string {
  const camera = VIEW_PRESETS[c.view].camera;
  const lines = Object.entries(camera).map(([k, v]) => `  ${k} ${v}`);
  const block = `# The ${c.view} view (project.ogt's create block; packages/create/bin/starters.ts).\nArtCamera {\n${lines.join("\n")}\n}\n`;
  const without = look.replace(/^(# The [^\n]* view \(project\.ogt's create block[^\n]*\n)?ArtCamera \{[^}]*\}\n?/m, "");
  return `${without.replace(/\n*$/, "\n")}${block}`;
}

/** An online game's room playtest: two processes, each seat a scripted bot, one world hash at the end. */
const ROOM_TEST = `// Two processes play one room in lockstep through a relay (docs/design.md
// §2.10): each drives its own seat by a different scripted bot (src/seats.ts
// \`bot:<n>\`), and both quit on tick 600, where both worlds hash the same
// and no og.Desync was sent. og new wrote it for players online-lockstep.
import { playtest } from "@origingame/engine/playtest";

export default playtest({
  name: "two processes in one room play the same game to tick 600",
  commands: "0:bot:1;10:quit",
  room: { players: 2, commands: ["0:bot:1;10:quit", "0:bot:2;10:quit"], name: "room" },
  check(run, t) {
    t.equal(run.peers.length, 1, "one peer beside the run");
    const peer = run.peers[0]!;
    t.equal(run.tick, 600, "the run quit on tick 600");
    t.equal(peer.tick, 600, "the peer quit on tick 600");
    t.ok(run.lockstep?.running === true && peer.lockstep?.running === true, "lockstep ran on both");
    t.equal(run.lockstep?.desync ?? null, null, "no desync on the run");
    t.equal(peer.lockstep?.desync ?? null, null, "no desync on the peer");
    t.ok(run.netHash.length > 0, "the run has a shared hash");
    t.equal(peer.netHash, run.netHash, \`both worlds hash \${run.netHash} at tick 600\`);
  },
});
`;

/**
 * Writes the selection into a scaffolded starter: its src/creation.ts (the
 * seats, controls and camera src/seats.ts reads), the UI style and the
 * genre's kit installed, the view's ArtCamera in the look, the platform's
 * publish settings, an online game's room playtest, and a README section.
 * Returns the files it added.
 */
export function applyCreation(dir: string, c: CreationChoices, lookFile: string, entry: string, starter: StarterName, notes: string[] = []): string[] {
  // The shipped translations are kept by the starter's directory (templates/locales/strings/game-2d).
  const own = (STARTERS[starter] as { dir?: string }).dir;
  const langs = applyLanguages(dir, c.lang, { entry, starter: own ?? starter, archetype: own === undefined });
  notes.push(...langs.notes);
  if (langs.load) notes.push(`the ${starter} starter draws its words as literals: move them into assets/strings/ (localization §1) before the language tables show anything`);
  const written: string[] = [...langs.files];
  const style = join(styleRoot(), c.ui);
  if (!existsSync(join(style, "style.ogt"))) throw new Error(`no installable UI style ${c.ui}`);
  const styleTarget = join(dir, "assets/styles", c.ui);
  mkdirSync(dirname(styleTarget), { recursive: true });
  cpSync(style, styleTarget, { recursive: true });
  written.push(...files(styleTarget, `assets/styles/${c.ui}`));
  // The mood asked of the preset (docs/art.md §2.10) goes beside `extends`; the cook lays its layer, the art brief reads it.
  let look = readFileSync(join(dir, lookFile), "utf8").replace(/^ui\s+"[^"]*"\s*$/gm, "").replace(/^mood\s+"[^"]*"\s*\n/gm, "");
  const moodLine = `mood "${c.mood}"`;
  look = /^extends\s+"[^"]*"\s*$/m.test(look) ? look.replace(/^(extends\s+"[^"]*")\s*$/m, `$1\n${moodLine}`) : `${look.replace(/\n*$/, "\n")}${moodLine}\n`;
  writeFileSync(join(dir, lookFile), artCamera(`${look.replace(/\n*$/, "\n")}ui "styles/${c.ui}/style.ogt"\n`, c));

  const kit = STARTERS[starter].kit as string | null;
  if (kit) {
    const target = join(dir, "assets/ui-kits", kit);
    if (!existsSync(target)) {
      mkdirSync(dirname(target), { recursive: true });
      cpSync(join(kitRoot(), kit), target, { recursive: true });
      written.push(...files(target, `assets/ui-kits/${kit}`));
    }
  }

  const manifest = join(dir, "project.ogt");
  const orientation = c.platform === "phone-portrait" ? "portrait" : c.platform === "phone-landscape" ? "landscape" : "any";
  let text = readFileSync(manifest, "utf8");
  const publish = /^publish[ \t]*\{([\s\S]*?)^\}/m;
  const settings = `\n  orientation ${orientation}\n  fit contain\n  dpr_cap ${c.platform.startsWith("phone-") ? 2 : 3}\n`;
  text = publish.test(text) ? text.replace(publish, (_, body: string) => `publish {${body.replace(/^\s*(orientation|fit|dpr_cap)\s+.*$/gm, "").replace(/\n+$/, "")}${settings}}`) : `${text}\npublish {${settings}}\n`;
  writeFileSync(manifest, text);

  const creation = join(dir, dirname(entry), "creation.ts");
  if (!existsSync(join(dir, dirname(entry), "seats.ts"))) throw new Error(`the ${starter} starter has no src/seats.ts: it cannot take creation choices`);
  writeFileSync(creation, creationModule(starter, c, langs.load ?? []));
  const tests = join(dir, "tests");
  const rooms = existsSync(tests) ? readdirSync(tests).filter((f) => f.endsWith(".og.ts") && /^\s*room:\s*\{/m.test(readFileSync(join(tests, f), "utf8"))) : [];
  if (c.players === "online-lockstep") {
    // A starter that plays online already has its room playtest (arena's duel).
    if (rooms.length === 0) {
      writeFileSync(join(tests, "room.og.ts"), ROOM_TEST);
      written.push("tests/room.og.ts");
    }
  } else {
    // A room playtest needs lockstep, which only online play starts: a starter committed online (arena) loses it.
    for (const f of rooms) {
      rmSync(join(tests, f));
      notes.push(`tests/${f} is a two-process room playtest, removed: ${c.players} starts no room (choose online-lockstep to keep it)`);
    }
  }

  const seats = c.players === "local-multiplayer"
    ? "Two seats on this device: WASD and the left keys against the arrows and the right keys, pad 1 against pad 2, or two sets of touch controls.\n"
    : c.players === "online-lockstep"
      ? "Two seats over a room in lockstep: og relay, then og run . --room ws://127.0.0.1:47900/game twice (tests/room.og.ts runs two peers and compares their worlds).\n"
      : "One seat, from the keys, the mouse, a pad or the touch controls.\n";
  const readme = join(dir, "README.md");
  // A section written before (choices applied again) is replaced, up to the next heading.
  const prose = readFileSync(readme, "utf8").replace(/\n*^## Creation choices\n[\s\S]*?(?=^## |(?![\s\S]))/m, "\n");
  writeFileSync(readme, `${prose.replace(/\n*$/, "\n")}
## Creation choices

project.ogt's create block holds the selection; og new wrote what it
starts into src/creation.ts (the ${starter} starter, its seats, controls and
camera), and src/seats.ts gives every seat's input to og.PlayerInput, the
only input the game reads. ${seats}
The ${c.view} view is the look's ArtCamera (${lookFile}); the UI wears
assets/styles/${c.ui}${kit ? `, and the ${kit} kit is installed under assets/ui-kits/${kit}` : ""}.
The platform's orientation and pixel ratio are in project.ogt's publish
block. The look carries the chosen mood, assets/strings holds the chosen
languages' tables, and the music is recorded for the audio step.
`);
  return written;
}
