import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.7.1 "Smoke_Variation2"
export default file.particles({
  "maxParticles": 124,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 33
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    1.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    13.685,
    13.685
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1023
    ],
    [
      0.5,
      0.5192
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
        1.0,
        0.9158,
        0.9574,
        0.0
      ]
    ],
    [
      0.3906,
      [
        1.0,
        0.9158,
        0.9574,
        0.0907
      ]
    ],
    [
      0.6,
      [
        1.0,
        0.9158,
        0.9574,
        0.1004
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9158,
        0.9574,
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
    0.0,
    0.0,
    -3.024
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Smoke"
  },
  "blend": "alpha",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-7-1.effect"
  }
});
