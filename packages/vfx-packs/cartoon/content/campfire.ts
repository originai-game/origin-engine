import { file } from '@origingame/engine/author';
// Campfire: Loops on the ground at the logs.
export default file.vfx2d({
 "recipe": {
  "name": "Campfire",
  "w": 124,
  "h": 172,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 91,
  "ramps": {
   "main": [
    "#b8321c",
    "#ee6a1e",
    "#ffb22e",
    "#ffe9a0"
   ],
   "alt": [
    "#d2461e",
    "#ff8a24",
    "#ffd040",
    "#fff6c8"
   ]
  },
  "layers": [
   {
    "type": "flame",
    "ramp": "$main",
    "w": 0.8,
    "hgt": 1.6,
    "taper": 0.5,
    "freq": 2,
    "rough": 0.35,
    "speed": 1,
    "flick": 0.3,
    "y": 0.9
   },
   {
    "type": "particles",
    "ramp": "$alt",
    "mode": "rise",
    "count": 5,
    "y": 0.2,
    "area": 0.3,
    "speed": [
     0.8,
     1.4
    ],
    "life": [
     0.4,
     0.8
    ],
    "wiggle": 0.3,
    "shape": "circle",
    "size": 3
   }
  ],
  "pivot": [
   0.5,
   0.7651
  ],
  "scale": 0.774194,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
