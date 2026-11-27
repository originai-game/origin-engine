import { file } from '@origingame/engine/author';
// Effekseer node 0.4.0.8 "EmberVariation"
export default file.particles({
  "maxParticles": 98,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.2,
      2.6,
      0.001
    ]
  },
  "spread": 135.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 3,
      "interval": 0.35
    }
  ],
  "duration": 0.7167,
  "looping": false,
  "lifetime": [
    0.05,
    0.65
  ],
  "speed": [
    8.8247,
    19.7326
  ],
  "size": [
    5.2,
    5.2
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.2188,
      0.4297
    ],
    [
      0.4531,
      0.6358
    ],
    [
      0.7031,
      0.8189
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
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.1406,
      [
        1.0,
        0.2195,
        0.0478,
        0.1062
      ]
    ],
    [
      0.2857,
      [
        1.0,
        0.2195,
        0.0478,
        0.1793
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
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
  "gravity": [
    0.0,
    0.0,
    -14.976
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Splash"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-4-0-8.effect"
  }
});
