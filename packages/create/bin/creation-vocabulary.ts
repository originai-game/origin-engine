/**
 * The creation vocabulary: every choice's values and the selection's type.
 * Imported by choices.ts (defaults and inference) and starters.ts (what each
 * choice starts) alike, so neither has to load the other first.
 */
export const CREATION_CHOICES = {
  genre: ["action-platformer", "shooter", "rpg", "puzzle-match", "strategy-tower-defense", "racing", "runner", "roguelike-card", "idle", "rhythm", "arena", "open-world"],
  view: ["side-2d", "top-down-2d", "isometric-2.5d", "third-person-3d", "first-person-3d"],
  mood: ["bright", "dark", "warm", "cold", "soft", "high-contrast"],
  ui: ["broadcast", "cartoon", "craft", "cyber", "dread", "fantasy", "foundry", "gilded", "kawaii", "pixel", "scifi", "storybook", "tactical", "wasteland"],
  platform: ["phone-portrait", "phone-landscape", "desktop", "multi"],
  input: ["touch", "keyboard-mouse", "gamepad"],
  players: ["single", "local-multiplayer", "online-lockstep"],
  music: ["chiptune", "orchestral", "electronic", "ambient", "folk", "rock"],
} as const;

export type ChoiceKey = keyof typeof CREATION_CHOICES;
export type CreationChoices = { readonly [K in ChoiceKey]: (typeof CREATION_CHOICES)[K][number] } & {
  readonly look: string;
  readonly lang: readonly string[];
};
export type CreationInput = Partial<CreationChoices>;
