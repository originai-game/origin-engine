import { file } from '@origingame/engine/author';
// Dash Lines: At the dashing body; +x is the way it dashes.
export default file.vfx2d({
 "recipe": {
  "name": "Dash Lines",
  "w": 192,
  "h": 128,
  "frames": 10,
  "fps": 18,
  "loop": false,
  "seed": 44,
  "ramps": {
   "main": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ],
   "smoke": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "spray",
    "count": 12,
    "x": 0.35,
    "area": 0.4,
    "areaX": 0.3,
    "dir": 180,
    "spread": 12,
    "speed": [
     1.6,
     2.4
    ],
    "drag": 3,
    "life": [
     0.35,
     0.6
    ],
    "emit": 0.35,
    "shape": "streak",
    "size": 11
   },
   {
    "type": "puffs",
    "ramp": "$smoke",
    "count": 4,
    "spread": 0.25,
    "rise": 0.05,
    "r0": 0.12,
    "r1": 0.22,
    "life": [
     0.4,
     0.8
    ],
    "dir": 180,
    "speed": 0.6,
    "decay": 0.6,
    "x": -0.35,
    "t0": 0.1
   }
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
