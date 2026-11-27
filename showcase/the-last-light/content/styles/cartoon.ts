import { file } from '@origingame/engine/author';
export default file.theme({
  "about": "Cartoon: a light panel under a coloured header, a thick clean outline, a gloss band and a 3D lip, round corners, chunky display type, bouncy motion (Brawl Stars, Clash Royale).",
  "color": { "source": "#ff8a3d", "accent": "#2ec4a6", "neutral": "#8a7560", "variant": "vibrant", "scheme": "light" },
  "material": "flat",
  "shape": "round",
  "type": "rounded-display",
  "motion": "bouncy",
  "ornament": "none",
  "effects": "sheen",
  "sound": "cute",
  "icons": "sticker",
  "density": "touch",
  "pack": {
    "variants": {"Ability.box": "size-18 compact:size-12", "Ability.box.ultimate": "size-26 compact:size-15"},
    "theme": {
      "duration": {"ready-pulse": 950, "flash": 280, "damage": 950, "damage-stagger": 70, "trail-hold": 350, "trail": 650, "hit": 480, "arc": 750},
      "scale": {"ready-pulse": 1.09, "damage-pop": 1.65, "combo-pop": 1.38, "press": 0.88},
      "opacity": {"sweep": 0.58, "flash": 0.72},
      "stroke": {"arc": 15}
    }
  }
});
