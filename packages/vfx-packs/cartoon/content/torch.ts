import { file } from '@origingame/engine/author';
// Torch Flame: Loops at a torch's head.
export default file.vfx2d({
 "recipe": {
  "name": "Torch Flame",
  "w": 64,
  "h": 112,
  "frames": 16,
  "fps": 12,
  "loop": true,
  "seed": 92,
  "ramps": {
   "main": [
    "#b8321c",
    "#ee6a1e",
    "#ffb22e",
    "#ffe9a0"
   ]
  },
  "layers": [
   {
    "type": "flame",
    "ramp": "$main",
    "w": 0.6,
    "hgt": 1.5,
    "taper": 0.5,
    "freq": 2,
    "rough": 0.3,
    "speed": 1,
    "flick": 0.3,
    "y": 0.75
   }
  ],
  "pivot": [
   0.5,
   0.7286
  ],
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
