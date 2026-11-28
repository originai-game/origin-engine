import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -18,
    18
  ],
  "blend": "alpha",
  "color": [
    0.1599999964237213,
    0.15000000596046448,
    0.15000000596046448,
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
        0.75
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
  "drag": 0.800000011920929,
  "duration": 2,
  "gravity": [
    0,
    0.8999999761581421,
    0
  ],
  "lifetime": [
    2.200000047683716,
    3.4000000953674316
  ],
  "lit": true,
  "maxParticles": 96,
  "noise": {
    "frequency": 0.6000000238418579,
    "scroll": 0.25,
    "strength": 0.30000001192092896
  },
  "rate": 22,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.11999999731779099,
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
    0.4000000059604645,
    0.6000000238418579
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      0.30000001192092896,
      1
    ],
    [
      1,
      3
    ]
  ],
  "softDistance": 0.4000000059604645,
  "speed": [
    0.30000001192092896,
    0.800000011920929
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
