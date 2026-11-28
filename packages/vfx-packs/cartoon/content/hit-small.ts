import { file } from '@origingame/engine/author';
// Tap Hit: Where a light hit lands.
export default file.vfx2d({
 "recipe": {
  "name": "Tap Hit",
  "w": 72,
  "h": 72,
  "frames": 8,
  "fps": 18,
  "loop": false,
  "seed": 12,
  "ramps": {
   "main": [
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
    "points": 5,
    "r": 0.7,
    "inner": 0.45,
    "pop": 0.25,
    "hollow": 0.9
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 5,
    "speed": [
     1.4,
     2
    ],
    "drag": 3,
    "life": [
     0.3,
     0.5
    ],
    "shape": "circle",
    "size": 4
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
