import { file } from '@origingame/engine/author';
export default file.theme({
  "about": "High fantasy: planed wood with a bevel, round corners, a classic serif, standard motion, runes on panels (CRPG inventories, tavern boards).",
  "color": { "source": "#c8902e", "accent": "#e3b65a", "neutral": "#7a5233", "variant": "tonal", "scheme": "dark" },
  "material": "wood",
  "shape": "round",
  "type": "classic-serif",
  "motion": "standard",
  "ornament": { "panel": "runes", "plate": "studs" },
  "effects": "dust",
  "sound": "wood",
  "icons": "sticker",
  "pack": {
    "variants": {"Ability.box": "size-16 compact:size-11", "Ability.box.ultimate": "size-24 compact:size-14"},
    "theme": {
      "duration": {"ready-pulse": 1700, "flash": 240, "damage": 950, "damage-stagger": 55, "trail-hold": 320, "trail": 580, "hit": 460, "arc": 720},
      "scale": {"ready-pulse": 1.05, "damage-pop": 1.48, "combo-pop": 1.24, "press": 0.94},
      "opacity": {"sweep": 0.68, "flash": 0.78},
      "stroke": {"arc": 12}
    }
  }
});
