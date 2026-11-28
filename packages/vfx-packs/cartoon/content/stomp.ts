import { file } from '@origingame/engine/author';
// Stomp: On the ground under a stomped enemy.
export default file.vfx2d({
 "recipe": {
  "name": "Stomp",
  "w": 176,
  "h": 128,
  "frames": 12,
  "fps": 18,
  "loop": false,
  "seed": 15,
  "ramps": {
   "main": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ],
   "alt": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ],
   "smoke": [
    "#a37650",
    "#cfa274",
    "#f0d6ae"
   ]
  },
  "layers": [
   {
    "type": "puffs",
    "ramp": "$smoke",
    "count": 6,
    "spread": 0.9,
    "flat": 0.25,
    "rise": 0.1,
    "r0": 0.14,
    "r1": 0.28,
    "life": [
     0.5,
     0.9
    ],
    "emit": 0.08,
    "dir": 0,
    "cone": 180,
    "speed": 1.2,
    "decay": 0.55,
    "y": 0.6
   },
   {
    "type": "star",
    "ramp": "$main",
    "points": 6,
    "r": 0.6,
    "inner": 0.45,
    "pop": 0.25,
    "hollow": 0.9,
    "y": 0.15
   },
   {
    "type": "ring",
    "ramp": "$alt",
    "r0": 0.2,
    "r1": 1.2,
    "th": 0.12,
    "squash": 0.3,
    "breakup": 0.4,
    "y": 0.65,
    "t1": 0.6
   }
  ],
  "pivot": [
   0.5,
   0.7812
  ],
  "scale": 0.75,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
