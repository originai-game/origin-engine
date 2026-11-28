import { file } from '@origingame/engine/author';
// Shield Bubble: Loops round a shielded character's middle.
export default file.vfx2d({
 "recipe": {
  "name": "Shield Bubble",
  "w": 144,
  "h": 144,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 86,
  "ramps": {
   "main": [
    "#2856b8",
    "#4a8ee8",
    "#9cd0ff"
   ]
  },
  "layers": [
   {
    "type": "bubble",
    "ramp": "$main",
    "r": 0.85,
    "th": 0.09,
    "hex": 0,
    "fill": 0,
    "shimmer": 1
   }
  ],
  "scale": 0.888889,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
