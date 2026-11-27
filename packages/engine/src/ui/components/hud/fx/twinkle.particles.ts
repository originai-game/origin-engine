import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.6000000238418579,
    0.8999999761581421
  ],
  "maxParticles": 8,
  "rate": 5,
  "rotation": [
    0,
    45
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      1,
      0
    ]
  },
  "size": [
    12,
    20
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.4000000059604645,
      1
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0,
    6
  ],
  "spread": 180,
  "texture": ref("@styles/shared/fx/sparkle.png")
});
