import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Glossy ice shards.
export default file.particles({
  "maxParticles": 20,
  "shape": {
    "kind": "sphere",
    "radius": 0.25
  },
  "spread": 80,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 14
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.8,
    1.1
  ],
  "speed": [
    2,
    3.6
  ],
  "size": [
    0.08,
    0.16
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.8,
      1
    ],
    [
      1,
      0
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "angularVelocity": [
    -360,
    360
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/low-poly/models/gem")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.392,
        0.761,
        1,
        1
      ]
    ],
    [
      1,
      [
        0.392,
        0.761,
        1,
        1
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0,
      [
        0.965,
        0.93,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.965,
        0.93,
        0.871,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0.35,
    "friction": 0.5,
    "against": "depth"
  }
});
