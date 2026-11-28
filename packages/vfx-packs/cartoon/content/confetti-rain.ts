import { file } from '@origingame/engine/author';
// Confetti Rain: Loops over a win screen's play area.
export default file.vfx2d({
 "recipe": {
  "name": "Confetti Rain",
  "w": 192,
  "h": 192,
  "frames": 24,
  "fps": 12,
  "loop": true,
  "seed": 68,
  "ramps": {
   "main": [
    "#b42a2a",
    "#e65a3e",
    "#ff9c6a"
   ],
   "alt": [
    "#2856b8",
    "#4a8ee8",
    "#9cd0ff"
   ],
   "smoke": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ]
  },
  "layers": [
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "fall",
    "y": 0.3,
    "fadeIn": 0.15,
    "fallScale": 0.62,
    "driftX": 0.06,
    "count": 8,
    "spanX": 0.68,
    "speed": [
     0.6,
     1
    ],
    "life": [
     0.8,
     0.95
    ],
    "wiggle": 0.2,
    "shape": "leaf",
    "size": 9
   },
   {
    "type": "particles",
    "ramp": "$alt",
    "mode": "fall",
    "y": 0.3,
    "fadeIn": 0.15,
    "fallScale": 0.62,
    "driftX": 0.06,
    "count": 8,
    "spanX": 0.68,
    "speed": [
     0.6,
     1
    ],
    "life": [
     0.8,
     0.95
    ],
    "wiggle": 0.2,
    "shape": "leaf",
    "size": 9,
    "seed": 3
   },
   {
    "type": "particles",
    "ramp": "$smoke",
    "mode": "fall",
    "y": 0.3,
    "fadeIn": 0.15,
    "fallScale": 0.62,
    "driftX": 0.06,
    "count": 8,
    "spanX": 0.68,
    "speed": [
     0.6,
     1
    ],
    "life": [
     0.8,
     0.95
    ],
    "wiggle": 0.2,
    "shape": "leaf",
    "size": 9,
    "seed": 5
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
