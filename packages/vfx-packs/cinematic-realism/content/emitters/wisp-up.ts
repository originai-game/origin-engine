import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.4000000059604645,
    0.38999998569488525,
    0.3799999952316284,
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
        0.4000000059604645
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
  "drag": 0.800000011920929,
  "duration": 2,
  "gravity": [
    0,
    0.6000000238418579,
    0
  ],
  "lifetime": [
    1.399999976158142,
    2.200000047683716
  ],
  "lit": true,
  "maxParticles": 16,
  "noise": {
    "frequency": 0.6000000238418579,
    "scroll": 0.30000001192092896,
    "strength": 0.20000000298023224
  },
  "rate": 5,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.5,
      0.20000000298023224
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
    0.25,
    0.4000000059604645
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.15000000596046448,
  "speed": [
    0.10000000149011612,
    0.30000001192092896
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
