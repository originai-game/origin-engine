import { file } from '@origingame/engine/author';
export default file.theme({
  "about": "Retro pixel: a whole-point staircase, a 1-point bevel, a pixel face, stepped motion (Celeste, Stardew Valley).",
  "color": { "source": "#f0a43a", "accent": "#7bd36a", "neutral": "#3d3366", "variant": "fidelity", "scheme": "dark" },
  "material": "pixel",
  "shape": "pixel",
  "type": "pixel",
  "motion": "stepped",
  "ornament": { "panel": "hairline" },
  "effects": "pixel-burst",
  "sound": "chiptune",
  "icons": "pixel",
  "pack": {
    "variants": {"Ability.box": "size-15 compact:size-10", "Ability.box.ultimate": "size-22 compact:size-13"},
    "theme": {
      "duration": {"ready-pulse": 1100, "flash": 150, "damage": 750, "damage-stagger": 50, "trail-hold": 320, "trail": 0, "hit": 320, "arc": 550},
      "scale": {"ready-pulse": 1.0, "damage-pop": 1.35, "combo-pop": 1.2, "press": 0.96},
      "opacity": {"sweep": 0.72, "flash": 0.85},
      "stroke": {"arc": 12}
    }
  }
});
