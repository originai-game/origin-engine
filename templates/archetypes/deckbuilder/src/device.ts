// This device's own, never in the world: the player's settings (the
// volumes, the language, reduced motion), loaded from og.platform's
// settings on the first ticks and saved by the settings' Done. They change
// presentation alone (the buses in og.Audio, each peer's own; the strings'
// locale; the camera's shake and the screens' dissolve), so in a room each
// player keeps their own and no peer's settings enter the world or its
// hash. src/ui.ts loads, applies and saves them; src/systems.ts's feel
// reads reduced motion.

/** The player's settings on this device, its settings request, and how often Done saved them. */
export const prefs = { master: 1, music: 0.7, effects: 1, language: 0, reducedMotion: false, loadRequest: 0, saves: 0 };
