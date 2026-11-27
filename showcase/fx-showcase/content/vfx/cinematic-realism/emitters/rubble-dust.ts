import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -12,
    12
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 8,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.800000011920929,
    0.7799999713897705,
    0.7400000095367432,
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
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.6000000238418579
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.3499999940395355
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
  "drag": 2,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.07999999821186066,
    0
  ],
  "lifetime": [
    2,
    3
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 9,
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
    0.800000011920929,
    1.2000000476837158
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2
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
