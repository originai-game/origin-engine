import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -12,
    12
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 7,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.20000000298023224,
    0.1899999976158142,
    0.1899999976158142,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.05999999865889549,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.5
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "drag": 2.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.05000000074505806,
    0
  ],
  "lifetime": [
    3,
    4.5
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.4000000059604645,
    "surface": false
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    0.8999999761581421,
    1.399999976158142
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      0.20000000298023224,
      1
    ],
    [
      1,
      1.7000000476837158
    ]
  ],
  "softDistance": 0.4000000059604645,
  "speed": [
    0.6000000238418579,
    1.600000023841858
  ],
  "spread": 180,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
