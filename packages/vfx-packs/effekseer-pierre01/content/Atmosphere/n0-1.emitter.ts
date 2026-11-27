import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "Particle"
export default file.particles({
  "maxParticles": 77,
  "shape": {
    "kind": "box",
    "halfExtents": [
      10.0,
      10.0,
      10.0
    ]
  },
  "spread": 62.0617,
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
    0.75,
    0.75
  ],
  "speed": [
    0.3842,
    0.4948
  ],
  "size": [
    0.075,
    0.075
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.7519,
        0.5356,
        0.0
      ]
    ],
    [
      0.4444,
      [
        1.0,
        0.7519,
        0.5356,
        1.0
      ]
    ],
    [
      0.5556,
      [
        1.0,
        0.7519,
        0.5356,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.7519,
        0.5356,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/Atmosphere/n0-1.effect"
  }
});
