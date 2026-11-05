import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import type { CreationChoices } from "./choices.ts";

/** Packs are also copied into the released create/ directory by pack-template.ts. */
function styleRoot(): string {
  const here = dirname(fileURLToPath(import.meta.url));
  for (const root of [join(here, "../../../templates/styles"), join(here, "../styles")]) {
    if (existsSync(join(root, "pixel/style.ogt"))) return root;
  }
  throw new Error("creation UI styles are missing; reinstall the engine dist");
}

function source(c: CreationChoices, arena: boolean): string {
  const local = c.players === "local-multiplayer";
  return `// Generated from project.ogt's create block. Keep gameplay on per-player inputs.
import { og } from "@origingame/engine";
import type { GameDesc, InputBinding, World } from "@origingame/engine";

export const creation = ${JSON.stringify(c, null, 2)};

function must<T>(r: { ok: true; value: T } | { ok: false; reason: string }): T {
  if (!r.ok) throw new Error(r.reason);
  return r.value;
}

export function creationGame(game: GameDesc) {
  return og.game({ ...game, start(world) {
    game.start?.(world);
    must(og.render.applyQuality(world, "${c.platform.startsWith("phone-") ? "low" : "medium"}"));
    for (const seat of ${local ? "[0, 1]" : "[0]"}) {
      const move: InputBinding[] = [];
      const action: InputBinding[] = [];
      if (creation.input === "keyboard-mouse") {
        move.push(seat === 0 ? { up: "KeyW", down: "KeyS", left: "KeyA", right: "KeyD" } : { up: "ArrowUp", down: "ArrowDown", left: "ArrowLeft", right: "ArrowRight" });
        action.push({ control: seat === 0 ? "Space" : "Enter" });
      }
${!local ? `      if (creation.input === "gamepad") {
        move.push({ control: "pad.leftXY" });
        action.push({ control: "pad.a" });
      }
` : ""}
      must(og.input.defineAction(world, { name: "move" + seat, kind: "axis2", bindings: move }));
      must(og.input.defineAction(world, { name: "action" + seat, kind: "button", bindings: action }));
    }
${arena ? `    const lock = must(world.resource(og.Lockstep));
    lock.enabled = ${c.players === "online-lockstep"};
    lock.players = 2;
    lock.inputDelay = 2;
    lock.hashEvery = 10;
` : ""}  } });
}

${arena ? `export function creationArenaCamera(world: World, width: number, height: number) {
  const vp = og.ui.viewport(world);
  if (vp.width <= 0 || vp.height <= 0) return;
  const portrait = vp.height > vp.width;
  const touch = creation.input === "touch";
  const side = touch && !portrait ? ${local ? 160 : 240} : 16;
  const left = vp.safeArea.left + side;
  const right = vp.width - vp.safeArea.right - side;
  const top = vp.safeArea.top + 136;
  const bottom = vp.height - vp.safeArea.bottom - (touch && portrait ? 240 : 24);
  const scale = Math.max(width / Math.max(1, right - left), height / Math.max(1, bottom - top));
  const camera = must(world.resource(og.Camera));
  camera.height = scale * vp.height;
  camera.position = { x: width / 2 + (vp.width - left - right) * scale / 2, y: height / 2 + (top + bottom - vp.height) * scale / 2, z: 50 };
}

` : ""}// Call once for each locally controlled seat each tick. Anchors inherit the
// host's live safe area; never bake a particular phone's notch into the game.
export function creationControls(world: World, seat = 0) {
  const vector = og.input.actionVector(world, "move" + seat);
  let x = vector.x, y = vector.y;
  let held = og.input.actionDown(world, "action" + seat);
  let pressed = og.input.actionPressed(world, "action" + seat);
  if (creation.input === "gamepad" && ${local}) {
    const pad = must(world.resource(og.Gamepad));
    const sx = pad.axis("leftX", seat), sy = pad.axis("leftY", seat);
    x = Math.abs(sx) > 0.2 ? sx : 0;
    y = Math.abs(sy) > 0.2 ? -sy : 0;
    held = pad.down("a", seat);
    pressed = pad.pressed("a", seat);
  }
  if (creation.input === "touch") {
    og.ui.anchor("CreationTouch" + seat, { anchor: seat === 0 ? "bottomLeft" : "bottomRight", direction: "${local ? "column" : "row"}", cross: "center", gap: 16 }, () => {
${c.view === "side-2d" ? `      const pad = og.ui.dpad("Move", { size: 112 });
      x = Number(pad.right) - Number(pad.left);
      y = Number(pad.up) - Number(pad.down);` : `      const stick = og.ui.joystick("Move", { size: 112 });
      x = stick.x; y = -stick.y;`}
      const action = og.ui.actionButton("Action", "Go", { size: 64, primary: true });
      held = action.pressed;
      pressed = action.clicked;
    });
  }
  return { x, y, held, pressed, buttons: held ? 1 : 0 };
}
`;
}

