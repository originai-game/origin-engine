// Eleven frames at 1280x720 compared with references/<tier>/: the title with
// the records in its footer, the lobby with three seated, the round's card
// at 3, the bump ring as two critters meet, the splash and banner as one is
// knocked out, the scoreboard, the berry rush with berries up, the crumble
// tiles shaking under the critters, the pause menu, the settings in French,
// and the podium. `OG_FX_CAPTURE=1 og test .` records this tier's references.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { playtest } from "@origingame/engine/playtest";
import { checkFrames, referencesDir } from "./frames.ts";
import { game, hold, pad, PAD0, script, tap } from "./lib.ts";

const FILES = ["title.png", "lobby.png", "card.png", "bump.png", "splash.png", "scores.png", "berry.png", "crumble.png", "pause.png", "settings-fr.png", "podium.png"];

export default playtest({
  name: "the title, lobby, card, a bump, a splash, the scoreboard, berries, cracking tiles, pause, the French settings and the podium match their reference frames",
  size: "1280x720",
  commands: script(
    "0.2:records:12:34.5",
    "0.8:screenshot:title.png",
    "1.0:seat:wasd",
    "1.1:seat:arrows",
    pad("a", PAD0, 1.2),
    "1.8:screenshot:lobby.png",
    "1.9:match",
    "3.0:screenshot:card.png",
    "3.1:go",
    hold("KeyA", 3.3, 0.8),
    hold("ArrowRight", 3.3, 0.8),
    "4.2:get:Game:bump.json",
    "4.2:screenshot:bump.png",
    "4.3:knock:2",
    "4.5:screenshot:splash.png",
    "4.7:knock:1",
    "8.0:screenshot:scores.png",
    "8.1:next",
    "8.2:go",
    "10.2:get:Game:berry.json",
    "10.2:screenshot:berry.png",
    "10.3:finish",
    "12.1:next",
    "12.2:go",
    "13.6:get:Game:crumble.json",
    "13.6:screenshot:crumble.png",
    "13.7:open_pause",
    "14.3:screenshot:pause.png",
    tap("settings", 14.4),
    "14.5:language:fr",
    "15.1:screenshot:settings-fr.png",
    "15.2:language:en",
    tap("Done", 15.3),
    tap("resume", 15.8),
    "16.0:podium:9,8,7",
    "17.0:screenshot:podium.png",
    "17.1:quit",
  ),
  check(run, t) {
    const project = dirname(referencesDir());
    for (const file of ["atlas.png", "icons.png"]) {
      const copied = createHash("sha256").update(readFileSync(resolve(project, "assets/styles/rumpus", file))).digest("hex");
      const source = createHash("sha256").update(readFileSync(resolve(project, "../../templates/styles/cartoon", file))).digest("hex");
      t.equal(copied, source, `${file} must match the cartoon pack: run bun tools/make-assets.ts`);
      if (copied !== source) return;
    }
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(game(run, "bump.json").bumps >= 1, "the critters met as the bump frame is taken");
    t.ok(game(run, "berry.json").berriesTaken === 0 && game(run, "berry.json").round === 1, "berries are up and untaken as the berry frame is taken");
    const crumble = game(run, "crumble.json");
    t.ok(crumble.round === 2 && crumble.cracks >= 1 && crumble.splashes === 2, `tiles are cracking under the still critters as the crumble frame is taken (${crumble.cracks} cracks)`);
    t.frame(FILES, () => {
      for (const f of FILES) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, FILES, "");
  },
});
