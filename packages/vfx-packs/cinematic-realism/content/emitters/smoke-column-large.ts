import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -8,
    8
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 20,
      "cycles": 1,
      "interval": 0,
      "time": 0.5
    }
  ],
  "color": [
    0.2800000011920929,
    0.25999999046325684,
    0.25,
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
      0.11999999731779099,
      [
        1,
        1,
        1,
        0.800000011920929
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
  "drag": 0.6000000238418579,
  "duration": 0.6000000238418579,
  "gravity": [
    0,
    0.800000011920929,
    0
  ],
  "lifetime": [
    5,
    7
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 1.600000023841858,
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
    3,
    4.5
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.7000000476837158
    ]
  ],
  "softDistance": 1,
  "speed": [
    0.4000000059604645,
    1
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
