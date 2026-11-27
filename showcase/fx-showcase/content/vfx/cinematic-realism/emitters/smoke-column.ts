import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -12,
    12
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 12,
      "cycles": 1,
      "interval": 0,
      "time": 0.3499999940395355
    }
  ],
  "color": [
    0.30000001192092896,
    0.2800000011920929,
    0.27000001072883606,
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
  "drag": 0.8999999761581421,
  "duration": 0.4000000059604645,
  "gravity": [
    0,
    0.6000000238418579,
    0
  ],
  "lifetime": [
    2.5999999046325684,
    3.5999999046325684
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 16,
  "rate": 0,
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
    1.2000000476837158,
    1.7999999523162842
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.600000023841858
    ]
  ],
  "softDistance": 0.5,
  "speed": [
    0.20000000298023224,
    0.5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
