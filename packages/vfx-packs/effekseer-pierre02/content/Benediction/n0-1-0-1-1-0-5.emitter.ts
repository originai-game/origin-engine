import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.1.0.5 "EmberVariation"
export default file.particles({
  "maxParticles": 210,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.6031,
      1.6031,
      1.6031
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 84,
      "cycles": 2,
      "interval": 0.0167
    }
  ],
  "duration": 0.0333,
  "looping": false,
  "lifetime": [
    0.05,
    0.65
  ],
  "speed": [
    4.8094,
    14.4281
  ],
  "size": [
    2.0039,
    2.0039
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      0.2188,
      0.5723
    ],
    [
      0.4531,
      0.7269
    ],
    [
      0.7031,
      0.8642
    ],
    [
      1.0,
      1.0
    ]
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
        0.8751,
        0.7519,
        0.5356,
        0.0
      ]
    ],
    [
      0.2857,
      [
        0.8751,
        0.7519,
        0.5356,
        0.0728
      ]
    ],
    [
      1.0,
      [
        0.8751,
        0.7519,
        0.5356,
        0.0
      ]
    ]
  ],
  "rotation": [
    -30.0,
    30.0
  ],
  "angularVelocity": [
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Benediction/n0-1-0-1-1-0-5.effect"
  }
});
