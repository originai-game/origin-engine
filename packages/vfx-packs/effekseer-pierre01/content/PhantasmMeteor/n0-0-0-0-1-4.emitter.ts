import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.4 "Dark"
export default file.particles({
  "maxParticles": 169,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3,
      "cycles": 9,
      "interval": 0.0167
    }
  ],
  "duration": 0.15,
  "looping": false,
  "lifetime": [
    1.1667,
    1.1667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    11.25,
    11.25
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5556
    ],
    [
      0.2188,
      0.6832
    ],
    [
      0.4531,
      0.7977
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
        0.7519,
        0.5356,
        0.0
      ]
    ],
    [
      0.2143,
      [
        1.0,
        0.7519,
        0.5356,
        0.1255
      ]
    ],
    [
      0.5714,
      [
        1.0,
        0.7519,
        0.5356,
        0.1255
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Smoke"
  },
  "blend": "subtractive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/PhantasmMeteor/n0-0-0-0-1-4.effect"
  }
});
