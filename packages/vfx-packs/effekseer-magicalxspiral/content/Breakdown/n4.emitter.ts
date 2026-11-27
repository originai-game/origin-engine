import { file } from '@origingame/engine/author';
// Effekseer node 4 "LightLine3"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.25,
    0.25
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.0,
    12.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6667
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.0417
    ],
    [
      1.0,
      0.25
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.7519,
        0.5356,
        1.0,
        0.6275
      ]
    ],
    [
      0.3333,
      [
        0.7519,
        0.5356,
        1.0,
        0.6275
      ]
    ],
    [
      1.0,
      [
        0.7519,
        0.5356,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -80.0,
    -50.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/ShockWave3"
  },
  "blend": "additive"
});
