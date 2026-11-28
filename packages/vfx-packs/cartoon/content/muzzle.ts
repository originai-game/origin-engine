import { file } from '@origingame/engine/author';
// Pop Muzzle: At the barrel's mouth, +x forward.
export default file.vfx2d({
 "recipe": {
  "name": "Pop Muzzle",
  "w": 96,
  "h": 64,
  "frames": 6,
  "fps": 20,
  "loop": false,
  "seed": 21,
  "ramps": {
   "main": [
    "#d2461e",
    "#ff8a24",
    "#ffd040",
    "#fff6c8"
   ],
   "smoke": [
    "#8e7f78",
    "#bcaea4",
    "#e9e0d6"
   ]
  },
  "layers": [
   {
    "type": "star",
    "ramp": "$main",
    "points": 6,
    "r": 0.62,
    "inner": 0.42,
    "pop": 0.3,
    "hollow": 0.8,
    "x": -0.2
   },
   {
    "type": "burst",
    "ramp": "$main",
    "r0": 0.1,
    "r1": 0.34,
    "grow": 0.2,
    "rough": 0.2,
    "cool": 0.6,
    "x": -0.1,
    "t1": 0.7
   },
   {
    "type": "puffs",
    "ramp": "$smoke",
    "count": 3,
    "spread": 0.15,
    "rise": 0.15,
    "r0": 0.08,
    "r1": 0.18,
    "life": [
     0.5,
     0.8
    ],
    "emit": 0.2,
    "dir": 0,
    "speed": 0.6,
    "decay": 0.5,
    "x": 0.1,
    "t0": 0.3
   }
  ],
  "impact": 0,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
