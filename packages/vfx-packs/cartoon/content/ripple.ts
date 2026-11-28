import { file } from '@origingame/engine/author';
// Ripples: Loops on the water line round a wading body or a float.
export default file.vfx2d({
 "recipe": {
  "name": "Ripples",
  "w": 128,
  "h": 48,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 53,
  "ramps": {
   "main": [
    "#7cc4ee",
    "#c6ecff",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "ring",
    "ramp": "$main",
    "r0": 0.2,
    "r1": 1.6,
    "th": 0.12,
    "squash": 0.22,
    "breakup": 0.15,
    "fade": 1
   },
   {
    "type": "ring",
    "ramp": "$main",
    "r0": 0.2,
    "r1": 1.6,
    "th": 0.12,
    "squash": 0.22,
    "breakup": 0.15,
    "fade": 1,
    "t0": 0.5,
    "t1": 1.5
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
