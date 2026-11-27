import { file } from '@origingame/engine/author';
// Effekseer node 0.3 "Spark Sparkles"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "sphere",
    "radius": 1.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 30
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    9.0,
    18.0
  ],
  "size": [
    0.035,
    0.25
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.1667,
      [
        1.0,
        1.0,
        1.0,
        0.249
      ]
    ],
    [
      0.6667,
      [
        1.0,
        1.0,
        1.0,
        0.249
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Particle01"
  },
  "blend": "additive"
});
