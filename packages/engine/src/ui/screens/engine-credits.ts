// What the art the engine ships asks every game to credit: the authors of the
// game icon set's game-icons.net silhouettes (CC BY 3.0,
// third_party/game-icons/ICONS.md). The Credits screen lists it after the
// game's own sections; a game that rolls its own credits lists it too.

/** The engine's credits sections, each { id, title (a string key), names }. */
export const engineCredits = [
  {
    id: "engine-icons",
    title: "screen.credits.icons",
    names: [
      { id: "game-icons", name: "game-icons.net", role: "CC BY 3.0" },
      { id: "lorc", name: "Lorc", role: "" },
      { id: "delapouite", name: "Delapouite", role: "" },
      { id: "carl-olsen", name: "Carl Olsen", role: "" },
      { id: "faithtoken", name: "Faithtoken", role: "" },
      { id: "lord-berandas", name: "Lord Berandas", role: "" },
      { id: "sbed", name: "Sbed", role: "" },
      { id: "skoll", name: "Skoll", role: "" },
    ],
  },
] as const;
