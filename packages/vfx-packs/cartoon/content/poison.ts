import { file } from '@origingame/engine/author';
// Poisoned: Loops round a poisoned character.
export default file.vfx2d({
 "recipe": {
  "name": "Poisoned",
  "w": 120,
  "h": 168,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 75,
  "ramps": {
   "main": [
    "#4e7e1a",
    "#8cc236",
    "#d0ee76"
   ]
  },
  "layers": [
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "rise",
    "count": 8,
    "y": 0.6,
    "area": 0.6,
    "speed": [
     0.6,
     1.2
    ],
    "life": [
     0.6,
     0.95
    ],
    "wiggle": 0.3,
    "shape": "bubble",
    "size": [
     3,
     6
    ]
   },
   {
    "type": "puffs",
    "ramp": "$main",
    "count": 3,
    "spread": 0.4,
    "rise": 0.4,
    "r0": 0.1,
    "r1": 0.2,
    "life": [
     0.6,
     0.95
    ],
    "decay": 0.6,
    "gain": 0.6,
    "y": 0.4
   }
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
