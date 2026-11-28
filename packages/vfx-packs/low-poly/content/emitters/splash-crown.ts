import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). A crown of drops thrown up by a splash.
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "cone",
    "angleDegrees": 22,
    "radius": 0.2
  },
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 18
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6,
    0.9
  ],
  "speed": [
    2.4,
    3.6
  ],
  "size": [
    0.07,
    0.12
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
