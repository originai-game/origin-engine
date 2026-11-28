// Written by templates/tools/ui-parts.ts: the pack's drawn parts as nine-slice images.
import { ref } from '@origingame/engine/author';

export const images = {
  "bar": { "src": ref("@styles/shared/ui/minimal/bar.png"), "slice": 16, "width": 4 },
  "button": { "src": ref("@styles/shared/ui/minimal/button.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "button-danger": { "src": ref("@styles/shared/ui/minimal/button-danger.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "button-danger-down": { "src": ref("@styles/shared/ui/minimal/button-danger-down.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "button-down": { "src": ref("@styles/shared/ui/minimal/button-down.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "button-primary": { "src": ref("@styles/shared/ui/minimal/button-primary.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "button-primary-down": { "src": ref("@styles/shared/ui/minimal/button-primary-down.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "check": { "src": ref("@styles/shared/ui/minimal/check.png"), "slice": 0, "width": 0 },
  "check-on": { "src": ref("@styles/shared/ui/minimal/check-on.png"), "slice": 0, "width": 0 },
  "cursor": { "src": ref("@styles/shared/ui/minimal/cursor.png"), "slice": 0, "width": 0 },
  "entry": { "src": ref("@styles/shared/ui/minimal/entry.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "entry-on": { "src": ref("@styles/shared/ui/minimal/entry-on.png"), "slice": [32, 36, 36, 36], "width": [8, 9, 9, 9] },
  "panel": { "src": ref("@styles/shared/ui/minimal/panel.png"), "slice": 64, "width": 16 },
  "plate": { "src": ref("@styles/shared/ui/minimal/plate.png"), "slice": 40, "width": 10 },
  "range": { "src": ref("@styles/shared/ui/minimal/range.png"), "slice": [0, 12, 0, 12], "width": [0, 3, 0, 3] },
  "slot": { "src": ref("@styles/shared/ui/minimal/slot.png"), "slice": 40, "width": 10 },
  "thumb": { "src": ref("@styles/shared/ui/minimal/thumb.png"), "slice": 0, "width": 0 },
  "track": { "src": ref("@styles/shared/ui/minimal/track.png"), "slice": [0, 12, 0, 12], "width": [0, 3, 0, 3] },
};
