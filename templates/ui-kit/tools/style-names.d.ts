/** Build-time catalogue of the kit's look documents. */
export function styleNames(): readonly string[];
/** Installed style copies override bundled packs of the same name. */
export function stylePaths(): Record<string, string>;
