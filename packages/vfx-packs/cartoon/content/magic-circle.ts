import { file } from '@origingame/engine/author';
// Spell Circle: Loops on the ground under a caster.
export default file.vfx2d({
 "recipe": {
  "name": "Spell Circle",
  "w": 160,
  "h": 80,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 83,
  "ramps": {
   "main": [
    "#5e34c0",
    "#9466ec",
    "#cfb0ff",
    "#f6eeff"
   ]
  },
  "layers": [
   {
    "type": "glyph",
    "ramp": "$main",
    "r": 0.95,
    "squash": 0.35,
    "rings": [
     1,
     0.82,
     0.55
    ],
    "star": 6,
    "step": 2,
    "ticks": 0,
    "runes": 0,
    "spin": 1
   }
  ],
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
