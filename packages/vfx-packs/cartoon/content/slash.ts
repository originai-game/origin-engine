import { file } from '@origingame/engine/author';
// Swoosh Slash: At the swing's centre, +x forward; turn it with the sprite.
export default file.vfx2d({
 "recipe": {
  "name": "Swoosh Slash",
  "w": 128,
  "h": 128,
  "frames": 12,
  "fps": 18,
  "loop": false,
  "seed": 14,
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
    "type": "slash",
    "ramp": "$main",
    "r": 0.72,
    "th": 0.24,
    "a0": -150,
    "a1": 40,
    "sweep": 0.3,
    "trail": 0.7,
    "erode": 0.6
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "burst",
    "count": 4,
    "x": 0.55,
    "y": -0.25,
    "speed": [
     0.8,
     1.4
    ],
    "drag": 3,
    "life": [
     0.3,
     0.5
    ],
    "shape": "star",
    "size": 5,
    "t0": 0.18
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
