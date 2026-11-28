import { file } from '@origingame/engine/author';
// Frozen: On the ground under a frozen character.
export default file.vfx2d({
 "recipe": {
  "name": "Frozen",
  "w": 160,
  "h": 176,
  "frames": 14,
  "fps": 15,
  "loop": false,
  "seed": 77,
  "ramps": {
   "main": [
    "#3482c8",
    "#7cc6ee",
    "#cdf2ff",
    "#ffffff"
   ],
   "spark": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "spikes",
    "ramp": "$main",
    "count": 6,
    "spread": 0.8,
    "hgt": 1.2,
    "width": 0.22,
    "lean": 0.35,
    "fan": 0.6,
    "grow": 0.2,
    "hold": 0.6,
    "y": 0.95
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 6,
    "y": 0.4,
    "speed": [
     0.8,
     1.4
    ],
    "drag": 3,
    "life": [
     0.3,
     0.6
    ],
    "shape": "diamond",
    "size": 3,
    "t0": 0.1
   }
  ],
  "pivot": [
   0.5,
   0.8636
  ],
  "scale": 0.8,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
