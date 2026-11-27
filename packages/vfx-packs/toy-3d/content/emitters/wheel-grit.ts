import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Grit.
export default file.particles({
  "maxParticles": 11,
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.08
  },
  "rate": 10,
  "duration": 0.5,
  "looping": true,
  "lifetime": [
    0.35,
    0.5
  ],
  "speed": [
    1.2,
    2
  ],
  "size": [
    0.025,
    0.04
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
    -420,
    420
  ],
  "gravity": [
    0,
    -9.8,
    0
  ],
  "drag": 0.6,
  "render": {
    "kind": "mesh",
    "mesh": ref("base/vfx/toy-3d/models/puff")
  },
  "lit": true,
  "roughness": 0.38,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.392,
        0.191,
        0.076,
        1
      ]
    ],
    [
      1,
      [
        0.392,
        0.191,
        0.076,
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
