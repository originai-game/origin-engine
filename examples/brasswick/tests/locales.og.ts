// Spanish, everywhere the game shows text: the title, the level select, the
// HUD, pause, the settings (every tab), and the results, each dumped after
// `language:es`. Every text is from es.ogt, no English word (the old
// templates' defaults, the settings tree's Apply, Revert and Defaults) is
// left on screen, accents and ¿¡ñ draw from the style's Latin-1
// faces, and every focusable control has a name. What a screen reader hears
// is Spanish too (FINDINGS.md #11, #12, fixed): the settings' language
// choice is named from the table, and a level card by its level and
// its stars ("0 de 3 estrellas"), never by the icons' English names.
import { playtest } from "@origingame/engine/playtest";
import { elements, game, press, script, tap, texts } from "./lib.ts";

/** Words in English a localised game must not leave on screen: the old templates' defaults and the settings tree's. */
const ENGLISH = ["Apply", "Revert", "Defaults", "Settings", "Audio", "Graphics", "Controls", "Language", "Master", "Music", "Effects", "Mute everything", "Done", "Paused", "Resume", "Quit to menu", "Retry", "Next", "Menu", "Share", "New best", "Level complete", "Levels", "Quality", "Low", "Medium", "High", "Show frame rate", "Reduce motion", "High contrast"];

export default playtest({
  name: "the title, levels, HUD, pause, every settings tab and the results in Spanish, with no English word left",
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
    tap("Row/graphics", 5.1),
    "5.6:dump_ui:display.json",
    tap("Row/language", 6.3),
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
    const screens = ["title", "levels", "hud", "pause", "sound", "display", "language", "results"];
    for (const s of screens) {
      const words = texts(run, `${s}.json`);
      const left = words.filter((w) => ENGLISH.some((e) => e.toLowerCase() === w.toLowerCase()));
      t.equal(left.join(", "), "", `${s}: no English word of a template on screen`);
    }
    const title = texts(run, "title.json");
    for (const w of ["Partida nueva", "Niveles", "Ajustes", "Continuar", "Un pequeño autómata de latón sale de la cámara"]) t.ok(title.includes(w), `the title says "${w}"`);
    const levels = texts(run, "levels.json");
    t.ok(levels.includes("1. La escalera de la caldera") && levels.includes("3. La puerta del farol"), "the level select names the levels in Spanish");
    t.ok(texts(run, "hud.json").includes("1. La escalera de la caldera"), "the HUD names the level");
    const pause = texts(run, "pause.json");
    for (const w of ["En pausa", "Seguir", "Volver al farol", "Reiniciar el nivel", "Salir al título"]) t.ok(pause.includes(w), `pause says "${w}"`);
    // The settings tree sets its tabs' and rows' names in capitals: compared without case.
    const has = (file: string, w: string) => texts(run, file).some((x) => x.toLowerCase() === w.toLowerCase());
    for (const w of ["Ajustes", "Sonido", "Pantalla", "Idioma", "Volumen", "Música", "Efectos", "Silenciar todo", "Hecho", "Por defecto", "Deshacer", "Aplicar"]) t.ok(has("sound.json", w), `the settings say "${w}"`);
    t.ok(has("display.json", "Reducir el movimiento"), "the display tab is Spanish");
    t.ok(has("language.json", "Español"), "the language tab lists Español");
    const results = texts(run, "results.json");
    for (const w of ["Nivel 1 superado", "Engranajes", "4 de 6", "Tiempo", "Mejor tiempo", "Caídas", "Siguiente nivel", "Jugar otra vez", "Niveles"]) t.ok(results.includes(w), `the results say "${w}"`);
    t.ok(results.some((w) => w.startsWith("Récord ")), "and the best as Récord");
    t.equal(game(run, "results-game.json").a11yUnnamed, 0, "no focusable control without a name");
    const names = elements(run, "language.json").map((e) => e.name);
    t.ok(names.includes("Idiomas"), "the settings' language choice is named in Spanish");
    t.ok(!names.includes("Languages"), "and not in English");
    const cards = elements(run, "levels.json").filter((e) => e.id.startsWith("Card#")).map((e) => e.name);
    t.equal(cards[0], "1. La escalera de la caldera 0 de 3 estrellas", "a level card is named by its level and its stars in Spanish");
    t.ok(!cards.some((n) => /\b(star|lock)\b/.test(n)), `no card is named by an icon's English name (${cards.join(" | ")})`);
    t.ok(levels.includes("Volver"), "the level select's Back button says Volver");
  },
});
