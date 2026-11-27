import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -12,
    12
  ],
  "blend": "alpha",
  "color": [
    0.14000000059604645,
    0.12999999523162842,
    0.12999999523162842,
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
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.699999988079071
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
  "drag": 0.5,
  "duration": 2,
  "gravity": [
    0,
    1.399999976158142,
    0
  ],
  "lifetime": [
    3.5,
    5
  ],
  "lit": true,
  "maxParticles": 64,
  "noise": {
    "frequency": 0.5,
    "scroll": 0.25,
    "strength": 0.30000001192092896
  },
  "rate": 10,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.6000000238418579,
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
    1,
    1.5
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      3.5999999046325684
    ]
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    0.5,
    1
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
