import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -30,
    30
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 16,
      "cycles": 1,
      "interval": 0,
      "time": 0
    },
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0.10000000149011612
    }
  ],
  "color": [
    1.100000023841858,
    1.1399999856948853,
    1.1799999475479126,
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
        0.8999999761581421
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.6000000238418579
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
  "drag": 0.6000000238418579,
  "duration": 0.20000000298023224,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    1.600000023841858,
    2.4000000953674316
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 30,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 8,
    "radius": 0.5
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
    1.5
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      2.200000047683716
    ]
  ],
  "softDistance": 0.5,
  "speed": [
    9,
    15
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
