import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.8 "Flash"
export default file.particles({
  "maxParticles": 37,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      12.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 90,
      "interval": 0.0167
    }
  ],
  "duration": 1.5,
  "looping": false,
  "lifetime": [
    0.4667,
    0.4667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    16.0,
    16.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
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
        1.0,
        0.9075,
        0.7519,
        0.0
      ]
    ],
    [
      0.1071,
      [
        1.0,
        0.9075,
        0.7519,
        0.1255
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.9075,
        0.7519,
        0.1255
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9075,
        0.7519,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Particle_Soft"
  },
  "blend": "additive"
});
