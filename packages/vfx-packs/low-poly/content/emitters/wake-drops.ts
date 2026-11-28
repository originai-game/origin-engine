import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Spray.
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "cone",
    "angleDegrees": 40,
    "radius": 0.1
  },
  "rate": 18,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.3,
    0.5
  ],
  "speed": [
    1,
    1.8
  ],
  "size": [
    0.04,
    0.07
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
    "mesh": ref("base/vfx/low-poly/models/drop")
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
