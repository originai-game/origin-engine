import { file } from '@origingame/engine/author';
// Heart Grab: Where a life or health pickup is collected.
export default file.vfx2d({
 "recipe": {
  "name": "Heart Grab",
  "w": 96,
  "h": 112,
  "frames": 12,
  "fps": 18,
  "loop": false,
  "seed": 63,
  "ramps": {
   "main": [
    "#cc3466",
    "#f2688e",
    "#ffbcd2"
   ],
   "spark": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "ring",
    "ramp": "$main",
    "r0": 0.2,
    "r1": 0.8,
    "th": 0.1,
    "breakup": 0.3,
    "t1": 0.6
   },
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "rise",
    "count": 4,
    "area": 0.5,
    "speed": [
     0.8,
     1.4
    ],
    "life": [
     0.45,
     0.8
    ],
    "shape": "heart",
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
    "shape": "heart",
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
