// What the shell (shell.ts) measures of the page: the canvas's pixel size
// with the DPR cap, the browser's small and large viewports, the device
// pixel ratio as it changes, the reader's preferences og-ui follows, the
// platform's editing keys, and the power preference kept for the next load.

/** The power preference kept for the next load under `key` (localStorage), false where none is or storage is closed. */
export function storedLowPower(key: string, storage: Pick<Storage, "getItem"> | null = safeStorage()): boolean {
  try {
    return (JSON.parse(storage?.getItem(key) ?? "null") as { lowPower?: unknown } | null)?.lowPower === true;
  } catch {
    return false;
  }
}

/** The page's localStorage, null where it is closed. */
export function safeStorage(): Storage | null {
  try {
    return globalThis.localStorage ?? null;
  } catch {
    return null;
  }
}

/**
 * Calls `changed` with the new device pixel ratio whenever it changes (a
 * window moved to a monitor of another scale, the page zoomed): a
 * `(resolution: <dpr>dppx)` query that stops matching, re-armed at the new
 * ratio each time. Answers the function that stops it.
 */
export function watchPixelRatio(
  changed: (dpr: number) => void,
  env: { matchMedia?: ((q: string) => MediaQueryList | undefined) | undefined; dpr: () => number } = {
    matchMedia: globalThis.matchMedia ? (q) => globalThis.matchMedia(q) : undefined,
    dpr: () => globalThis.devicePixelRatio || 1,
  },
): () => void {
  let media: MediaQueryList | undefined;
  let at = env.dpr();
  const fire = () => {
    media?.removeEventListener("change", fire);
    const was = at;
    arm();
    if (at !== was) changed(at);
  };
  const arm = () => {
    at = env.dpr();
    media = env.matchMedia?.(`(resolution: ${at}dppx)`);
    media?.addEventListener?.("change", fire);
  };
  arm();
  return () => media?.removeEventListener?.("change", fire);
}

/** The pixel size of a canvas laid out at `cssWidth`×`cssHeight` on this display, with the DPR cap. */
export function pixelSize(cssWidth: number, cssHeight: number, devicePixelRatio: number, dprCap = 2): { width: number; height: number; dpr: number } {
  const dpr = Math.max(0.5, Math.min(devicePixelRatio || 1, dprCap > 0 ? dprCap : 2));
  return { width: Math.max(1, Math.round(cssWidth * dpr)), height: Math.max(1, Math.round(cssHeight * dpr)), dpr };
}

/**
 * The play surface's size with the browser's own controls shown and hidden
 * (CSS's small and large viewports, og-ui's `Viewport.small`/`large`): the
 * canvas's `width`×`height` moved by how far the page's 100svw/100svh and
 * 100lvw/100lvh are from its 100dvw/100dvh, which `measure` reads. Empty
 * where the browser has no such units (the surface's own size then).
 */
export function viewportExtremes(
  width: number,
  height: number,
  measure: (w: string, h: string) => [number, number] | undefined = measureViewport,
): { smallWidth?: number; smallHeight?: number; largeWidth?: number; largeHeight?: number } {
  const [d, s, l] = [measure("100dvw", "100dvh"), measure("100svw", "100svh"), measure("100lvw", "100lvh")];
  if (!d || !s || !l || !(d[0] > 0 && d[1] > 0 && s[1] > 0 && l[1] > 0)) return {};
  return {
    smallWidth: Math.max(1, width - Math.max(0, d[0] - s[0])),
    smallHeight: Math.max(1, height - Math.max(0, d[1] - s[1])),
    largeWidth: width + Math.max(0, l[0] - d[0]),
    largeHeight: height + Math.max(0, l[1] - d[1]),
  };
}

/** The CSS px a fixed box of `w`×`h` takes on this page. */
function measureViewport(w: string, h: string): [number, number] | undefined {
  const doc = globalThis.document;
  if (!doc?.body || !globalThis.CSS?.supports?.("height", "1dvh")) return undefined;
  const probe = doc.createElement("div");
  probe.style.cssText = `position:fixed;left:0;top:0;visibility:hidden;pointer-events:none;width:${w};height:${h}`;
  doc.body.appendChild(probe);
  const r = probe.getBoundingClientRect();
  probe.remove();
  return [r.width, r.height];
}

/** The page's preferences og-ui follows, each with the viewport field it sets. */
export const PAGE_PREFERENCES = [
  ["reducedMotion", "(prefers-reduced-motion: reduce)"],
  ["highContrast", "(prefers-contrast: more)"],
  ["reducedTransparency", "(prefers-reduced-transparency: reduce)"],
] as const;

/** The page's motion, contrast and transparency preferences and the platform's editing keys, as the viewport carries them to og-ui. */
export function pagePreferences(): { reducedMotion: boolean; highContrast: boolean; reducedTransparency: boolean; macKeys: boolean } {
  const query = (q: string) => globalThis.matchMedia?.(q).matches ?? false;
  const [motion, contrast, transparency] = PAGE_PREFERENCES.map(([, q]) => query(q));
  return { reducedMotion: motion!, highContrast: contrast!, reducedTransparency: transparency!, macKeys: macKeys() };
}

/** Whether the page runs on an Apple platform, whose text fields take Command and Option (og-ui's `Viewport.macKeys`). */
export function macKeys(nav: { platform?: string; userAgentData?: { platform?: string } } | undefined = globalThis.navigator as never): boolean {
  const platform = nav?.userAgentData?.platform || nav?.platform || "";
  return /^(mac|iphone|ipad|ipod)/i.test(platform);
}
