import { file } from '@origingame/engine/author';
export default file.vfx2d({
  "recipe": {
    "cleanup": true,
    "dither": 0,
    "fps": 15,
    "frames": 16,
    "h": 32,
    "layers": [
      {
        "ramp": "$main",
        "rise": 0.55,
        "scale": 2,
        "text": "999!",
        "type": "text",
        "y": 0.3
      },
      {
        "count": 6,
        "drag": 2.5,
        "life": [
          0.3,
          0.5
        ],
        "mode": "burst",
        "ramp": "$spark",
        "shape": "star",
        "size": 1,
        "speed": [
          1,
          1.8
        ],
        "type": "particles",
        "y": 0.2
      }
    ],
    "loop": false,
    "name": "Ember Damage Number",
    "outline": "outside",
    "palette": null,
    "pivot": [
      0.5,
      0.5
    ],
    "ramps": {
      "alt": "lava",
      "main": "fire",
      "smoke": "smoke",
      "spark": "fire"
    },
    "seed": 1,
    "w": 48
  }
});
