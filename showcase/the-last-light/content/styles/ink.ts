import { file } from '@origingame/engine/author';
export default file.theme({
  "about": "Ink on rice paper: a faint wash, a brushed edge, Song type, calm motion, cinnabar accents (wuxia and xianxia games).",
  "color": { "source": "#b8322a", "accent": "#b8322a", "neutral": "#5a5248", "variant": "muted", "scheme": "light" },
  "material": "ink",
  "shape": "brush",
  "type": "song",
  "motion": "calm",
  "ornament": { "panel": "brush-corner" },
  "effects": "ink-splash",
  "sound": "soft",
  "icons": "ink",
  "density": "comfortable",
  "pack": {
    "variants": {"Ability.box": "size-16 compact:size-11", "Ability.box.ultimate": "size-24 compact:size-14"},
    "theme": {
      "duration": {"ready-pulse": 2000, "flash": 280, "damage": 1050, "damage-stagger": 65, "trail-hold": 360, "trail": 640, "hit": 520, "arc": 800},
      "scale": {"ready-pulse": 1.03, "damage-pop": 1.35, "combo-pop": 1.18, "press": 0.97},
      "opacity": {"sweep": 0.55, "flash": 0.65},
      "stroke": {"arc": 9}
    }
  }
});
