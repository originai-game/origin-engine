import { file } from '@origingame/engine/author';
// Pow Hit: Where the hit lands; a struck enemy or wall.
export default file.vfx2d({
 "recipe": {
  "name": "Pow Hit",
  "w": 144,
  "h": 144,
  "frames": 10,
  "fps": 18,
  "loop": false,
  "seed": 11,
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
   "spark": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ]
  },
  "layers": [
   {
    "type": "star",
    "ramp": "$main",
    "points": 8,
    "r": 0.78,
    "inner": 0.42,
    "sharp": 0.6,
    "jitter": 0.25,
    "pop": 0.22,
    "hollow": 0.9
   },
   {
    "type": "ring",
    "ramp": "$alt",
    "r0": 0.25,
    "r1": 0.95,
    "th": 0.12,
    "breakup": 0.5,
    "t0": 0.05,
    "t1": 0.7,
    "gain": 0.8
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 7,
    "speed": [
     1.6,
     2.4
    ],
    "drag": 2.6,
    "life": [
     0.35,
     0.6
    ],
    "shape": "star",
    "size": 6,
    "t0": 0.05
   },
   {
    "type": "flash",
    "ramp": "$spark",
    "r": 0.45,
    "dur": 0.12
   }
  ],
  "scale": 0.777778,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
