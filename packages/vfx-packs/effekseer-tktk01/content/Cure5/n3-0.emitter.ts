import { file } from '@origingame/engine/author';
// Effekseer node 3.0 "星型"
export default file.particles({
  "maxParticles": 32,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0833
    }
  ],
  "duration": 0.7667,
  "looping": false,
  "lifetime": [
    0.0167,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    26.6667,
    26.6667
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    45.0,
    45.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/star"
  },
  "blend": "additive"
});
