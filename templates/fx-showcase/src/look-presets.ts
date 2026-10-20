// The presets the looks station cycles (looks.ts), in a module of their
// own with no engine import, so the playtest's script (tests/
// looks-script.ts) and the browser reference tiers (packages/engine/
// tests/web/reference.ts --looks) read the same list.

/** The presets the station cycles, in the catalogue's order. */
export const LOOK_PRESETS = [
  "expressionist-painting", "bold-comic-illustration", "watercolor-wash", "pixel-3d-hybrid", "faceted-low-poly",
  "sculpted-clay", "8bit-pixel", "voxel-worlds", "illuminated-manuscript", "rubber-hose-animation",
  "pencil-hatching", "gothic-contour", "lush-painted-worlds", "decorative-storybook", "classical-painting-collage",
  "detailed-doodle", "black-figure-pottery", "heavy-ink-shadows", "monochrome-pixel", "16bit-pixel",
  "detailed-pixel", "luminous-pixel", "topdown-tile-pixel", "low-resolution-3d", "coarse-pixel",
  "anime-cel-3d", "inked-comic-3d", "paper-craft", "miniature-diorama", "painterly-3d",
  "cinematic-realism", "muted-cinematic-3d", "layered-silhouette", "minimal-geometric", "neon-vector",
  "monochrome-dither-3d", "luminous-layered-painting",
] as const;
