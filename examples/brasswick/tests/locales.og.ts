// Spanish, everywhere the game shows text: the title, the level select, the
// HUD, pause, the settings (every tab), and the results, each dumped after
// `language:es`. Every text is from es.ogt, none of the templates' English
// defaults is left on screen, accents and ¿¡ñ draw from the skin's Latin-1
// faces, and every focusable control has a name. What a screen reader hears
// is Spanish too (FINDINGS.md #11, #12, fixed): the settings' tab row and
// language list are named from the table, and a level card by its level and
// its stars ("0 de 3 estrellas"), never by the icons' English names.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, press, script, tap, texts } from "./lib.ts";

/** Words the templates draw by default, which a localised game must replace. */
const ENGLISH = ["Settings", "Audio", "Graphics", "Controls", "Language", "Master", "Music", "Effects", "Mute everything", "Done", "Paused", "Resume", "Quit to menu", "Retry", "Next", "Menu", "Share", "New best", "Level complete", "Levels", "Quality", "Low", "Medium", "High", "Show frame rate", "Reduce motion", "High contrast"];

export default playtest({
  name: "the title, levels, HUD, pause, every settings tab and the results in Spanish, with no English word of a template left",
  commands: script(
    "0.2:language:es",
    "0.2:unlock:3",
    "1.0:dump_ui:title.json",
    "1.0:get:Game:title-game.json",
    tap("levels", 1.1),
    "1.8:dump_ui:levels.json",
    press("Escape", 1.9),
    tap("new", 2.4),
    "3.5:dump_ui:hud.json",
    press("Escape", 3.6),
    "4.2:dump_ui:pause.json",
    tap("settings", 4.3),
    "5.0:dump_ui:sound.json",
    tap("Tab#1", 5.1),
    "5.6:dump_ui:display.json",
    tap("Tab#2", 5.7),
    "6.2:dump_ui:controls.json",
    tap("Tab#3", 6.3),
    "6.8:dump_ui:language.json",
    tap("Done", 6.9),
    tap("resume", 7.4),
    "7.8:gears:4",
    "7.9:finish",
    "9.4:dump_ui:results.json",
    "9.4:get:Game:results-game.json",
    "9.5:quit",
  ),
  check(run, t) {
    t.equal(run.warnings.join("; "), "", "the script ran without warnings");
    const screens = ["title", "levels", "hud", "pause", "sound", "display", "controls", "language", "results"];
    for (const s of screens) {
      const words = texts(run, `${s}.json`);
      const left = words.filter((w) => ENGLISH.includes(w));
      t.equal(left.join(", "), "", `${s}: no English word of a template on screen`);
    }
    const title = texts(run, "title.json");
    for (const w of ["Partida nueva", "Niveles", "Ajustes", "Continuar", "Un pequeño autómata de latón sale de la cámara"]) t.ok(title.includes(w), `the title says "${w}"`);
    const levels = texts(run, "levels.json");
    t.ok(levels.includes("1. La escalera de la caldera") && levels.includes("3. La puerta del farol"), "the level select names the levels in Spanish");
    t.ok(texts(run, "hud.json").includes("1. La escalera de la caldera"), "the HUD names the level");
    const pause = texts(run, "pause.json");
    for (const w of ["En pausa", "Seguir", "Volver al farol", "Reiniciar el nivel", "Salir al título"]) t.ok(pause.includes(w), `pause says "${w}"`);
    const sound = texts(run, "sound.json");
    for (const w of ["Ajustes", "Sonido", "Pantalla", "Controles", "Idioma", "Volumen", "Música", "Efectos", "Silenciar todo", "Hecho"]) t.ok(sound.includes(w), `the settings say "${w}"`);
    t.ok(texts(run, "display.json").includes("Reducir el movimiento"), "the display tab is Spanish");
    t.ok(texts(run, "language.json").includes("Español"), "the language tab lists Español");
    const results = texts(run, "results.json");
    for (const w of ["Nivel 1 superado", "Engranajes", "4 de 6", "Tiempo", "Mejor tiempo", "Caídas", "Siguiente nivel", "Jugar otra vez", "Niveles"]) t.ok(results.includes(w), `the results say "${w}"`);
    t.ok(results.some((w) => w.startsWith("Récord ")), "and the best as Récord");
    t.equal(game(run, "results-game.json").a11yUnnamed, 0, "no focusable control without a name");
    const names = elements(run, "language.json").map((e) => e.name);
    t.ok(names.includes("Apartado") && names.includes("Idiomas"), "the settings' tab row and language list are named in Spanish");
    t.ok(!names.includes("Section") && !names.includes("Languages"), "and not in English");
    const cards = elements(run, "levels.json").filter((e) => e.kind === "cell").map((e) => e.name);
    t.equal(cards[0], "1. La escalera de la caldera 0 de 3 estrellas", "a level card is named by its level and its stars in Spanish");
    t.ok(!cards.some((n) => /\b(star|lock)\b/.test(n)), `no card is named by an icon's English name (${cards.join(" | ")})`);
    t.ok(levels.includes("Volver"), "the level select's Back button says Volver");
  },
});
