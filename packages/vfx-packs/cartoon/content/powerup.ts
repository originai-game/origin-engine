import { file } from '@origingame/engine/author';
// Power Up: Round a character taking a power-up.
export default file.vfx2d({
 "recipe": {
  "name": "Power Up",
  "w": 156,
  "h": 188,
  "frames": 16,
  "fps": 15,
  "loop": false,
  "seed": 64,
  "ramps": {
   "main": [
    "#e09a14",
    "#ffd23c",
    "#fff3a8"
   ],
   "spark": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "rays",
    "ramp": "$main",
    "count": 12,
    "r": 1,
    "width": 0.18,
    "gain": 0.7,
    "spin": 0.2,
    "t1": 0.8
   },
   {
    "type": "ring",
    "ramp": "$main",
    "r0": 0.3,
    "r1": 1,
    "th": 0.14,
    "breakup": 0.3,
    "t1": 0.7
   },
   {
    "type": "particles",
    "ramp": "$spark",
    "mode": "rise",
    "count": 10,
    "area": 0.7,
    "speed": [
     0.8,
     1.5
    ],
    "life": [
     0.45,
     0.8
    ],
    "emit": 0.5,
    "shape": "star",
    "size": 5,
    "twinkle": 2
   }
  ],
  "scale": 0.820513,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
