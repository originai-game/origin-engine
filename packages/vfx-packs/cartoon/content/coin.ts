import { file } from '@origingame/engine/author';
// Coin Grab: Where a coin is collected.
export default file.vfx2d({
 "recipe": {
  "name": "Coin Grab",
  "w": 96,
  "h": 112,
  "frames": 12,
  "fps": 18,
  "loop": false,
  "seed": 61,
  "ramps": {
   "main": [
    "#b8700f",
    "#ecaa1e",
    "#ffdc58",
    "#fff6c8"
   ],
   "smoke": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ],
   "spark": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ]
  },
  "layers": [
   {
    "type": "rays",
    "ramp": "$smoke",
    "count": 8,
    "r": 0.8,
    "width": 0.16,
    "gain": 0.55,
    "t1": 0.6
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 6,
    "speed": [
     1.2,
     1.8
    ],
    "drag": 3,
    "life": [
     0.35,
     0.6
    ],
    "shape": "star",
    "size": 5,
    "t0": 0.1
   },
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "burst",
    "count": 1,
    "speed": [
     0,
     0
    ],
    "life": [
     0.55,
     0.6
    ],
    "shape": "coin",
    "size": 11,
    "gravity": -2.5
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
