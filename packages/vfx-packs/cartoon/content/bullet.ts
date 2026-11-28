import { file } from '@origingame/engine/author';
// Pellet: Loops on the moving shot; +x is where it flies.
export default file.vfx2d({
 "recipe": {
  "name": "Pellet",
  "w": 96,
  "h": 48,
  "frames": 8,
  "fps": 16,
  "loop": true,
  "seed": 22,
  "ramps": {
   "main": [
    "#d2461e",
    "#ff8a24",
    "#ffd040",
    "#fff6c8"
   ]
  },
  "layers": [
   {
    "type": "comet",
    "ramp": "$main",
    "r": 0.32,
    "len": 1.3,
    "speed": 2,
    "rough": 0.2,
    "x": 0.6
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
