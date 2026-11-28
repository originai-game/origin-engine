import { file } from '@origingame/engine/author';
// Chimney Smoke: Loops at a chimney's top.
export default file.vfx2d({
 "recipe": {
  "name": "Chimney Smoke",
  "w": 96,
  "h": 176,
  "frames": 24,
  "fps": 12,
  "loop": true,
  "seed": 93,
  "ramps": {
   "main": [
    "#8e7f78",
    "#bcaea4",
    "#e9e0d6"
   ]
  },
  "layers": [
   {
    "type": "puffs",
    "ramp": "$main",
    "count": 6,
    "spread": 0.15,
    "rise": 1.8,
    "r0": 0.18,
    "r1": 0.42,
    "life": [
     0.8,
     1
    ],
    "dir": -90,
    "cone": 20,
    "speed": 0.5,
    "drift": 0.3,
    "decay": 0.4,
    "flat": 1,
    "y": 1.1
   }
  ],
  "pivot": [
   0.5,
   0.8273
  ],
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
