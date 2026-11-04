// The roster and wider-game previews at desktop size (1280×720 CSS px at
// 1×): the party lobby (not ready, ready, in queue, the ready check,
// accepted), the character select (hovered, locked in), the versus screen,
// the draft (a pick, locked in, the ban phase), the world map (its fog, a
// place's detail, kinds turned off) and the song select (selected, a row
// tapped, a locked song, the preview paused); then each right to left, in
// Japanese and at 150 % text, each frame compared with
// crates/og-ui/references/<tier>/kit-genres-<file>.
import { playtest } from "@origingame/engine/playtest";
import { checkFrames } from "./frames.ts";
import { genresScript } from "./patterns.ts";
import { ui } from "./lib.ts";

const script = genresScript(false);

export default playtest({
  name: "the roster and wider-game previews in their states at desktop size",
  size: "1280x720",
  commands: script.commands,
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    t.ok(run.quit, "the script reached quit");
    t.ok(ui(run, "lobby.json").some((e) => e.role === "button" && e.name.startsWith("Lumen") && e.name.includes("Party leader")), "the local player's card names the leader");
    t.ok(ui(run, "lobby-found.json").some((e) => e.text === "Match found"), "the ready check stands over the lobby");
    t.ok(ui(run, "select-locked.json").some((e) => e.id === "Label" && e.text === "Locked in"), "Lock in tapped locked the hovered fighter in");
    t.ok(ui(run, "draft-locked.json").some((e) => e.role === "listitem" && e.name.startsWith("Lumen, Vesper, locked")), "the draft's action locked the hovered Vesper into the player's slot");
    t.ok(ui(run, "map-detail.json").some((e) => e.role === "region" && e.name === "Amberhold"), "a tap on Amberhold's pin opened its detail");
    t.ok(ui(run, "songs.json").some((e) => e.role === "option" && e.name.startsWith("Crystal Echo") && e.states.includes("selected")), "the wheel stands on Crystal Echo");
    t.frame(script.files, () => {
      for (const f of script.files) t.equal(run.png(f).width, 1280, `${f} is 1280 wide`);
    });
    checkFrames(run, t, script.files, "kit-genres-");
  },
});
