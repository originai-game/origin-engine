import { file } from '@origingame/engine/author';
// Anime action RPG: dusk-violet glass with cut corners, an angled display
// face over a soft rounded text face, and the current choice on a slanted
// ribbon with a coral leading edge (Honkai: Star Rail, Genshin Impact,
// Persona 5's slant in a quieter key).
export default file.theme({
  "about": "Anime RPG: dusk-violet glass with cut corners and a lit top edge, an angled display face over a rounded text face, the current choice on a slanted ribbon with a coral edge, standard motion (Honkai: Star Rail, Genshin Impact).",
  "color": { "source": "#f2c46d", "accent": "#ff6f91", "neutral": "#3b2a5c", "variant": "tonal", "scheme": "dark" },
  "material": "glass",
  "shape": "cut",
  "type": "anime",
  "motion": "standard",
  "ornament": { "panel": "hairline" },
  "effects": "glow",
  "sound": "glass",
  "icons": "glow",
  "density": "comfortable",
  "pack": {
    "variants": {
      "Ability.box": "size-16 compact:size-11", "Ability.box.ultimate": "size-26 compact:size-15",
      "Screen.backdrop": "bg-linear-to-br from-stage-hi via-stage-mid to-stage-lo",
      "Screen.glow": "bg-radial from-stage-glow/40 to-transparent to-60%",
      "Screen.texture": "bg-[repeating-conic-gradient(from_0deg_at_100%_45%,rgb(255_255_255/0.05)_0%,rgb(255_255_255/0.05)_0.4%,transparent_0.4%,transparent_1.6%)]",
      "Screen.heading": "font-display italic font-bold text-display-lg leading-tight tracking-display-lg text-fg text-shadow-sm text-shadow-bg",
      "Title.wordmark": "font-display italic font-bold text-display-3xl leading-none tracking-display-3xl text-fg text-shadow-sm text-shadow-stage-glow",
      "MenuList.frame": "flex-col gap-1",
      "MenuList.indicator": "absolute inset-0 -skew-x-12 bg-linear-to-r from-accent/45 via-accent/15 to-transparent border-s-4 border-accent shadow-glow shadow-accent/40 pointer-events-none",
      "MenuList.indicator.danger": "from-danger/45 via-danger/15 border-danger shadow-danger/40",
      "MenuList.label": "text-start font-display italic font-bold text-3xl leading-tight tracking-wide text-shadow-xs text-shadow-bg portrait:text-4xl",
      "MenuList.label.md": "text-2xl portrait:text-3xl",
      "MenuList.line": "text-start text-base font-semibold text-fg-muted group-data-current/entry:text-fg whitespace-nowrap overflow-hidden"
    },
    "theme": {
      "fontSize": { "2xs": [13, 1.25], "xs": [14, 1.3], "meta": [15, 1.3], "caption": [15, 1.3], "sm": [16, 1.35], "body": [17, 1.4], "reading": [18, 1.45], "base": [18, 1.35], "lg": [21, 1.3], "xl": [24, 1.25], "2xl": [28, 1.2], "3xl": [32, 1.2], "4xl": [38, 1.15] },
      "colors": { "stage-hi": "#4a2a6e", "stage-mid": "#22163f", "stage-lo": "#0d0a1c", "stage-glow": "#ff8fab" },
      "duration": {"ready-pulse": 1600, "flash": 260, "damage": 1000, "damage-stagger": 60, "trail-hold": 300, "trail": 600, "hit": 450, "arc": 700},
      "scale": {"ready-pulse": 1.04, "damage-pop": 1.5, "combo-pop": 1.25, "press": 0.94},
      "opacity": {"sweep": 0.6, "flash": 0.8},
      "stroke": {"arc": 10}
    }
  }
});
