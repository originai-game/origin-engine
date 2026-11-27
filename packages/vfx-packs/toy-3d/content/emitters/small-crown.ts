import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A small splash's drops.
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "cone",
    "angleDegrees": 22,
    "radius": 0.1
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 10
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.4,
    0.6
  ],
  "speed": [
    1.6,
    2.4
  ],
  "size": [
    0.05,
    0.08
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
    -60,
    60
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.4,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/drop")
  },
  "lit": true,
  "roughness": 0.12,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.068,
        0.392,
        0.871,
        1
      ]
    ],
    [
      1,
      [
        0.068,
        0.392,
        0.871,
        1
      ]
    ]
  ],
  "collision": {
    "bounce": 0,
    "friction": 0,
    "against": "depth",
    "die": true
  }
});
