import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "color": [
    1,
    0.9200000166893005,
    0.7799999713897705,
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
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.2199999988079071
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
  "duration": 2,
  "gravity": [
    0,
    0.15000000596046448,
    0
  ],
  "lifetime": [
    1,
    1.7999999523162842
  ],
  "lit": true,
  "maxParticles": 80,
  "rate": 36,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.11999999731779099
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
    0.6000000238418579
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.800000011920929,
    2
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
