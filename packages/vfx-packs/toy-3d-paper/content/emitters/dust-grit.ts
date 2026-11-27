import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). Grit bouncing out of it.
export default file.particles({
  "maxParticles": 12,
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.08
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 8
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.5,
    0.8
  ],
  "speed": [
    1.6,
    2.8
  ],
  "size": [
    0.03,
    0.05
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
    "mesh": ref("base/vfx/toy-3d-paper/models/coinFlat")
  },
  "lit": true,
  "roughness": 1,
  "translucency": 0.6,
  "blend": "alpha",
  "colorOverLife": [
    [
      0,
      [
        0.351,
        0.219,
        0.119,
        1
      ]
    ],
    [
      1,
      [
        0.351,
        0.219,
        0.119,
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
