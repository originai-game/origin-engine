// The UI's haptic patterns on the page (docs/ui.md §3.1): each frame
// answer's `haptics` (the patterns the style's feedback played, already at
// the player's strength, og.ui.setHaptics) rumbles every connected pad
// that has a vibration actuator and vibrates a phone. A pattern is a list
// of [startMs, ms, low, high] pulses: `low` the heavy motor, `high` the
// light one; a phone's one motor takes the larger, and the Vibration API
// has no strength, so a pulse below a fifth is left out and a weak one is
// shortened.

/** A pulse: [startMs, ms, low, high]. */
export type HapticPulse = [number, number, number, number];

/** The pattern navigator.vibrate takes (on, off, on…) for `pulses`; empty when nothing is strong enough to feel. */
export function vibrationPattern(pulses: readonly HapticPulse[]): number[] {
  const out: number[] = [];
  let at = 0;
  const sorted = [...pulses].sort((a, b) => a[0] - b[0]);
  for (const [start, ms, low, high] of sorted) {
    const k = Math.max(low, high);
    if (k < 0.2 || ms <= 0) continue;
    const on = Math.max(8, Math.round(ms * Math.min(1, k * 1.5)));
    const gap = Math.max(0, Math.round(start - at));
    if (out.length === 0) {
      // The pattern starts on; a delay before the first pulse is an off of length 0 then the gap.
      if (gap > 0) out.push(0, gap);
    } else if (out.length % 2 === 1) {
      out.push(gap);
    }
    out.push(on);
    at = start + on;
  }
  return out;
}

interface Actuator {
  playEffect?: (kind: string, params: Record<string, number>) => Promise<unknown>;
}
interface RumblePad {
  connected?: boolean;
  vibrationActuator?: Actuator | null;
}

/** Plays `patterns` on the page's pads and the phone's motor. */
export function playHaptics(patterns: readonly (readonly HapticPulse[])[]): void {
  const nav = globalThis.navigator as
    | { getGamepads?: () => ReadonlyArray<RumblePad | null>; vibrate?: (p: number[]) => boolean }
    | undefined;
  if (!nav) return;
  for (const pulses of patterns) {
    const pads = nav.getGamepads?.() ?? [];
    for (const pad of pads) {
      const act = pad?.connected ? pad.vibrationActuator : null;
      if (!act?.playEffect) continue;
      for (const [start, ms, low, high] of pulses) {
        void act
          .playEffect("dual-rumble", {
            startDelay: Math.max(0, start),
            duration: Math.max(0, ms),
            strongMagnitude: Math.min(1, Math.max(0, low)),
            weakMagnitude: Math.min(1, Math.max(0, high)),
          })
          .catch(() => {});
      }
    }
    const pattern = vibrationPattern(pulses);
    if (pattern.length > 0) {
      try {
        nav.vibrate?.(pattern);
      } catch {
        // A page without user activation refuses; nothing to do.
      }
    }
  }
}
