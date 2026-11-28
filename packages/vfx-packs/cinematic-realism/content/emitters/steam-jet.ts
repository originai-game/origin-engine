import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    2,
    2,
    2.049999952316284,
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
      0.10000000149011612,
      [
        1,
        1,
        1,
        0.4000000059604645
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        1,
        0.25
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
  "drag": 3.5,
  "duration": 2,
  "gravity": [
    0,
    0.6000000238418579,
    0
  ],
  "lifetime": [
    0.5,
    1
  ],
  "lit": true,
  "maxParticles": 96,
  "rate": 60,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 9,
    "radius": 0.019999999552965164
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
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      8
    ]
  ],
  "softDistance": 0.15000000596046448,
  "speed": [
    5,
    8
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