/** Apply supported starter wiring; genre and camera selection belong to the next mapping step. */
export function applyCreation(dir: string, c: CreationChoices, lookFile: string, entry: string, archetype: string | undefined, profile: string): string[] {
  const styles = styleRoot();
  const style = join(styles, c.ui);
  if (!existsSync(join(style, "style.ogt"))) throw new Error(`no installable UI style ${c.ui}`);
  const target = join(dir, "assets/styles", c.ui);
  mkdirSync(dirname(target), { recursive: true });
  cpSync(style, target, { recursive: true });
  const look = readFileSync(join(dir, lookFile), "utf8").replace(/^ui\s+"[^"]*"\s*$/gm, "");
  writeFileSync(join(dir, lookFile), `${look}\nui "styles/${c.ui}/style.ogt"\n`);

  const manifest = join(dir, "project.ogt");
  const orientation = c.platform === "phone-portrait" ? "portrait" : c.platform === "phone-landscape" ? "landscape" : "any";
  let text = readFileSync(manifest, "utf8");
  const publish = /^publish[ \t]*\{([\s\S]*?)^\}/m;
  const settings = `\n  orientation ${orientation}\n  fit contain\n  dpr_cap ${c.platform.startsWith("phone-") ? 2 : 3}\n`;
  text = publish.test(text) ? text.replace(publish, (_, body: string) => `publish {${body.replace(/^\s*(orientation|fit|dpr_cap)\s+.*$/gm, "")}${settings}}`) : `${text}\npublish {${settings}}\n`;
  writeFileSync(manifest, text);

  const main = join(dir, entry);
  let code = readFileSync(main, "utf8");
  code = `import { creationGame${!archetype ? ", creationControls" : ""} } from "./creation.ts";\n${code}`;
  code = code.replace("export default og.game({", "export default creationGame({");
  writeFileSync(join(dirname(main), "creation.ts"), source(c, archetype === "arena"));
  if (archetype === "arena") {
    const gameFile = join(dirname(main), "game.ts");
    let game = readFileSync(gameFile, "utf8");
    game = `import { creationControls, creationArenaCamera } from "./creation.ts";\n${game}`;
    game = game.replace('"Hud", { reads: [Duel, og.Lockstep] }', '"Hud", { reads: [Duel, og.Lockstep], writes: [og.Camera] }');
    game = game.replace('  const net = og.net.status(world);', '  creationArenaCamera(world, WIDTH, HEIGHT);\n  const net = og.net.status(world);');
    const from = game.indexOf('export const Submit = defineSystem("Submit"');
    const to = game.indexOf("/** Each fighter", from);
    if (from < 0 || to < 0) throw new Error("arena Submit contract changed");
    const local = c.players === "local-multiplayer";
    const submit = local
      ? `export const Submit = defineSystem("Submit", { reads: [og.Input, og.Gamepad], writes: [og.PlayerInput] }, (q, time, world) => {
  const inputs = [creationControls(world, 0), creationControls(world, 1)];
  for (const a of q.archetypes()) {
    const p = a.column(og.PlayerInput);
    for (let i = 0; i < a.count; i++) {
      const input = inputs[p.player[i]!]!;
      p.x[i] = input.x; p.y[i] = input.y; p.buttons[i] = input.buttons; p.tick[i] = time.tick;
    }
  }
});\n\n`
      : game.slice(from, to).replace(/  const input = must\(world.resource\(og.Input\)\);[\s\S]*?\n\}\);/, `  const input = creationControls(world);
  must(og.net.submit(world, { x: input.x, y: input.y, buttons: input.buttons }));
});`);
    game = game.slice(0, from) + submit + game.slice(to);
    if (local) {
      game = game.replace('"Local: Red"', '"Local: Red and Blue"');
    }
    game = game.replace('og.ui.anchor("Status", { anchor: "bottom", padding: { left: 16, top: 12,', 'og.ui.anchor("Status", { anchor: "top", padding: { left: 16, top: 72,');
    writeFileSync(gameFile, game);
    if (c.players !== "online-lockstep") {
      writeFileSync(join(dir, "tests/duel.og.ts"), `import { playtest } from "@origingame/engine/playtest";
export default playtest({
  name: "local seats do not start a network room",
  commands: "0.1:dump_entity:fighter-0:red.json;0.1:dump_entity:fighter-1:blue.json;0.2:quit",
  check(run, t) {
    t.equal(run.lockstep?.running ?? false, false, "local play never waits for a room");
    t.equal(run.peers.length, 0, "no remote peers");
    t.equal(run.json<{ components: { "og.PlayerInput": { player: number } } }>("red.json").components["og.PlayerInput"].player, 0, "red owns seat zero");
    t.equal(run.json<{ components: { "og.PlayerInput": { player: number } } }>("blue.json").components["og.PlayerInput"].player, 1, "blue owns seat one");
  },
});\n`);
    }
    // The old solo test hard-codes keyboard input and the old HUD text.
    writeFileSync(join(dir, "tests/solo.og.ts"), `import { playtest } from "@origingame/engine/playtest";
export default playtest({
  name: "the selected controls boot outside a room",
  commands: "0.2:expect:ui:text:Red 0;0.2:expect:ui:text:Blue 0;0.3:quit",
});\n`);
  } else if (!archetype && profile === "2d") {
    // The starter already stores HUD touch values in Game, before Hero runs.
    const from = code.indexOf("  // A d-pad and Jump where");
    const to = code.indexOf("  if (g.paused)", from);
    if (from < 0 || to < 0) throw new Error("2D starter HUD contract changed");
    code = code.slice(0, from) + `  const controls = creationControls(world);
  let moveX = controls.x;
  let jump = controls.held;
` + code.slice(to);
    code = code.replace('{ writes: [Game] }, (_q, _time, world) => {', '{ reads: [og.Input, og.Gamepad], writes: [Game] }, (_q, _time, world) => {');
    code = code.replace(/^    const stick = pad\.axis\("leftX", 0\);\n/m, "");
    code = code.replace(/^    const run = Math\.max[^\n]+g\.touchX\)\);/m, "    const run = g.touchX;");
    code = code.replace(/^    const jump = [^\n]+g\.touchJump;/m, "    const jump = g.touchJump;");
  } else if (!archetype) {
    code = code.replace("reads: [Spin, og.Input]", "reads: [Spin, og.Input, og.Gamepad]");
    code = code.replace("  const started = run.value.started;", "  const started = run.value.started;\n  const controls = creationControls(world);");
    code = code.replace('start.clicked || input.value.pressed("KeyS")', 'start.clicked || controls.pressed || input.value.pressed("KeyS")');
  }
  writeFileSync(main, code);
  const readme = join(dir, "README.md");
  writeFileSync(readme, `${readFileSync(readme, "utf8")}\n## Creation choices\n\nThe selection is in project.ogt's create block. src/creation.ts applies input\nand quality; publish applies orientation. UI anchors use the host's live\nsafe area. The selected UI pack is installed under assets/styles/${c.ui}.\nMood, music and languages are recorded for subsequent art/audio/localization\nwork. Genre-to-archetype and view camera presets are not applied yet.\n${c.players === "local-multiplayer" ? "Two seats: WASD + Space and arrows + Enter, or gamepads 0 and 1.\n" : c.players === "online-lockstep" ? "Run og check (Lockstep enables its lint), then og relay and two og run . --room ws://127.0.0.1:47900/game processes. tests/duel.og.ts checks a two-peer room.\n" : ""}`);
  function files(root: string, prefix: string): string[] {
    return readdirSync(root, { withFileTypes: true }).flatMap(e => e.isDirectory() ? files(join(root, e.name), `${prefix}/${e.name}`) : [`${prefix}/${e.name}`]);
  }
  return [`${dirname(entry)}/creation.ts`, ...files(target, `assets/styles/${c.ui}`)];
}
