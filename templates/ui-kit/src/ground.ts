// The ground every page stands on: the game's scene (its painted scene,
// its HUD's gameplay frame, the style's backdrop picture, the documents'
// key art), or the pack's own ground (`demo:ground:pack`), its backdrop
// with no picture, on which the baseline sheets judge each component and
// pattern as a piece of the pack on its own (the owner, 2026-11-04: how
// the UI sits in a particular game's scene is that game's work).

let pack = false;

/** The pages stand on the pack's own ground, not a scene. */
export function packGround(): boolean {
  return pack;
}

/** `demo:ground:pack` or `demo:ground:scene` (the default). */
export function setGround(value: string): void {
  pack = value === "pack";
}

/** A texture for a scene's art, or 0 on the pack's own ground. */
export function sceneArt<T>(texture: T | null | undefined): T | 0 {
  return pack || texture === null || texture === undefined ? 0 : texture;
}
