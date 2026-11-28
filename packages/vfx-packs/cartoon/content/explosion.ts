import { file } from '@origingame/engine/author';
// Boom: On the ground where it goes off.
export default file.vfx2d({
 "recipe": {
  "name": "Boom",
  "w": 232,
  "h": 232,
  "frames": 18,
  "fps": 15,
  "loop": false,
  "seed": 31,
  "ramps": {
   "main": [
    "#b8321c",
    "#ee6a1e",
    "#ffb22e",
    "#ffe9a0"
   ],
   "alt": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ],
   "smoke": [
    "#8e7f78",
    "#bcaea4",
    "#e9e0d6"
   ],
   "spark": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ]
  },
  "layers": [
   {
    "type": "puffs",
    "ramp": "$smoke",
    "count": 9,
    "spread": 0.55,
    "rise": 0.35,
    "r0": 0.18,
    "r1": 0.36,
    "life": [
     0.55,
     0.95
    ],
    "emit": 0.12,
    "gain": 0.8,
    "decay": 0.5,
    "flat": 0.6,
    "t0": 0.18,
    "y": 0.1
   },
   {
    "type": "burst",
    "ramp": "$main",
    "r0": 0.12,
    "r1": 0.66,
    "grow": 0.35,
    "rough": 0.28,
    "cool": 0.5,
    "erode": 0.85,
    "erodeStart": 0.45,
    "t1": 0.8,
    "y": 0.15
   },
   {
    "type": "ring",
    "ramp": "$alt",
    "r0": 0.2,
    "r1": 0.95,
    "th": 0.08,
    "squash": 0.3,
    "breakup": 0.5,
    "y": 0.75,
    "t1": 0.55
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 12,
    "speed": [
     1.6,
     2.6
    ],
    "drag": 2,
    "gravity": 1.2,
    "life": [
     0.4,
     0.7
    ],
    "shape": "star",
    "size": 7,
    "t0": 0.05,
    "y": 0.15
   },
   {
    "type": "flash",
    "ramp": "$spark",
    "r": 0.6,
    "dur": 0.1,
    "y": 0.15
   }
  ],
  "pivot": [
   0.5,
   0.831
  ],
  "scale": 0.827586,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
