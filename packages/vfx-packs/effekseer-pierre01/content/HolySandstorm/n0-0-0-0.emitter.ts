import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "Smoke"
export default file.particles({
  "maxParticles": 153,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2,
      0.2,
      0.2
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 240,
      "interval": 0.0167
    }
  ],
  "duration": 4.0,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.6,
    1.8
  ],
  "size": [
    11.0,
    11.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4545
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
        0.5356,
        0.4423,
        0.2195,
        0.0
      ]
    ],
    [
      0.1719,
      [
        0.5356,
        0.4423,
        0.2195,
        0.0537
      ]
    ],
    [
      0.5333,
      [
        0.5356,
        0.4423,
        0.2195,
        0.251
      ]
    ],
    [
      0.75,
      [
        0.5356,
        0.4423,
        0.2195,
        0.251
      ]
    ],
    [
      0.9219,
      [
        0.5356,
        0.4423,
        0.2195,
        0.0673
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.4423,
        0.2195,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -300.0,
    300.0
  ],
  "gravity": [
    13.9762,
    0.0,
    52.16
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/HolySandstorm/n0-0-0-0.effect"
  }
});
