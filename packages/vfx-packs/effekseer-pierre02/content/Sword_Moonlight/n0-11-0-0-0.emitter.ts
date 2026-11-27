import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.0 "Dark"
export default file.particles({
  "maxParticles": 45,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    2.6667,
    2.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    22.5,
    22.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6
    ],
    [
      0.4531,
      0.8179
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
        0.0005,
        0.0012,
        0.0055,
        0.0
      ]
    ],
    [
      0.0938,
      [
        0.0005,
        0.0012,
        0.0055,
        0.251
      ]
    ],
    [
      0.75,
      [
        0.0005,
        0.0012,
        0.0055,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0005,
        0.0012,
        0.0055,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-0.effect"
  }
});
