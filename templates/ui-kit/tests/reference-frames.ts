/** The package's reference budget: main desktop/phone surfaces and distinctive visual states. */
export const REFERENCE_FRAMES: Readonly<Record<string, readonly string[]>> = {
  card: [
    "kit-patterns-cards-idle.png", "kit-patterns-phone-cards-idle.png",
    "kit-patterns-cards-aim-valid.png", "kit-genres3-deck-idle.png", "kit-patterns-phone-rewards-idle.png",
  ],
  gacha: ["kit-patterns-gacha-idle.png", "kit-patterns-phone-gacha-idle.png", "kit-patterns-gacha-reveal-legendary.png"],
  idle: ["kit-patterns-generators-idle.png", "kit-patterns-phone-generators-idle.png", "kit-patterns-generators-ascend-holding.png"],
  liveops: ["kit-patterns-store-idle.png", "kit-patterns-phone-store-idle.png", "kit-patterns-pass-claim.png"],
  photo: ["kit-genres3-photo-idle.png", "kit-genres3-phone-photo-idle.png", "kit-genres3-photo-frame.png"],
  puzzle: ["kit-patterns-board-idle.png", "kit-patterns-phone-board-idle.png", "kit-patterns-board-star.png"],
  racing: ["kit-racing-idle.png", "kit-racing-phone-idle.png", "kit-racing-checkpoint.png", "kit-racing-phone-text-150.png"],
  rhythm: ["kit-desktop-rhythmHud.png", "kit-phone-rhythmHud.png", "kit-genres-songs-idle.png"],
  rpg: [
    "kit-patterns-gear-idle.png", "kit-patterns-phone-gear-idle.png", "kit-patterns-gear-drag-invalid.png",
    "kit-desktop-rpgHud.png", "kit-story-dialogue-choices.png",
  ],
  shooter: ["kit-desktop-shooterHud.png", "kit-phone-shooterHud.png", "kit-genres3-loadout-optic.png"],
  sports: ["kit-desktop-sportsHud.png", "kit-phone-sportsHud.png", "kit-desktop-matchFacts.png"],
  story: ["kit-story-vn-line.png", "kit-story-phone-vn-line.png", "kit-genres3-cg-viewer.png", "kit-genres3-routes-tower.png"],
  strategy: [
    "kit-genres2-rts-idle.png", "kit-genres2-phone-rts-idle.png", "kit-genres2-rts-target.png",
    "kit-genres2-ledger-underfunded.png", "kit-starship-scifi-sectorMap.png",
  ],
  versus: [
    "kit-genres-lobby-idle.png", "kit-genres-phone-lobby-idle.png", "kit-genres-draft-ban.png",
    "kit-desktop-players.png", "kit-genres-select-locked.png",
  ],
  materials: ["kit-desktop-materials.png", "kit-phone-materials.png"],
  flight: [],
  "ui-kit": [
    "kit-desktop-widgets.png", "kit-phone-widgets.png", "kit-desktop-text-compose.png",
    "kit-components-overlays-submenu.png", "kit-controls-bindings-conflict.png",
  ],
};

const names = new Set(Object.values(REFERENCE_FRAMES).flat());

/** Remove only unselected captures, preserving every command's timestamp and order. */
export function selectReferenceFrames(script: { commands: string }, prefix: string): { commands: string; files: string[] } {
  const files: string[] = [];
  const commands = script.commands.split(";").filter(step => {
    const shot = /^[^:]+:screenshot:(.+)$/.exec(step);
    if (!shot) return true;
    const file = shot[1]!;
    if (!names.has(prefix + file)) return false;
    files.push(file);
    return true;
  }).join(";");
  return { commands, files };
}
