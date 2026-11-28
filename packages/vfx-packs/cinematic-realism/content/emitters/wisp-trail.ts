import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.800000011920929,
    0.800000011920929,
    0.800000011920929,
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
        0.5
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        1,
        0.2800000011920929
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
    0,
    0
  ],
  "lifetime": [
    1,
    1.600000023841858
  ],
  "lit": true,
  "maxParticles": 128,
  "noise": {
    "frequency": 1,
    "scroll": 0.30000001192092896,
    "strength": 0.11999999731779099
  },
  "rate": 70,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.019999999552965164,
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
    0.18000000715255737,
    0.2800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.5999999046325684
    ]
  ],
  "softDistance": 0.10000000149011612,
  "speed": [
    0.019999999552965164,
    0.11999999731779099
  ],
  "spread": 180,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
