import { file } from '@origingame/engine/author';
// Splash: On the water line where something falls in.
export default file.vfx2d({
 "recipe": {
  "name": "Splash",
  "w": 184,
  "h": 184,
  "frames": 14,
  "fps": 16,
  "loop": false,
  "seed": 51,
  "ramps": {
   "main": [
    "#2a6cc0",
    "#3f9fe6",
    "#8fd8ff",
    "#eefaff"
   ],
   "alt": [
    "#7cc4ee",
    "#c6ecff",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "pillar",
    "ramp": "$main",
    "w": 0.32,
    "hgt": 0.9,
    "speed": 1,
    "rough": 0.35,
    "grow": 0.3,
    "y": 0.7,
    "t1": 0.6
   },
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "spray",
    "count": 14,
    "y": 0.6,
    "dir": -90,
    "spread": 110,
    "speed": [
     1.4,
     2.4
    ],
    "gravity": 3.2,
    "drag": 0.6,
    "life": [
     0.5,
     0.9
    ],
    "shape": "drop",
    "size": 6
   },
   {
    "type": "ring",
    "ramp": "$alt",
    "r0": 0.2,
    "r1": 1.08,
    "th": 0.1,
    "squash": 0.25,
    "breakup": 0.3,
    "y": 0.75
   }
  ],
  "pivot": [
   0.5,
   0.813
  ],
  "scale": 0.782609,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
