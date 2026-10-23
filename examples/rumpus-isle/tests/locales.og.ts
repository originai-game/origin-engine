// French, everywhere the game shows text: the title, the lobby, the round's
// card and HUD, pause, the settings (every tab), the banner, the scoreboard
// and the podium, each dumped after `language:fr`. Every text is from
// fr.ogt, none of the templates' English defaults is left on screen, the
// accents and the French spaces before ? and ! draw from the style's faces,
// the critters' names stay their names, and every focusable control has a
// name.
import { playtest } from "@origingame/engine/playtest";
import { game, press, script, tap, texts } from "./lib.ts";

/** Words the templates draw by default, which a localised game must replace. */
const ENGLISH = ["Settings", "Audio", "Graphics", "Controls", "Language", "Master", "Music", "Effects", "Mute everything", "Done", "Paused", "Resume", "Quit to menu", "Quit to title", "Play", "Back", "Start", "Ready", "Next round", "Quality", "Low", "Medium", "High", "Show frame rate", "Reduce motion", "Reduced motion", "High contrast"];

export default playtest({
  name: "the title, lobby, card, HUD, pause, every settings tab, the banner, the scoreboard and the podium in French, with no English word of a template left",
  commands: script(
    "0.2:language:fr",
    "0.6:dump_ui:title.json",
    "0.7:seat:wasd",
    "0.8:seat:arrows",
    "1.2:dump_ui:lobby.json",
    "1.3:match",
    "1.8:dump_ui:card.json",
    "1.9:go",
    press("Escape", 2.0),
    "2.5:dump_ui:pause.json",
    tap("settings", 2.6),
    "3.2:dump_ui:sound.json",
    tap("Tab#1", 3.3),
    "3.8:dump_ui:display.json",
    tap("Tab#2", 3.9),
    "4.4:dump_ui:controls.json",
    tap("Tab#3", 4.5),
    "5.0:dump_ui:language.json",
    tap("Done", 5.1),
    tap("resume", 5.6),
    "6.0:knock:1",
    "6.3:dump_ui:over.json",
    "8.5:dump_ui:scores.json",
    "8.6:podium:3,2",
    "9.2:dump_ui:podium.json",
    "9.2:get:Game:podium-game.json",
    "9.3:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const screens = ["title", "lobby", "card", "pause", "sound", "display", "controls", "language", "over", "scores", "podium"];
    for (const s of screens) {
      const words = texts(run, `${s}.json`);
      const left = words.filter((w) => ENGLISH.includes(w));
      t.equal(left.join(", "), "", `${s}: no English word of a template on screen`);
    }
    const title = texts(run, "title.json");
    for (const w of ["Rumpus Isle", "Jouer", "Réglages", "Une fête de bousculades à deux, trois ou quatre sur un écran", "Des manettes, ou deux joueurs sur un clavier"]) t.ok(title.includes(w), `the title says "${w}"`);
    const lobby = texts(run, "lobby.json");
    for (const w of ["Qui joue ?", "P1 · Pip", "ZQSD + Espace", "Flèches + Entrée", "Choisit", "Appuyez pour jouer", "Couleur", "Corail", "Crème", "Prêt", "Retour", "Commencer"]) t.ok(lobby.includes(w), `the lobby says "${w}"`);
    const card = texts(run, "card.json");
    for (const w of ["Manche 1 sur 3", "Anneau de bousculade", "1/3 · Anneau de bousculade", "Pause"]) t.ok(card.includes(w), `the card and the HUD say "${w}"`);
    t.ok(card.some((w) => w.startsWith("Poussez les autres dans la mer")), "the card says how in French");
    const pause = texts(run, "pause.json");
    for (const w of ["En pause", "Reprendre", "Réglages", "Retour au salon", "Quitter vers le titre"]) t.ok(pause.includes(w), `pause says "${w}"`);
    const sound = texts(run, "sound.json");
    for (const w of ["Réglages", "Son", "Graphismes", "Commandes", "Langue", "Général", "Musique", "Effets", "Tout couper", "Terminé"]) t.ok(sound.includes(w), `the settings say "${w}"`);
    t.ok(texts(run, "display.json").includes("Animations réduites"), "the graphics tab is French");
    t.ok(texts(run, "controls.json").includes("Vibration"), "the controls tab is French");
    t.ok(texts(run, "language.json").includes("Français"), "the language tab lists Français");
    const over = texts(run, "over.json");
    t.ok(over.includes("Pip remporte la manche !"), "the banner is French, the critter's name its own");
    t.ok(over.includes("2e"), "Juno's place reads 2e");
    const scores = texts(run, "scores.json");
    for (const w of ["Manche 1 : Anneau de bousculade", "1er", "2e", "P2 · Juno", "Manche suivante"]) t.ok(scores.includes(w), `the scoreboard says "${w}"`);
    t.ok(scores.some((w) => w.endsWith(" pts")), "and counts the points in pts");
    t.ok(scores.some((w) => w.startsWith("De nouveaux joueurs peuvent")), "and the drop-in hint");
    const podium = texts(run, "podium.json");
    for (const w of ["Pip règne sur Rumpus Isle !", "Rejouer", "Nouveaux joueurs", "Titre"]) t.ok(podium.includes(w), `the podium says "${w}"`);
    t.equal(game(run, "podium-game.json").a11yUnnamed, 0, "no focusable control without a name");
  },
});
