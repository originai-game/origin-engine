import { file } from '@origingame/engine/author';
// Ice Shatter: At the centre of shattered ice or glass.
export default file.vfx2d({
 "recipe": {
  "name": "Ice Shatter",
  "w": 208,
  "h": 192,
  "frames": 14,
  "fps": 16,
  "loop": false,
  "seed": 103,
  "ramps": {
   "main": [
    "#3482c8",
    "#7cc6ee",
    "#cdf2ff",
    "#ffffff"
   ],
   "spark": [
    "#d8cfc4",
    "#f4efe8",
    "#ffffff"
   ]
  },
  "layers": [
   {
    "type": "flash",
    "ramp": "$spark",
    "r": 0.25,
    "dur": 0.08
   },
   {
    "type": "ring",
    "ramp": "$spark",
    "r0": 0.12,
    "r1": 0.55,
    "th": 0.07,
    "breakup": 0.7,
    "t1": 0.4
   },
   {
    "type": "particles",
    "ramp": "$main",
    "mode": "burst",
    "count": 14,
    "dir": -90,
    "spread": 220,
    "speed": [
     1.3,
     2.1
    ],
    "gravity": 3,
    "drag": 1.8,
    "life": [
     0.5,
     0.9
    ],
    "shape": "shard",
    "size": [
     7,
     11
    ]
   }
  ],
  "scale": 0.75,
  "supersample": 4,
  "cleanup": false,
  "outline": "outside",
  "outlineColor": "#3a2416",
  "line": 2,
  "innerLine": 1
 }
});
