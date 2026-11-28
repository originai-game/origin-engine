import { file, ref } from '@origingame/engine/author';

// MIT Origin Engine (scripts/vfx-toy.ts). One drop let go each cycle.
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0,
  "rate": 0,
  "bursts": [
    {
      "time": 0,
      "count": 1
    }
  ],
  "duration": 1.2,
  "looping": true,
  "lifetime": [
    0.56,
    0.56
  ],
  "speed": [
    0,
    0
  ],
  "size": [
    0.1,
    0.1
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      1,
      1
    ]
  ],
  "rotation": [
    0,
    360
  ],
  "angularVelocity": [
    0,
    0
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
  },
  "subEmitters": [
    {
      "trigger": "death",
      "emitter": ref("base/vfx/low-poly/emitters/drip-plip"),
      "count": 6,
      "inheritVelocity": 0
    }
  ]
});
