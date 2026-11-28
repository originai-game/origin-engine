import { file } from '@origingame/engine/author';
export default file.theme({
  "about": "Anime RPG: dark translucent glass with a lit top edge, soft corners, a clean sans, standard motion (Honkai: Star Rail, Genshin Impact).",
  "color": { "source": "#e8c37a", "accent": "#7fd4ff", "neutral": "#3a4a6b", "variant": "tonal", "scheme": "dark" },
  "material": "glass",
  "shape": "soft",
  "type": "sans",
  "motion": "standard",
  "ornament": { "panel": "hairline" },
  "effects": "glow",
  "sound": "glass",
  "icons": "glow",
  "density": "comfortable",
  "pack": {
    "variants": {
      "Ability.box": "size-16 compact:size-11", "Ability.box.ultimate": "size-26 compact:size-15",
      "Screen.backdrop": "bg-linear-to-b from-stage-hi to-stage-lo",
      "Screen.glow": "bg-radial from-stage-glow to-transparent to-60% opacity-dim",
      "MenuList.frame": "flex-col gap-1",
      "MenuList.label": "text-start text-3xl leading-tight font-semibold tracking-wide text-shadow-xs text-shadow-bg",
      "MenuList.label.md": "text-2xl"
    },
    "theme": {
      "fontSize": { "2xs": [13, 1.25], "xs": [14, 1.3], "meta": [15, 1.3], "caption": [15, 1.3], "sm": [16, 1.35], "body": [17, 1.4], "reading": [18, 1.45], "base": [18, 1.35], "lg": [21, 1.3], "xl": [24, 1.25], "2xl": [28, 1.2], "3xl": [32, 1.2], "4xl": [38, 1.15] },
      "colors": { "stage-hi": "#24345a", "stage-lo": "#0b1020", "stage-glow": "#7fd4ff" },
      "duration": {"ready-pulse": 1600, "flash": 260, "damage": 1000, "damage-stagger": 60, "trail-hold": 300, "trail": 600, "hit": 450, "arc": 700},
      "scale": {"ready-pulse": 1.04, "damage-pop": 1.5, "combo-pop": 1.25, "press": 0.94},
      "opacity": {"sweep": 0.6, "flash": 0.8},
      "stroke": {"arc": 10}
    }
  }
});
