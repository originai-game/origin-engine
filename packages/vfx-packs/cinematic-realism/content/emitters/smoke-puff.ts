import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -15,
    15
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 6,
      "cycles": 1,
      "interval": 0,
      "time": 0.20000000298023224
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
        0.699999988079071
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.44999998807907104
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
  "drag": 1,
  "duration": 0.30000001192092896,
  "gravity": [
    0,
    0.5,
    0
  ],
  "lifetime": [
    2,
    3
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 8,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.30000001192092896,
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
    0.699999988079071,
    1
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      1.7999999523162842
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.20000000298023224,
    0.5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
