import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "火尻尾"
export default file.particles({
  "maxParticles": 210,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 8
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.25,
    0.9167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    3.0,
    5.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5234,
        0.0802,
        0.3922
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.6338,
        0.2166,
        0.288
      ]
    ],
    [
      0.5156,
      [
        1.0,
        0.7483,
        0.4136,
        0.19
      ]
    ],
    [
      0.7656,
      [
        1.0,
        0.8733,
        0.6816,
        0.0919
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
