import { file } from '@origingame/engine/author';
// Land Dust: At the feet on landing.
export default file.vfx2d({
 "recipe": {
  "name": "Land Dust",
  "w": 192,
  "h": 112,
  "frames": 12,
  "fps": 18,
  "loop": false,
  "seed": 42,
  "ramps": {
   "main": [
    "#a37650",
    "#cfa274",
    "#f0d6ae"
   ]
  },
  "layers": [
   {
    "type": "puffs",
    "ramp": "$main",
    "count": 8,
    "spread": 0.9,
    "flat": 0.2,
    "rise": 0.12,
    "r0": 0.12,
    "r1": 0.28,
    "life": [
     0.5,
     0.9
    ],
    "emit": 0.05,
    "dir": -90,
    "cone": 180,
    "speed": 1.4,
    "decay": 0.5,
    "y": 0.55
   },
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "burst",
    "count": 6,
    "y": 0.55,
    "dir": -90,
    "spread": 140,
    "speed": [
     1,
     1.6
    ],
    "gravity": 2.4,
    "drag": 1.2,
    "ground": 0.7,
    "life": [
     0.35,
     0.6
    ],
    "shape": "circle",
    "size": 4
   }
  ],
  "pivot": [
   0.5,
   0.7143
  ],
  "scale": 0.571429,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
