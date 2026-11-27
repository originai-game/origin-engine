import { file } from '@origingame/engine/author';
// Effekseer node 5 "maru2"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.02,
      0.02,
      0.02
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.3,
    1.3
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      1.0,
      1.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.2195,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
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
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/Particle02"
  },
  "blend": "additive"
});
