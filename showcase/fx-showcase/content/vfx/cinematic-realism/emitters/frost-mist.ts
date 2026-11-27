import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.8500000238418579,
    0.9300000071525574,
    1,
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
      0.20000000298023224,
      [
        1,
        1,
        1,
        0.41999998688697815
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
  "drag": 1.2000000476837158,
  "duration": 2,
  "gravity": [
    0,
    -0.5,
    0
  ],
  "lifetime": [
    1.600000023841858,
    2.5999999046325684
  ],
  "maxParticles": 24,
  "rate": 7,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.30000001192092896,
      0.6000000238418579,
      0.25
    ]
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
    0.3499999940395355,
    0.550000011920929
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
  "softDistance": 0.20000000298023224,
  "speed": [
    0.05000000074505806,
    0.25
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
