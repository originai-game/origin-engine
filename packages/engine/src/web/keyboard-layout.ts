// The player's keyboard layout, for glyphs (og.input.setKeyboardLayout): the
// shell tells the engine what each physical key prints, so prompts read Z Q S D
// on AZERTY while the bindings stay on the physical keys (KeyboardEvent.code).
//
// Where the browser has navigator.keyboard.getLayoutMap() (Chrome, Edge) the
// whole map is read at once and again when the window regains focus (the
// player may have switched layouts) or the keyboard says its layout changed.
// Elsewhere (Firefox, Safari) the map is learnt from the keys as they are
// pressed: an unmodified key whose `key` is one character says what that code
// prints; until a key is pressed its glyph is the US layout's.

/** A key and what the layout prints on it (IDL input.KeyLabel). */
export interface KeyLabel {
  readonly code: string;
  readonly label: string;
}

interface LayoutNavigator {
  readonly keyboard?: { getLayoutMap?(): Promise<{ entries(): IterableIterator<[string, string]> }> };
}

/** The layout map's labels, sorted by code; null where the browser has no layout map (or refuses it: a frame without the permission). */
export async function readLayoutMap(nav: LayoutNavigator | undefined = globalThis.navigator as LayoutNavigator | undefined): Promise<KeyLabel[] | null> {
  const get = nav?.keyboard?.getLayoutMap;
  if (!get) return null;
  try {
    const map = await get.call(nav!.keyboard);
    return [...map.entries()].map(([code, label]) => ({ code, label })).sort((a, b) => (a.code < b.code ? -1 : a.code > b.code ? 1 : 0));
  } catch {
    return null;
  }
}

/** A keydown as the learner reads it. */
export interface KeyEventLike {
  readonly code: string;
  readonly key: string;
  readonly shiftKey?: boolean;
  readonly ctrlKey?: boolean;
  readonly altKey?: boolean;
  readonly metaKey?: boolean;
  getModifierState?(key: string): boolean;
}

/** The layout learnt from the keys pressed, for a browser without a layout map. */
export class LayoutLearner {
  readonly #labels = new Map<string, string>();

  /** Learn from one keydown; true when what the layout prints changed (send `labels()` again). */
  learn(e: KeyEventLike): boolean {
    // The unshifted character only: Shift, AltGr and shortcuts type others.
    if (!e.code || e.shiftKey || e.ctrlKey || e.altKey || e.metaKey || e.getModifierState?.("AltGraph")) return false;
    if ([...e.key].length !== 1 || e.key.trim() === "") return false;
    const label = e.key.toLowerCase();
    if (this.#labels.get(e.code) === label) return false;
    this.#labels.set(e.code, label);
    return true;
  }

  labels(): KeyLabel[] {
    return [...this.#labels].map(([code, label]) => ({ code, label })).sort((a, b) => (a.code < b.code ? -1 : a.code > b.code ? 1 : 0));
  }
}
